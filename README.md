# rnm - Batch File Renamer

A modern TUI tool for batch renaming files, written in Rust.

## Features

- Modern, btop-inspired terminal interface
- Vim-style keybindings for efficient navigation
- **8 rename modes:**
  - Search/Replace
  - Regex with capture groups ($1, $2, ...)
  - Sequential numbering with padding
  - Prefix add/remove
  - Suffix add/remove
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

#### Search/Replace
```bash
rnm --dry-run --search "old" --replace "new"
rnm --search "IMG_" --replace "photo_" --yes
```

#### Regex with Capture Groups
```bash
# Rename IMG_001.jpg to photo_001.jpg
rnm --mode regex --search "IMG_(\d+)" --replace 'photo_$1' --dry-run

# Swap parts: vacation_2024.jpg to 2024_vacation.jpg
rnm --mode regex --search "(.+)_(\d+)" --replace '$2_$1' --dry-run
```

#### Numbering
```bash
# file_### uses 3-digit padding: file_001, file_002, ...
rnm --pattern "photo_###" --dry-run

# Start at 10 with 4-digit padding
rnm --pattern "image_####" --start 10 --dry-run
```

#### Prefix/Suffix
```bash
# Add prefix
rnm --prefix "backup_" --dry-run

# Remove prefix
rnm --remove-prefix "backup_" --dry-run

# Add suffix (before extension)
rnm --suffix "_v2" --dry-run

# Remove suffix
rnm --remove-suffix "_old" --dry-run
```

#### Case Transformation
```bash
rnm --mode upper --dry-run    # UPPERCASE
rnm --mode lower --dry-run    # lowercase
rnm --mode title --dry-run    # Title Case
```

### Presets

```bash
# Save a preset
rnm --search "IMG_" --replace "photo_" --save-preset photo-rename

# List available presets
rnm --list-presets

# Use a preset
rnm --preset photo-rename --dry-run
rnm --preset photo-rename --yes
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
| `t` | Toggle add/remove (prefix/suffix modes) |

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
  -n, --dry-run                        Preview changes without renaming
  -s, --search <SEARCH>                Search pattern (search/replace or regex)
  -r, --replace <REPLACE>              Replace pattern
  -m, --mode <MODE>                    Mode: search, regex, numbering, prefix,
                                       suffix, upper, lower, title
      --pattern <PATTERN>              Numbering pattern (e.g., "photo_###")
      --start <START>                  Starting number [default: 1]
      --prefix <PREFIX>                Add prefix to filenames
      --suffix <SUFFIX>                Add suffix (before extension)
      --remove-prefix <REMOVE_PREFIX>  Remove prefix from filenames
      --remove-suffix <REMOVE_SUFFIX>  Remove suffix (before extension)
  -p, --preset <PRESET>                Load a saved preset
  -y, --yes                            Skip confirmation prompt
      --save-preset <SAVE_PRESET>      Save settings as preset
      --list-presets                   List available presets
  -h, --help                           Print help
  -V, --version                        Print version
```

## Interface Layout

```
+-- Files (/path) [A-Z] ----------------------------+
| * image001.jpg                                    |
|   image002.jpg                                    |
|   document.pdf                                    |
+---------------------------------------------------+
+-- Operation --------------------------------------+
| Mode: [Regex]  (m: switch)                        |
| Regex:   IMG_(\d+)                                |
| Replace: photo_$1                                 |
+---------------------------------------------------+
+-- Preview ----------------------------------------+
| IMG_001.jpg  ->  photo_001.jpg                    |
| IMG_002.jpg  ->  photo_002.jpg                    |
+---------------------------------------------------+
+-- Help -------------------------------------------+
| j/k Nav  Space Sel  m Mode  s Sort  t Toggle  ... |
+---------------------------------------------------+
```

## Rename Modes

| Mode | Description | Example |
|------|-------------|---------|
| **Search/Replace** | Simple text replacement | `old` -> `new` |
| **Regex** | Full regex with capture groups | `IMG_(\d+)` -> `photo_$1` |
| **Numbering** | Sequential numbers with padding | `photo_###` -> `photo_001` |
| **Prefix** | Add/remove text at start | `backup_` + `file.txt` |
| **Suffix** | Add/remove text before extension | `file` + `_v2` + `.txt` |
| **Uppercase** | Convert to UPPERCASE | `file.txt` -> `FILE.txt` |
| **Lowercase** | Convert to lowercase | `FILE.TXT` -> `file.txt` |
| **Title Case** | Capitalize each word | `hello_world` -> `Hello_World` |

## Configuration

Configuration is stored at `~/.config/rnm/config.toml`:

```toml
# Default rename mode
default_mode = "SearchReplace"

# Default sort order
default_sort = "Name"

# Saved presets
[presets.photo-rename]
name = "photo-rename"
mode = "SearchReplace"
search = "IMG_"
replace = "photo_"
```

## Tech Stack

- [ratatui](https://ratatui.rs/) - TUI Framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal Backend
- [clap](https://clap.rs/) - CLI Argument Parsing
- [regex](https://docs.rs/regex) - Regular Expressions
- [serde](https://serde.rs/) + [toml](https://github.com/toml-rs/toml) - Configuration

## License

MIT
