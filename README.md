# rnm - Batch File Renamer

A modern TUI tool for batch renaming files, written in Rust.

## Features

- Modern, btop-inspired terminal interface
- Vim-style keybindings for efficient navigation
- Multiple rename modes:
  - Search/Replace
  - UPPERCASE
  - lowercase
  - Title Case
- File sorting (name, size, extension, date)
- Live preview of rename operations
- Selective file renaming (individual or batch)
- Glob pattern support
- Non-interactive CLI mode with dry-run support
- Preset system for saving and reusing rename configurations
- Configuration file support

## Installation

```bash
cargo build --release
# Binary will be at target/release/rnm
```

## Usage

### Interactive TUI Mode

```bash
# Current directory
rnm

# Specific directory
rnm /path/to/folder

# Glob pattern (e.g., all JPG files)
rnm "*.jpg"
rnm "/path/to/*.png"
```

### Non-Interactive CLI Mode

```bash
# Dry-run: Preview changes without renaming
rnm --dry-run --search "old" --replace "new"

# Execute rename with confirmation
rnm --search "IMG_" --replace "photo_"

# Skip confirmation (use with caution!)
rnm --search "old" --replace "new" --yes

# Case transformation modes
rnm --mode upper --dry-run          # UPPERCASE
rnm --mode lower --dry-run          # lowercase
rnm --mode title --dry-run          # Title Case

# Use on specific files
rnm "*.jpg" --mode lower --dry-run
```

### Presets

```bash
# Save a preset
rnm --search "IMG_" --replace "photo_" --save-preset my-preset

# List available presets
rnm --list-presets

# Use a preset
rnm --preset my-preset --dry-run
rnm --preset my-preset --yes
```

## Keybindings

### File List Navigation
| Key | Action |
|-----|--------|
| `j` / Down | Next file |
| `k` / Up | Previous file |
| `Space` | Toggle file selection |
| `a` | Select/deselect all files |

### Modes & Sorting
| Key | Action |
|-----|--------|
| `m` | Cycle rename mode |
| `s` | Cycle sort order |

### Panel Navigation
| Key | Action |
|-----|--------|
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `Esc` | Return to file list |

### Actions
| Key | Action |
|-----|--------|
| `Enter` | Execute rename operation |
| `?` | Show help |
| `q` | Quit |
| `Ctrl+C` | Force quit |

## CLI Options

```
Options:
  -n, --dry-run                    Preview changes without actually renaming
  -s, --search <SEARCH>            Search pattern for find/replace mode
  -r, --replace <REPLACE>          Replace pattern for find/replace mode
  -m, --mode <MODE>                Rename mode: search, upper, lower, title
  -p, --preset <PRESET>            Load a saved preset by name
  -y, --yes                        Skip confirmation prompt
      --save-preset <SAVE_PRESET>  Save current settings as a preset
      --list-presets               List available presets
  -h, --help                       Print help
  -V, --version                    Print version
```

## Interface Layout

```
+-- Files (/path) [A-Z] ----------------------------+
| * image001.jpg                                    |
|   image002.jpg                                    |
|   document.pdf                                    |
+---------------------------------------------------+
+-- Operation --------------------------------------+
| Mode: [Search/Replace]  (m: switch)               |
| Search:   ______                                  |
| Replace:  ______                                  |
+---------------------------------------------------+
+-- Preview ----------------------------------------+
| image001.jpg  ->  photo001.jpg                    |
| image002.jpg  ->  photo002.jpg                    |
+---------------------------------------------------+
+-- Help -------------------------------------------+
| j/k Nav  Space Select  m Mode  s Sort  Enter Run  |
+---------------------------------------------------+
```

## Configuration

Configuration is stored at `~/.config/rnm/config.toml`:

```toml
# Default rename mode
default_mode = "SearchReplace"  # or "Uppercase", "Lowercase", "TitleCase"

# Default sort order
default_sort = "Name"  # or "NameDesc", "Size", "SizeDesc", "Extension", "Date", "DateDesc"

# Saved presets
[presets.my-preset]
name = "my-preset"
mode = "SearchReplace"
search = "old"
replace = "new"
```

## Tech Stack

- [ratatui](https://ratatui.rs/) - TUI Framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal Backend
- [clap](https://clap.rs/) - CLI Argument Parsing
- [serde](https://serde.rs/) + [toml](https://github.com/toml-rs/toml) - Configuration

## License

MIT
