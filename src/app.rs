use crate::models::{AppItem, Message, UpdatableApp};
use crate::ui::build_view;
use crate::winget::{get_updatable_apps, update_single_app};
use iced::{Element, Task};

/// Main application state.
pub struct AppState {
    /// All loaded app items.
    pub apps: Vec<AppItem>,
    /// Whether the app list is currently loading.
    pub loading: bool,
    /// Status message shown to the user.
    pub status_message: String,
    /// Whether updates are currently running.
    pub updating: bool,
    /// Whether the confirmation dialog is visible.
    pub show_confirmation: bool,
    /// Apps pending update as `(name, id)` pairs.
    pub pending_updates: Vec<(String, String)>,
    /// Whether the results dialog is visible.
    pub show_results_dialog: bool,
    /// Per-app update result strings.
    pub update_results: Vec<String>,
    /// Current search / filter query.
    pub search_query: String,
    /// Total number of apps to update in the current batch.
    pub update_total: usize,
    /// Number of apps updated so far in the current batch.
    pub update_completed: usize,
    /// All app IDs queued for the current update batch (`update_completed` is the cursor).
    pub update_queue: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            apps: Vec::new(),
            loading: true,
            status_message: String::from("Loading updatable apps..."),
            updating: false,
            show_confirmation: false,
            pending_updates: Vec::new(),
            show_results_dialog: false,
            update_results: Vec::new(),
            search_query: String::new(),
            update_total: 0,
            update_completed: 0,
            update_queue: Vec::new(),
        }
    }
}

impl AppState {
    /// Creates a new `AppState` and returns it with an initial load task.
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(async { get_updatable_apps() }, Message::AppsLoaded),
        )
    }

    /// Returns the count of currently selected apps.
    pub fn selected_count(&self) -> usize {
        self.apps.iter().filter(|a| a.selected).count()
    }

    /// Returns indices of apps visible after filtering by the search query.
    pub fn visible_indices(&self) -> Vec<usize> {
        self.apps
            .iter()
            .enumerate()
            .filter(|(_, item)| item.matches_search(&self.search_query))
            .map(|(i, _)| i)
            .collect()
    }

    /// Updates the application state based on the given message.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadApps => self.handle_load_apps(),
            Message::AppsLoaded(result) => self.handle_apps_loaded(result),
            Message::ToggleApp(index) => self.handle_toggle_app(index),
            Message::UpdateSelected => self.handle_update_selected(),
            Message::UpdateSingleComplete(result) => self.handle_update_single_complete(result),
            Message::SelectAll => self.handle_select_all(),
            Message::DeselectAll => self.handle_deselect_all(),
            Message::ConfirmUpdate => self.handle_confirm_update(),
            Message::CancelUpdate => self.handle_cancel_update(),
            Message::CloseResultsDialog => self.handle_close_results_dialog(),
            Message::SearchChanged(query) => self.handle_search_changed(query),
        }
    }

    /// Renders the view.
    pub fn view(&self) -> Element<'_, Message> {
        build_view(self)
    }

    // ── Message handlers ─────────────────────────────────────────────

    fn handle_load_apps(&mut self) -> Task<Message> {
        self.loading = true;
        self.search_query.clear();
        self.status_message = String::from("Loading updatable apps...");
        Task::perform(async { get_updatable_apps() }, Message::AppsLoaded)
    }

    fn handle_apps_loaded(&mut self, result: Result<Vec<UpdatableApp>, String>) -> Task<Message> {
        self.loading = false;
        match result {
            Ok(apps) => {
                let count = apps.len();
                self.apps = apps.into_iter().map(AppItem::from).collect();
                self.status_message = format!("{count} app(s) available for update");
            }
            Err(e) => {
                self.status_message = format!("Error: {e}");
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
        let selected: Vec<(String, String)> = self
            .apps
            .iter()
            .filter(|a| a.selected)
            .map(|a| (a.app.name.clone(), a.app.id.clone()))
            .collect();

        if selected.is_empty() {
            self.status_message = String::from("No apps selected");
            return Task::none();
        }

        self.pending_updates = selected;
        self.show_confirmation = true;
        Task::none()
    }

    fn handle_confirm_update(&mut self) -> Task<Message> {
        self.show_confirmation = false;
        self.update_queue = self
            .pending_updates
            .iter()
            .map(|(_, id)| id.clone())
            .collect();
        self.update_total = self.update_queue.len();
        self.update_completed = 0;
        self.update_results.clear();
        self.updating = true;

        if self.update_queue.is_empty() {
            return Task::none();
        }

        let id = self.update_queue[0].clone();
        self.status_message = format!("Updating 0/{} app(s)...", self.update_total);
        Task::perform(
            async move {
                match update_single_app(&id) {
                    Ok(msg) | Err(msg) => msg,
                }
            },
            Message::UpdateSingleComplete,
        )
    }

    fn handle_cancel_update(&mut self) -> Task<Message> {
        self.show_confirmation = false;
        self.pending_updates.clear();
        self.status_message = String::from("Update cancelled");
        Task::none()
    }

    fn handle_update_single_complete(&mut self, result: String) -> Task<Message> {
        self.update_results.push(result);
        self.update_completed += 1;

        if self.update_completed < self.update_total {
            let id = self.update_queue[self.update_completed].clone();
            self.status_message = format!(
                "Updating {}/{} app(s)...",
                self.update_completed, self.update_total
            );
            return Task::perform(
                async move {
                    match update_single_app(&id) {
                        Ok(msg) | Err(msg) => msg,
                    }
                },
                Message::UpdateSingleComplete,
            );
        }

        // All updates finished.
        self.updating = false;
        self.show_results_dialog = true;
        self.status_message = String::from("Update complete");

        for item in &mut self.apps {
            item.selected = false;
        }

        // Auto-refresh the list
        self.loading = true;
        Task::perform(async { get_updatable_apps() }, Message::AppsLoaded)
    }

    fn handle_select_all(&mut self) -> Task<Message> {
        let visible = self.visible_indices();
        for idx in visible {
            self.apps[idx].selected = true;
        }
        Task::none()
    }

    fn handle_deselect_all(&mut self) -> Task<Message> {
        let visible = self.visible_indices();
        for idx in visible {
            self.apps[idx].selected = false;
        }
        Task::none()
    }

    fn handle_close_results_dialog(&mut self) -> Task<Message> {
        self.show_results_dialog = false;
        Task::none()
    }

    fn handle_search_changed(&mut self, query: String) -> Task<Message> {
        self.search_query = query;
        Task::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_app(name: &str, id: &str) -> UpdatableApp {
        UpdatableApp::new(
            name.into(),
            id.into(),
            "1.0".into(),
            "2.0".into(),
            "winget".into(),
        )
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.apps.is_empty());
        assert!(state.loading);
        assert!(!state.updating);
        assert!(!state.show_confirmation);
        assert!(!state.show_results_dialog);
        assert!(state.search_query.is_empty());
    }

    #[test]
    fn test_selected_count() {
        let mut state = AppState::default();
        state.apps.push(AppItem::new(sample_app("A", "A.A"), true));
        state
            .apps
            .push(AppItem::new(sample_app("B", "B.B"), false));
        state.apps.push(AppItem::new(sample_app("C", "C.C"), true));
        assert_eq!(state.selected_count(), 2);
    }

    #[test]
    fn test_visible_indices_with_search() {
        let mut state = AppState::default();
        state
            .apps
            .push(AppItem::from_app(sample_app("Firefox", "Mozilla.Firefox")));
        state
            .apps
            .push(AppItem::from_app(sample_app("Chrome", "Google.Chrome")));
        state
            .apps
            .push(AppItem::from_app(sample_app("Edge", "Microsoft.Edge")));

        state.search_query = "fire".into();
        assert_eq!(state.visible_indices(), vec![0]);

        state.search_query.clear();
        assert_eq!(state.visible_indices(), vec![0, 1, 2]);
    }

    #[test]
    fn test_handle_select_all() {
        let mut state = AppState::default();
        state
            .apps
            .push(AppItem::from_app(sample_app("A", "A.A")));
        state
            .apps
            .push(AppItem::from_app(sample_app("B", "B.B")));

        let _ = state.handle_select_all();
        assert!(state.apps.iter().all(|a| a.selected));
    }

    #[test]
    fn test_handle_select_all_respects_filter() {
        let mut state = AppState::default();
        state
            .apps
            .push(AppItem::from_app(sample_app("Firefox", "Mozilla.Firefox")));
        state
            .apps
            .push(AppItem::from_app(sample_app("Chrome", "Google.Chrome")));

        state.search_query = "fire".into();
        let _ = state.handle_select_all();
        assert!(state.apps[0].selected);
        assert!(!state.apps[1].selected); // Chrome not visible
    }

    #[test]
    fn test_handle_deselect_all() {
        let mut state = AppState::default();
        state.apps.push(AppItem::new(sample_app("A", "A.A"), true));

        let _ = state.handle_deselect_all();
        assert!(!state.apps[0].selected);
    }

    #[test]
    fn test_handle_toggle_app() {
        let mut state = AppState::default();
        state
            .apps
            .push(AppItem::from_app(sample_app("A", "A.A")));

        let _ = state.handle_toggle_app(0);
        assert!(state.apps[0].selected);

        let _ = state.handle_toggle_app(0);
        assert!(!state.apps[0].selected);
    }

    #[test]
    fn test_handle_toggle_app_out_of_bounds() {
        let mut state = AppState::default();
        // Should not panic
        let _ = state.handle_toggle_app(99);
    }

    #[test]
    fn test_handle_update_selected_empty() {
        let mut state = AppState::default();
        state.loading = false;
        let _ = state.handle_update_selected();
        assert_eq!(state.status_message, "No apps selected");
        assert!(!state.show_confirmation);
    }

    #[test]
    fn test_handle_update_selected_shows_confirmation() {
        let mut state = AppState::default();
        state
            .apps
            .push(AppItem::new(sample_app("My App", "My.App"), true));

        let _ = state.handle_update_selected();
        assert!(state.show_confirmation);
        assert_eq!(state.pending_updates.len(), 1);
        assert_eq!(
            state.pending_updates[0],
            ("My App".into(), "My.App".into())
        );
    }

    #[test]
    fn test_handle_cancel_update() {
        let mut state = AppState::default();
        state.show_confirmation = true;
        state
            .pending_updates
            .push(("A".into(), "A.Id".into()));

        let _ = state.handle_cancel_update();
        assert!(!state.show_confirmation);
        assert!(state.pending_updates.is_empty());
        assert_eq!(state.status_message, "Update cancelled");
    }

    #[test]
    fn test_handle_close_results_dialog() {
        let mut state = AppState::default();
        state.show_results_dialog = true;
        let _ = state.handle_close_results_dialog();
        assert!(!state.show_results_dialog);
    }

    #[test]
    fn test_handle_search_changed() {
        let mut state = AppState::default();
        let _ = state.handle_search_changed("firefox".into());
        assert_eq!(state.search_query, "firefox");
    }

    #[test]
    fn test_handle_load_apps_resets_state() {
        let mut state = AppState::default();
        state.loading = false;
        state.search_query = "test".into();
        let _ = state.handle_load_apps();
        assert!(state.loading);
        assert!(state.search_query.is_empty());
        assert_eq!(state.status_message, "Loading updatable apps...");
    }

    #[test]
    fn test_handle_apps_loaded_ok() {
        let mut state = AppState::default();
        let apps = vec![
            sample_app("Firefox", "Mozilla.Firefox"),
            sample_app("Chrome", "Google.Chrome"),
        ];
        let _ = state.handle_apps_loaded(Ok(apps));
        assert!(!state.loading);
        assert_eq!(state.apps.len(), 2);
        assert!(state.status_message.contains("2"));
    }

    #[test]
    fn test_handle_apps_loaded_err() {
        let mut state = AppState::default();
        let _ = state.handle_apps_loaded(Err("Connection failed".into()));
        assert!(!state.loading);
        assert!(state.apps.is_empty());
        assert!(state.status_message.contains("Error"));
        assert!(state.status_message.contains("Connection failed"));
    }

    #[test]
    fn test_handle_apps_loaded_empty() {
        let mut state = AppState::default();
        let _ = state.handle_apps_loaded(Ok(vec![]));
        assert!(!state.loading);
        assert!(state.apps.is_empty());
        assert!(state.status_message.contains("0"));
    }

    #[test]
    fn test_handle_confirm_update_sets_progress() {
        let mut state = AppState::default();
        state
            .pending_updates
            .push(("App A".into(), "A.App".into()));
        state
            .pending_updates
            .push(("App B".into(), "B.App".into()));

        let _ = state.handle_confirm_update();
        assert!(!state.show_confirmation);
        assert!(state.updating);
        assert_eq!(state.update_total, 2);
        assert_eq!(state.update_completed, 0);
        assert_eq!(state.update_queue, vec!["A.App", "B.App"]);
        assert!(state.status_message.contains("0/2"));
    }

    #[test]
    fn test_handle_confirm_update_empty_queue() {
        let mut state = AppState::default();
        // pending_updates is empty
        let _ = state.handle_confirm_update();
        assert!(!state.show_confirmation);
        assert_eq!(state.update_total, 0);
    }

    #[test]
    fn test_handle_update_single_complete_continues() {
        let mut state = AppState::default();
        state.updating = true;
        state.update_total = 3;
        state.update_completed = 0;
        state.update_queue = vec!["A.App".into(), "B.App".into(), "C.App".into()];

        let _ = state.handle_update_single_complete("SUCCESS:A.App - done".into());

        assert_eq!(state.update_completed, 1);
        assert!(state.updating);
        assert!(!state.show_results_dialog);
        assert_eq!(state.update_results.len(), 1);
        assert!(state.status_message.contains("1/3"));
    }

    #[test]
    fn test_handle_update_single_complete_finishes() {
        let mut state = AppState::default();
        state.updating = true;
        state.update_total = 1;
        state.update_completed = 0;
        state.update_queue = vec!["A.App".into()];
        state.apps.push(AppItem::new(sample_app("A", "A.App"), true));

        let _ = state.handle_update_single_complete("SUCCESS:A.App - done".into());

        assert_eq!(state.update_completed, 1);
        assert!(!state.updating);
        assert!(state.show_results_dialog);
        assert!(!state.apps[0].selected);
        assert_eq!(state.update_results, vec!["SUCCESS:A.App - done"]);
        assert_eq!(state.status_message, "Update complete");
    }

    #[test]
    fn test_handle_update_single_complete_second_of_two() {
        let mut state = AppState::default();
        state.updating = true;
        state.update_total = 2;
        state.update_completed = 1; // first already done
        state.update_results = vec!["SUCCESS:A - done".into()];
        state.update_queue = vec!["A.App".into(), "B.App".into()];
        state.apps.push(AppItem::new(sample_app("A", "A.App"), true));
        state.apps.push(AppItem::new(sample_app("B", "B.App"), true));

        let _ = state.handle_update_single_complete("FAILURE:B - err".into());

        assert_eq!(state.update_completed, 2);
        assert!(!state.updating);
        assert!(state.show_results_dialog);
        assert_eq!(state.update_results.len(), 2);
    }

    #[test]
    fn test_progress_defaults() {
        let state = AppState::default();
        assert_eq!(state.update_total, 0);
        assert_eq!(state.update_completed, 0);
        assert!(state.update_queue.is_empty());
    }
}

