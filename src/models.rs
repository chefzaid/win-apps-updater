use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents an application that has an available update.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdatableApp {
    pub name: String,
    pub id: String,
    pub version: String,
    pub available: String,
    pub source: String,
}

impl UpdatableApp {
    /// Creates a new `UpdatableApp` instance.
    pub fn new(
        name: String,
        id: String,
        version: String,
        available: String,
        source: String,
    ) -> Self {
        Self {
            name,
            id,
            version,
            available,
            source,
        }
    }
}

impl fmt::Display for UpdatableApp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) {} → {}",
            self.name, self.id, self.version, self.available
        )
    }
}

/// Wraps an `UpdatableApp` with UI selection state.
#[derive(Debug, Clone)]
pub struct AppItem {
    pub app: UpdatableApp,
    pub selected: bool,
}

impl AppItem {
    /// Creates a new `AppItem` with the given selection state.
    pub fn new(app: UpdatableApp, selected: bool) -> Self {
        Self { app, selected }
    }

    /// Creates a new unselected `AppItem`.
    pub fn from_app(app: UpdatableApp) -> Self {
        Self::new(app, false)
    }

    /// Returns `true` if this item matches the given search query (case-insensitive).
    pub fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let q = query.to_lowercase();
        self.app.name.to_lowercase().contains(&q) || self.app.id.to_lowercase().contains(&q)
    }
}

impl From<UpdatableApp> for AppItem {
    fn from(app: UpdatableApp) -> Self {
        Self::from_app(app)
    }
}

/// All application messages / events.
#[derive(Debug, Clone)]
pub enum Message {
    /// Trigger loading the list of updatable apps.
    LoadApps,
    /// Apps have been loaded (success or error).
    AppsLoaded(Result<Vec<UpdatableApp>, String>),
    /// Toggle selection of an app at the given index.
    ToggleApp(usize),
    /// Initiate updating selected apps (shows confirmation).
    UpdateSelected,
    /// A single app update completed with its result string.
    UpdateSingleComplete(String),
    /// Select all visible apps.
    SelectAll,
    /// Deselect all visible apps.
    DeselectAll,
    /// Confirm the update after showing the confirmation dialog.
    ConfirmUpdate,
    /// Cancel the pending update.
    CancelUpdate,
    /// Close the results dialog.
    CloseResultsDialog,
    /// Search / filter text changed.
    SearchChanged(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_updatable_app_creation() {
        let app = UpdatableApp::new(
            "Test App".into(),
            "Test.App".into(),
            "1.0.0".into(),
            "1.0.1".into(),
            "winget".into(),
        );

        assert_eq!(app.name, "Test App");
        assert_eq!(app.id, "Test.App");
        assert_eq!(app.version, "1.0.0");
        assert_eq!(app.available, "1.0.1");
        assert_eq!(app.source, "winget");
    }

    #[test]
    fn test_updatable_app_display() {
        let app = UpdatableApp::new(
            "Firefox".into(),
            "Mozilla.Firefox".into(),
            "120.0".into(),
            "121.0".into(),
            "winget".into(),
        );
        let display = format!("{app}");
        assert!(display.contains("Firefox"));
        assert!(display.contains("Mozilla.Firefox"));
        assert!(display.contains("120.0"));
        assert!(display.contains("121.0"));
    }

    #[test]
    fn test_app_item_creation() {
        let app = UpdatableApp::new(
            "Test".into(),
            "Test.App".into(),
            "1.0".into(),
            "2.0".into(),
            "winget".into(),
        );

        let item = AppItem::from_app(app.clone());
        assert!(!item.selected);
        assert_eq!(item.app.name, "Test");

        let selected_item = AppItem::new(app, true);
        assert!(selected_item.selected);
    }

    #[test]
    fn test_app_item_from_trait() {
        let app = UpdatableApp::new(
            "Test".into(),
            "Test.App".into(),
            "1.0".into(),
            "2.0".into(),
            "winget".into(),
        );
        let item: AppItem = app.into();
        assert!(!item.selected);
    }

    #[test]
    fn test_matches_search_by_name() {
        let item = AppItem::from_app(UpdatableApp::new(
            "Visual Studio Code".into(),
            "Microsoft.VSCode".into(),
            "1.0".into(),
            "2.0".into(),
            "winget".into(),
        ));
        assert!(item.matches_search("visual"));
        assert!(item.matches_search("studio"));
        assert!(item.matches_search("VISUAL"));
        assert!(!item.matches_search("firefox"));
    }

    #[test]
    fn test_matches_search_by_id() {
        let item = AppItem::from_app(UpdatableApp::new(
            "Visual Studio Code".into(),
            "Microsoft.VSCode".into(),
            "1.0".into(),
            "2.0".into(),
            "winget".into(),
        ));
        assert!(item.matches_search("Microsoft"));
        assert!(item.matches_search("vscode"));
    }

    #[test]
    fn test_matches_search_empty_query() {
        let item = AppItem::from_app(UpdatableApp::new(
            "Test".into(),
            "Test.App".into(),
            "1.0".into(),
            "2.0".into(),
            "winget".into(),
        ));
        assert!(item.matches_search(""));
    }
}

