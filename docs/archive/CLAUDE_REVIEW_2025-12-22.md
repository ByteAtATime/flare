# Flare Gap Analysis & Review
**Date:** 2025-12-22
**Reviewer:** Claude Opus 4.5
**Focus:** Extensions compatibility, Downloads Manager, overall gaps
**Last Updated:** 2025-12-22 (post-fixes review)

---

## Executive Summary

Flare is approximately **70% feature-complete** compared to Raycast (up from 60% after recent fixes).

### ✅ Recently Fixed (This Branch)
- React Reconciler stubs now work (no more crashes)
- `usePersistentState` actually persists data
- Database indices added for performance
- N+1 query fixed in file indexer
- TcpListener port crash fixed
- Structured logging via tracing
- CPU monitor runs in background thread

### Remaining Pain Points
1. **Extensions**: AppleScript shims still limited, some APIs missing
2. **Downloads Manager**: Does not exist
3. **System Integration**: Window management, system commands, per-command hotkeys missing
4. **Code Quality**: Some unsafe `.unwrap()` calls remain

---

## 1. Extensions: Why Many Don't Work

### 1.1 ~~Critical: React Reconciler Stubs~~ ✅ FIXED

**Location:** `sidecar/src/hostConfig.ts:383-412`

~~10 React Reconciler methods throw "Function not implemented" errors instead of being no-ops.~~

**Status:** All 10 methods now return safe no-op values (void, false, null, Date.now()).

---

### 1.2 ~~Critical: usePersistentState is Fake~~ ✅ FIXED

**Location:** `sidecar/src/api/index.ts:97-139`

**Status:** Now properly persists to LocalStorage with:
- `useEffect` to load on mount
- `isLoading` state for async load tracking
- `useCallback` memoized setter that persists on every change
- Proper JSON parse/stringify with error handling

---

### 1.3 Important: AppleScript Shim is Minimal

**Location:** `src-tauri/src/extension_shims.rs:80-114`

Only 4 AppleScript patterns are supported:

| Pattern | Linux Equivalent |
|---------|-----------------|
| `tell application "X" to activate` | `gtk-launch` / `xdg-open` |
| `tell application "X" to quit` | `pkill -f` |
| `display notification` | `notify-send` |
| `set volume N` | `pactl` / `amixer` |

Everything else returns:
```
"AppleScript not supported on Linux. Script: {script}"
```

**Common unsupported operations:**
- `tell application "System Events"` (keystroke simulation)
- `do shell script` (should map to child_process)
- `tell application "Finder"` (file operations)
- `tell application "Safari"` (browser control)
- Property access (`get name of application`)

---

### 1.4 Important: Missing/Incomplete APIs

| Raycast API | Status | Location |
|-------------|--------|----------|
| `Clipboard.copy/paste` | ✅ Works | `sidecar/src/api/clipboard.ts` |
| `Clipboard.read (HTML)` | ❌ Not supported | `src-tauri/src/clipboard.rs:42` |
| `LocalStorage` | ✅ Works | `sidecar/src/api/utils.ts` |
| `Cache` | ✅ Works | `sidecar/src/api/cache.ts` |
| `usePersistentState` | ❌ Stub only | `sidecar/src/api/index.ts:97` |
| `runAppleScript` | ⚠️ 4 patterns only | `src-tauri/src/extension_shims.rs` |
| `BrowserExtension` | ⚠️ CSS only, no JS eval | `sidecar/src/api/browserExtension.ts` |
| `getSelectedFinderItems` | ✅ Works (Linux equiv) | `sidecar/src/api/environment.ts` |
| `getSelectedText` | ✅ Works | `sidecar/src/api/environment.ts` |
| `showInFinder` | ✅ Works (xdg-open) | `sidecar/src/api/environment.ts` |
| `trash` | ✅ Works | `sidecar/src/api/environment.ts` |
| `OAuth` | ⚠️ Works but unclear packageName | `sidecar/src/api/oauth.ts:151` |
| `AI.ask` | ✅ Works | `sidecar/src/api/ai.ts` |

---

### 1.5 Path Translation Gaps

**Location:** `src-tauri/src/extension_shims.rs:17-74`

Path translation exists but is incomplete:

| macOS Path | Translated To |
|------------|--------------|
| `/Applications/X.app` | `/usr/share/applications/x.desktop` |
| `/Library/` | `/usr/lib/` |
| `~/Library/Application Support/` | `~/.local/share/` |
| `~/Library/Preferences/` | `~/.config/` |
| `/Users/` | `/home/` |

**Problem:** Many extensions hardcode paths without using Raycast APIs, so translation never happens.

---

### 1.6 Extension Compatibility Estimate

| Category | % Working | Notes |
|----------|-----------|-------|
| Pure UI (lists, forms, details) | 90% | Most work fine |
| Clipboard-based | 80% | HTML not supported |
| HTTP/API extensions | 95% | Work well |
| AppleScript automation | 10% | Only basic commands |
| Native binary bundled | 0% | macOS binaries fail |
| System Events | 5% | Almost nothing works |
| Browser control | 20% | CSS queries only |

---

## 2. Downloads Manager: Does Not Exist

### Current State

The file indexer watches `~/Downloads` (`src-tauri/src/file_search/indexer.rs:20`), but this is only for **file search**, not download management.

### What's Missing

| Feature | Status |
|---------|--------|
| Download progress tracking | ❌ Not implemented |
| Download pause/resume/cancel | ❌ Not implemented |
| Download history | ❌ Not implemented |
| Downloads UI view | ❌ Not implemented |
| Browser integration | ❌ Not implemented |
| Download notifications | ❌ Not implemented |

### Recommended Implementation

1. Create `src-tauri/src/downloads/` module:
   - `manager.rs` - Track active downloads
   - `history.rs` - SQLite storage for download history
   - `monitor.rs` - Watch ~/Downloads for new files

2. Create UI in `src/lib/components/DownloadsView.svelte`

3. Add commands:
   - `list_downloads` - Get download history
   - `open_download` - Open file/folder
   - `clear_download_history` - Clean up

---

## 3. Other Major Gaps

### 3.1 Critical Missing Features

| Feature | Priority | Effort | Notes |
|---------|----------|--------|-------|
| Window Management | Critical | 2 weeks | X11 via x11rb, Wayland per-compositor |
| System Commands | Critical | 1 week | shutdown, restart, sleep, lock |
| Per-Command Hotkeys | Critical | 1 week | Currently only global app toggle |
| System Tray | High | 3 days | No background indicator |

### 3.2 Code Quality Issues

#### Unsafe `.unwrap()` Calls (32+ instances)

**High-risk locations:**

| File | Risk | Issue |
|------|------|-------|
| `browser_extension.rs:170` | **Critical** | `TcpListener::bind().expect()` - crashes if port taken |
| `soulver.rs:10` | High | `CString::new().expect()` - crashes on invalid path |
| `snippets/engine.rs:22-28` | Medium | `Regex::new().unwrap()` - unlikely to fail |
| `snippets/manager.rs` | Medium | Many unwraps in tests |

**Fix:** Replace with `?` operator or `match` statements.

#### ~~TcpListener Port Binding~~ ✅ FIXED

**Location:** `src-tauri/src/browser_extension.rs:170`

**Status:** Now uses proper `match` with `tracing::error!` and graceful return instead of crashing.

---

## 4. TODO Comments in Codebase

### TypeScript/Svelte TODOs

| Location | Comment | Priority |
|----------|---------|----------|
| `src/lib/assets.ts:44` | `// TODO: better heuristic?` | Low |
| `src/lib/assets.ts:68` | `// TODO: better heuristic?` | Low |
| `src/lib/assets.ts:74` | `// TODO: actually handle adjustContrast` | Low |
| `src/lib/components/CommandDeeplinkConfirm.svelte:39` | `<!-- TODO: implement "always open" -->` | Medium |
| `src/lib/components/nodes/shared/actions.ts:8` | `// TODO: naming?` | Low |
| `sidecar/src/api/oauth.ts:151` | `// TODO: what does this mean?` (packageName) | Medium |

### Rust TODOs

No TODO comments found in Rust code.

---

## 5. Performance Issues

### 5.1 ~~N+1 Query in File Indexer~~ ✅ FIXED

**Location:** `src-tauri/src/file_search/indexer.rs`

**Status:** Fixed in commit `55a7bd0`. Now uses batch query with HashMap lookup.

### 5.2 ~~Missing Database Indices~~ ✅ FIXED

**Status:** All 6 indices added in commit `55a7bd0`:
- `idx_ai_generations_created`
- `idx_ai_conversations_updated`
- `idx_clipboard_content_type`
- `idx_clipboard_pinned`
- `idx_clipboard_last_copied`
- `idx_snippets_keyword`

---

## 6. Recommended Action Plan

### ✅ Completed Quick Wins

| # | Task | Status | Source |
|---|------|--------|--------|
| 1 | Fix React Reconciler stubs (no-op, don't throw) | ✅ Done | Current branch |
| 2 | Implement `usePersistentState` properly | ✅ Done | Current branch |
| 3 | Add database indices | ✅ Done | Commit `55a7bd0` |
| 4 | Fix TcpListener crash on port conflict | ✅ Done | Current branch |
| 5 | N+1 query fix in file indexer | ✅ Done | Commit `55a7bd0` |
| 6 | Replace println!/eprintln! with tracing | ✅ Done | Commit `8ff7426` |
| 7 | CPU monitor background thread | ✅ Done | Commit `8ff7426` |
| 8 | Remove debug console.log statements | ✅ Done | Commit `55a7bd0` |

### Remaining Quick Wins

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 1 | Add more AppleScript shims (open URL, do shell script) | 4 hours | Medium |
| 2 | Replace remaining `.unwrap()` with safe handling | 1 day | High |

### Medium Term (1-2 weeks)

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 3 | Create Downloads Manager module | 2 days | Medium |
| 4 | Window management (X11) | 1 week | High |
| 5 | System commands (shutdown/lock/sleep) | 2 days | High |
| 6 | Per-command global hotkeys | 1 week | High |

### Long Term (1+ months)

- Wayland window management (compositor-specific)
- Full AppleScript parser/translator
- Extension compatibility scoring system
- Fork/adapt top 10 popular Raycast extensions for Linux

---

## 7. Files Referenced

| File | Purpose |
|------|---------|
| `sidecar/src/hostConfig.ts` | React Reconciler configuration |
| `sidecar/src/api/index.ts` | Raycast API exports |
| `sidecar/src/api/*.ts` | Individual API implementations |
| `src-tauri/src/extension_shims.rs` | macOS API compatibility |
| `src-tauri/src/browser_extension.rs` | WebSocket server |
| `src-tauri/src/file_search/indexer.rs` | File indexing |
| `src-tauri/src/clipboard.rs` | Clipboard operations |
| `TODO.md` | Existing task tracking |

---

## 8. Conclusion

Flare has made significant progress. **8 of the original quick wins are now complete.**

### What's Working Well Now
- ✅ Extension React rendering (reconciler fixed)
- ✅ Extension state persistence (usePersistentState fixed)
- ✅ Database performance (indices + N+1 fix)
- ✅ Logging infrastructure (tracing)
- ✅ System monitoring (background CPU thread)
- ✅ Stability (TcpListener crash fixed)

### Remaining Focus Areas
1. **Extension Compatibility**: Expand AppleScript shims, add missing APIs
2. **Feature Gaps**: Downloads Manager, Window Management, System Commands
3. **Code Quality**: ~30 remaining `.unwrap()` calls need safe handling

### Estimated Timeline
- **Current State:** ~70% Raycast feature parity (up from 60%)
- **To 90% parity:** 6-8 weeks of focused development
- **Key blockers:** Window management (X11/Wayland complexity)

---

*This review supplements the existing TODO.md with specific technical findings.*
*Updated after fixes on 2025-12-22.*
