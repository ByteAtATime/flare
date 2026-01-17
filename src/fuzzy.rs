use nucleo::{Config, Matcher, Utf32Str};
use rayon::prelude::*;
use std::cell::RefCell;
use std::thread_local;

// TODO: what in the rust sorcery is this
thread_local! {
    static MATCHER: RefCell<Matcher> = RefCell::new(Matcher::new(Config::DEFAULT));
}

pub fn fuzzy_filter<T, F>(query: &str, items: &[T], get_text: F) -> Vec<(usize, T, i32)>
where
    T: Clone + Sync + Send,
    F: Fn(&T) -> String + Sync,
{
    if query.is_empty() {
        return items
            .iter()
            .enumerate()
            .map(|(idx, item)| (idx, item.clone(), 0))
            .collect();
    }

    items
        .par_iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            let text = get_text(item);
            MATCHER.with(|matcher| {
                let mut matcher = matcher.borrow_mut();
                let mut query_buf = Vec::new();
                let query_str = Utf32Str::new(query, &mut query_buf);
                let mut text_buf = Vec::new();
                let text_str = Utf32Str::new(&text, &mut text_buf);
                let mut indices: Vec<u32> = Vec::new();
                let score = matcher.fuzzy_indices(text_str, query_str, &mut indices);

                score.map(|s| (idx, item.clone(), s as i32))
            })
        })
        .collect()
}

pub fn fuzzy_filter_and_sort<T, F>(query: &str, items: &[T], get_text: F) -> Vec<T>
where
    T: Clone + Sync + Send,
    F: Fn(&T) -> String + Sync,
{
    let mut filtered = fuzzy_filter(query, items, get_text);
    filtered.par_sort_by(|a, b| b.2.cmp(&a.2));
    filtered.into_iter().map(|(_, item, _)| item).collect()
}
