/// workspace_modal.rs — "Open or Create Workspace" dialog
///
/// Shown on first launch (or when no valid last workspace exists).
/// Lets the user either create a new named workspace or pick an existing .cards file.

use iced::{
    Alignment, Border, Color, Element, Length, Padding, Radians, Shadow,
    gradient,
    widget::{button, column, container, row, text, text_input, Space},
};
use iced::Theme as IcedTheme;
use std::path::PathBuf;
use crate::button_style::CardButtonStyle;
use crate::theme::Theme;
use crate::file_picker::{self, FilePickerState, FilePickerMessage};

#[derive(Debug, Clone)]
pub enum WorkspaceModalState {
    /// Idle — user hasn't chosen action yet
    Idle,
    /// User chose "New Workspace", entering a name
    CreatingNew {
        /// New workspace name
        name: String,
        /// File picker state for choosing save location
        picker: Option<FilePickerState>,
        /// Chosen save directory
        save_dir: PathBuf,
        /// True when this dialog was opened after an import — confirm should
        /// save the current in-memory state rather than create an empty workspace.
        is_import: bool,
    },
    /// User chose "Open Workspace", picking an existing file
    OpeningExisting {
        /// File picker state for opening a file
        picker: FilePickerState,
    },
    /// User chose "Import Workspace" from the welcome screen
    ImportingWorkspace {
        picker: FilePickerState,
    },
}

impl Default for WorkspaceModalState {
    fn default() -> Self { Self::Idle }
}

/// Messages produced by the workspace modal
#[derive(Debug, Clone)]
pub enum WorkspaceModalMessage {
    /// Choose to create a new workspace
    ChooseNew,
    /// Choose to open an existing workspace
    ChooseOpen,
    /// Choose to import a workspace from an export file
    ChooseImport,
    /// Input for new workspace name
    NewNameInput(String),
    /// Browse button pressed to choose save directory
    BrowseSaveDir,
    /// Confirm the creation of a new workspace
    ConfirmNew,
    /// Cancel the creation of a new workspace
    CancelNew,
    /// Messages from the file picker component
    FilePicker(FilePickerMessage),
    /// No-op — used to swallow backdrop mouse events so the canvas can't be panned
    Noop,
}

/// Render the workspace modal overlay.
/// Returns an `Element` that covers the whole screen.
pub fn view<'a>(
    state: &'a WorkspaceModalState,
    theme: Theme,
    accent: Color,
) -> Element<'static, WorkspaceModalMessage> {
    // If file picker is active, delegate to its view
    match state {
        WorkspaceModalState::CreatingNew { picker: Some(fp), .. } => {
            return file_picker::view(fp, theme, accent)
                .map(WorkspaceModalMessage::FilePicker);
        }
        WorkspaceModalState::OpeningExisting { picker } => {
            return file_picker::view(picker, theme, accent)
                .map(WorkspaceModalMessage::FilePicker);
        }
        WorkspaceModalState::ImportingWorkspace { picker } => {
            return file_picker::view(picker, theme, accent)
                .map(WorkspaceModalMessage::FilePicker);
        }
        _ => {}
    }

    let bg = theme.sidebar_background();
    let text_color = theme.button_text();
    let accent_bg = theme.accent_bg_from(accent);
    let shadow_color = theme.sidebar_shadow();

    // ── Buttons ──────────────────────────────────────────────────────────

    let primary_btn_style = CardButtonStyle {
        background: accent_bg,
        background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.35),
        text_color,
        border_color: accent,
        shadow_color: Color::TRANSPARENT,
    };

    let secondary_btn_style = CardButtonStyle {
        background: Color::TRANSPARENT,
        background_hovered: accent_bg,
        text_color,
        border_color: theme.button_border(),
        shadow_color: Color::TRANSPARENT,
    };

    // Clone all state data upfront so the match arms work with owned values
    let (cloned_name, cloned_save_dir) = match state {
        WorkspaceModalState::CreatingNew { name, save_dir, .. } => {
            (name.clone(), save_dir.clone())
        }
        _ => (String::new(), std::path::PathBuf::new()),
    };

    let body: Element<'static, WorkspaceModalMessage> = match state {
        WorkspaceModalState::Idle => {
            let new_btn = button(
                container(text("New Workspace").size(14))
                    .align_x(Alignment::Center).align_y(Alignment::Center)
                    .width(Length::Fill).height(Length::Fill),
            )
            .width(160).height(40)
            .class(primary_btn_style.clone())
            .on_press(WorkspaceModalMessage::ChooseNew);

            let open_btn = button(
                container(text("Open Workspace…").size(14))
                    .align_x(Alignment::Center).align_y(Alignment::Center)
                    .width(Length::Fill).height(Length::Fill),
            )
            .width(160).height(40)
            .class(secondary_btn_style.clone())
            .on_press(WorkspaceModalMessage::ChooseOpen);

            let import_btn = button(
                container(text("Import Workspace…").size(14))
                    .align_x(Alignment::Center).align_y(Alignment::Center)
                    .width(Length::Fill).height(Length::Fill),
            )
            .width(160).height(40)
            .class(secondary_btn_style)
            .on_press(WorkspaceModalMessage::ChooseImport);

            column![
                text("Welcome to Cards")
                    .size(20)
                    .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() })
                    .color(text_color),
                Space::with_height(8),
                text("Create a new workspace or open an existing one.")
                    .size(14)
                    .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.7)),
                Space::with_height(28),
                row![new_btn, Space::with_width(12), open_btn].align_y(Alignment::Center),
                Space::with_height(10),
                row![import_btn].align_y(Alignment::Center),
            ]
            .spacing(0)
            .into()
        }

        WorkspaceModalState::CreatingNew { .. } => {
            let name = &cloned_name;
            let save_dir = &cloned_save_dir;

            let input = text_input("Workspace name…", name)
                .on_input(WorkspaceModalMessage::NewNameInput)
                .on_submit(WorkspaceModalMessage::ConfirmNew)
                .size(14)
                .padding(Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
                .style(move |_theme: &IcedTheme, status| {
                    use iced::widget::text_input::Style;
                    Style {
                        background: iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.08)),
                        border: Border {
                            color: match status {
                                iced::widget::text_input::Status::Focused => accent,
                                _ => Color::from_rgba(0.5, 0.5, 0.5, 0.4),
                            },
                            width: 1.5,
                            radius: 6.0.into(),
                        },
                        icon: accent,
                        placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        value: text_color,
                        selection: Color::from_rgba(accent.r, accent.g, accent.b, 0.3),
                    }
                });

            let dir_label = save_dir.to_string_lossy().to_string();
            let dir_display = if dir_label.len() > 45 {
                format!("…{}", &dir_label[dir_label.len().saturating_sub(44)..])
            } else {
                dir_label
            };

            let can_confirm = !name.trim().is_empty();
            let confirm_btn: Element<'static, WorkspaceModalMessage> = if can_confirm {
                button(
                    container(text("Create").size(14))
                        .align_x(Alignment::Center).align_y(Alignment::Center)
                        .width(Length::Fill).height(Length::Fill),
                )
                .width(100).height(36)
                .class(primary_btn_style)
                .on_press(WorkspaceModalMessage::ConfirmNew)
                .into()
            } else {
                container(
                    text("Create").size(14)
                        .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.35)),
                )
                .width(100).height(36)
                .align_x(Alignment::Center).align_y(Alignment::Center)
                .style(move |_: &IcedTheme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
                    border: Border { color: Color::from_rgba(0.5, 0.5, 0.5, 0.2), width: 1.5, radius: 6.0.into() },
                    ..Default::default()
                })
                .into()
            };

            let cancel_btn = button(
                container(text("Back").size(14))
                    .align_x(Alignment::Center).align_y(Alignment::Center)
                    .width(Length::Fill).height(Length::Fill),
            )
            .width(80).height(36)
            .class(secondary_btn_style.clone())
            .on_press(WorkspaceModalMessage::CancelNew);

            let browse_btn = button(
                container(text("Browse…").size(13))
                    .align_x(Alignment::Center).align_y(Alignment::Center)
                    .width(Length::Fill).height(Length::Fill),
            )
            .height(28)
            .padding(Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 10.0 })
            .class(secondary_btn_style)
            .on_press(WorkspaceModalMessage::BrowseSaveDir);

            column![
                text("New Workspace")
                    .size(18)
                    .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() })
                    .color(text_color),
                Space::with_height(20),
                text("Workspace name").size(13)
                    .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.7)),
                Space::with_height(6),
                input,
                Space::with_height(14),
                text("Save location").size(13)
                    .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.7)),
                Space::with_height(6),
                row![
                    container(text(dir_display).size(12)
                        .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.6)))
                        .width(Length::Fill).align_y(Alignment::Center),
                    Space::with_width(8),
                    browse_btn,
                ]
                .align_y(Alignment::Center),
                Space::with_height(20),
                row![cancel_btn, Space::with_width(Length::Fill), confirm_btn]
                    .align_y(Alignment::Center),
            ]
            .spacing(0)
            .width(320)
            .into()
        }

        _ => Space::with_height(0).into(),
    };

    // ── Card container ────────────────────────────────────────────────────

    // Same gradient as file_picker: accent tint at top, plain bg at bottom
    let grad_top = Color {
        r: bg.r * (1.0 - accent.a * 0.18) + accent.r * (accent.a * 0.18),
        g: bg.g * (1.0 - accent.a * 0.18) + accent.g * (accent.a * 0.18),
        b: bg.b * (1.0 - accent.a * 0.18) + accent.b * (accent.a * 0.18),
        a: 1.0,
    };
    let card_gradient = gradient::Linear::new(Radians(std::f32::consts::PI * 0.75))
        .add_stop(0.0, grad_top)
        .add_stop(1.0, bg);

    let inner_card = container(body)
        .padding(Padding::new(32.0))
        .style(move |_: &IcedTheme| iced::widget::container::Style {
            background: Some(iced::Background::Gradient(iced::Gradient::Linear(card_gradient))),
            border: Border {
                color: Color::from_rgba(accent.r, accent.g, accent.b, 0.3),
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: shadow_color,
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 32.0,
            },
            text_color: None,
        });

    // The Overlay widget handles backdrop-click detection (fires CancelNew when
    // the user clicks outside the card). We just return the centred card here.
    container(inner_card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_: &IcedTheme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
            border: Border::default(),
            shadow: Shadow::default(),
            text_color: None,
        })
        .into()
}












