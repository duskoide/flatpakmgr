# flatpakmgr

A terminal user interface (TUI) for managing Flatpak applications, built with Rust and Ratatui.

## Features

- Browse installed Flatpak applications and runtimes
- View detailed app information (version, runtime, license, permissions)
- Search and install new apps from remote repositories
- Update and uninstall applications
- Manage Flatpak remotes (enable/disable)
- View operation history
- Job management with progress tracking
- Responsive layout that adapts to terminal size

## Install

Download the latest release from [GitHub Releases](https://github.com/duskoide/flatpakmgr/releases):

```bash
# Download and extract
curl -sL https://github.com/duskoide/flatpakmgr/releases/latest/download/flatpakmgr-linux-x86_64.tar.gz | tar xz

# Move to PATH
sudo mv flatpakmgr /usr/local/bin/
```

Or build from source:

```bash
cargo install --path .
```

## Build

```bash
cargo build --release
```

The binary will be at `target/release/flatpakmgr`.

## Usage

```bash
flatpakmgr [OPTIONS]

Options:
      --user              Use user installation
      --system            Use system installation
      --installation <N>  Use specific installation
      --no-system         Disable system installation
  -v, --verbose           Enable verbose logging
  -h, --help              Print help
  -V, --version           Print version
```

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Show help |
| `J` | Show jobs |

### Navigation
| Key | Action |
|-----|--------|
| `1`-`5` | Switch tabs (Apps, Runtimes, Remotes, History, Install) |
| `Tab` | Cycle focus: Tabs → List → Detail |
| `Shift+Tab` | Focus tabs |
| `Esc` | Focus tabs / close modal |

### Apps Tab
| Key | Action |
|-----|--------|
| `j`/`↓` | Next app |
| `k`/`↑` | Previous app |
| `r` | Refresh list |
| `u` | Update selected app |
| `U` | Update all |
| `d` | Uninstall selected app |
| `p` | Show permissions |

### Remotes Tab
| Key | Action |
|-----|--------|
| `e` | Toggle enable/disable remote |

### Install Tab
| Key | Action |
|-----|--------|
| `/` | Focus search |
| `Enter` | Install selected package |

## Architecture

```
src/
├── app/           # Application state and input handling
│   ├── mod.rs     # App struct, refresh helpers
│   ├── input.rs   # Keyboard input handlers
│   ├── mode.rs    # Focus/Mode/Tab enums
│   └── tabs/      # Per-tab state (AppsTab, RuntimesTab, etc.)
├── flatpak_service/  # Flatpak CLI abstraction
│   ├── mod.rs     # FlatpakService with async commands
│   ├── parse.rs   # Output parsers
│   ├── types.rs   # Data types (AppRef, Permission, etc.)
│   └── job.rs     # Background job manager
├── ui/            # TUI rendering
│   ├── mod.rs     # Root draw function
│   ├── layout.rs  # Main layout with responsive checks
│   ├── tabs/      # Per-tab renderers
│   ├── modals/    # Modal dialogs (help, jobs, permissions, confirm)
│   ├── status_bar.rs
│   └── toast.rs
├── config.rs      # Configuration file support
├── telemetry.rs   # Logging/tracing setup
├── lib.rs
└── main.rs        # Entry point, event loop
```

## Configuration

Config is stored at `~/.config/flatpakmgr/config.json` (Linux) or equivalent platform directory.

```json
{
  "default_installation": "system"
}
```

## License

MIT License

Copyright (c) 2026 duskoide

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.