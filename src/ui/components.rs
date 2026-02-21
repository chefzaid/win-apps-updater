use crate::app::AppState;
use crate::models::Message;
use iced::{
    widget::{
        button, checkbox, column, container, horizontal_rule, progress_bar, row, scrollable,
        stack, text, text_input, Column,
    },
    Alignment, Color, Element, Font, Length,
};

// ── Colour palette ───────────────────────────────────────────────────

const ACCENT: Color = Color::from_rgb(0.30, 0.56, 0.93);
const SURFACE: Color = Color::from_rgb(0.13, 0.13, 0.17);
const SURFACE_LIGHT: Color = Color::from_rgb(0.17, 0.17, 0.22);
const HEADER_BG: Color = Color::from_rgb(0.10, 0.10, 0.14);
const ROW_ALT: Color = Color::from_rgb(0.15, 0.15, 0.19);
const ROW_NORMAL: Color = Color::from_rgb(0.12, 0.12, 0.16);
const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.60);
const SUCCESS_CLR: Color = Color::from_rgb(0.30, 0.80, 0.30);
const FAILURE_CLR: Color = Color::from_rgb(0.90, 0.25, 0.25);
const WARNING_CLR: Color = Color::from_rgb(1.0, 0.65, 0.15);
const INFO_CLR: Color = Color::from_rgb(0.40, 0.70, 1.0);
const OVERLAY_BG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.70);
const DIALOG_BG: Color = Color::from_rgb(0.16, 0.16, 0.20);

const BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    family: iced::font::Family::SansSerif,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

// ── Public entry point ───────────────────────────────────────────────

/// Builds the complete view for the application.
pub fn build_view(state: &AppState) -> Element<'_, Message> {
    let base = build_base_content(state);

    let mut layers: Vec<Element<'_, Message>> = vec![base];

    if state.show_confirmation {
        layers.push(build_confirmation_overlay(state));
    }
    if state.show_results_dialog {
        layers.push(build_results_overlay(state));
    }

    stack(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── Base content ─────────────────────────────────────────────────────

fn build_base_content(state: &AppState) -> Element<'_, Message> {
    let title_bar = build_title_bar(state);
    let toolbar = build_toolbar(state);
    let search = build_search_bar(state);
    let app_list = build_app_list(state);
    let status_bar = build_status_bar(state);

    let mut content = Column::new().spacing(0).width(Length::Fill).height(Length::Fill);
    content = content.push(title_bar);
    content = content.push(toolbar);
    if state.updating {
        content = content.push(build_progress_bar(state));
    }
    content = content.push(search);
    content = content.push(app_list);
    content = content.push(status_bar);
    content.into()
}

fn build_title_bar(state: &AppState) -> Element<'_, Message> {
    let title = text("Windows Apps Updater")
        .size(26)
        .color(Color::WHITE)
        .font(BOLD);

    let selected = state.selected_count();
    let badge: Element<'_, Message> = if selected > 0 {
        text(format!("{selected} selected"))
            .size(13)
            .color(ACCENT)
            .into()
    } else {
        text("").size(13).into()
    };

    container(
        row![title, badge]
            .spacing(16)
            .align_y(Alignment::Center),
    )
    .padding([16, 24])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    })
    .into()
}

fn build_toolbar(state: &AppState) -> Element<'_, Message> {
    let enabled = !state.updating && !state.loading;

    let refresh = styled_button("Refresh", enabled, Message::LoadApps);
    let select_all = styled_button("Select All", enabled, Message::SelectAll);
    let deselect_all = styled_button("Deselect All", enabled, Message::DeselectAll);

    let update_btn = if state.updating {
        styled_button_accent("Updating...", false, Message::UpdateSelected)
    } else {
        styled_button_accent(
            "Update Selected",
            enabled && state.selected_count() > 0,
            Message::UpdateSelected,
        )
    };

    container(
        row![refresh, select_all, deselect_all, update_btn]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding([10, 24])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(SURFACE_LIGHT)),
        ..Default::default()
    })
    .into()
}

fn build_search_bar(state: &AppState) -> Element<'_, Message> {
    let input = text_input("Filter apps by name or ID...", &state.search_query)
        .on_input(Message::SearchChanged)
        .size(14)
        .padding(10);

    container(input)
        .padding([8, 24])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(SURFACE)),
            ..Default::default()
        })
        .into()
}

fn build_status_bar(state: &AppState) -> Element<'_, Message> {
    let visible_count = state.visible_indices().len();
    let total_count = state.apps.len();

    let filter_info = if state.search_query.is_empty() {
        String::new()
    } else {
        format!(" (showing {visible_count} of {total_count})")
    };

    container(
        text(format!("{}{filter_info}", state.status_message))
            .size(13)
            .color(TEXT_MUTED),
    )
    .padding([10, 24])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        ..Default::default()
    })
    .into()
}

// ── App list ─────────────────────────────────────────────────────────

fn build_app_list(state: &AppState) -> Element<'_, Message> {
    let mut list = Column::new().spacing(0).width(Length::Fill);

    if state.loading {
        list = list.push(
            container(text("Loading...").size(16).color(TEXT_MUTED))
                .padding(40)
                .center_x(Length::Fill),
        );
    } else if state.apps.is_empty() {
        list = list.push(
            container(
                column![
                    text("All apps are up to date!")
                        .size(18)
                        .color(SUCCESS_CLR),
                    text("Click Refresh to check again.")
                        .size(13)
                        .color(TEXT_MUTED),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .padding(40)
            .center_x(Length::Fill),
        );
    } else {
        list = list.push(build_list_header());

        let visible = state.visible_indices();
        if visible.is_empty() {
            list = list.push(
                container(
                    text("No apps match your filter.")
                        .size(14)
                        .color(TEXT_MUTED),
                )
                .padding(20)
                .center_x(Length::Fill),
            );
        } else {
            for (row_num, &idx) in visible.iter().enumerate() {
                let bg = if row_num % 2 == 0 {
                    ROW_NORMAL
                } else {
                    ROW_ALT
                };
                list = list.push(build_app_row(&state.apps[idx], idx, state.updating, bg));
            }
        }
    }

    scrollable(list)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn build_list_header() -> Element<'static, Message> {
    container(
        row![
            text("").width(Length::Fixed(40.0)),
            text("Application")
                .size(12)
                .color(TEXT_MUTED)
                .font(BOLD)
                .width(Length::FillPortion(4)),
            text("ID")
                .size(12)
                .color(TEXT_MUTED)
                .font(BOLD)
                .width(Length::FillPortion(4)),
            text("Installed")
                .size(12)
                .color(TEXT_MUTED)
                .font(BOLD)
                .width(Length::FillPortion(2)),
            text("Available")
                .size(12)
                .color(TEXT_MUTED)
                .font(BOLD)
                .width(Length::FillPortion(2)),
        ]
        .spacing(8)
        .padding([0, 8])
        .align_y(Alignment::Center),
    )
    .padding([8, 16])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(HEADER_BG)),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn build_app_row(
    item: &crate::models::AppItem,
    index: usize,
    updating: bool,
    bg: Color,
) -> Element<'_, Message> {
    let cb = if updating {
        checkbox("", item.selected)
    } else {
        checkbox("", item.selected).on_toggle(move |_| Message::ToggleApp(index))
    };

    container(
        row![
            container(cb).width(Length::Fixed(40.0)),
            text(&item.app.name)
                .size(14)
                .width(Length::FillPortion(4)),
            text(&item.app.id)
                .size(13)
                .color(TEXT_MUTED)
                .width(Length::FillPortion(4)),
            text(&item.app.version)
                .size(13)
                .width(Length::FillPortion(2)),
            text(&item.app.available)
                .size(13)
                .color(ACCENT)
                .width(Length::FillPortion(2)),
        ]
        .spacing(8)
        .padding([0, 8])
        .align_y(Alignment::Center),
    )
    .padding([6, 16])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg)),
        ..Default::default()
    })
    .into()
}

// ── Dialogs ──────────────────────────────────────────────────────────

fn build_confirmation_overlay(state: &AppState) -> Element<'_, Message> {
    let mut apps_col = Column::new().spacing(4);
    for (name, id) in &state.pending_updates {
        apps_col = apps_col.push(
            text(format!("  {name}  ({id})"))
                .size(13)
                .color(Color::from_rgb(0.8, 0.8, 0.85)),
        );
    }

    let note = text("These apps may need to be closed before updating.")
        .size(12)
        .color(WARNING_CLR);

    let buttons = row![
        styled_button_accent("Yes, Proceed", true, Message::ConfirmUpdate),
        styled_button("Cancel", true, Message::CancelUpdate),
    ]
    .spacing(12);

    let dialog = container(
        column![
            text("Confirm Update").size(22).color(Color::WHITE).font(BOLD),
            horizontal_rule(1),
            text(format!(
                "The following {} app(s) will be updated:",
                state.pending_updates.len()
            ))
            .size(14),
            container(scrollable(apps_col)).max_height(250),
            note,
            buttons,
        ]
        .spacing(16)
        .padding(28)
        .max_width(560),
    )
    .style(dialog_style);

    overlay_backdrop(dialog)
}

fn build_results_overlay(state: &AppState) -> Element<'_, Message> {
    // Tally results
    let mut ok_count = 0usize;
    let mut fail_count = 0usize;
    let mut warn_count = 0usize;
    for r in &state.update_results {
        if r.starts_with("SUCCESS:") {
            ok_count += 1;
        } else if r.starts_with("FAILURE:") {
            fail_count += 1;
        } else {
            warn_count += 1;
        }
    }

    // Summary badges
    let summary = row![
        result_badge(&format!("{ok_count} succeeded"), SUCCESS_CLR),
        result_badge(&format!("{fail_count} failed"), FAILURE_CLR),
        result_badge(&format!("{warn_count} other"), WARNING_CLR),
    ]
    .spacing(8);

    // Per-app result rows
    let mut results_col = Column::new().spacing(2);
    for (i, result) in state.update_results.iter().enumerate() {
        let (icon_txt, label, color) = format_result_row(result);
        let bg = if i % 2 == 0 { ROW_NORMAL } else { ROW_ALT };

        let result_row = container(
            row![
                container(
                    text(icon_txt).size(12).color(Color::WHITE).font(BOLD)
                )
                .padding([2, 8])
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(color)),
                    border: iced::Border { radius: 4.0.into(), ..Default::default() },
                    ..Default::default()
                }),
                text(label).size(13).color(Color::from_rgb(0.85, 0.85, 0.88)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([6, 12])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            ..Default::default()
        });

        results_col = results_col.push(result_row);
    }

    // Header row with title and close button
    let header = row![
        text("Update Results")
            .size(22)
            .color(Color::WHITE)
            .font(BOLD)
            .width(Length::Fill),
        button(text("X").size(14).font(BOLD))
            .on_press(Message::CloseResultsDialog)
            .padding([4, 10])
            .style(close_button_style),
    ]
    .align_y(Alignment::Center);

    // "Done" button at the bottom
    let done_btn = container(
        styled_button_accent("Done", true, Message::CloseResultsDialog),
    )
    .width(Length::Fill)
    .center_x(Length::Fill);

    let dialog = container(
        column![
            header,
            horizontal_rule(1),
            summary,
            container(
                scrollable(results_col)
                    .height(Length::Fixed(280.0))
                    .width(Length::Fill),
            )
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.11, 0.11, 0.14))),
                border: iced::Border {
                    color: Color::from_rgb(0.22, 0.22, 0.28),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }),
            done_btn,
        ]
        .spacing(14)
        .padding(28)
        .width(Length::Fill)
        .max_width(620),
    )
    .width(Length::FillPortion(3))
    .center_x(Length::Fill)
    .style(dialog_style);

    overlay_backdrop(dialog)
}

/// Small coloured badge for the results summary row.
fn result_badge<'a>(label: &str, color: Color) -> Element<'a, Message> {
    container(
        text(label.to_string()).size(12).color(Color::WHITE).font(BOLD),
    )
    .padding([3, 10])
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(Color {
            a: 0.25,
            ..color
        })),
        border: iced::Border {
            color: Color { a: 0.5, ..color },
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Returns `(badge_text, description, badge_color)` for one result line.
fn format_result_row(result: &str) -> (String, String, Color) {
    if let Some(rest) = result.strip_prefix("SUCCESS:") {
        ("OK".into(), rest.to_string(), SUCCESS_CLR)
    } else if let Some(rest) = result.strip_prefix("FAILURE:") {
        ("FAIL".into(), rest.to_string(), FAILURE_CLR)
    } else if let Some(rest) = result.strip_prefix("[!]") {
        ("WARN".into(), rest.trim().to_string(), WARNING_CLR)
    } else if let Some(rest) = result.strip_prefix("[i]") {
        ("INFO".into(), rest.trim().to_string(), INFO_CLR)
    } else {
        ("—".into(), result.to_string(), TEXT_MUTED)
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn overlay_backdrop<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(OVERLAY_BG)),
            ..Default::default()
        })
        .into()
}

fn dialog_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(DIALOG_BG)),
        border: iced::Border {
            color: Color::from_rgb(0.3, 0.3, 0.38),
            width: 1.5,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

fn close_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(0.35, 0.15, 0.15),
        button::Status::Pressed => Color::from_rgb(0.50, 0.15, 0.15),
        _ => Color::from_rgb(0.22, 0.22, 0.27),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.85, 0.85, 0.85),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn styled_button(label: &str, enabled: bool, msg: Message) -> Element<'_, Message> {
    let btn = button(text(label).size(13))
        .padding([8, 16])
        .style(toolbar_button_style);

    if enabled { btn.on_press(msg) } else { btn }.into()
}

fn styled_button_accent(label: &str, enabled: bool, msg: Message) -> Element<'_, Message> {
    let btn = button(text(label).size(13))
        .padding([8, 16])
        .style(accent_button_style);

    if enabled { btn.on_press(msg) } else { btn }.into()
}

fn toolbar_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(0.25, 0.25, 0.30),
        button::Status::Pressed => Color::from_rgb(0.20, 0.20, 0.25),
        _ => Color::from_rgb(0.22, 0.22, 0.27),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::from_rgb(0.85, 0.85, 0.88),
        border: iced::Border {
            color: Color::from_rgb(0.3, 0.3, 0.35),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn accent_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(0.35, 0.60, 0.96),
        button::Status::Pressed => Color::from_rgb(0.25, 0.50, 0.86),
        _ => ACCENT,
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

// ── Progress bar ─────────────────────────────────────────────────────

fn build_progress_bar(state: &AppState) -> Element<'_, Message> {
    let pct = if state.update_total > 0 {
        state.update_completed as f32 / state.update_total as f32 * 100.0
    } else {
        0.0
    };

    let label = text(format!(
        "{}/{} apps updated",
        state.update_completed, state.update_total
    ))
    .size(12)
    .color(TEXT_MUTED);

    container(
        column![
            label,
            progress_bar(0.0..=100.0, pct)
                .height(6)
                .style(progress_bar_style),
        ]
        .spacing(4),
    )
    .padding([8, 24])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(SURFACE)),
        ..Default::default()
    })
    .into()
}

fn progress_bar_style(_theme: &iced::Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: iced::Background::Color(Color::from_rgb(0.20, 0.20, 0.25)),
        bar: iced::Background::Color(ACCENT),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 3.0.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_result_row_success() {
        let (badge, label, color) = format_result_row("SUCCESS:App.Id - updated successfully");
        assert_eq!(badge, "OK");
        assert!(label.contains("App.Id"));
        assert_eq!(color, SUCCESS_CLR);
    }

    #[test]
    fn test_format_result_row_failure() {
        let (badge, label, color) = format_result_row("FAILURE:App.Id - download error");
        assert_eq!(badge, "FAIL");
        assert!(label.contains("App.Id"));
        assert_eq!(color, FAILURE_CLR);
    }

    #[test]
    fn test_format_result_row_warning() {
        let (badge, label, color) = format_result_row("[!] App.Id - needs to be closed");
        assert_eq!(badge, "WARN");
        assert!(label.contains("App.Id"));
        assert_eq!(color, WARNING_CLR);
    }

    #[test]
    fn test_format_result_row_info() {
        let (badge, label, color) = format_result_row("[i] App.Id - already up to date");
        assert_eq!(badge, "INFO");
        assert!(label.contains("App.Id"));
        assert_eq!(color, INFO_CLR);
    }

    #[test]
    fn test_format_result_row_plain() {
        let (badge, label, color) = format_result_row("Some unknown format");
        assert_eq!(badge, "\u{2014}");
        assert_eq!(label, "Some unknown format");
        assert_eq!(color, TEXT_MUTED);
    }

    #[test]
    fn test_progress_bar_style_colors() {
        let style = progress_bar_style(&iced::Theme::Dark);
        // Verify it produces the expected accent bar colour
        assert_eq!(style.bar, iced::Background::Color(ACCENT));
    }

    #[test]
    fn test_toolbar_button_style_variants() {
        let default = toolbar_button_style(&iced::Theme::Dark, button::Status::Active);
        let hovered = toolbar_button_style(&iced::Theme::Dark, button::Status::Hovered);
        let pressed = toolbar_button_style(&iced::Theme::Dark, button::Status::Pressed);
        assert!(default.background.is_some());
        assert!(hovered.background.is_some());
        assert!(pressed.background.is_some());
        // Hovered should be brighter than default
        assert_ne!(default.background, hovered.background);
    }

    #[test]
    fn test_accent_button_style_variants() {
        let default = accent_button_style(&iced::Theme::Dark, button::Status::Active);
        let hovered = accent_button_style(&iced::Theme::Dark, button::Status::Hovered);
        assert_eq!(default.text_color, Color::WHITE);
        assert_eq!(hovered.text_color, Color::WHITE);
        assert_ne!(default.background, hovered.background);
    }

    #[test]
    fn test_close_button_style_has_background() {
        let active = close_button_style(&iced::Theme::Dark, button::Status::Active);
        let hovered = close_button_style(&iced::Theme::Dark, button::Status::Hovered);
        let pressed = close_button_style(&iced::Theme::Dark, button::Status::Pressed);
        // Active has a subtle dark background (not transparent)
        assert!(active.background.is_some());
        // Hovered turns reddish - should differ from active
        assert_ne!(active.background, hovered.background);
        // Pressed differs from hovered
        assert_ne!(hovered.background, pressed.background);
    }

    #[test]
    fn test_dialog_style_has_border() {
        let style = dialog_style(&iced::Theme::Dark);
        assert!(style.border.width > 0.0);
    }
}

