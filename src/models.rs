use serde::{Deserialize, Serialize};

/// Represents an application that has an available update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdatableApp {
    pub name: String,
    pub id: String,
    pub version: String,
    pub available: String,
    pub source: String,
}

impl UpdatableApp {
    /// Creates a new UpdatableApp
    pub fn new(name: String, id: String, version: String, available: String, source: String) -> Self {
        Self {
            name,
            id,
            version,
            available,
            source,
        }
    }
}

/// Represents an app item in the UI with selection state
#[derive(Debug, Clone)]
pub struct AppItem {
    pub app: UpdatableApp,
    pub selected: bool,
}

impl AppItem {
    /// Creates a new AppItem with the given app and selection state
    pub fn new(app: UpdatableApp, selected: bool) -> Self {
        Self { app, selected }
    }

    /// Creates a new unselected AppItem
    pub fn from_app(app: UpdatableApp) -> Self {
        Self::new(app, false)
    }
}

/// Messages that can be sent in the application
#[derive(Debug, Clone)]
pub enum Message {
    /// Load the list of updatable apps
    LoadApps,
    /// Apps have been loaded (success or error)
    AppsLoaded(Result<Vec<UpdatableApp>, String>),
    /// Toggle selection of an app at the given index
    ToggleApp(usize),
    /// Start updating selected apps
    UpdateSelected,
    /// Update process has completed with results
    UpdateComplete(Vec<String>),
    /// Select all apps
    SelectAll,
    /// Deselect all apps
    DeselectAll,
    /// Confirm the update after showing confirmation dialog
    ConfirmUpdate,
    /// Cancel the update
    CancelUpdate,
    /// Close the results dialog
    CloseResultsDialog,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_updatable_app_creation() {
        let app = UpdatableApp::new(
            "Test App".to_string(),
            "Test.App".to_string(),
            "1.0.0".to_string(),
            "1.0.1".to_string(),
            "winget".to_string(),
        );

        assert_eq!(app.name, "Test App");
        assert_eq!(app.id, "Test.App");
        assert_eq!(app.version, "1.0.0");
        assert_eq!(app.available, "1.0.1");
        assert_eq!(app.source, "winget");
    }

    #[test]
    fn test_app_item_creation() {
        let app = UpdatableApp::new(
            "Test".to_string(),
            "Test.App".to_string(),
            "1.0".to_string(),
            "2.0".to_string(),
            "winget".to_string(),
        );

        let item = AppItem::from_app(app.clone());
        assert!(!item.selected);
        assert_eq!(item.app.name, "Test");

        let selected_item = AppItem::new(app, true);
        assert!(selected_item.selected);
    }
}

