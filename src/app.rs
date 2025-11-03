use crate::models::{AppItem, Message, UpdatableApp};
use crate::ui::build_view;
use crate::winget::{get_updatable_apps, update_single_app};
use iced::{Element, Task};

/// Main application state
pub struct AppState {
    pub apps: Vec<AppItem>,
    pub loading: bool,
    pub status_message: String,
    pub updating: bool,
    pub show_confirmation: bool,
    pub apps_needing_close: Vec<String>,
    pub pending_update_ids: Vec<String>,
    pub show_results_dialog: bool,
    pub update_results: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            apps: Vec::new(),
            loading: true,
            status_message: String::from("Loading updatable apps..."),
            updating: false,
            show_confirmation: false,
            apps_needing_close: Vec::new(),
            pending_update_ids: Vec::new(),
            show_results_dialog: false,
            update_results: Vec::new(),
        }
    }
}

impl AppState {
    /// Creates a new AppState and returns it with an initial task to load apps
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(async { get_updatable_apps() }, Message::AppsLoaded),
        )
    }

    /// Updates the application state based on the message
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadApps => self.handle_load_apps(),
            Message::AppsLoaded(result) => self.handle_apps_loaded(result),
            Message::ToggleApp(index) => self.handle_toggle_app(index),
            Message::UpdateSelected => self.handle_update_selected(),
            Message::UpdateComplete(results) => self.handle_update_complete(results),
            Message::SelectAll => self.handle_select_all(),
            Message::DeselectAll => self.handle_deselect_all(),
            Message::ConfirmUpdate => self.handle_confirm_update(),
            Message::CancelUpdate => self.handle_cancel_update(),
            Message::CloseResultsDialog => self.handle_close_results_dialog(),
        }
    }

    /// Renders the view
    pub fn view(&self) -> Element<'_, Message> {
        build_view(self)
    }

    fn handle_load_apps(&mut self) -> Task<Message> {
        self.loading = true;
        self.status_message = String::from("Loading updatable apps...");
        Task::perform(async { get_updatable_apps() }, Message::AppsLoaded)
    }

    fn handle_apps_loaded(&mut self, result: Result<Vec<UpdatableApp>, String>) -> Task<Message> {
        self.loading = false;
        match result {
            Ok(apps) => {
                self.apps = apps.into_iter().map(AppItem::from_app).collect();
                self.status_message = format!("{} app(s) available for update", self.apps.len());
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
        Task::none()
    }

    fn handle_toggle_app(&mut self, index: usize) -> Task<Message> {
        if let Some(app) = self.apps.get_mut(index) {
            app.selected = !app.selected;
        }
        Task::none()
    }

    fn handle_update_selected(&mut self) -> Task<Message> {
        let selected_ids: Vec<String> = self
            .apps
            .iter()
            .filter(|app| app.selected)
            .map(|app| app.app.id.clone())
            .collect();

        if selected_ids.is_empty() {
            self.status_message = String::from("No apps selected");
            return Task::none();
        }

        // Show confirmation dialog
        self.show_confirmation = true;
        self.apps_needing_close = selected_ids.clone();
        self.pending_update_ids = selected_ids;

        Task::none()
    }

    fn handle_confirm_update(&mut self) -> Task<Message> {
        self.show_confirmation = false;

        let selected_ids = self.pending_update_ids.clone();
        self.updating = true;
        self.status_message = format!("Updating {} app(s)...", selected_ids.len());

        Task::perform(
            update_apps_sequential(selected_ids),
            Message::UpdateComplete,
        )
    }

    fn handle_cancel_update(&mut self) -> Task<Message> {
        self.show_confirmation = false;
        self.pending_update_ids.clear();
        self.apps_needing_close.clear();
        self.status_message = String::from("Update cancelled");
        Task::none()
    }

    fn handle_update_complete(&mut self, results: Vec<String>) -> Task<Message> {
        self.updating = false;
        self.update_results = results;
        self.show_results_dialog = true;
        self.status_message = String::from("Update complete");

        // Deselect all apps
        for app_item in &mut self.apps {
            app_item.selected = false;
        }

        // Auto-refresh to update the list
        self.loading = true;
        Task::perform(async { get_updatable_apps() }, Message::AppsLoaded)
    }

    fn handle_select_all(&mut self) -> Task<Message> {
        for app in &mut self.apps {
            app.selected = true;
        }
        Task::none()
    }

    fn handle_deselect_all(&mut self) -> Task<Message> {
        for app in &mut self.apps {
            app.selected = false;
        }
        Task::none()
    }

    fn handle_close_results_dialog(&mut self) -> Task<Message> {
        self.show_results_dialog = false;
        Task::none()
    }
}

/// Updates apps sequentially and returns results
async fn update_apps_sequential(app_ids: Vec<String>) -> Vec<String> {
    let mut results = Vec::new();

    for app_id in app_ids.iter() {
        let result = update_single_app(app_id);
        results.push(match result {
            Ok(msg) => msg,
            Err(msg) => msg,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.apps.is_empty());
        assert!(state.loading);
        assert!(!state.updating);
        assert!(!state.show_confirmation);
        assert!(!state.show_results_dialog);
    }

    #[test]
    fn test_handle_select_all() {
        let mut state = AppState::default();
        let app = UpdatableApp::new(
            "Test".to_string(),
            "Test.App".to_string(),
            "1.0".to_string(),
            "2.0".to_string(),
            "winget".to_string(),
        );
        state.apps.push(AppItem::from_app(app));

        let _ = state.handle_select_all();
        assert!(state.apps[0].selected);
    }

    #[test]
    fn test_handle_deselect_all() {
        let mut state = AppState::default();
        let app = UpdatableApp::new(
            "Test".to_string(),
            "Test.App".to_string(),
            "1.0".to_string(),
            "2.0".to_string(),
            "winget".to_string(),
        );
        state.apps.push(AppItem::new(app, true));

        let _ = state.handle_deselect_all();
        assert!(!state.apps[0].selected);
    }

    #[test]
    fn test_handle_toggle_app() {
        let mut state = AppState::default();
        let app = UpdatableApp::new(
            "Test".to_string(),
            "Test.App".to_string(),
            "1.0".to_string(),
            "2.0".to_string(),
            "winget".to_string(),
        );
        state.apps.push(AppItem::from_app(app));

        let _ = state.handle_toggle_app(0);
        assert!(state.apps[0].selected);

        let _ = state.handle_toggle_app(0);
        assert!(!state.apps[0].selected);
    }
}

