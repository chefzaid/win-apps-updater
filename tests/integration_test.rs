use win_apps_updater::models::{AppItem, UpdatableApp};
use win_apps_updater::winget::parse_winget_output;

// ── Model tests ──────────────────────────────────────────────────────

#[test]
fn test_updatable_app_equality() {
    let app1 = UpdatableApp::new(
        "Test App".into(),
        "Test.App".into(),
        "1.0.0".into(),
        "1.0.1".into(),
        "winget".into(),
    );

    let app2 = UpdatableApp::new(
        "Test App".into(),
        "Test.App".into(),
        "1.0.0".into(),
        "1.0.1".into(),
        "winget".into(),
    );

    assert_eq!(app1, app2);
}

#[test]
fn test_updatable_app_inequality() {
    let app1 = UpdatableApp::new(
        "App A".into(),
        "A.A".into(),
        "1.0".into(),
        "2.0".into(),
        "winget".into(),
    );
    let app2 = UpdatableApp::new(
        "App B".into(),
        "B.B".into(),
        "1.0".into(),
        "2.0".into(),
        "winget".into(),
    );
    assert_ne!(app1, app2);
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
    let s = format!("{app}");
    assert!(s.contains("Firefox"));
    assert!(s.contains("Mozilla.Firefox"));
    assert!(s.contains("120.0"));
    assert!(s.contains("121.0"));
}

#[test]
fn test_app_item_selection() {
    let app = UpdatableApp::new(
        "Test".into(),
        "Test.App".into(),
        "1.0".into(),
        "2.0".into(),
        "winget".into(),
    );

    let mut item = AppItem::from_app(app);
    assert!(!item.selected);

    item.selected = true;
    assert!(item.selected);
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

// ── Search / filter tests ────────────────────────────────────────────

#[test]
fn test_matches_search_name_case_insensitive() {
    let item = AppItem::from_app(UpdatableApp::new(
        "Visual Studio Code".into(),
        "Microsoft.VSCode".into(),
        "1.0".into(),
        "2.0".into(),
        "winget".into(),
    ));
    assert!(item.matches_search("visual"));
    assert!(item.matches_search("VISUAL"));
    assert!(item.matches_search("Studio Code"));
}

#[test]
fn test_matches_search_by_id() {
    let item = AppItem::from_app(UpdatableApp::new(
        "Chrome".into(),
        "Google.Chrome".into(),
        "1.0".into(),
        "2.0".into(),
        "winget".into(),
    ));
    assert!(item.matches_search("Google"));
    assert!(item.matches_search("google.chrome"));
}

#[test]
fn test_matches_search_empty_always_true() {
    let item = AppItem::from_app(UpdatableApp::new(
        "Any".into(),
        "Any.App".into(),
        "1.0".into(),
        "2.0".into(),
        "winget".into(),
    ));
    assert!(item.matches_search(""));
}

#[test]
fn test_matches_search_no_match() {
    let item = AppItem::from_app(UpdatableApp::new(
        "Firefox".into(),
        "Mozilla.Firefox".into(),
        "1.0".into(),
        "2.0".into(),
        "winget".into(),
    ));
    assert!(!item.matches_search("chrome"));
}

// ── Winget parsing tests ─────────────────────────────────────────────

#[test]
fn test_parse_winget_empty() {
    let result = parse_winget_output("");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_parse_winget_no_updates_text() {
    let result = parse_winget_output("No applicable upgrade found.\n");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_parse_winget_typical_output() {
    let output = "\
Name                           Id                          Version        Available      Source
-------------------------------------------------------------------------------------------------
Microsoft Visual Studio Code   Microsoft.VisualStudioCode  1.85.0         1.85.1         winget
Google Chrome                  Google.Chrome               120.0.6099.109 120.0.6099.130 winget
2 upgrades available.";

    let apps = parse_winget_output(output).unwrap();
    assert_eq!(apps.len(), 2);

    assert_eq!(apps[0].name, "Microsoft Visual Studio Code");
    assert_eq!(apps[0].id, "Microsoft.VisualStudioCode");
    assert_eq!(apps[0].version, "1.85.0");
    assert_eq!(apps[0].available, "1.85.1");
    assert_eq!(apps[0].source, "winget");

    assert_eq!(apps[1].name, "Google Chrome");
    assert_eq!(apps[1].id, "Google.Chrome");
}

#[test]
fn test_parse_winget_multi_word_app_name() {
    let output = "\
Name                            Id                         Version   Available   Source
--------------------------------------------------------------------------------------
Microsoft Windows Terminal      Microsoft.WindowsTerminal  1.18.0    1.19.0      winget
";
    let apps = parse_winget_output(output).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "Microsoft Windows Terminal");
    assert_eq!(apps[0].id, "Microsoft.WindowsTerminal");
    assert_eq!(apps[0].version, "1.18.0");
    assert_eq!(apps[0].available, "1.19.0");
}

#[test]
fn test_parse_winget_single_app() {
    let output = "\
Name       Id          Version  Available  Source
-------------------------------------------------
Notepad++  Notepad++.N  8.5.0   8.6.0     winget
1 upgrades available.";

    let apps = parse_winget_output(output).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "Notepad++");
}

#[test]
fn test_parse_winget_stops_at_footer() {
    let output = "\
Name    Id       Version  Available  Source
-------------------------------------------
App     Test.App 1.0      2.0        winget
1 upgrades available.
Some trailing text
";
    let apps = parse_winget_output(output).unwrap();
    assert_eq!(apps.len(), 1);
}

// ── Serialization round-trip ─────────────────────────────────────────

#[test]
fn test_updatable_app_serde_round_trip() {
    let app = UpdatableApp::new(
        "Test App".into(),
        "Test.App".into(),
        "1.0.0".into(),
        "1.0.1".into(),
        "winget".into(),
    );

    let json = serde_json::to_string(&app).unwrap();
    let deserialized: UpdatableApp = serde_json::from_str(&json).unwrap();
    assert_eq!(app, deserialized);
}

// ── AppState tests (via public API) ──────────────────────────────────

#[test]
fn test_app_state_default_values() {
    let state = win_apps_updater::app::AppState::default();
    assert!(state.apps.is_empty());
    assert!(state.loading);
    assert!(!state.updating);
    assert!(!state.show_confirmation);
    assert!(!state.show_results_dialog);
    assert!(state.search_query.is_empty());
    assert!(state.pending_updates.is_empty());
    assert!(state.update_results.is_empty());
}

#[test]
fn test_app_state_selected_count() {
    let mut state = win_apps_updater::app::AppState::default();
    state.apps.push(AppItem::new(
        UpdatableApp::new("A".into(), "A".into(), "1".into(), "2".into(), "w".into()),
        true,
    ));
    state.apps.push(AppItem::new(
        UpdatableApp::new("B".into(), "B".into(), "1".into(), "2".into(), "w".into()),
        false,
    ));
    assert_eq!(state.selected_count(), 1);
}

#[test]
fn test_app_state_visible_indices() {
    let mut state = win_apps_updater::app::AppState::default();
    state.apps.push(AppItem::from_app(UpdatableApp::new(
        "Firefox".into(),
        "Mozilla.Firefox".into(),
        "1".into(),
        "2".into(),
        "w".into(),
    )));
    state.apps.push(AppItem::from_app(UpdatableApp::new(
        "Chrome".into(),
        "Google.Chrome".into(),
        "1".into(),
        "2".into(),
        "w".into(),
    )));

    // No filter — all visible
    assert_eq!(state.visible_indices(), vec![0, 1]);

    // Filter to Firefox only
    state.search_query = "fire".into();
    assert_eq!(state.visible_indices(), vec![0]);

    // No match
    state.search_query = "edge".into();
    assert!(state.visible_indices().is_empty());
}

// ── Wrapped winget output tests ──────────────────────────────────────

#[test]
fn test_parse_wrapped_winget_output() {
    // Simulate winget output wrapped at 80 columns.
    // Column positions: Name=0, Id=35, Version=74, Available=91, Source=108
    let header = format!(
        "{:<35}{:<39}{:<17}{:<17}{}",
        "Name", "Id", "Version", "Available", "Source"
    );
    let sep = "-".repeat(114);
    let data = format!(
        "{:<35}{:<39}{:<17}{:<17}{}",
        "Microsoft Visual Studio Code",
        "Microsoft.VisualStudioCode",
        "1.85.0",
        "1.85.1",
        "winget"
    );
    let footer = "1 upgrades available.";

    // Wrap all wide lines at 80 chars
    let wrap = |s: &str| -> String {
        if s.len() <= 80 {
            s.to_string()
        } else {
            format!("{}\n{}", &s[..80], &s[80..])
        }
    };

    let output = format!(
        "{}\n{}\n{}\n{footer}",
        wrap(&header),
        wrap(&sep),
        wrap(&data),
    );

    let apps = parse_winget_output(&output).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "Microsoft Visual Studio Code");
    assert_eq!(apps[0].id, "Microsoft.VisualStudioCode");
    assert_eq!(apps[0].version, "1.85.0");
    assert_eq!(apps[0].available, "1.85.1");
    assert!(apps[0].source.contains("winget"));
}

