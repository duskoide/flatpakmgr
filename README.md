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

MIT