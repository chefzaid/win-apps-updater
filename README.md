# Windows Apps Updater

A lightweight, modern GUI for batch-updating Windows applications through **winget** (Windows Package Manager).

## Features

- **Modern dark UI** - clean design with styled controls, alternating rows, and overlay dialogs
- **List updatable apps** - detects every application with a pending update via `winget upgrade`
- **Selective updates** - pick which applications to update with checkboxes (select all / deselect all)
- **Batch updates** - update multiple applications in one click
- **Progress tracking** - overall progress bar showing how many apps have been updated
- **Search & filter** - instantly narrow the list by name or ID

## Getting Started

### Prerequisites

- **Windows 10 / 11** - winget must be installed (ships with modern Windows) 
- **Rust toolchain** - Latest stable - [install](https://www.rust-lang.org/tools/install) |

### Build & Run

```bash
git clone https://github.com/chefzaid/win-apps-updater.git
cd win-apps-updater
cargo build --release
```

The executable is written to `target/release/WinAppsUpdater.exe`.

## How It Works

1. Runs `winget upgrade --include-unknown` in the background
2. Sanitises the raw output (strips progress-spinner characters, unwraps line-wrapped tables)
3. Parses the column-aligned table using header positions - correctly handles multi-word app names
4. Displays the results in an Iced GUI table
5. For each selected app, runs `winget upgrade --id <id>` sequentially with live progress

## Project Structure

```
src/
  main.rs            Entry point & window configuration
  lib.rs             Library re-exports for testing
  models.rs          Data types: UpdatableApp, AppItem, Message
  app.rs             Application state & update logic (Elm architecture)
  winget.rs          Winget CLI integration, output parsing & sanitisation
  ui/
    mod.rs           UI module exports
    components.rs    View builders, styles & layout
    icon.rs          Programmatic window-icon generation
tests/
  integration_test.rs
build.rs             Generates multi-size .ico & embeds it via winresource
```

## Technology Stack

| Component       | Choice                                                           |
|-----------------|------------------------------------------------------------------|
| Language        | Rust 2021 Edition                                                |
| GUI             | [Iced](https://github.com/iced-rs/iced) v0.13 (Elm architecture) |
| Async           | [Tokio](https://tokio.rs/)                                       |
| Serialisation   | [Serde](https://serde.rs/)                                       |
| Package manager | winget CLI                                                       |
| Build           | Cargo with LTO + symbol stripping                                |

## Testing

```bash
cargo test            # all tests (unit + integration)
cargo test --lib      # unit tests only
cargo test --test integration_test  # integration tests only
```

## Roadmap

- [ ] Show release date of latest update
- [ ] Settings panel
- [ ] Automatic update scheduling
- [ ] System-tray notifications

## License

GPL-3.0 - see [LICENSE](LICENSE) for details.
