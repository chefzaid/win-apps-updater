# Windows Apps Updater

A modern, lightweight GUI application for Windows that simplifies application management and updates using `winget` (Windows Package Manager).

## ✨ Features

- 📋 **List Updatable Apps**: Detects all applications that have updates available via winget
- ✅ **Selective Updates**: Choose which applications to update using checkboxes
- 🎨 **Modern UI**: Clean, dark-themed interface with intuitive layout
- ⚡ **Batch Updates**: Update multiple applications simultaneously

## Prerequisites

- Windows 11 or later

## Building from Source

### Requirements

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

## Usage

1. Run the application:
   ```bash
   cargo run --release
   ```
   Or double-click the `WinAppsUpdater.exe` file.

2. The application will automatically load the list of updatable applications on startup.
   - You can click **Refresh** at any time to reload the list

3. Select the applications you want to update by checking the boxes next to them.
   - Use **Select All** to select all applications
   - Use **Deselect All** to clear all selections

4. Click the **Update Selected** button to update the chosen applications.
   - A confirmation dialog will appear listing the apps to be updated
   - Click **Yes, Proceed** to continue or **Cancel** to abort

5. Wait for the updates to complete. During updates:
   - All buttons and checkboxes are disabled to prevent conflicts
   - Status message shows "Updating X app(s)..."
   - Updates run silently in the background

6. When updates complete, a **Results Dialog** appears showing:
   - **Green text**: Successful updates
   - **Red text**: Failed updates
   - **Orange text** `[!]`: Warnings (apps that need to be closed)
   - **Light blue text** `[i]`: Info (apps already up to date)
   - Click the **X** button in the top-right corner to close the dialog

7. After successful updates, the list automatically refreshes to show remaining updates.

## How It Works

The application uses the `winget upgrade` command to:
- Detect applications with available updates
- Parse the output to display application names, current versions, and available versions
- Execute `winget upgrade --id <app-id>` for each selected application

## 🏗️ Architecture

This application follows the **Elm Architecture** pattern (Model-View-Update) as implemented by the Iced framework:

1. **Model** (`AppState`): Holds all application state
2. **Update** (`update` method): Handles messages and updates state
3. **View** (`view` method): Renders UI based on current state

### Key Design Principles

- **Separation of Concerns**: Each module has a single, well-defined responsibility
- **Immutability**: State changes only through the update function
- **Type Safety**: Leverages Rust's type system to prevent errors at compile time
- **Testability**: Pure functions and clear interfaces make testing straightforward
- **Message-Driven**: All user interactions and async operations communicate via messages

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

### Module Breakdown

- **`main.rs`**: Minimal entry point - only launches the application
- **`lib.rs`**: Exposes modules for testing and potential library use
- **`models.rs`**: Core data structures with unit tests
  - `UpdatableApp`: Represents an app with update information
  - `AppItem`: Wraps app with selection state
  - `Message`: All application messages/events
- **`app.rs`**: Application state management with unit tests
  - `AppState`: Main application state
  - Message handlers for all user interactions
  - Business logic for updates and state transitions
- **`winget.rs`**: Windows Package Manager integration with unit tests
  - Executes winget commands
  - Parses winget output
  - Handles update operations
- **`ui/`**: User interface components
  - `components.rs`: All view builders (dialogs, buttons, lists)
  - `icon.rs`: Programmatic icon generation with tests

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

## 📝 License

This project is open source and available under the MIT License.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Follow Rust best practices and idioms
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure `cargo test` passes
5. Ensure `cargo clippy` has no warnings
6. Format code with `cargo fmt`

## 🔧 Troubleshooting

### "winget command failed"
- Ensure winget is installed and accessible from the command line
- Try running `winget upgrade` manually in PowerShell to verify it works
- Check that winget is up to date: `winget --version`

### No updates showing
- Some applications may not be available in the winget repository
- Try running `winget upgrade --include-unknown` manually to see what winget detects
- Click the **Refresh** button to reload the list

### Application won't start
- Ensure you have the latest Visual C++ Redistributable installed
- Check that your Windows version is up to date
- Try running from command line to see any error messages: `.\WinAppsUpdater.exe`

### Update results dialog doesn't show
- Make sure you're clicking the **X** button in the top-right corner of the dialog
- The dialog appears after updates complete - look for a semi-transparent overlay

### App shows "needs to be closed" warning `[!]`
- Close the running application that needs to be updated
- Click **Update Selected** again to retry the update
- The app automatically detects when winget reports that an application must be closed before updating

### Compilation errors
- Ensure you have the latest stable Rust toolchain: `rustup update stable`
- Clean the build: `cargo clean` then rebuild
- Check that all dependencies are compatible: `cargo update`

## 🚀 Future Enhancements

- [ ] Show the release date of latest update
- [ ] Show a progress bar for each app during updates
- [ ] Add search/filter functionality
- [ ] Add settings panel for customization
- [ ] Add automatic update scheduling
- [ ] Add notifications for available updates
