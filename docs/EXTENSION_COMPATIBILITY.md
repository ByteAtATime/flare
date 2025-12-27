# Extension Compatibility Layer

## Overview

The extension compatibility layer provides Linux equivalents for macOS-specific APIs commonly used in Raycast extensions. This allows many Raycast extensions to run on Flareup without modification.

## Features

### 1. Path Translation

Automatically translates macOS paths to Linux equivalents:

- `/Applications/` → `/usr/share/applications/`
- `/Library/` → `/usr/lib/`
- `/Users/` → `/home/`
- `~/Library/Application Support/` → `~/.local/share/`
- `~/Library/Preferences/` → `~/.config/`

### 2. AppleScript Shim

Provides Linux equivalents for common AppleScript commands:

#### Supported Patterns

- **Application Activation**: `tell application "AppName" to activate`
  - Translated to: `gtk-launch` or `xdg-open`
  
- **Application Quit**: `tell application "AppName" to quit`
  - Translated to: `pkill -f appname`
  
- **Notifications**: `display notification "message" with title "title"`
  - Translated to: `notify-send "title" "message"`
  
- **Volume Control**: `set volume N`
  - Translated to: `pactl set-sink-volume @DEFAULT_SINK@ N%` or `amixer set Master N%`

### 3. System Information

Provides cross-platform system information:

- Platform detection (always returns "linux")
- Architecture (x86_64, aarch64, etc.)
- Hostname
- Desktop environment (GNOME, KDE, etc.)

## Usage in Extensions

Extensions using macOS-specific APIs will automatically use the shims when running on Flareup:

```typescript
import { runAppleScript } from "@raycast/api";

// This will work on Linux through the shim layer
await runAppleScript('display notification "Hello" with title "Flareup"');
```

## API Reference

### Rust Backend

Located in `src-tauri/src/extension_shims.rs`:

- `PathShim::translate_path(path: &str) -> String`
- `AppleScriptShim::run_apple_script(script: &str) -> ShimResult`
- `SystemShim::get_system_info() -> HashMap<String, String>`

### Tauri Commands

- `shim_translate_path(path: String) -> String`
- `shim_run_applescript(script: String) -> ShimResult`
- `shim_get_system_info() -> HashMap<String, String>`

### TypeScript/Sidecar API

Located in `sidecar/src/api/shims.ts`:

```typescript
// Translate a macOS path to Linux equivalent
const linuxPath = await translatePath("/Applications/Safari.app");

// Run AppleScript with automatic translation
const result = await runAppleScript('tell application "Firefox" to activate');

// Get system information
const sysInfo = await getSystemInfo();
```

## Limitations

### Not Supported

1. **Complex AppleScript**: Only common patterns are supported. Complex scripts with conditionals, loops, or custom handlers will not work.

2. **Native Binaries**: Extensions that bundle macOS-specific binaries (Mach-O format) cannot be shimmed. Flareup will detect these at install time and warn you.

3. **System-Specific Features**: Some macOS features have no Linux equivalent (e.g., specific Finder operations, macOS-only system preferences).

### Partial Support

1. **Application Launching**: Works for applications with desktop files. May not work for all applications.

2. **Volume Control**: Requires PulseAudio/PipeWire or ALSA. May not work on all audio setups.

## Extension Compatibility Heuristics

When installing extensions, Flareup runs heuristic checks to detect potential incompatibilities:

1. **Mach-O Binary Detection**: Warns if macOS-only executable files are found in the extension
2. **AppleScript Detection**: Warns if `runAppleScript` is used
3. **Path Detection**: Warns if hardcoded macOS paths are found

Users are prompted to confirm installation if potential issues are detected.

## Testing

Run the Rust tests:

```bash
cargo test --package flare --lib extension_shims::tests
```

## Future Enhancements

- [ ] Support for more AppleScript patterns
- [ ] Automatic path rewriting in extension code
- [ ] Extension compatibility database
- [ ] Better error messages for unsupported features
- [ ] Fallback mechanisms for common operations
