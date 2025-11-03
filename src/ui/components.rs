use crate::app::AppState;
use crate::models::Message;
use iced::{
    widget::{button, checkbox, column, container, horizontal_rule, row, scrollable, text, Column},
    Alignment, Color, Element, Length,
};

/// Builds the main view for the application
pub fn build_view(state: &AppState) -> Element<'_, Message> {
    let title = text("Windows Apps Updater").size(32);

    let button_row = build_button_row(state);
    let status = build_status_text(state);
    let scrollable_list = build_app_list(state);

    let mut content = column![title, button_row, status, scrollable_list]
        .spacing(10)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    // Add confirmation dialog if needed
    if state.show_confirmation {
        content = content.push(build_confirmation_dialog(state));
    }

    // Add results dialog if needed
    if state.show_results_dialog {
        content = content.push(build_results_dialog(state));
    }

    content.into()
}

/// Builds the button row with Refresh, Select All, Deselect All, and Update buttons
fn build_button_row(state: &AppState) -> Element<'_, Message> {
    let refresh_button = create_button("Refresh", !state.updating, Message::LoadApps);
    let select_all_button = create_button("Select All", !state.updating, Message::SelectAll);
    let deselect_all_button =
        create_button("Deselect All", !state.updating, Message::DeselectAll);

    let update_button = if state.updating {
        button("Updating...").padding(10)
    } else {
        button("Update Selected")
            .on_press(Message::UpdateSelected)
            .padding(10)
    };

    row![
        refresh_button,
        select_all_button,
        deselect_all_button,
        update_button,
    ]
    .spacing(10)
    .padding(10)
    .into()
}

/// Creates a button with optional enabled state
fn create_button(label: &str, enabled: bool, message: Message) -> button::Button<'_, Message> {
    if enabled {
        button(label).on_press(message).padding(10)
    } else {
        button(label).padding(10)
    }
}

/// Builds the status text display
fn build_status_text(state: &AppState) -> Element<'_, Message> {
    text(&state.status_message)
        .size(14)
        .width(Length::Fill)
        .into()
}

/// Builds the scrollable list of apps
fn build_app_list(state: &AppState) -> Element<'_, Message> {
    let mut app_list = Column::new().spacing(5).padding(10);

    if state.loading {
        app_list = app_list.push(text("Loading..."));
    } else if state.apps.is_empty() {
        app_list = app_list.push(text("No apps to display"));
    } else {
        app_list = app_list.push(build_header_row());
        app_list = app_list.push(horizontal_rule(1));

        for (index, app_item) in state.apps.iter().enumerate() {
            app_list = app_list.push(build_app_row(app_item, index, state.updating));

            if index < state.apps.len() - 1 {
                app_list = app_list.push(horizontal_rule(1));
            }
        }
    }

    scrollable(app_list)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

/// Builds the header row for the app list
fn build_header_row() -> Element<'static, Message> {
    row![
        text("").width(Length::Fixed(30.0)),
        text("Application")
            .width(Length::FillPortion(3))
            .size(14),
        text("Installed Version")
            .width(Length::FillPortion(2))
            .size(14),
        text("Latest Version")
            .width(Length::FillPortion(2))
            .size(14),
    ]
    .spacing(10)
    .into()
}

/// Builds a single app row
fn build_app_row(
    app_item: &crate::models::AppItem,
    index: usize,
    updating: bool,
) -> Element<'_, Message> {
    let checkbox_widget = if updating {
        checkbox("", app_item.selected)
    } else {
        checkbox("", app_item.selected).on_toggle(move |_| Message::ToggleApp(index))
    };

    row![
        checkbox_widget,
        text(&app_item.app.name).width(Length::FillPortion(3)),
        text(&app_item.app.version).width(Length::FillPortion(2)),
        text(&app_item.app.available).width(Length::FillPortion(2)),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

/// Builds the confirmation dialog
fn build_confirmation_dialog(state: &AppState) -> Element<'_, Message> {
    let mut apps_text = String::from("The following apps will be updated:\n\n");
    for app_id in &state.apps_needing_close {
        apps_text.push_str(&format!("• {}\n", app_id));
    }
    apps_text.push_str("\nThey may need to be closed before updating. Continue?");

    let dialog = column![
        text("Confirm Update").size(24),
        text(apps_text).size(14),
        row![
            button("Yes, Proceed")
                .on_press(Message::ConfirmUpdate)
                .padding(10),
            button("Cancel")
                .on_press(Message::CancelUpdate)
                .padding(10),
        ]
        .spacing(10),
    ]
    .spacing(20)
    .padding(30)
    .width(Length::Fixed(500.0));

    create_dialog_overlay(dialog)
}

/// Builds the results dialog
fn build_results_dialog(state: &AppState) -> Element<'_, Message> {
    let mut results_column = Column::new().spacing(5);

    for result in &state.update_results {
        let (result_text, color) = parse_result_status(result);
        results_column = results_column.push(text(result_text).size(14).color(color));
    }

    let header = row![
        text("Update Results").size(24).width(Length::Fill),
        button("X")
            .on_press(Message::CloseResultsDialog)
            .padding(5)
            .style(create_close_button_style),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let inner_dialog = container(
        column![header, scrollable(results_column).height(Length::Fixed(300.0)),]
            .spacing(20)
            .padding(30),
    )
    .width(Length::Fixed(600.0))
    .style(create_dialog_style);

    create_dialog_overlay(inner_dialog)
}

/// Parses result status and returns text and color
fn parse_result_status(result: &str) -> (String, Color) {
    if result.starts_with("SUCCESS:") {
        (
            result.replace("SUCCESS:", ""),
            Color::from_rgb(0.0, 0.8, 0.0),
        )
    } else if result.starts_with("FAILURE:") {
        (
            result.replace("FAILURE:", ""),
            Color::from_rgb(0.9, 0.0, 0.0),
        )
    } else if result.starts_with("[!]") {
        (result.to_string(), Color::from_rgb(1.0, 0.6, 0.0))
    } else if result.starts_with("[i]") {
        (result.to_string(), Color::from_rgb(0.4, 0.7, 1.0))
    } else {
        (result.to_string(), Color::WHITE)
    }
}

/// Creates a dialog overlay with semi-transparent background
fn create_dialog_overlay<'a, T: Into<Element<'a, Message>>>(
    content: T,
) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.0, 0.0, 0.0, 0.7,
            ))),
            ..Default::default()
        })
        .into()
}

/// Style for the dialog container
fn create_dialog_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.5, 0.5, 0.5),
            width: 2.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the close button
fn create_close_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        text_color: Color::from_rgb(0.8, 0.8, 0.8),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

