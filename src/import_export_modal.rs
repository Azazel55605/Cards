/// import_export_modal.rs — UI for the Import / Export dialogs
///
/// The modal is driven by `ImportExportState` stored in the app.
/// It reuses the file picker for path selection and adds a format
/// selector (Export) or result display (Import).

use iced::{
    Alignment, Border, Color, Element, Length, Padding, Radians, Shadow,
    gradient,
    widget::{button, column, container, row, scrollable, svg, text, Space},
};
use iced::Theme as IcedTheme;

use crate::button_style::CardButtonStyle;
use crate::file_picker::{FilePickerMessage, FilePickerState, FilePickerMode};
use crate::import_export::ExportFormat;
use crate::theme::Theme;
use crate::svg_style::SvgStyle;

// ── Operation kind ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IEKind {
    ExportWorkspace,
    ExportBoard,
    ImportWorkspace,
    ImportBoard,
}

impl IEKind {
    pub fn is_export(&self) -> bool {
        matches!(self, IEKind::ExportWorkspace | IEKind::ExportBoard)
    }
    pub fn is_workspace(&self) -> bool {
        matches!(self, IEKind::ExportWorkspace | IEKind::ImportWorkspace)
    }
    pub fn title(&self) -> &'static str {
        match self {
            IEKind::ExportWorkspace => "Export Workspace",
            IEKind::ExportBoard     => "Export Board",
            IEKind::ImportWorkspace => "Import Workspace",
            IEKind::ImportBoard     => "Import Board",
        }
    }
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ImportExportState {
    pub kind: IEKind,
    /// File picker for choosing path
    pub picker: FilePickerState,
    /// Selected export format (Export only)
    pub format: ExportFormat,
    /// Result message (success / error / warning) shown after operation
    pub result: Option<ImportExportResult>,
}

#[derive(Debug, Clone)]
pub struct ImportExportResult {
    pub success: bool,
    pub message: String,
    pub warnings: Vec<String>,
}

impl ImportExportState {
    pub fn new_export_workspace(workspace_name: &str, start_dir: std::path::PathBuf) -> Self {
        let safe = workspace_name.replace(' ', "_");
        let picker = FilePickerState::new(
            FilePickerMode::Save { default_name: safe },
            start_dir,
            "Export Workspace",
        );
        Self { kind: IEKind::ExportWorkspace, picker, format: ExportFormat::CardsWorkspace, result: None }
    }

    pub fn new_export_board(board_name: &str, start_dir: std::path::PathBuf) -> Self {
        let safe = board_name.replace(' ', "_");
        let picker = FilePickerState::new(
            FilePickerMode::Save { default_name: safe },
            start_dir,
            "Export Board",
        );
        Self { kind: IEKind::ExportBoard, picker, format: ExportFormat::CardsBoard, result: None }
    }

    pub fn new_import_workspace(start_dir: std::path::PathBuf) -> Self {
        let picker = FilePickerState::new(
            FilePickerMode::Open { filter_exts: vec![] }, // accept all, validated after
            start_dir,
            "Import Workspace",
        );
        Self { kind: IEKind::ImportWorkspace, picker, format: ExportFormat::CardsWorkspace, result: None }
    }

    pub fn new_import_board(start_dir: std::path::PathBuf) -> Self {
        let picker = FilePickerState::new(
            FilePickerMode::Open { filter_exts: vec![] },
            start_dir,
            "Import Board",
        );
        Self { kind: IEKind::ImportBoard, picker, format: ExportFormat::CardsBoard, result: None }
    }

    /// Build the final output path, appending the correct extension for exports.
    pub fn resolved_path(&self) -> Option<std::path::PathBuf> {
        if self.kind.is_export() {
            let name = self.picker.file_name.trim().to_string();
            if name.is_empty() { return None; }
            let ext = self.format.extension();
            let file = if name.ends_with(&format!(".{}", ext)) {
                name
            } else {
                // Strip any other extension first
                let stem = std::path::Path::new(&name)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or(name);
                format!("{}.{}", stem, ext)
            };
            Some(self.picker.current_dir.join(file))
        } else {
            // Import: use selected file directly
            let name = self.picker.file_name.trim().to_string();
            if name.is_empty() { return None; }
            Some(self.picker.current_dir.join(name))
        }
    }
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ImportExportMessage {
    /// File picker internal events
    Picker(FilePickerMessage),
    /// User selected an export format
    SetFormat(ExportFormat),
    /// Confirm / execute the operation
    Confirm,
    /// Dismiss the modal
    Cancel,
    /// Dismiss the result panel and return to file picker
    DismissResult,
}

// ── View ──────────────────────────────────────────────────────────────────────

pub fn view(
    state: &ImportExportState,
    theme: Theme,
    accent: Color,
) -> Element<'static, ImportExportMessage> {
    if let Some(ref result) = state.result {
        return result_view(result, state.kind.title(), theme, accent);
    }

    if !state.kind.is_export() {
        return crate::file_picker::view(&state.picker, theme, accent)
            .map(ImportExportMessage::Picker);
    }

    // ── Export mode ───────────────────────────────────────────────────────────
    // We inject the format selector as the `extra` element inside view_with_extra.
    // Format buttons emit FilePickerMessage::Extra(idx) which the ImportExport
    // message handler maps back to SetFormat.
    // idx 0 = first format, 1 = second format, etc.

    let text_color = theme.button_text();
    let accent_bg  = theme.accent_bg_from(accent);
    let border_col = theme.button_border();
    let dim_color  = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.55);
    let sep_color  = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.12);

    let formats: &[ExportFormat] = if state.kind == IEKind::ExportWorkspace {
        &[ExportFormat::CardsWorkspace, ExportFormat::Json]
    } else {
        &[ExportFormat::CardsBoard, ExportFormat::Json]
    };

    // Build chip buttons emitting Extra(idx)
    let mut fmt_row: iced::widget::Row<'static, FilePickerMessage> = row![
        container(text("Format:").size(12).color(dim_color))
            .align_y(Alignment::Center),
        Space::with_width(10),
    ]
    .align_y(Alignment::Center)
    .spacing(6);

    for (idx, &fmt) in formats.iter().enumerate() {
        let selected = state.format == fmt;
        fmt_row = fmt_row.push(
            button(
                container(text(fmt.label()).size(12))
                    .align_x(Alignment::Center).align_y(Alignment::Center)
                    .width(Length::Shrink).height(Length::Fill),
            )
            .height(28)
            .padding(Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 10.0 })
            .class(CardButtonStyle {
                background: if selected { accent_bg } else { Color::TRANSPARENT },
                background_hovered: accent_bg,
                text_color,
                border_color: if selected { accent } else { border_col },
                shadow_color: Color::TRANSPARENT,
            })
            .on_press(FilePickerMessage::Extra(idx as u8)),
        );
    }

    // Thin separator + chip row = the extra element
    let sep_line: Element<'static, FilePickerMessage> = container(Space::with_height(1))
        .width(Length::Fill).height(1)
        .style(move |_: &IcedTheme| iced::widget::container::Style {
            background: Some(iced::Background::Color(sep_color)),
            ..Default::default()
        })
        .into();

    let extra: Element<'static, FilePickerMessage> = column![
        sep_line,
        container(fmt_row)
            .width(Length::Fill)
            .padding(Padding { top: 6.0, right: 0.0, bottom: 6.0, left: 0.0 }),
    ]
    .spacing(0)
    .into();

    // Map Extra(idx) → SetFormat; everything else → Picker(...)
    let formats_copy: Vec<ExportFormat> = formats.to_vec();
    crate::file_picker::view_with_extra(&state.picker, theme, accent, Some(extra))
        .map(move |fp_msg| match fp_msg {
            FilePickerMessage::Extra(idx) => {
                formats_copy.get(idx as usize)
                    .copied()
                    .map(ImportExportMessage::SetFormat)
                    .unwrap_or(ImportExportMessage::Cancel)
            }
            other => ImportExportMessage::Picker(other),
        })
}

// ── Result panel ──────────────────────────────────────────────────────────────

fn result_view(
    result: &ImportExportResult,
    title: &str,
    theme: Theme,
    accent: Color,
) -> Element<'static, ImportExportMessage> {
    let bg = theme.sidebar_background();
    let text_color = theme.button_text();
    let accent_bg = theme.accent_bg_from(accent);
    let shadow_color = theme.sidebar_shadow();
    let sep_color = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.12);

    let ok_color   = Color::from_rgb(0.2, 0.8, 0.4);
    let err_color  = Color::from_rgb(0.9, 0.3, 0.3);
    let warn_color = Color::from_rgb(0.9, 0.75, 0.2);

    let status_col = if result.success { ok_color } else { err_color };
    let status_icon_handle = if result.success {
        svg::Handle::from_memory(include_bytes!("icons/check.svg").as_slice())
    } else {
        svg::Handle::from_memory(include_bytes!("icons/cross.svg").as_slice())
    };
    let warn_icon_handle = svg::Handle::from_memory(include_bytes!("icons/exclamation.svg").as_slice());

    let mut content = column![
        row![
            container(
                svg(status_icon_handle)
                    .width(18)
                    .height(18)
                    .class(SvgStyle { color: status_col })
            )
            .align_y(Alignment::Center),
            Space::with_width(10),
            text(result.message.clone()).size(14).color(text_color),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(8)
    .width(Length::Fill);

    if !result.warnings.is_empty() {
        content = content.push(Space::with_height(4));
        for w in &result.warnings {
            content = content.push(
                row![
                    container(
                        svg(warn_icon_handle.clone())
                            .width(13)
                            .height(13)
                            .class(SvgStyle { color: warn_color })
                    )
                    .align_y(Alignment::Center),
                    Space::with_width(6),
                    text(w.clone()).size(12).color(warn_color),
                ]
                .align_y(Alignment::Center),
            );
        }
    }

    let dismiss_btn = button(
        container(text(if result.success { "Done" } else { "Close" }).size(13))
            .align_x(Alignment::Center).align_y(Alignment::Center)
            .width(Length::Fill).height(Length::Fill),
    )
    .height(32).width(Length::Shrink)
    .padding(Padding { top: 0.0, right: 16.0, bottom: 0.0, left: 16.0 })
    .class(CardButtonStyle {
        background: accent_bg,
        background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.35),
        text_color, border_color: accent, shadow_color: Color::TRANSPARENT,
    })
    .on_press(ImportExportMessage::DismissResult);

    let sep = container(Space::with_height(1))
        .width(Length::Fill).height(1)
        .style(move |_: &IcedTheme| iced::widget::container::Style {
            background: Some(iced::Background::Color(sep_color)),
            ..Default::default()
        });

    let grad_top = Color {
        r: bg.r * (1.0 - accent.a * 0.15) + accent.r * (accent.a * 0.15),
        g: bg.g * (1.0 - accent.a * 0.15) + accent.g * (accent.a * 0.15),
        b: bg.b * (1.0 - accent.a * 0.15) + accent.b * (accent.a * 0.15),
        a: 1.0,
    };
    let panel_gradient = gradient::Linear::new(Radians(std::f32::consts::PI * 0.75))
        .add_stop(0.0, grad_top)
        .add_stop(1.0, bg);

    let title_str = title.to_string();

    let panel = container(
        column![
            text(title_str)
                .size(15)
                .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() })
                .color(text_color),
            Space::with_height(12),
            sep,
            Space::with_height(16),
            scrollable(content.padding(4))
                .height(Length::Fixed(120.0))
                .width(Length::Fill),
            Space::with_height(16),
            container(row![
                Space::with_width(Length::Fill),
                dismiss_btn,
            ])
            .width(Length::Fill)
            .height(40)
            .align_y(Alignment::Center),
        ]
        .spacing(0)
        .width(Length::Fill),
    )
    .width(Length::Fixed(480.0))
    .padding(Padding::new(28.0))
    .style(move |_: &IcedTheme| iced::widget::container::Style {
        background: Some(iced::Background::Gradient(iced::Gradient::Linear(panel_gradient))),
        border: Border {
            color: Color::from_rgba(accent.r, accent.g, accent.b, 0.3),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: shadow_color,
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 28.0,
        },
        text_color: None,
    });

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_: &IcedTheme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
            ..Default::default()
        })
        .into()
}








