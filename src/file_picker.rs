/// file_picker.rs — Reusable custom file-picker modal
///
/// Renders a full-screen dimmed overlay with a centred panel that lets the
/// user browse the filesystem directory-by-directory and pick a file or
/// choose a save location.
///
/// Two modes:
///   - Open  : pick an existing file (filtered by extension)
///   - Save  : type a filename in the current directory
///
/// Usage:
///   1. Construct a `FilePickerState` and store it in your app state.
///   2. Call `file_picker::view(&state, theme, accent)` to get an
///      `Element<FilePickerMessage>`.
///   3. Map the message and handle `FilePickerMessage::Picked(path)` /
///      `FilePickerMessage::Cancelled`.

use std::path::PathBuf;
use std::collections::HashSet;
use iced::{
    Alignment, Border, Color, Element, Length, Padding, Radians, Shadow,
    gradient,
    widget::{button, column, container, row, scrollable, svg, text, text_input, Space},
};
use iced::Theme as IcedTheme;
use crate::button_style::CardButtonStyle;
use crate::svg_style::SvgStyle;
use crate::theme::Theme;

// ── Mode ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FilePickerMode {
    /// Pick an existing file — only show files whose extension is in the list
    /// (empty list = show all files)
    Open { filter_exts: Vec<String> },
    /// Choose a directory + filename to save to
    Save { default_name: String },
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FilePickerState {
    pub mode: FilePickerMode,
    /// Currently browsed directory
    pub current_dir: PathBuf,
    /// Directory entries in current_dir (dirs first, then files)
    pub entries: Vec<DirEntry>,
    /// Filename input (Save mode) or selected file name (Open mode)
    pub file_name: String,
    /// Error string shown below the input
    pub error: Option<String>,
    /// Optional title shown at the top
    pub title: String,
    /// Which drive roots have their sub-tree expanded in the places sidebar
    pub expanded_drives: HashSet<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

impl FilePickerState {
    /// Create a new picker rooted at `start_dir`.
    pub fn new(mode: FilePickerMode, start_dir: PathBuf, title: impl Into<String>) -> Self {
        let default_name = match &mode {
            FilePickerMode::Save { default_name } => default_name.clone(),
            FilePickerMode::Open { .. } => String::new(),
        };
        let mut s = Self {
            mode,
            current_dir: start_dir,
            entries: Vec::new(),
            file_name: default_name,
            error: None,
            title: title.into(),
            expanded_drives: HashSet::new(),
        };
        s.refresh_entries();
        s
    }

    /// Reload `entries` from `current_dir`.
    pub fn refresh_entries(&mut self) {
        self.entries.clear();
        let Ok(rd) = std::fs::read_dir(&self.current_dir) else { return };

        let mut dirs: Vec<DirEntry> = Vec::new();
        let mut files: Vec<DirEntry> = Vec::new();

        for entry in rd.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip hidden files
            if name.starts_with('.') { continue; }
            let is_dir = path.is_dir();
            let de = DirEntry { name, path, is_dir };
            if is_dir { dirs.push(de); } else { files.push(de); }
        }

        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        // In Open mode, filter files by extension (empty list = show all)
        if let FilePickerMode::Open { ref filter_exts } = self.mode {
            if !filter_exts.is_empty() {
                files.retain(|f| {
                    f.path.extension()
                        .map(|e| {
                            let ext = e.to_string_lossy().to_lowercase();
                            filter_exts.iter().any(|fe| fe.to_lowercase() == ext)
                        })
                        .unwrap_or(false)
                });
            }
        }

        self.entries = dirs.into_iter().chain(files).collect();
    }

    /// Navigate into a subdirectory.
    pub fn enter(&mut self, path: PathBuf) {
        self.current_dir = path;
        self.refresh_entries();
        self.error = None;
    }

    /// Navigate up one level.
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
            self.current_dir = parent;
            self.refresh_entries();
            self.error = None;
        }
    }

    /// Build the confirmed path (for Save: dir + filename; for Open: selected file).
    pub fn confirmed_path(&self) -> Option<PathBuf> {
        let name = self.file_name.trim().to_string();
        if name.is_empty() { return None; }
        match &self.mode {
            FilePickerMode::Save { .. } => {
                let mut name = name;
                if !name.ends_with(".cards") {
                    name.push_str(".cards");
                }
                Some(self.current_dir.join(name))
            }
            FilePickerMode::Open { .. } => {
                let path = self.current_dir.join(&name);
                if path.exists() { Some(path) } else { None }
            }
        }
    }
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FilePickerMessage {
    EnterDir(PathBuf),
    GoUp,
    FileNameInput(String),
    SelectFile(String),
    Confirm,
    Cancel,
    GoToPlace(PathBuf),
    ToggleDrive(PathBuf),
    /// Generic passthrough for callers that embed extra UI inside view_with_extra.
    /// The u8 index is caller-defined.
    Extra(u8),
}

// ── Places helpers ────────────────────────────────────────────────────────────

struct Place {
    label: String,
    path: PathBuf,
    /// true = drives section, false = shortcuts section
    is_drive: bool,
}

/// Build the list of quick-access places.
fn build_places() -> Vec<Place> {
    let mut places: Vec<Place> = Vec::new();

    // ── User shortcuts ────────────────────────────────────────────────────────
    if let Some(h) = dirs::home_dir() {
        places.push(Place { label: "Home".into(),      path: h.clone(),                          is_drive: false });
        places.push(Place { label: "Desktop".into(),   path: h.join("Desktop"),                  is_drive: false });
        places.push(Place { label: "Documents".into(), path: h.join("Documents"),                is_drive: false });
        places.push(Place { label: "Downloads".into(), path: h.join("Downloads"),                is_drive: false });
    }

    // ── Mounted drives (Linux: /proc/mounts) ─────────────────────────────────
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/mounts") {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 3 { continue; }
                let dev   = parts[0];
                let mount = parts[1];
                let fs    = parts[2];
                // Keep only real block devices and common virtual FS we care about
                let want_dev = dev.starts_with("/dev/sd")
                    || dev.starts_with("/dev/nvme")
                    || dev.starts_with("/dev/mmcblk")
                    || dev.starts_with("/dev/mapper")
                    || dev.starts_with("/dev/hd");
                let want_fs = matches!(fs, "ext4"|"ext3"|"ext2"|"btrfs"|"xfs"|"vfat"|"ntfs"|"exfat"|"f2fs"|"fuseblk");
                if !want_dev && !want_fs { continue; }
                // Skip root (already accessible via Home) and common pseudo-mounts
                if mount == "/" { continue; }
                let path = PathBuf::from(mount);
                if !path.exists() { continue; }
                // Human-readable label: last path component, or the device name
                let label = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| dev.to_string());
                // Avoid duplicates
                if !places.iter().any(|p| p.path == path) {
                    places.push(Place { label, path, is_drive: true });
                }
            }
        }
    }

    // ── macOS /Volumes ────────────────────────────────────────────────────────
    #[cfg(target_os = "macos")]
    {
        if let Ok(rd) = std::fs::read_dir("/Volumes") {
            for entry in rd.flatten() {
                let path = entry.path();
                let label = entry.file_name().to_string_lossy().to_string();
                if !places.iter().any(|p| p.path == path) {
                    places.push(Place { label, path, is_drive: true });
                }
            }
        }
    }

    // ── Windows drive letters ─────────────────────────────────────────────────
    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let path = PathBuf::from(format!("{}:\\", letter as char));
            if path.exists() {
                places.push(Place {
                    label: format!("{}:", letter as char),
                    path,
                    is_drive: true,
                });
            }
        }
    }

    places
}

// ── View ──────────────────────────────────────────────────────────────────────

pub fn view(
    state: &FilePickerState,
    theme: Theme,
    accent: Color,
) -> Element<'static, FilePickerMessage> {
    view_with_extra(state, theme, accent, None)
}

/// Like `view` but injects `extra` as an additional row inside the panel,
/// between the file-list separator and the bottom action row.
/// Used by the export modal to embed the format selector.
pub fn view_with_extra(
    state: &FilePickerState,
    theme: Theme,
    accent: Color,
    extra: Option<Element<'static, FilePickerMessage>>,
) -> Element<'static, FilePickerMessage> {
    let bg = theme.sidebar_background();
    let text_color = theme.button_text();
    let accent_bg = theme.accent_bg_from(accent);
    let shadow_color = theme.sidebar_shadow();
    let border_col = theme.button_border();
    let dim_color = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.55);
    let sep_color = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.12);
    let icon_color = theme.icon_color();

    // ── Panel dimensions ──────────────────────────────────────────────────────
    const PLACES_W: f32 = 150.0;
    const PANEL_W: f32 = 700.0;
    const PANEL_PAD: f32 = 20.0;
    const LIST_H: f32 = 300.0;
    const BODY_H: f32 = LIST_H + 28.0 + 8.0 + 8.0 + 1.0;
    // Extra row height (format selector strip: sep(1) + padding(12) + chips(28))
    const EXTRA_H: f32 = 1.0 + 12.0 + 28.0;
    let base_panel_h: f32 = 28.0 + 12.0 + 1.0 + BODY_H + 1.0 + 12.0 + 40.0 + PANEL_PAD * 2.0;
    let panel_h: f32 = if extra.is_some() { base_panel_h + EXTRA_H } else { base_panel_h };

    let btn = move |label: String, msg: FilePickerMessage| -> Element<'static, FilePickerMessage> {
        button(
            container(text(label).size(13))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .height(32)
        .width(Length::Shrink)
        .padding(Padding { top: 0.0, right: 12.0, bottom: 0.0, left: 12.0 })
        .class(CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: accent_bg,
            text_color,
            border_color: border_col,
            shadow_color: Color::TRANSPARENT,
        })
        .on_press(msg)
        .into()
    };

    let primary_btn = move |label: String, msg: FilePickerMessage| -> Element<'static, FilePickerMessage> {
        button(
            container(text(label).size(13))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .height(32)
        .width(Length::Shrink)
        .padding(Padding { top: 0.0, right: 14.0, bottom: 0.0, left: 14.0 })
        .class(CardButtonStyle {
            background: accent_bg,
            background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.35),
            text_color,
            border_color: accent,
            shadow_color: Color::TRANSPARENT,
        })
        .on_press(msg)
        .into()
    };

    // SVG handles
    let close_icon      = svg::Handle::from_memory(include_bytes!("icons/close.svg").as_slice());
    let arrow_up_icon   = svg::Handle::from_memory(include_bytes!("icons/arrow-up.svg").as_slice());
    let arrow_up_icon2  = svg::Handle::from_memory(include_bytes!("icons/arrow-up.svg").as_slice());

    // ── Title bar ─────────────────────────────────────────────────────────────
    let title_bar = row![
        text(state.title.clone())
            .size(15)
            .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .color(text_color),
        Space::with_width(Length::Fill),
        button(
            container(
                svg(close_icon).width(16).height(16)
                    .class(SvgStyle { color: text_color })
            )
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .height(30).width(30)
        .class(CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: accent_bg,
            text_color,
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        })
        .on_press(FilePickerMessage::Cancel),
    ]
    .align_y(Alignment::Center)
    .spacing(8);

    // ── Places sidebar ────────────────────────────────────────────────────────
    let places = build_places();

    let mut places_col = column![].spacing(0).width(Length::Fill);

    // Section label helper
    let section_label = |label: &'static str| -> Element<'static, FilePickerMessage> {
        container(
            text(label).size(10)
                .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.45))
                .font(iced::Font { weight: iced::font::Weight::Bold, ..Default::default() })
        )
        .padding(Padding { top: 8.0, right: 0.0, bottom: 2.0, left: 10.0 })
        .width(Length::Fill)
        .into()
    };

    places_col = places_col.push(section_label("PLACES"));

    // ── Shortcut places (Home, Desktop, etc.) ─────────────────────────────────
    for place in places.iter().filter(|p| !p.is_drive) {
        let is_active = state.current_dir == place.path;
        let place_path = place.path.clone();
        let place_label = place.label.clone();
        let icon_h = svg::Handle::from_memory(include_bytes!("icons/folder.svg").as_slice());
        let icon_col = if is_active { accent } else { Color { a: 0.6, ..icon_color } };

        places_col = places_col.push(
            button(
                row![
                    container(svg(icon_h).width(12).height(12).class(SvgStyle { color: icon_col }))
                        .align_y(Alignment::Center)
                        .padding(Padding { top: 0.0, right: 5.0, bottom: 0.0, left: 8.0 }),
                    text(place_label).size(12)
                        .color(if is_active { text_color } else { dim_color }),
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .height(26).width(Length::Fill)
            .class(CardButtonStyle {
                background: if is_active { Color::from_rgba(accent.r, accent.g, accent.b, 0.18) } else { Color::TRANSPARENT },
                background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.12),
                text_color, border_color: Color::TRANSPARENT, shadow_color: Color::TRANSPARENT,
            })
            .on_press(FilePickerMessage::GoToPlace(place_path)),
        );
    }

    // ── Drives — tree view (expand/collapse) ───────────────────────────────────
    let drives: Vec<&Place> = places.iter().filter(|p| p.is_drive).collect();
    if !drives.is_empty() {
        places_col = places_col.push(
            container(Space::with_height(1)).width(Length::Fill).height(1)
                .style(move |_: &IcedTheme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(sep_color)),
                    ..Default::default()
                })
        );
        places_col = places_col.push(section_label("DRIVES"));

        for drive in drives {
            let is_expanded = state.expanded_drives.contains(&drive.path);
            let is_active   = state.current_dir == drive.path
                || state.current_dir.starts_with(&drive.path);
            let drive_path  = drive.path.clone();
            let drive_label = drive.label.clone();

            // ── Chevron icon (▶ / ▼) rendered as text ────────────────────
            let chevron_char = if is_expanded { "▾" } else { "▸" };
            let chevron_col  = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.5);
            let drive_icon_h = svg::Handle::from_memory(include_bytes!("icons/folder.svg").as_slice());
            let drive_icon_col = if is_active { accent } else { Color { a: 0.7, ..icon_color } };

            let toggle_path = drive_path.clone();
            let nav_path    = drive_path.clone();

            places_col = places_col.push(
                row![
                    // Chevron toggle button
                    button(
                        container(
                            text(chevron_char).size(11).color(chevron_col)
                        )
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    )
                    .height(26).width(20)
                    .class(CardButtonStyle {
                        background: Color::TRANSPARENT,
                        background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.08),
                        text_color, border_color: Color::TRANSPARENT, shadow_color: Color::TRANSPARENT,
                    })
                    .on_press(FilePickerMessage::ToggleDrive(toggle_path)),
                    // Drive label button
                    button(
                        row![
                            container(svg(drive_icon_h).width(12).height(12).class(SvgStyle { color: drive_icon_col }))
                                .align_y(Alignment::Center)
                                .padding(Padding { top: 0.0, right: 5.0, bottom: 0.0, left: 2.0 }),
                            text(drive_label).size(12)
                                .color(if is_active { text_color } else { dim_color }),
                        ]
                        .align_y(Alignment::Center)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    )
                    .height(26).width(Length::Fill)
                    .class(CardButtonStyle {
                        background: if state.current_dir == drive_path {
                            Color::from_rgba(accent.r, accent.g, accent.b, 0.18)
                        } else { Color::TRANSPARENT },
                        background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.12),
                        text_color, border_color: Color::TRANSPARENT, shadow_color: Color::TRANSPARENT,
                    })
                    .on_press(FilePickerMessage::GoToPlace(nav_path)),
                ]
                .spacing(0)
                .width(Length::Fill),
            );

            // ── Sub-entries when expanded ─────────────────────────────────
            if is_expanded {
                // Read one level of subdirs from the drive root
                let sub_dirs: Vec<(String, PathBuf)> = std::fs::read_dir(&drive_path)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        let n = e.file_name().to_string_lossy().to_string();
                        if p.is_dir() && !n.starts_with('.') { Some((n, p)) } else { None }
                    })
                    .take(12) // cap at 12 to avoid a huge list
                    .collect::<Vec<_>>()
                    .into_iter()
                    .collect();

                let mut sorted = sub_dirs;
                sorted.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

                for (sub_name, sub_path) in sorted {
                    let sub_is_active = state.current_dir == sub_path;
                    let sub_nav = sub_path.clone();
                    let sub_icon = svg::Handle::from_memory(include_bytes!("icons/folder.svg").as_slice());
                    let sub_icon_col = if sub_is_active { accent } else { Color { a: 0.55, ..icon_color } };

                    places_col = places_col.push(
                        button(
                            row![
                                // Indent
                                Space::with_width(20),
                                container(svg(sub_icon).width(11).height(11).class(SvgStyle { color: sub_icon_col }))
                                    .align_y(Alignment::Center)
                                    .padding(Padding { top: 0.0, right: 4.0, bottom: 0.0, left: 0.0 }),
                                text(sub_name).size(11)
                                    .color(if sub_is_active { text_color } else { dim_color }),
                            ]
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .height(Length::Fill),
                        )
                        .height(24).width(Length::Fill)
                        .class(CardButtonStyle {
                            background: if sub_is_active { Color::from_rgba(accent.r, accent.g, accent.b, 0.18) } else { Color::TRANSPARENT },
                            background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.10),
                            text_color, border_color: Color::TRANSPARENT, shadow_color: Color::TRANSPARENT,
                        })
                        .on_press(FilePickerMessage::GoToPlace(sub_nav)),
                    );
                }
            }
        }
    }

    // No background — let the panel gradient show through
    let places_sidebar = container(
        scrollable(
            container(places_col)
                .width(Length::Fill)
                .padding(Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
        )
        .height(Length::Fixed(BODY_H))
        .width(Length::Fill)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(2).scroller_width(2),
        ))
    )
    .width(Length::Fixed(PLACES_W));

    // Vertical separator between sidebar and file list
    let vert_sep = container(Space::with_width(1))
        .width(1)
        .height(Length::Fill)
        .style(move |_: &IcedTheme| iced::widget::container::Style {
            background: Some(iced::Background::Color(sep_color)),
            ..Default::default()
        });

    // ── Path breadcrumb ───────────────────────────────────────────────────────
    let can_go_up = state.current_dir.parent().is_some();
    let path_str = {
        let s = state.current_dir.to_string_lossy().to_string();
        if s.len() > 38 { format!("…{}", &s[s.len().saturating_sub(37)..]) } else { s }
    };

    let up_icon_color = if can_go_up { text_color }
        else { Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.28) };

    let up_row = row![
        svg(if can_go_up { arrow_up_icon } else { arrow_up_icon2 })
            .width(12).height(12)
            .class(SvgStyle { color: up_icon_color }),
        Space::with_width(4),
        text("Up").size(12).color(up_icon_color),
    ]
    .align_y(Alignment::Center);

    let up_btn: Element<'static, FilePickerMessage> = if can_go_up {
        button(
            container(up_row)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Shrink)
                .height(Length::Fill),
        )
        .height(28)
        .padding(Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 8.0 })
        .class(CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: accent_bg,
            text_color,
            border_color: border_col,
            shadow_color: Color::TRANSPARENT,
        })
        .on_press(FilePickerMessage::GoUp)
        .into()
    } else {
        let dim = up_icon_color;
        button(
            container(up_row)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Shrink)
                .height(Length::Fill),
        )
        .height(28)
        .padding(Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 8.0 })
        .class(CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: Color::TRANSPARENT,
            text_color: dim,
            border_color: Color::from_rgba(border_col.r, border_col.g, border_col.b, 0.28),
            shadow_color: Color::TRANSPARENT,
        })
        .into()
    };

    let is_save_mode = matches!(state.mode, FilePickerMode::Save { .. });

    let path_row: Element<'static, FilePickerMessage> = {
        let mut r = row![
            up_btn,
            Space::with_width(6),
            container(text(path_str).size(11).color(dim_color))
                .width(Length::Fill)
                .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center)
        .spacing(0);

        if is_save_mode {
            r = r.push(Space::with_width(8)).push(
                button(
                    container(text("Select Folder").size(12))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .width(Length::Shrink)
                        .height(Length::Fill),
                )
                .height(28)
                .padding(Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 10.0 })
                .class(CardButtonStyle {
                    background: accent_bg,
                    background_hovered: Color::from_rgba(accent.r, accent.g, accent.b, 0.25),
                    text_color,
                    border_color: accent,
                    shadow_color: Color::TRANSPARENT,
                })
                .on_press(FilePickerMessage::Confirm)
            );
        }
        r.into()
    };

    // ── Separator ─────────────────────────────────────────────────────────────
    let sep = move |c: Color| -> Element<'static, FilePickerMessage> {
        container(Space::with_height(1))
            .width(Length::Fill)
            .height(1)
            .style(move |_: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(c)),
                ..Default::default()
            })
            .into()
    };

    // ── File list ─────────────────────────────────────────────────────────────
    let mut list = column![].spacing(1).width(Length::Fill);

    if state.entries.is_empty() {
        list = list.push(
            container(text("(empty directory)").size(12).color(dim_color))
                .padding(Padding { top: 8.0, right: 0.0, bottom: 8.0, left: 8.0 }),
        );
    }

    for entry in &state.entries {
        let is_selected = !entry.is_dir && state.file_name == entry.name;

        let entry_icon_handle = if entry.is_dir {
            svg::Handle::from_memory(include_bytes!("icons/folder.svg").as_slice())
        } else {
            svg::Handle::from_memory(include_bytes!("icons/file.svg").as_slice())
        };
        let entry_icon_color = if entry.is_dir {
            Color { a: 0.85, ..accent }
        } else {
            Color { a: 0.65, ..icon_color }
        };

        let row_style = CardButtonStyle {
            background:         if is_selected { accent_bg } else { Color::TRANSPARENT },
            background_hovered: accent_bg,
            text_color,
            border_color:       if is_selected { accent } else { Color::TRANSPARENT },
            shadow_color:       Color::TRANSPARENT,
        };

        let msg = if entry.is_dir {
            FilePickerMessage::EnterDir(entry.path.clone())
        } else {
            FilePickerMessage::SelectFile(entry.name.clone())
        };

        list = list.push(
            button(
                row![
                    container(
                        svg(entry_icon_handle).width(13).height(13)
                            .class(SvgStyle { color: entry_icon_color })
                    )
                    .align_y(Alignment::Center)
                    .padding(Padding { top: 0.0, right: 6.0, bottom: 0.0, left: 6.0 }),
                    text(entry.name.clone()).size(13).color(text_color),
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .height(28)
            .width(Length::Fill)
            .class(row_style)
            .on_press(msg),
        );
    }

    let file_list = scrollable(
        container(list).width(Length::Fill).padding(Padding::new(4.0)),
    )
    .height(Length::Fixed(LIST_H))
    .width(Length::Fill)
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new().width(4).scroller_width(4),
    ))
    .style(move |_: &IcedTheme, _| {
        use iced::widget::scrollable::{Rail, Scroller};
        let thumb  = Color { a: 0.45, ..accent };
        let track  = Color { a: 0.08, ..accent };
        let make_rail = |bg| Rail {
            background: Some(iced::Background::Color(bg)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            scroller: Scroller { color: thumb, border: Border { radius: 3.0.into(), ..Default::default() } },
        };
        iced::widget::scrollable::Style {
            container: iced::widget::container::Style::default(),
            vertical_rail: make_rail(track),
            horizontal_rail: make_rail(Color::TRANSPARENT),
            gap: None,
        }
    });

    // ── Right pane (path + file list) ─────────────────────────────────────────
    let right_pane = column![
        Space::with_height(8),
        path_row,
        Space::with_height(8),
        sep(sep_color),
        file_list,
    ]
    .spacing(0)
    .width(Length::Fill);

    // ── Main body: places | sep | file list ───────────────────────────────────
    let body = row![
        places_sidebar,
        vert_sep,
        container(right_pane).width(Length::Fill).padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 8.0 }),
    ]
    .spacing(0)
    .width(Length::Fill);

    // ── Bottom row ────────────────────────────────────────────────────────────
    let bottom_row: Element<'static, FilePickerMessage> = if is_save_mode {
        let save_path = {
            let s = state.current_dir.to_string_lossy().to_string();
            if s.len() > 60 { format!("…{}", &s[s.len().saturating_sub(59)..]) } else { s }
        };
        row![
            column![
                text("Save to:").size(10)
                    .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.5)),
                text(save_path).size(12).color(text_color),
            ]
            .spacing(2)
            .width(Length::Fill),
            Space::with_width(8),
            btn("Cancel".to_string(), FilePickerMessage::Cancel),
            Space::with_width(6),
            primary_btn("Select".to_string(), FilePickerMessage::Confirm),
        ]
        .align_y(Alignment::Center)
        .spacing(0)
        .into()
    } else {
        let input = text_input("Selected:", &state.file_name)
            .on_input(FilePickerMessage::FileNameInput)
            .on_submit(FilePickerMessage::Confirm)
            .size(13)
            .padding(Padding { top: 6.0, right: 10.0, bottom: 6.0, left: 10.0 })
            .style(move |_: &IcedTheme, status| {
                use iced::widget::text_input::Style;
                Style {
                    background: iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.08)),
                    border: Border {
                        color: match status {
                            iced::widget::text_input::Status::Focused => accent,
                            _ => Color::from_rgba(0.5, 0.5, 0.5, 0.4),
                        },
                        width: 1.5,
                        radius: 5.0.into(),
                    },
                    icon: accent,
                    placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                    value: text_color,
                    selection: Color::from_rgba(accent.r, accent.g, accent.b, 0.3),
                }
            });

        let can_confirm = !state.file_name.trim().is_empty();
        let open_btn: Element<'static, FilePickerMessage> = if can_confirm {
            primary_btn("Open".to_string(), FilePickerMessage::Confirm)
        } else {
            container(text("Open").size(13)
                .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.3)))
            .height(32)
            .padding(Padding { top: 0.0, right: 14.0, bottom: 0.0, left: 14.0 })
            .align_y(Alignment::Center)
            .style(move |_: &IcedTheme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
                border: Border { color: Color::from_rgba(0.5, 0.5, 0.5, 0.25), width: 1.5, radius: 5.0.into() },
                ..Default::default()
            })
            .into()
        };

        row![
            container(input).width(Length::Fill),
            Space::with_width(8),
            btn("Cancel".to_string(), FilePickerMessage::Cancel),
            Space::with_width(6),
            open_btn,
        ]
        .align_y(Alignment::Center)
        .spacing(0)
        .into()
    };

    let error_row: Element<'static, FilePickerMessage> = if let Some(ref err) = state.error {
        text(err.clone()).size(11).color(Color::from_rgb(0.9, 0.3, 0.3)).into()
    } else {
        Space::with_height(0).into()
    };

    let bottom_row = container(bottom_row)
        .width(Length::Fill)
        .height(40)
        .align_y(Alignment::Center);

    // ── Gradient ──────────────────────────────────────────────────────────────
    let grad_top = Color {
        r: bg.r * (1.0 - accent.a * 0.15) + accent.r * (accent.a * 0.15),
        g: bg.g * (1.0 - accent.a * 0.15) + accent.g * (accent.a * 0.15),
        b: bg.b * (1.0 - accent.a * 0.15) + accent.b * (accent.a * 0.15),
        a: 1.0,
    };
    let panel_gradient = gradient::Linear::new(Radians(std::f32::consts::PI * 0.75))
        .add_stop(0.0, grad_top)
        .add_stop(1.0, bg);

    // ── Panel ─────────────────────────────────────────────────────────────────
    let mut panel_col = column![
        title_bar,
        Space::with_height(12),
        sep(sep_color),
        // Body: places sidebar + file list side by side
        container(body)
            .width(Length::Fill)
            .height(Length::Fixed(BODY_H)),
        sep(sep_color),
    ]
    .spacing(0)
    .width(Length::Fill);

    // Inject the extra row (e.g. format selector) if provided
    if let Some(extra_elem) = extra {
        panel_col = panel_col.push(extra_elem);
        panel_col = panel_col.push(sep(sep_color));
    }

    panel_col = panel_col
        .push(Space::with_height(12))
        .push(bottom_row)
        .push(error_row);

    let panel = container(panel_col)
        .width(Length::Fixed(PANEL_W))
        .height(Length::Fixed(panel_h))
        .padding(Padding::new(PANEL_PAD))
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

    // ── Full-screen dimmed backdrop + centred panel ───────────────────────────
    // The Overlay widget draws the dim layer and handles backdrop-click detection.
    // We just return the centered panel here; the dim color is applied via the
    // container so it fills the whole viewport.
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







