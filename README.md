# Windows Apps Updater

A modern, lightweight GUI application for Windows that simplifies application management and updates using `winget` (Windows Package Manager).

## ✨ Features

- 🎨 **Modern UI**: Clean, dark-themed interface with styled buttons, alternating rows, and proper overlay dialogs
- 📋 **List Updatable Apps**: Detects all applications that have updates available via winget
- ✅ **Selective Updates**: Choose which applications to update using checkboxes
- ⚡ **Batch Updates**: Update multiple applications simultaneously
- 🔍 **Search & Filter**: Instantly filter the app list by name or ID

## Building from Source

### Requirements

- Windows 11 or later
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### Build Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/chefzaid/WinAppsUpdater.git
   cd WinAppsUpdater
   ```

2. Build the release version:
   ```bash
   cargo build --release
   ```

3. The executable will be located at:
   ```
   target/release/WinAppsUpdater.exe
   ```

## How It Works

The application uses the `winget upgrade` command to:
- Detect applications with available updates
- Parse the column-aligned output to accurately extract application names (even multi-word), IDs, versions, and sources
- Execute `winget upgrade --id <app-id>` for each selected application

## 🛠️ Technology Stack

- **Language**: Rust (2021 Edition)
- **GUI Framework**: [Iced](https://github.com/iced-rs/iced) v0.13 (Elm-inspired architecture)
- **Async Runtime**: [Tokio](https://tokio.rs/) (for async operations)
- **Serialization**: [Serde](https://serde.rs/) (for data structures)
- **Package Manager Integration**: winget CLI
- **Build System**: Cargo with release optimizations (LTO, strip)

## 📁 Project Structure

The project follows Rust best practices with a clean, modular architecture:

```
WinAppsUpdater/
├── src/
│   ├── main.rs              # Application entry point (22 lines)
│   ├── lib.rs               # Library exports for testing
│   ├── models.rs            # Data structures (UpdatableApp, AppItem, Message)
│   ├── app.rs               # Application state and business logic
│   ├── winget.rs            # Winget CLI integration and parsing
│   └── ui/
│       ├── mod.rs           # UI module exports
│       ├── components.rs    # UI view builders and components
│       └── icon.rs          # Application icon generation
├── tests/
│   └── integration_test.rs  # Integration tests
├── Cargo.toml               # Project configuration and dependencies
├── build.rs                 # Build script for embedding icon
├── icon.ico                 # Application icon file
└── README.md                # This file
```

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test
```

## 🚀 Future Enhancements

- [x] Add search/filter functionality
- [ ] Show the release date of latest update
- [ ] Show a progress bar for each app during updates
- [ ] Add settings panel for customization
- [ ] Add automatic update scheduling
- [ ] Add notifications for available updates

## 📝 License

GPL-3.0 License (see LICENSE file for details)
