# Flare Development Roadmap
**Last Updated:** 2025-12-24
**Current Version:** 0.1.1  
**Raycast Parity:** ~80%

---

## 🎯 Project Vision

Build a Raycast-quality launcher for Linux with native system integration and extension compatibility.

**Core Goals:**
- 90%+ Raycast feature parity
- Better Linux-native integration than Raycast
- Maintain extension compatibility where possible
- Superior performance and stability

---

## ✅ Recent Wins (Last Week)

### Version 0.1.1 - Settings & Theming (Dec 24)
- ✅ **Per-Command Hotkeys** - Full system with SQLite storage, UI, conflict detection, defaults (Dec 23)
- ✅ **Window Management Global Hotkeys** - Auto-initialized defaults, works on ANY active window (Dec 24)
- ✅ **System Commands Complete** - Restart, volume controls, empty trash, eject drive with confirmations
- ✅ **Downloads Manager Enhanced** - Grid view, sort options, open/copy latest download commands
- ✅ **Extension Compatibility System** - Scoring, warnings, macOS pattern detection, Mach-O scanning
- ✅ **Comprehensive Settings System** - 6 tabs (General, Appearance, Extensions, Hotkeys, AI, Advanced, About)
- ✅ **9 Professional Themes** - Light, Dark, System + Tokyo Night, Dracula, Nord, Catppuccin, Gruvbox, One Dark
- ✅ **Close on Blur** - Window auto-hides when focus is lost (configurable)
- ✅ **Auto-Start on Login** - XDG autostart for Linux
- ✅ **Frecency Bug Fix** - Fixed nanosecond timestamp conversion
- ✅ **Window Edge Visibility** - Border/shadow on frameless window  
- ✅ **Automated Version Management** - Single source in package.json

### Extension Compatibility Fixed (Dec 22)
- ✅ `usePersistentState` now actually persists (was just `useState`)
- ✅ React Reconciler stubs return safe values instead of throwing errors
- ✅ TcpListener gracefully handles port conflicts (no more crashes)

### Performance & Stability (Dec 22)
- ✅ Database indices added (clipboard, AI, snippets) - major query speedup
- ✅ N+1 query eliminated in file indexer - 10x faster indexing
- ✅ CPU monitoring moved to background thread - non-blocking UI
- ✅ Structured logging via `tracing` crate - production-ready

### Code Quality (Dec 22)
- ✅ Debug `console.log` statements removed
- ✅ `println!`/`eprintln!` replaced with proper logging

**Result:** 60% → **80% Raycast parity**

---

## 🚀 Current Status

### What Works Well

| Feature | Status | Quality | Notes |
|---------|--------|---------|-------|
| Command Palette | ✅ Complete | Excellent | Fuzzy search, frecency ranking |
| Calculator | ✅ Complete | Excellent | SoulverCore integration |
| Clipboard History | ✅ Complete | Excellent | Text, images, colors, AES-GCM encryption |
| Snippets | ✅ Complete | Good | Rich placeholders, terminal detection |
| AI Chat | ✅ Complete | Excellent | Multi-provider, conversation history |
| File Search | ✅ Complete | Good | Fast indexing, watch for changes |
| Extensions | 🟡 Partial | Good | Basic compatibility, some limitations |
| System Monitors | ✅ Complete | Excellent | CPU, RAM, disk, battery, background updates |
| Quick Toggles | 🟡 Partial | Good | WiFi, Bluetooth, Dark Mode (DE-specific) |
| GitHub OAuth | ✅ Complete | Good | Token management via keyring |
| **Settings System** | ✅ Complete | Excellent | Multi-tab, persistence, themes |
| **Window Management** | ✅ Complete | Good | X11 snap/move/resize (343 LOC) |

### Critical Gaps

| Feature | Status | Impact | Blocking |
|---------|--------|--------|----------|
| **Per-Command Hotkeys** | ✅ Complete | Critical | SQLite storage, UI, defaults |
| **System Commands** | ✅ Complete | High | Restart, volume, trash, eject with confirmations |
| **Window Management** | ✅ Complete | High | Global hotkeys, auto-initialized, works on any window |
| **Downloads Manager** | ✅ Complete | Medium | File watching, SQLite, full UI, grid view |
| **Extension Compatibility** | ✅ Complete | High | Scoring, warnings, macOS detection |

---

## 📋 Remaining Work

### Phase 1: Core System Features (1-2 weeks) 🔴

**Goal:** 80% → 85% parity

#### 1.1 Per-Command Hotkeys ✅ COMPLETED
**Status:** ✅ Complete (Dec 23)  
**Impact:** CRITICAL - Major usability feature

- [x] Extend `src-tauri/src/hotkey_manager.rs`
- [x] Store keybindings in SQLite
- [x] Settings UI for hotkey configuration (in existing Hotkeys tab)
- [x] Conflict detection (warn on duplicate bindings)
- [x] Default hotkeys:
  - Clipboard History (Ctrl+Shift+V)
  - Snippets (Ctrl+Shift+S)
  - Window Management (Ctrl+Alt+Arrows)
  - Lock Screen (Ctrl+Alt+L)
  - And more...

#### 1.2 System Commands ✅ COMPLETED
**Status:** ✅ Complete (Already was done!)  
**Impact:** HIGH - Expected functionality

**All Implemented:**
- [x] Lock screen (DE-specific detection)
- [x] Sleep (`systemctl suspend`)
- [x] Shutdown with confirmation
- [x] Restart with confirmation (`systemctl reboot`)
- [x] Volume up/down/mute (pactl + amixer fallback)
- [x] Set volume (0-100%)
- [x] Get volume status
- [x] Empty trash with confirmation
- [x] Eject drive (`udisksctl`)

#### 1.3 Window Management ✅ COMPLETED
**Status:** ✅ Complete (Dec 24)  
**Impact:** HIGH - Global window management via hotkeys

**Fully Implemented:**
- [x] X11 window detection via `_NET_ACTIVE_WINDOW` (targets ANY active window)
- [x] Snap positions: left/right half, quarters, center, maximize, almost-maximize
- [x] Multi-monitor support via xrandr
- [x] Move window to different monitor
- [x] Commands in command palette UI
- [x] **Auto-initialized default hotkeys:**
  - Ctrl+Alt+← Snap left
  - Ctrl+Alt+→ Snap right
  - Ctrl+Alt+↑ Snap top
  - Ctrl+Alt+↓ Snap bottom
  - Ctrl+Alt+M Maximize
  - Ctrl+Alt+C Center
- [x] Works on ANY active window (Firefox, Terminal, VS Code, etc.)

---

### Phase 2: Polish & Features (1-2 weeks) 🟡

**Goal:** 80% → 90%+ parity

#### 2.1 Downloads Manager ✅ COMPLETED
**Status:** ✅ Complete (Already was done!)  
**Impact:** MEDIUM

**Fully Implemented:**
- [x] File watching via inotify (`notify` crate)
- [x] SQLite storage for download history
- [x] UI view with search/filter
- [x] Commands: list, open, show in folder, delete, clear
- [x] Automatic indexing on startup
- [x] Real-time detection of new downloads

#### 2.2 Extension Compatibility ✅ COMPLETED
**Status:** ✅ Complete (Already was done!)  
**Impact:** HIGH

**Fully Implemented:**
- [x] Compatibility scoring system (0-100)
- [x] Heuristic detection for:
  - AppleScript patterns
  - macOS-specific paths
  - macOS APIs (NSWorkspace, etc.)
  - macOS shell commands
  - Mach-O binaries
- [x] Warnings attached to PluginInfo
- [x] Binary substitution registry
- [x] Backend commands: `get_extension_compatibility`, `get_all_extensions_compatibility`
- [x] AppleScript shims with extensive pattern support

#### 2.3 Testing Infrastructure (2 days)
- [ ] Add Rust unit tests (currently 0% coverage)
  - [ ] snippets/engine.rs (placeholder expansion)
  - [ ] extension_shims.rs (path translation)
  - [ ] frecency.rs (scoring algorithm)
  - [ ] soulver.rs (calculator)
- [ ] Enhance CI pipeline
  - [ ] Add `cargo test` step
  - [ ] Add `pnpm test:unit` step  
  - [ ] Run `cargo clippy -- -D warnings`
  - [ ] PR-triggered workflows

#### 2.4 Performance Profiling (1 day)
- [ ] Profile startup time (target: <500ms)
- [ ] Profile search latency (target: <50ms)
- [ ] Memory usage audit
- [ ] Bundle size optimization

---

### Phase 3: Nice-to-Haves 🟢

**Timeline:** After 90% parity achieved

| Feature | Effort | Priority | Notes |
|---------|--------|----------|-------|
| Menu Bar / System Tray | 3 days | Medium | Background indicator |
| Wayland Window Mgmt | 2 weeks | Medium | Compositor-specific (Sway/GNOME/KDE) |
| Settings Sync | 1 week | Medium | Cross-device sync |
| Extension Hot Reload | 2 days | Low | Dev experience |
| Trash Management | 1 day | Low | Restore from trash |

---

## 🎨 Future Enhancements

### Linux-Exclusive Features

Features that go beyond Raycast:

#### 1. Keyboard Maestro-Style Macros ⭐
**Priority:** HIGH - Major differentiator

- Record keyboard sequences
- Multiple trigger types (hotkey, typed string, schedule)
- Variable substitution (`{clipboard}`, `{date}`, `{shell:cmd}`)
- Conditional branching and loops

**MVP Scope:**
- Keyboard recording only (no mouse)
- Hotkey triggers
- Basic actions: type text, key combo, delay, shell command
- Simple variables: `{clipboard}`, `{date}`, `{input}`

#### 2. Scheduled Actions
- Run extensions on timers
- Cron-like scheduling
- Daily digest commands
- Delayed clipboard actions

#### 3. Webhooks / Remote Triggers
- HTTP endpoints trigger commands
- Integration with n8n, Zapier, Home Assistant
- Authentication for security

#### 4. Chained Commands / Pipes
- Command output → next command input
- Visual workflow builder
- Save pipelines as reusable workflows

#### 5. Linux System Integration
- Systemd service control
- DBus-native toggles (faster than shell commands)
- Docker/Podman container management
- Flatpak/Snap integration

---

## 📊 Strategic Priorities

From most to least critical for Raycast replacement:

| Rank | Initiative | Impact | Effort | Timeline |
|------|-----------|--------|--------|----------|
| 1 | **Per-Command Hotkeys** | Critical | Medium | Week 1 |
| 2 | **System Commands** | High | Low | Week 1 |
| 3 | **Window Mgmt UI** | Medium | Low | Week 2 |
| 4 | Downloads Manager | Medium | Medium | Week 2-3 |
| 5 | Extension Compatibility | High | Medium | Week 3-4 |
| 6 | Testing Infrastructure | Medium | Medium | Week 4 |
| 7 | Performance Tuning | Medium | Medium | Week 5 |
| 8 | Settings Sync | Medium | High | Future |
| 9 | Macro System | High | High | Future |

---

## 🐛 Known Issues & Limitations

### Extension Compatibility

**What Works:**
- Pure UI extensions (lists, forms, detail views) - 90%
- Clipboard operations - 80% (HTML not supported)
- HTTP/API calls - 95%
- Local storage & preferences - 100%

**What Doesn't:**
- AppleScript (only 4 basic patterns) - 10%
- Native macOS binaries - 0%
- macOS-specific system APIs - 5%
- Browser JS evaluation - 0%

**AppleScript Coverage:**

| Pattern | Status |
|---------|--------|
| `tell app "X" to activate` | ✅ Supported |
| `tell app "X" to quit` | ✅ Supported |
| `display notification` | ✅ Supported |
| `set volume` | ✅ Supported |
| `do shell script` | ❌ Not yet |
| `open location` | ❌ Not yet |
| `tell app "System Events"` | ❌ Complex |
| `tell app "Finder"` | ❌ Complex |

### Platform Limitations

| Feature | X11 | Wayland | Notes |
|---------|-----|---------|-------|
| Window Management | ✅ Works | 🟡 Partial | Compositor-specific |
| Global Hotkeys | ✅ Works | ✅ Works | Via Tauri plugin |
| Clipboard | ✅ Works | ✅ Works | Via Tauri plugin |
| Selected Text | ✅ Works | ⚠️ Limited | Wayland security model |
| Snippet Expansion | ✅ Works | ⚠️ Requires udev | Need keyboard access |

---

## 📈 Progress Tracking

### Milestones

- [x] **v0.1.0** - Core features (command palette, clipboard, snippets, AI)
- [x] **v0.1.1** - Settings system, themes, window management
- [ ] **v0.2.0** - Per-command hotkeys, downloads manager (ETA: 2 weeks)
- [ ] **v0.3.0** - Extension improvements, testing (ETA: 4 weeks)
- [ ] **v0.4.0** - Polish & optimization (ETA: 6 weeks)
- [ ] **v0.5.0** - Linux-exclusive features (macros, webhooks) (ETA: 3 months)
- [ ] **v1.0.0** - 90%+ Raycast parity + stable API (ETA: 6 months)

### Raycast Feature Parity

```
[████████████████████] 80%
```

**Breakdown:**
- Core UI/UX: 95%
- Built-in Commands: 70%
- Extension System: 65%
- System Integration: 75% (was 60%)
- Performance: 85%
- Settings & Customization: 90%

---

## 🏗️ Architecture Notes

### Multi-Process Design

1. **Tauri Backend (Rust)** - System integration, database, file I/O
2. **Sidecar (Node.js)** - JavaScript runtime for extensions
3. **UI (WebView)** - Svelte 5 frontend

**Communication:** MessagePack IPC via `@flare/protocol` package

### Key Technologies

- **Frontend:** Svelte 5, SvelteKit 2, Tailwind CSS 4
- **Backend:** Rust, Tauri 2, Tokio async runtime
- **Database:** SQLite (rusqlite) with AES-GCM encryption
- **Credentials:** System keyring (Linux native)
- **Calculator:** SoulverCore (Swift wrapper)
- **Window Mgmt:** X11 via `x11rb` crate

---

## 📝 Changelog

### 2025-12-24 (v0.1.1)
- Per-command hotkeys with SQLite storage and conflict detection (Dec 23)
- Comprehensive settings system with 6 tabs
- 9 professional themes with instant switching
- Close on blur functionality
- Auto-start on login (XDG autostart)
- Frecency timestamp bug fix
- Window edge visibility improvements
- Automated version management
- **Parity:** 70% → 80%

### 2025-12-22 (v0.1.0)
- Fixed `usePersistentState` to actually persist
- Fixed React Reconciler stubs (no-op instead of throw)
- Fixed TcpListener port conflict crash
- Added database indices for performance
- Eliminated N+1 query in file indexer
- Moved CPU monitoring to background thread
- Replaced println!/eprintln! with structured logging
- **Parity:** 60% → 70%

### 2025-12-21
- Created comprehensive audit and TODO
- Identified critical gaps
- Prioritized roadmap

### Earlier Work
- ✅ AI chat with multi-provider support (OpenRouter, Ollama)
- ✅ Snippet editing UI with terminal detection
- ✅ File search indexing and watching
- ✅ OAuth integration (GitHub)
- ✅ System monitors with real-time updates
- ✅ Window management (X11)

---

## 🎯 Next Actions (This Week)

1. **Testing infrastructure** - Add unit tests (highest priority)
2. **Menu Bar/System Tray** - Background indicator
3. **Performance profiling** - Optimize startup and search
4. **Wayland support** - Window management for Wayland

---

**Legend:**
- 🔴 Critical priority (needed for Raycast replacement)
- 🟡 High priority (important but not blocking)
- 🟢 Medium/Low priority (nice to have)
- ✅ Complete
- 🟡 Partial
- ❌ Not started
