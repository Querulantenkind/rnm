# rnm - Batch File Renamer

A modern TUI tool for batch renaming files, written in Rust.

## Features

- Modern, btop-inspired terminal interface
- Vim-style keybindings for efficient navigation
- Live preview of rename operations
- Selective file renaming (individual or batch)
- Glob pattern support
- Confirmation dialog before executing changes

## Installation

```bash
cargo build --release
# Binary will be at target/release/rnm
```

## Usage

```bash
# Current directory
rnm

# Specific directory
rnm /path/to/folder

# Glob pattern (e.g., all JPG files)
rnm *.jpg
rnm /path/to/*.png
```

## Keybindings

### File List Navigation
| Key | Action |
|-----|--------|
| `j` / Down | Next file |
| `k` / Up | Previous file |
| `Space` | Toggle file selection |
| `a` | Select/deselect all files |

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

## Interface Layout

```
+-- Files ---------------------------------------+
| * image001.jpg                                 |
|   image002.jpg                                 |
|   document.pdf                                 |
+------------------------------------------------+
+-- Operation -----------------------------------+
| Mode: [Search/Replace]                         |
| Search:   ______                               |
| Replace:  ______                               |
+------------------------------------------------+
+-- Preview -------------------------------------+
| image001.jpg  ->  photo001.jpg                 |
| image002.jpg  ->  photo002.jpg                 |
+------------------------------------------------+
+-- Help ----------------------------------------+
| j/k: Navigation  Tab: Next field  Enter: Run   |
+------------------------------------------------+
```

## Tech Stack

- [ratatui](https://ratatui.rs/) - TUI Framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal Backend
- [clap](https://clap.rs/) - CLI Argument Parsing

## License

MIT
