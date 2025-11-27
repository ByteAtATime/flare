use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const HALF_LIFE_DAYS: f64 = 15.0;
const MS_PER_DAY: f64 = 24.0 * 60.0 * 60.0 * 1000.0;

const POINTS_VISIT: f64 = 100.0;

// if a user clicks the same thing within BURST_WINDOW_MS, give less points
const POINTS_BURST: f64 = 5.0;
const BURST_WINDOW_MS: u64 = 2 * 60 * 1000;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FrecencyEntry {
    score: f64,
    last_visited_at: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FrecencyStore {
    entries: HashMap<String, FrecencyEntry>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl FrecencyStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        if let Ok(file_content) = fs::read_to_string(&path) {
            if let Ok(mut store) = serde_json::from_str::<FrecencyStore>(&file_content) {
                store.file_path = Some(path);
                return store;
            }
        }

        Self {
            entries: HashMap::new(),
            file_path: Some(path),
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn compute_decayed_score(entry: &FrecencyEntry, current_time: u64) -> f64 {
        let days_elapsed = (current_time.saturating_sub(entry.last_visited_at)) as f64 / MS_PER_DAY;

        let lambda = 2.0_f64.ln() / HALF_LIFE_DAYS;

        entry.score * (-lambda * days_elapsed).exp()
    }

    pub fn visit(&mut self, id: String) {
        let now = Self::now();

        let new_entry = if let Some(old_entry) = self.entries.get(&id) {
            let decayed_score = Self::compute_decayed_score(old_entry, now);

            let is_burst = (now - old_entry.last_visited_at) < BURST_WINDOW_MS;
            let points = if is_burst { POINTS_BURST } else { POINTS_VISIT };

            FrecencyEntry {
                score: decayed_score + points,
                last_visited_at: now,
            }
        } else {
            FrecencyEntry {
                score: POINTS_VISIT,
                last_visited_at: now,
            }
        };

        self.entries.insert(id, new_entry);

        let _ = self.save();
    }

    pub fn sort<T, F>(&self, items: &mut [T], id_extractor: F)
    where
        F: Fn(&T) -> &str,
    {
        let now = Self::now();

        let mut scores: HashMap<String, f64> = HashMap::with_capacity(items.len());

        for item in items.iter() {
            let id = id_extractor(item);
            if let Some(entry) = self.entries.get(id) {
                scores.insert(id.to_string(), Self::compute_decayed_score(entry, now));
            }
        }

        items.sort_by(|a, b| {
            let id_a = id_extractor(a);
            let id_b = id_extractor(b);

            let score_a = scores.get(id_a);
            let score_b = scores.get(id_b);

            match (score_a, score_b) {
                (Some(sa), Some(sb)) => sb.partial_cmp(sa).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    }

    // atomic save operation: if the write isn't successful, it will not corrupt the data
    // fs::rename should be atomic, i.e. it either fully renames or doesn't
    pub fn save(&self) -> io::Result<()> {
        if let Some(path) = &self.file_path {
            let json = serde_json::to_string(self)?;

            let tmp_path = path.with_extension("tmp");
            fs::write(&tmp_path, json)?;

            fs::rename(tmp_path, path)?;
        }
        Ok(())
    }

    pub fn reset(&mut self, id: &str) {
        self.entries.remove(id);
        let _ = self.save();
    }
}
