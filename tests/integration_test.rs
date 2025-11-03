use win_apps_updater::models::{AppItem, UpdatableApp};

#[test]
fn test_updatable_app_equality() {
    let app1 = UpdatableApp::new(
        "Test App".to_string(),
        "Test.App".to_string(),
        "1.0.0".to_string(),
        "1.0.1".to_string(),
        "winget".to_string(),
    );

    let app2 = UpdatableApp::new(
        "Test App".to_string(),
        "Test.App".to_string(),
        "1.0.0".to_string(),
        "1.0.1".to_string(),
        "winget".to_string(),
    );

    assert_eq!(app1, app2);
}

#[test]
fn test_app_item_selection() {
    let app = UpdatableApp::new(
        "Test".to_string(),
        "Test.App".to_string(),
        "1.0".to_string(),
        "2.0".to_string(),
        "winget".to_string(),
    );

    let mut item = AppItem::from_app(app);
    assert!(!item.selected);

    item.selected = true;
    assert!(item.selected);
}

