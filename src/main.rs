#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod theme;
mod button_style;
mod dot_grid;
mod overlay;
mod sidebar;
mod settings;
mod svg_style;
mod config;
mod card;
mod context_menu;
mod app_menu;
mod markdown;
mod custom_text_editor;
mod card_toolbar;
mod card_layer;
mod card_shelf;
mod connection_toolbar;
mod icon_util;
mod positioned;
mod text_document;
mod text_renderer;
mod markdown_parser;
mod text_processor;
mod workspace;
mod workspace_modal;
mod file_picker;
mod import_export;
mod import_export_modal;
mod zoom_bar;

use iced::widget::{button, column, container, row, svg, text, Space, scrollable, text_editor, pick_list, mouse_area, stack, text_input};
use iced::{Element, Length, Point, Rectangle, Theme as IcedTheme, Subscription, Vector, Task};
use iced::{Border, Color, Shadow};
use iced::time;
use iced::event::{self, Event};
use iced::mouse;
use iced::{Padding, Alignment};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use theme::Theme;
use button_style::CardButtonStyle;
use dot_grid::{DotGrid, DotGridMessage};
use overlay::Overlay;
use sidebar::Sidebar;
use settings::{SettingsModal, SettingsCategory};
use svg_style::SvgStyle;
use config::{Config, FontFamily, AccentColor};
use context_menu::ContextMenu;
use app_menu::{AppMenu, AppMenuItem};
use card::{Card, CardIcon, CardType};
use workspace::{WorkspaceFile, BoardData, CardData, ConnectionData};
use workspace_modal::{WorkspaceModalState, WorkspaceModalMessage};
use file_picker::{FilePickerState, FilePickerMessage, FilePickerMode};
use import_export_modal::{ImportExportState, ImportExportMessage, IEKind, ImportExportResult};
use card_toolbar::{CardToolbar, ToolbarItem};
use card_layer::CardLayer;
use card_shelf::{CardShelf, SHELF_HEIGHT, GHOST_CARD_W, GHOST_TOP_BAR_H};
use zoom_bar::ZoomBar;

// Application constants (not user-configurable)
const SIDEBAR_WIDTH: f32 = 250.0;
const DOT_SPACING: f32 = 30.0;
const DOT_RADIUS: f32 = 2.0;
const ANIMATION_DURATION_MS: f32 = 250.0;
/// Auto-save workspace every N seconds
const AUTO_SAVE_INTERVAL_SECS: f32 = 30.0;

// Custom text editor style with visible cursor
struct TransparentTextEditorStyle {
    theme: Theme,
    accent_color: Color,
}

impl text_editor::Catalog for TransparentTextEditorStyle {
    type Class<'a> = IcedTheme;

    fn default<'a>() -> Self::Class<'a> {
        <IcedTheme as Default>::default()
    }

    fn style(&self, _class: &Self::Class<'_>, status: text_editor::Status) -> text_editor::Style {
        let cursor_color = match self.theme {
            Theme::Light => Color::from_rgb8(0, 0, 0),
            Theme::Dark => Color::from_rgb8(255, 255, 255),
        };

        let accent = self.accent_color;
        let accent_glow = self.theme.accent_glow_from(self.accent_color);
        let text_value = match self.theme {
            Theme::Light => Color::from_rgb8(0, 0, 0),
            Theme::Dark => Color::from_rgb8(255, 255, 255),
        };

        match status {
            text_editor::Status::Active => text_editor::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                icon: cursor_color,
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                value: text_value,
                selection: accent_glow,
            },
            text_editor::Status::Hovered => text_editor::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border {
                    color: Color::from_rgba(accent.r, accent.g, accent.b, 0.5),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                icon: cursor_color,
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                value: text_value,
                selection: accent_glow,
            },
            text_editor::Status::Focused => text_editor::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border {
                    color: accent,
                    width: 2.0,
                    radius: 4.0.into(),
                },
                icon: cursor_color,
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                value: text_value,
                selection: accent_glow,
            },
            text_editor::Status::Disabled => text_editor::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                icon: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                value: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                selection: Color::from_rgba(accent.r, accent.g, accent.b, 0.2),
            },
        }
    }
}

const APP_NAME: &str = "Cards";
const APP_VERSION: &str = "0.1.9";

#[derive(Debug, Clone, Copy, PartialEq)]
enum BoardAnimationType {
    None,
    AddBoard,
    DeleteBoard,
    ButtonPositionChange,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct FontSize(f32);

impl FontSize {
    const SIZES: &'static [FontSize] = &[
        FontSize(10.0),
        FontSize(12.0),
        FontSize(14.0),
        FontSize(16.0),
        FontSize(18.0),
        FontSize(20.0),
        FontSize(24.0),
    ];
}

impl std::fmt::Display for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}pt", self.0 as i32)
    }
}

pub fn main() -> iced::Result {
    let config = Config::load();

    // On Windows the .ico is embedded as a Win32 resource via build.rs /
    // winresource, so the OS picks it up automatically without any runtime
    // call here.  On other platforms we leave the icon unset — macOS reads
    // it from the app bundle and Linux from the XDG desktop entry.
    iced::application(APP_NAME, Cards::update, Cards::view)
        .subscription(Cards::subscription)
        .theme(Cards::theme)
        .run_with(move || Cards::new(config))
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleSidebar,
    ToggleTheme,
    SetTheme(Theme),
    ToggleSettings,
    CloseSettings,
    SelectSettingsCategory(SettingsCategory),
    SetSidebarOpenOnStart(bool),
    SetAnimationsEnabled(bool),
    SetFontFamily(FontFamily),
    SetFontSize(f32),
    SetAccentColor(AccentColor),
    Tick(Instant),
    DotGridMessage(DotGridMessage),
    EventOccurred(Event),
    // Context menu messages
    ShowContextMenu(Point),
    HideContextMenu,
    AddCard,
    AddCardOfType(CardType),
    // Card messages
    ShowCardIconMenu(usize),
    ChangeCardIcon(usize, CardIcon),
    ChangeCardColor(usize, Color),
    ChangeCardType(usize, CardType),
    HideCardIconMenu,
    ShowCardTypeMenu(usize),
    HideCardTypeMenu,
    // Card content editing
    StartEditingCard(usize),
    CardEditorAction(usize, text_editor::Action),
    KeyboardInput(iced::keyboard::Event),
    StopEditingCard,
    // Toolbar messages
    FormatBold,
    FormatItalic,
    FormatStrikethrough,
    FormatCode,
    FormatCodeBlock,
    FormatHeading,
    FormatBullet,
    DuplicateCard(usize),
    DeleteCard(usize),
    // Board messages
    AddNewBoard,
    SelectBoard(usize),
    DeleteBoard(usize),
    BoardHover(Option<usize>),
    StartRenamingBoard(usize),
    BoardRenameInput(String),
    FinishRenamingBoard,
    CancelRenamingBoard,
    // Settings messages
    SetNewBoardButtonAtTop(bool),
    SetDebugMode(bool),
    SetConfirmCardDelete(bool),
    // Confirmation dialog messages
    ShowDeleteConfirmDialog(usize),
    ConfirmDeleteCard,
    CancelDeleteCard,
    // App menu messages
    ToggleAppMenu,
    CloseAppMenu,
    MenuFileNewBoard,
    MenuFileNewWorkspace,
    MenuFileOpenWorkspace,
    MenuFileOpenRecent(String),
    MenuFileQuit,
            MenuViewResetCanvas,
            MenuViewToggleSidebar,
            MenuViewToggleTheme,
    MenuHelpAbout,
    MenuHelpKeyboardShortcuts,
    // Recent submenu hover state
    MenuRecentSubmenuOpen,
    MenuRecentSubmenuClose,
    // Workspace messages
    WorkspaceModal(WorkspaceModalMessage),
    WorkspaceLoaded(WorkspaceFile),
    SaveWorkspace,
    // Import / Export
    MenuFileImportExport,
    MenuImportExportSubmenuOpen,
    MenuImportExportSubmenuClose,
    MenuExportWorkspace,
    MenuExportBoard,
    MenuImportWorkspace,
    MenuImportBoard,
    ImportExport(ImportExportMessage),
    ShelfDragStart(CardType, Point),
    // Connection toolbar actions
    ConnSetLineStyle { from_card: usize, from_side: card::CardSide, to_card: usize, to_side: card::CardSide, style: card::LineStyle },
    ConnToggleArrowFrom { from_card: usize, from_side: card::CardSide, to_card: usize, to_side: card::CardSide },
    ConnToggleArrowTo   { from_card: usize, from_side: card::CardSide, to_card: usize, to_side: card::CardSide },
    ConnDelete          { from_card: usize, from_side: card::CardSide, to_card: usize, to_side: card::CardSide },
    // Image picker
    OpenImagePicker(usize),
    ImagePickerMsg(FilePickerMessage),
    CloseImagePicker,
    // Zoom
    ZoomIn,
    ZoomOut,
    ZoomReset,
    MenuViewResetZoom,
}

struct Cards {
    theme: Theme,
    sidebar_open: bool,
    sidebar_offset: f32,
    animating: bool,
    animation_start_offset: f32,
    animation_progress: f32,
    dot_grid: DotGrid,
    canvas_offset: Vector,
    // Canvas recentering animation
    canvas_animating: bool,
    canvas_animation_start: Vector,
    canvas_animation_progress: f32,
    // Settings state
    settings_open: bool,
    settings_category: SettingsCategory,
    settings_animating: bool,
    settings_animation_progress: f32,
    settings_opening: bool,  // Track if opening (true) or closing (false)
    // Theme transition
    theme_transitioning: bool,
    theme_transition_progress: f32,
    next_theme: Option<Theme>,
    // Context menu state
    context_menu_position: Option<Point>,
    pending_card_position: Option<Point>,  // Store position for card creation
    mouse_position: Option<Point>,          // Last known screen mouse position
    // Card customization menu
    card_icon_menu_position: Option<Point>,
    card_icon_menu_card_id: Option<usize>,
    // Card type menu (popup on right icon click)
    card_type_menu_position: Option<Point>,
    card_type_menu_card_id: Option<usize>,
    // Card editing state
    editing_card_id: Option<usize>,
    selected_card_id: Option<usize>,  // Track selected card for toolbar
    // Multi-card box-selection
    selected_card_ids: std::collections::HashSet<usize>,
    /// Previous world position of the batch-dragged card (for delta computation)
    last_drag_world_pos: Option<Point>,
    // Board management
    boards: Vec<String>,  // List of board names
    active_board_index: usize,  // Currently active board
    hovered_board_index: Option<usize>,  // Track which board is being hovered
    editing_board_index: Option<usize>,  // Track which board is being renamed
    board_rename_value: String,  // Current value during rename
    board_cards: HashMap<usize, Vec<card::Card>>,  // Store cards for each board
    board_connections: HashMap<usize, Vec<card::Connection>>,  // Store connections per board
    selected_conn: Option<card::Connection>,  // Currently selected connection (click-to-select)
    // Board animations
    board_list_animating: bool,  // Animation for add/delete/reorder
    board_list_animation_progress: f32,
    board_list_animation_type: BoardAnimationType,
    animating_board_index: Option<usize>,  // Track which board is being animated
    // Configuration
    config: Config,
    // Active accent color (resolved from config)
    accent_color: Color,
    // Pending card deletion confirmation
    confirm_delete_card_id: Option<usize>,
    // App menu state
    app_menu_open: bool,
    app_menu_animating: bool,
    app_menu_opening: bool,
    app_menu_animation_progress: f32,
    // Cache SVG handles
    icon_menu_left: svg::Handle,
    icon_menu_right: svg::Handle,
    icon_moon: svg::Handle,
    icon_sun: svg::Handle,
    icon_settings: svg::Handle,
    icon_close: svg::Handle,
    icon_add: svg::Handle,
    icon_duplicate: svg::Handle,
    icon_delete: svg::Handle,
    icon_app: svg::Handle,
    icon_menu: svg::Handle,
    icon_type_text: svg::Handle,
    icon_type_markdown: svg::Handle,
    icon_fmt_bold: svg::Handle,
    icon_fmt_italic: svg::Handle,
    icon_fmt_strikethrough: svg::Handle,
    icon_fmt_code: svg::Handle,
    icon_fmt_codeblock: svg::Handle,
    icon_fmt_heading: svg::Handle,
    icon_fmt_bullet: svg::Handle,
    icon_type_image: svg::Handle,
    // Image picker (card_id, picker state)
    image_picker: Option<(usize, FilePickerState)>,
    /// Zoom level for the canvas (1.0 = 100%)
    canvas_zoom: f32,
    /// Whether the Ctrl key is currently held (for Ctrl+scroll zoom)
    ctrl_held: bool,
    window_size: iced::Size,
    last_tick: Instant,
    // Workspace persistence
    workspace_path: Option<std::path::PathBuf>,
    workspace_modal: Option<WorkspaceModalState>,
    /// True when the "Open Recent" submenu is hovered open
    recent_submenu_open: bool,
    /// True when unsaved changes exist — triggers a save at the end of update()
    workspace_dirty: bool,
    /// True when only the canvas position changed (debounced save)
    canvas_position_dirty: bool,
    /// Timestamp of the last file write, for debouncing position-only saves
    last_save_instant: Instant,
    /// Seconds since last save; we save every AUTO_SAVE_INTERVAL_SECS
    time_since_last_save: f32,
    // Import / Export modal
    import_export_modal: Option<ImportExportState>,
    import_export_submenu_open: bool,
    // Shelf drag state
    shelf_drag: Option<CardType>,
}

const SIDEBAR_HIDDEN_OFFSET: f32 = -280.0;
const BOARD_ANIMATION_DURATION_MS: f32 = 150.0; // Faster animation for board changes

impl Cards {
    fn new(mut config: Config) -> (Self, Task<Message>) {
        // Clean up any recent workspaces whose files no longer exist
        config.prune_missing_recents();

        let theme: Theme = config.appearance.theme.into();
        let sidebar_open = config.general.sidebar_open_on_start;
        let sidebar_offset = if sidebar_open { 0.0 } else { SIDEBAR_HIDDEN_OFFSET };
        let accent_color = config.appearance.accent_color.to_color();

        let mut dot_grid = DotGrid::new(theme.dot_color(), theme.background());
        dot_grid.set_dot_spacing(DOT_SPACING);
        dot_grid.set_dot_radius(DOT_RADIUS);
        dot_grid.set_card_colors(
            theme.card_background(),
            theme.card_border(),
            theme.card_text(),
        );
        dot_grid.set_accent_color(config.appearance.accent_color.to_color());
        // Set debug mode from config
        dot_grid.set_debug_mode(config.general.debug_mode);
        // Auto-select best available font if the configured one isn't installed
        if !config.appearance.font.family.is_available() {
            if let Some(best) = FontFamily::best_available() {
                config.appearance.font.family = best;
            }
            // If none of our named fonts are available, iced::Font::MONOSPACE
            // (the system monospace) will be used via to_iced_font() fallback.
        }

        // Set font from config
        let font = config.appearance.font.family.to_iced_font();
        if config.general.debug_mode {
            println!("DEBUG: Initializing with font family: {:?}, size: {}", config.appearance.font.family, config.appearance.font.size);
        }
        dot_grid.set_font(font, config.appearance.font.size);
        if config.general.debug_mode {
            println!("DEBUG: Font applied to DotGrid");
        }

        let mut cards = Cards {
            theme,
            sidebar_open,
            sidebar_offset,
            animating: false,
            animation_start_offset: 0.0,
            animation_progress: 0.0,
            dot_grid,
            canvas_offset: Vector::new(0.0, 0.0),
            canvas_animating: false,
            canvas_animation_start: Vector::new(0.0, 0.0),
            canvas_animation_progress: 0.0,
            settings_open: false,
            settings_category: SettingsCategory::default(),
            settings_animating: false,
            settings_animation_progress: 0.0,
            settings_opening: false,
            theme_transitioning: false,
            theme_transition_progress: 0.0,
            next_theme: None,
            context_menu_position: None,
            pending_card_position: None,
            mouse_position: None,
            card_icon_menu_position: None,
            card_icon_menu_card_id: None,
            card_type_menu_position: None,
            card_type_menu_card_id: None,
            editing_card_id: None,
            selected_card_id: None,
            selected_card_ids: std::collections::HashSet::new(),
            last_drag_world_pos: None,
            boards: vec!["Board 1".to_string()],
            active_board_index: 0,
            hovered_board_index: None,
            editing_board_index: None,
            board_rename_value: String::new(),
            board_cards: {
                let mut map = HashMap::new();
                map.insert(0, Vec::new());  // Initialize first board with empty cards
                map
            },
            board_connections: {
                let mut map = HashMap::new();
                map.insert(0, Vec::new());
                map
            },
            selected_conn: None,
            board_list_animating: false,
            board_list_animation_progress: 0.0,
            board_list_animation_type: BoardAnimationType::None,
            animating_board_index: None,
            config,
            accent_color,
            confirm_delete_card_id: None,
            app_menu_open: false,
            app_menu_animating: false,
            app_menu_opening: false,
            app_menu_animation_progress: 0.0,
            icon_menu_left: svg::Handle::from_memory(include_bytes!("icons/menu-left.svg")),
            icon_menu_right: svg::Handle::from_memory(include_bytes!("icons/menu-right.svg")),
            icon_moon: svg::Handle::from_memory(include_bytes!("icons/moon.svg")),
            icon_sun: svg::Handle::from_memory(include_bytes!("icons/sun.svg")),
            icon_settings: svg::Handle::from_memory(include_bytes!("icons/settings.svg")),
            icon_close: svg::Handle::from_memory(include_bytes!("icons/close.svg")),
            icon_add: svg::Handle::from_memory(include_bytes!("icons/add.svg")),
            icon_duplicate: svg::Handle::from_memory(include_bytes!("icons/duplicate.svg")),
            icon_delete: svg::Handle::from_memory(include_bytes!("icons/delete.svg")),
            icon_app: svg::Handle::from_memory(include_bytes!("icons/app.svg")),
            icon_menu: svg::Handle::from_memory(include_bytes!("icons/menu.svg")),
            icon_type_text: svg::Handle::from_memory(include_bytes!("icons/type-text.svg")),
            icon_type_markdown: svg::Handle::from_memory(include_bytes!("icons/type-markdown.svg")),
            icon_fmt_bold: svg::Handle::from_memory(include_bytes!("icons/fmt-bold.svg")),
            icon_fmt_italic: svg::Handle::from_memory(include_bytes!("icons/fmt-italic.svg")),
            icon_fmt_strikethrough: svg::Handle::from_memory(include_bytes!("icons/fmt-strikethrough.svg")),
            icon_fmt_code: svg::Handle::from_memory(include_bytes!("icons/fmt-code.svg")),
            icon_fmt_codeblock: svg::Handle::from_memory(include_bytes!("icons/fmt-codeblock.svg")),
            icon_fmt_heading: svg::Handle::from_memory(include_bytes!("icons/fmt-heading.svg")),
            icon_fmt_bullet: svg::Handle::from_memory(include_bytes!("icons/fmt-bullet.svg")),
            icon_type_image: svg::Handle::from_memory(include_bytes!("icons/type-image.svg")),
            image_picker: None,
            canvas_zoom: 1.0,
            ctrl_held: false,
            window_size: iced::Size::new(800.0, 600.0),
            last_tick: Instant::now(),
            workspace_path: None,
            workspace_modal: None,
            recent_submenu_open: false,
            workspace_dirty: false,
            canvas_position_dirty: false,
            last_save_instant: Instant::now(),
            time_since_last_save: 0.0,
            import_export_modal: None,
            import_export_submenu_open: false,
            shelf_drag: None,
        };
        cards.update_exclude_region();

        // ── Workspace bootstrap ───────────────────────────────────────────────
        // Try to load the last opened workspace from the config.
        // If none (first launch) or the file is gone / corrupt, show the modal.
        let last_ws = cards.config.general.last_workspace.clone();
        let needs_modal = if let Some(ref path_str) = last_ws {
            let path = std::path::PathBuf::from(path_str);
            if path.exists() {
                match WorkspaceFile::load(&path) {
                    Ok(ws) => {
                        cards.apply_workspace(ws, path);
                        false
                    }
                    Err(e) => {
                        eprintln!("Failed to load last workspace '{}': {}", path_str, e);
                        true
                    }
                }
            } else {
                eprintln!("Last workspace file not found: {}", path_str);
                true
            }
        } else {
            true  // First launch — no workspace recorded
        };

        if needs_modal {
            cards.workspace_modal = Some(WorkspaceModalState::Idle);
        }

        (cards, Task::none())
    }

    // Helper function to convert icondata to complete SVG

    fn update(&mut self, message: Message) -> Task<Message> {
        // Keep canvas blocked state in sync with modal state on every update tick.
        self.sync_grid_blocked();

        match message {
            Message::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                if self.config.general.enable_animations {
                    self.animating = true;
                    self.animation_start_offset = self.sidebar_offset;
                    self.animation_progress = 0.0;
                } else {
                    self.sidebar_offset = if self.sidebar_open { 0.0 } else { SIDEBAR_HIDDEN_OFFSET };
                    self.update_exclude_region();
                }
            }
            Message::ToggleTheme => {
                let new_theme = self.theme.toggle();
                if self.config.general.enable_animations {
                    // Start theme transition animation
                    self.next_theme = Some(new_theme);
                    self.theme_transitioning = true;
                    self.theme_transition_progress = 0.0;
                } else {
                    // Instant theme change
                    self.theme = new_theme;
                    self.update_theme_colors();
                }
                if let Err(e) = self.config.set_theme(new_theme) {
                    eprintln!("Failed to save theme: {}", e);
                }
            }
            Message::SetTheme(theme) => {
                if self.theme != theme {
                    if self.config.general.enable_animations {
                        // Start theme transition animation
                        self.next_theme = Some(theme);
                        self.theme_transitioning = true;
                        self.theme_transition_progress = 0.0;
                    } else {
                        // Instant theme change
                        self.theme = theme;
                        self.update_theme_colors();
                    }
                    if let Err(e) = self.config.set_theme(theme) {
                        eprintln!("Failed to save theme: {}", e);
                    }
                }
            }
            Message::ToggleSettings => {
                if self.config.general.enable_animations {
                    if self.settings_open {
                        // Start closing animation
                        self.settings_opening = false;
                        self.settings_animating = true;
                        self.settings_animation_progress = 0.0;
                    } else {
                        // Start opening animation
                        self.settings_open = true;
                        self.settings_opening = true;
                        self.settings_animating = true;
                        self.settings_animation_progress = 0.0;
                        self.dot_grid.set_effect_enabled(false);
                        self.update_exclude_region();
                    }
                } else {
                    // Instant toggle
                    self.settings_open = !self.settings_open;
                    self.dot_grid.set_effect_enabled(!self.settings_open && self.config.general.enable_animations);
                    self.update_exclude_region();
                }
                self.context_menu_position = None;
                self.pending_card_position = None;
            }
            Message::CloseSettings => {
                if self.config.general.enable_animations && self.settings_open {
                    // Start closing animation
                    self.settings_opening = false;
                    self.settings_animating = true;
                    self.settings_animation_progress = 0.0;
                } else {
                    // Instant close
                    self.settings_open = false;
                    self.dot_grid.set_effect_enabled(self.config.general.enable_animations);
                    self.update_exclude_region();
                }
            }
            Message::SelectSettingsCategory(category) => {
                self.settings_category = category;
            }
            Message::SetSidebarOpenOnStart(open) => {
                if let Err(e) = self.config.set_sidebar_open_on_start(open) {
                    eprintln!("Failed to save sidebar setting: {}", e);
                }
            }
            Message::SetAnimationsEnabled(enabled) => {
                if let Err(e) = self.config.set_animations_enabled(enabled) {
                    eprintln!("Failed to save animations setting: {}", e);
                }
                self.dot_grid.set_effect_enabled(enabled && !self.settings_open);
            }
            Message::SetFontFamily(family) => {
                if self.config.general.debug_mode {
                    println!("DEBUG: SetFontFamily called with {:?}", family);
                }
                if let Err(e) = self.config.set_font_family(family) {
                    eprintln!("Failed to save font family setting: {}", e);
                }
                // Update DotGrid with new font
                let font = family.to_iced_font();
                self.dot_grid.set_font(font, self.config.appearance.font.size);
                if self.config.general.debug_mode {
                    println!("DEBUG: Font applied to DotGrid");
                }
            }
            Message::SetFontSize(size) => {
                if self.config.general.debug_mode {
                    println!("DEBUG: SetFontSize called with {}", size);
                }
                if let Err(e) = self.config.set_font_size(size) {
                    eprintln!("Failed to save font size setting: {}", e);
                }
                // Update DotGrid with new font size
                let font = self.config.appearance.font.family.to_iced_font();
                self.dot_grid.set_font(font, size);
                if self.config.general.debug_mode {
                    println!("DEBUG: Font size applied to DotGrid");
                }
            }
            Message::SetAccentColor(accent) => {
                self.accent_color = accent.to_color();
                self.dot_grid.set_accent_color(self.accent_color);
                if let Err(e) = self.config.set_accent_color(accent) {
                    eprintln!("Failed to save accent color setting: {}", e);
                }
            }
            Message::Tick(_instant) => {
                // Calculate delta time since last tick
                let now = Instant::now();
                let delta_time = now.duration_since(self.last_tick).as_secs_f32();
                self.last_tick = now;

                // Update card animations
                self.dot_grid.update_card_animation(delta_time);

                // Clear cards cache if editing to show blinking cursor
                if self.editing_card_id.is_some() {
                    self.dot_grid.clear_cards_cache();
                }

                // Advance connection animation when a drag is in progress
                if self.dot_grid.pending_conn().is_some() {
                    self.dot_grid.advance_conn_anim(delta_time);
                    self.dot_grid.clear_cards_cache();
                }

                let animation_duration = ANIMATION_DURATION_MS;

                // Animate sidebar
                self.animation_progress += 16.0 / animation_duration;

                if self.animation_progress >= 1.0 {
                    self.animation_progress = 1.0;
                    self.animating = false;
                    self.sidebar_offset = if self.sidebar_open { 0.0 } else { SIDEBAR_HIDDEN_OFFSET };
                } else {
                    let t = self.animation_progress;
                    let eased = 1.0 - (1.0 - t).powi(3);

                    let target = if self.sidebar_open { 0.0 } else { SIDEBAR_HIDDEN_OFFSET };
                    self.sidebar_offset = self.animation_start_offset + (target - self.animation_start_offset) * eased;
                }

                // Animate canvas recentering
                if self.canvas_animating {
                    self.canvas_animation_progress += 16.0 / animation_duration;

                    if self.canvas_animation_progress >= 1.0 {
                        self.canvas_animation_progress = 1.0;
                        self.canvas_animating = false;
                        self.canvas_offset = Vector::new(0.0, 0.0);
                        self.canvas_position_dirty = true;
                    } else {
                        let t = self.canvas_animation_progress;
                        // Use ease-out cubic for smooth deceleration
                        let eased = 1.0 - (1.0 - t).powi(3);

                        let target = Vector::new(0.0, 0.0);
                        self.canvas_offset.x = self.canvas_animation_start.x + (target.x - self.canvas_animation_start.x) * eased;
                        self.canvas_offset.y = self.canvas_animation_start.y + (target.y - self.canvas_animation_start.y) * eased;
                    }

                    self.dot_grid.set_offset(self.canvas_offset);
                }

                // Animate board list changes (faster animation)
                if self.board_list_animating {
                    self.board_list_animation_progress += 16.0 / BOARD_ANIMATION_DURATION_MS;

                    if self.board_list_animation_progress >= 1.0 {
                        if self.config.general.debug_mode {
                            println!("DEBUG: Animation complete");
                        }
                        self.board_list_animation_progress = 1.0;
                        self.board_list_animating = false;

                        // Handle post-animation actions
                        match self.board_list_animation_type {
                            BoardAnimationType::DeleteBoard => {
                                // Actually delete the board after animation
                                if let Some(index) = self.animating_board_index {
                                    if index < self.boards.len() {
                                        self.boards.remove(index);

                                        // Remove the deleted board's cards and reindex
                                        let mut new_board_cards = HashMap::new();
                                        for (board_idx, cards) in self.board_cards.iter() {
                                            if *board_idx < index {
                                                // Boards before the deleted one keep their index
                                                new_board_cards.insert(*board_idx, cards.clone());
                                            } else if *board_idx > index {
                                                // Boards after the deleted one shift down by 1
                                                new_board_cards.insert(*board_idx - 1, cards.clone());
                                            }
                                            // Skip board at 'index' - it's being deleted
                                        }
                                        self.board_cards = new_board_cards;

                                        // Adjust active board index if needed
                                        if self.active_board_index >= self.boards.len() {
                                            self.active_board_index = self.boards.len().saturating_sub(1);
                                            let new_cards = self.board_cards.get(&self.active_board_index).cloned().unwrap_or_default();
                                            self.load_cards_with_positions(new_cards);
                                        } else if self.active_board_index > index {
                                            self.active_board_index = self.active_board_index.saturating_sub(1);
                                            let new_cards = self.board_cards.get(&self.active_board_index).cloned().unwrap_or_default();
                                            self.load_cards_with_positions(new_cards);
                                        } else if self.active_board_index == index {
                                            let new_cards = self.board_cards.get(&self.active_board_index).cloned().unwrap_or_default();
                                            self.load_cards_with_positions(new_cards);
                                        }

                                        self.dot_grid.clear_cards_cache();
                                        self.workspace_dirty = true;
                                    }
                                }
                            }
                            _ => {}
                        }

                        self.board_list_animation_type = BoardAnimationType::None;
                        self.animating_board_index = None;
                    } else {
                        if self.config.general.debug_mode {
                            println!("DEBUG: Animation progress: {:.2}", self.board_list_animation_progress);
                        }
                    }
                }

                // Animate settings modal
                if self.settings_animating {
                    self.settings_animation_progress += 16.0 / 200.0; // 200ms animation

                    if self.settings_animation_progress >= 1.0 {
                        self.settings_animation_progress = 1.0;
                        self.settings_animating = false;

                        if !self.settings_opening {
                            // Animation complete for closing
                            self.settings_open = false;
                            self.dot_grid.set_effect_enabled(self.config.general.enable_animations);
                            self.update_exclude_region();
                        }
                    }
                }

                // Animate app menu open/close
                if self.app_menu_animating {
                    // 180ms animation
                    let delta = 16.0 / 180.0;
                    if self.app_menu_opening {
                        self.app_menu_animation_progress += delta;
                        if self.app_menu_animation_progress >= 1.0 {
                            self.app_menu_animation_progress = 1.0;
                            self.app_menu_animating = false;
                        }
                    } else {
                        self.app_menu_animation_progress -= delta;
                        if self.app_menu_animation_progress <= 0.0 {
                            self.app_menu_animation_progress = 0.0;
                            self.app_menu_animating = false;
                            self.app_menu_open = false;
                        }
                    }
                }

                // Animate theme transition
                if self.theme_transitioning {
                    self.theme_transition_progress += 16.0 / 1000.0; // 1000ms (1 second) animation for smooth diagonal wipe

                    // Switch theme early (at 5% progress) so the wipe reveals the new theme
                    if self.theme_transition_progress >= 0.05 && self.next_theme.is_some() {
                        if let Some(new_theme) = self.next_theme {
                            if self.theme != new_theme {
                                self.theme = new_theme;
                                self.update_theme_colors();
                            }
                        }
                    }

                    if self.theme_transition_progress >= 1.0 {
                        self.theme_transition_progress = 1.0;
                        self.theme_transitioning = false;
                        self.next_theme = None;
                    }
                }

                // Save current board's cards to keep them synced
                self.save_current_board_cards();

                // Flush any dirty flag set during Tick (e.g. animated board delete)
                if self.workspace_dirty && self.workspace_path.is_some() {
                    self.workspace_dirty = false;
                    self.canvas_position_dirty = false;
                    self.last_save_instant = Instant::now();
                    self.save_workspace_to_file();
                }

                self.update_exclude_region();
            }
            Message::DotGridMessage(msg) => {
                // Block all canvas interaction while any modal is open
                if self.workspace_modal.is_some()
                    || self.settings_open
                    || self.confirm_delete_card_id.is_some()
                {
                    return Task::none();
                }
                match msg {
                    DotGridMessage::Pan(delta) => {
                        self.context_menu_position = None;
                        self.pending_card_position = None;
                        self.card_icon_menu_position = None;
                        self.card_icon_menu_card_id = None;
                        self.selected_conn = None;
                        self.canvas_offset.x += delta.x;
                        self.canvas_offset.y += delta.y;
                        self.dot_grid.set_offset(self.canvas_offset);
                        self.canvas_position_dirty = true;
                    }
                    DotGridMessage::RightClick(pos) => {
                        // Check if click is within sidebar bounds
                        if !self.is_point_in_sidebar(pos) {
                            self.context_menu_position = Some(pos);
                            self.pending_card_position = Some(pos);
                            self.card_icon_menu_position = None;
                            self.card_icon_menu_card_id = None;
                        }
                    }
                    DotGridMessage::CardRightClickIcon(card_id) => {
                        if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id) {
                            let screen_pos = Point::new(
                                card.current_position.x + self.canvas_offset.x + 25.0,
                                card.current_position.y + self.canvas_offset.y + 25.0,
                            );
                            self.card_icon_menu_position = Some(screen_pos);
                            self.card_icon_menu_card_id = Some(card_id);
                            self.context_menu_position = None;
                            self.pending_card_position = None;
                            self.card_type_menu_position = None;
                            self.card_type_menu_card_id = None;
                        }
                    }
                    DotGridMessage::CardTypeIconClick(card_id) => {
                        if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id) {
                            // Position menu just below the right icon
                            let screen_pos = Point::new(
                                card.current_position.x + self.canvas_offset.x + card.width - 30.0,
                                card.current_position.y + self.canvas_offset.y + 30.0,
                            );
                            self.card_type_menu_position = Some(screen_pos);
                            self.card_type_menu_card_id = Some(card_id);
                            self.card_icon_menu_position = None;
                            self.card_icon_menu_card_id = None;
                            self.context_menu_position = None;
                            self.pending_card_position = None;
                        }
                    }
                    DotGridMessage::CardLeftClickBar(card_id, _pos) => {
                        // Start dragging — stop editing but keep (or set) selection
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            card.is_dragging = true;
                            card.is_editing = false;
                        }
                        self.editing_card_id = None;
                        self.selected_conn = None;
                        self.last_drag_world_pos = None;
                        // Bring card to front so it renders on top while dragging
                        self.dot_grid.bring_card_to_front(card_id);
                        // If card is not part of multi-selection, clear multi-select.
                        // Do NOT change single-card selection here — selection is
                        // set/confirmed on release (CardDrop).  Keeping the existing
                        // selection state means the view doesn't restructure on press,
                        // so the drag gesture registers immediately without a second click.
                        if !self.selected_card_ids.contains(&card_id) {
                            self.selected_card_ids.clear();
                            self.dot_grid.clear_selected_cards();
                            // leave selected_card_id / single_selected_card unchanged
                        } else {
                            // Mark all selected cards as dragging too
                            for card in self.dot_grid.cards_mut().iter_mut() {
                                if self.selected_card_ids.contains(&card.id) {
                                    card.is_dragging = true;
                                }
                            }
                        }
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CardLeftClickBody(card_id) => {
                        self.selected_conn = None;
                        // Special case: if card_id is usize::MAX, it means stop editing
                        if card_id == usize::MAX {
                            // Stop editing any card and update checkbox positions
                            let card_ids: Vec<usize> = self.dot_grid.cards_mut()
                                .iter()
                                .filter(|c| c.is_editing)
                                .map(|c| c.id)
                                .collect();

                            for card in self.dot_grid.cards_mut().iter_mut() {
                                if card.is_editing {
                                    card.is_editing = false;
                                }
                            }

                            // Update checkbox + link positions for cards that were editing
                            for id in card_ids {
                                self.dot_grid.update_card_checkbox_positions(id);
                                self.dot_grid.update_card_link_positions(id);
                            }

                            self.editing_card_id = None;
                            self.selected_card_id = None;
                            self.dot_grid.set_single_selected_card(None);
                            self.dot_grid.clear_cards_cache();
                        } else {
                            // First, stop editing ALL cards and update their checkbox positions
                            let previously_editing: Vec<usize> = self.dot_grid.cards_mut()
                                .iter()
                                .filter(|c| c.is_editing)
                                .map(|c| c.id)
                                .collect();

                            for card in self.dot_grid.cards_mut().iter_mut() {
                                card.is_editing = false;
                            }

                            // Update checkbox + link positions for previously editing cards
                            for id in previously_editing {
                                self.dot_grid.update_card_checkbox_positions(id);
                                self.dot_grid.update_card_link_positions(id);
                            }

                            // For image cards, open the image picker instead of editing
                            let is_image_card = self.dot_grid.cards().iter()
                                .find(|c| c.id == card_id)
                                .map(|c| c.card_type == CardType::Image)
                                .unwrap_or(false);

                            if is_image_card {
                                self.selected_card_id = Some(card_id);
                                self.selected_card_ids.clear();
                                self.dot_grid.clear_selected_cards();
                                self.dot_grid.set_single_selected_card(Some(card_id));
                                self.context_menu_position = None;
                                self.card_icon_menu_position = None;
                                // Open image file picker
                                let start_dir = dirs::picture_dir()
                                    .or_else(dirs::home_dir)
                                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                                let picker = FilePickerState::new(
                                    FilePickerMode::Open { filter_exts: vec![
                                        "png".into(), "jpg".into(), "jpeg".into(),
                                        "gif".into(), "bmp".into(), "webp".into(),
                                        "svg".into(),
                                    ]},
                                    start_dir,
                                    "Select Image",
                                );
                                self.image_picker = Some((card_id, picker));
                                self.dot_grid.clear_cards_cache();
                            } else {
                            // Start editing the card and select it
                            self.editing_card_id = Some(card_id);
                            self.selected_card_id = Some(card_id);
                            // Single click clears box selection
                            self.selected_card_ids.clear();
                            self.dot_grid.clear_selected_cards();
                            // Editing border handles visual — clear single-select indicator
                            self.dot_grid.set_single_selected_card(None);
                            // Bring to front for correct layering
                            self.dot_grid.bring_card_to_front(card_id);
                            if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                card.is_editing = true;
                                // Move cursor to end of text
                                card.content.move_cursor_to_end();
                            }
                            // Close any open menus
                            self.context_menu_position = None;
                            self.card_icon_menu_position = None;
                            self.dot_grid.clear_cards_cache();
                            }
                        }
                    }
                    DotGridMessage::CardDrag(card_id, pos, drag_offset) => {
                        let world_pos = Point::new(
                            pos.x - self.canvas_offset.x - drag_offset.x,
                            pos.y - self.canvas_offset.y - drag_offset.y,
                        );

                        if self.selected_card_ids.contains(&card_id) && self.selected_card_ids.len() > 1 {
                            // Batch move: apply delta to all selected cards
                            if let Some(prev) = self.last_drag_world_pos {
                                let dx = world_pos.x - prev.x;
                                let dy = world_pos.y - prev.y;
                                let selected = self.selected_card_ids.clone();
                                for card in self.dot_grid.cards_mut().iter_mut() {
                                    if selected.contains(&card.id) {
                                        let new_pos = Point::new(
                                            card.current_position.x + dx,
                                            card.current_position.y + dy,
                                        );
                                        card.target_position = new_pos;
                                        card.current_position = new_pos;
                                    }
                                }
                            } else {
                                // First frame of batch drag: just move the primary card
                                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                    if card.is_dragging {
                                        card.target_position = world_pos;
                                        card.current_position = world_pos;
                                    }
                                }
                            }
                            self.last_drag_world_pos = Some(world_pos);
                        } else {
                            // Single card drag
                            if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                if card.is_dragging {
                                    card.target_position = world_pos;
                                    card.current_position = world_pos;
                                }
                            }
                        }
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CardResizeStart(card_id, _pos) => {
                        // Resize start is handled in canvas state
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CardResize(card_id, pos) => {
                        let dot_spacing = self.dot_grid.dot_spacing();
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            // Calculate world position
                            let world_pos = Point::new(
                                pos.x - self.canvas_offset.x,
                                pos.y - self.canvas_offset.y,
                            );

                            // Calculate new size from mouse position
                            let new_width = (world_pos.x - card.current_position.x).max(Card::MIN_WIDTH);
                            let new_height = (world_pos.y - card.current_position.y).max(Card::MIN_HEIGHT);

                            // Snap to grid for target size
                            let snapped_width = ((new_width / dot_spacing).round() * dot_spacing).max(Card::MIN_WIDTH);
                            let snapped_height = ((new_height / dot_spacing).round() * dot_spacing).max(Card::MIN_HEIGHT);

                            // ONLY set target - let animation smooth the transition
                            card.target_width = snapped_width;
                            card.target_height = snapped_height;
                        }
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CardResizeEnd(card_id) => {
                        let dot_spacing = self.dot_grid.dot_spacing();
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            let final_width = ((card.width / dot_spacing).round() * dot_spacing).max(Card::MIN_WIDTH);
                            let final_height = ((card.height / dot_spacing).round() * dot_spacing).max(Card::MIN_HEIGHT);
                            card.target_width = final_width;
                            card.target_height = final_height;
                        }
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                    DotGridMessage::CardDrop(card_id) => {
                        let dot_spacing = self.dot_grid.dot_spacing();
                        self.last_drag_world_pos = None;
                        if self.selected_card_ids.contains(&card_id) && self.selected_card_ids.len() > 1 {
                            // Snap all selected cards
                            let selected = self.selected_card_ids.clone();
                            for card in self.dot_grid.cards_mut().iter_mut() {
                                if selected.contains(&card.id) {
                                    card.is_dragging = false;
                                    let snapped = Card::snap_to_grid(card.current_position, dot_spacing);
                                    card.current_position = snapped;
                                    card.target_position = snapped;
                                }
                            }
                            // Keep multi-selection active after move
                        } else {
                            if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                card.is_dragging = false;
                                let snapped = Card::snap_to_grid(card.current_position, dot_spacing);
                                card.current_position = snapped;
                                card.target_position = snapped;
                            }
                            // Select the card on release so the first click-and-drag works immediately
                            self.selected_card_id = Some(card_id);
                            self.dot_grid.set_single_selected_card(Some(card_id));
                        }
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                    DotGridMessage::CheckboxToggle(card_id, line_index) => {
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            let text = card.content.text();
                            let card_type = card.card_type;
                            // For Markdown cards, toggle directly in plain text.
                            // For legacy <md>-tagged cards, use the tag-aware helper.
                            let updated_text = if card_type == crate::card::CardType::Markdown {
                                Self::toggle_checkbox_markdown(&text, line_index)
                            } else {
                                Self::toggle_checkbox_in_text(&text, line_index, self.config.general.debug_mode)
                            };
                            card.content.set_text(updated_text);
                            self.dot_grid.clear_cards_cache();
                            self.dot_grid.update_card_checkbox_positions(card_id);
                            self.dot_grid.update_card_link_positions(card_id);
                            self.workspace_dirty = true;
                        }
                    }
                    DotGridMessage::LinkClick(url) => {
                        // Open URL in default browser, ignore errors silently
                        let _ = open::that(&url);
                    }
                    DotGridMessage::BoxSelectEnd(rect) => {
                        self.selected_conn = None;
                        let canvas_offset = self.dot_grid.offset();
                        let mut ids = std::collections::HashSet::new();
                        for card in self.dot_grid.cards() {
                            let card_screen = Rectangle {
                                x: card.current_position.x + canvas_offset.x,
                                y: card.current_position.y + canvas_offset.y,
                                width: card.width,
                                height: card.height,
                            };
                            // Select cards that intersect the drag rectangle
                            let intersects = rect.x < card_screen.x + card_screen.width
                                && rect.x + rect.width > card_screen.x
                                && rect.y < card_screen.y + card_screen.height
                                && rect.y + rect.height > card_screen.y;
                            if intersects {
                                ids.insert(card.id);
                            }
                        }
                        self.selected_card_ids = ids.clone();
                        self.dot_grid.set_selected_cards(ids);
                        // Clear single-card selection (box select replaces it)
                        self.editing_card_id = None;
                        self.selected_card_id = None;
                        self.dot_grid.set_single_selected_card(None);
                        for card in self.dot_grid.cards_mut().iter_mut() {
                            card.is_editing = false;
                        }
                    }
                    DotGridMessage::CardTextClick(card_id, click_pos) => {
                        // Get canvas offset first (before borrowing cards_mut)
                        let canvas_offset = self.dot_grid.offset();

                        // Handle mouse click in text editor
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            if card.is_editing {
                                // Calculate relative position within the card's text area
                                let card_screen_x = card.current_position.x + canvas_offset.x;
                                let card_screen_y = card.current_position.y + canvas_offset.y;
                                let top_bar_height = 30.0;
                                let padding = 10.0;

                                let relative_x = (click_pos.x - card_screen_x - padding).max(0.0);
                                let relative_y = (click_pos.y - card_screen_y - top_bar_height - padding).max(0.0);

                                card.content.click_at_position(relative_x, relative_y);
                                self.dot_grid.clear_cards_cache();
                            }
                        }
                    }
                    DotGridMessage::DeleteConnection { from_card, from_side, to_card, to_side } => {
                        self.dot_grid.connections_mut().retain(|c| {
                            !(c.from_card == from_card && c.from_side == from_side
                                && c.to_card == to_card && c.to_side == to_side)
                        });
                        self.selected_conn = None;
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                    DotGridMessage::ConnectionClick { from_card, from_side, to_card, to_side } => {
                        self.selected_conn = self.dot_grid.connections().iter().find(|c| {
                            c.from_card == from_card && c.from_side == from_side
                            && c.to_card == to_card && c.to_side == to_side
                        }).copied();
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::ConnectionComplete { from_card, from_side, to_card, to_side } => {
                        use crate::card::Connection;
                        // Prevent duplicate connections on the same pair of sides
                        let already = self.dot_grid.connections().iter().any(|c| {
                            (c.from_card == from_card && c.from_side == from_side
                                && c.to_card == to_card && c.to_side == to_side)
                            || (c.from_card == to_card && c.from_side == to_side
                                && c.to_card == from_card && c.to_side == from_side)
                        });
                        if !already {
                            self.dot_grid.add_connection(Connection::new(from_card, from_side, to_card, to_side));
                            self.workspace_dirty = true;
                        }
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::ConnectionCancel => {
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CardTextDrag(card_id, drag_pos) => {
                        // Get canvas offset first (before borrowing cards_mut)
                        let canvas_offset = self.dot_grid.offset();

                        // Handle mouse drag in text editor for selection
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            if card.is_editing {
                                // Calculate relative position within the card's text area
                                let card_screen_x = card.current_position.x + canvas_offset.x;
                                let card_screen_y = card.current_position.y + canvas_offset.y;
                                let top_bar_height = 30.0;
                                let padding = 10.0;

                                let relative_x = (drag_pos.x - card_screen_x - padding).max(0.0);
                                let relative_y = (drag_pos.y - card_screen_y - top_bar_height - padding).max(0.0);

                                card.content.drag_to_position(relative_x, relative_y);
                                self.dot_grid.clear_cards_cache();
                            }
                        }
                    }
                }
            }
            Message::EventOccurred(event) => {
                match event {
                    Event::Keyboard(iced::keyboard::Event::ModifiersChanged(mods)) => {
                        self.ctrl_held = mods.control();
                    }
                    Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        self.mouse_position = Some(position);
                    }
                    Event::Window(iced::window::Event::Resized(size)) => {
                        self.window_size = size;
                        self.update_exclude_region();
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                        // If we're editing a board name and user clicks outside (not captured by widget),
                        // finish the rename
                        if self.editing_board_index.is_some() {
                            if let Some(index) = self.editing_board_index {
                                if index < self.boards.len() && !self.board_rename_value.trim().is_empty() {
                                    self.boards[index] = self.board_rename_value.trim().to_string();
                                }
                            }
                            self.editing_board_index = None;
                            self.board_rename_value.clear();
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        if let Some(card_type) = self.shelf_drag.take() {
                            if let Some(pos) = self.mouse_position {
                                let shelf_top = self.window_size.height - SHELF_HEIGHT;
                                if pos.y < shelf_top {
                                    // Drop position: ghost top-left anchored to cursor in top bar
                                    let screen_pos = Point::new(
                                        pos.x - GHOST_CARD_W / 2.0,
                                        pos.y - GHOST_TOP_BAR_H / 2.0,
                                    );
                                    let card_id = self.dot_grid.add_card(screen_pos, self.accent_color);
                                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                        card.card_type = card_type;
                                    }
                                    self.dot_grid.clear_cards_cache();
                                    self.workspace_dirty = true;
                                }
                            }
                        }
                    }
                    Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                        if !self.settings_open {
                            if self.ctrl_held {
                                // Ctrl+scroll → zoom toward cursor
                                let scroll_y = match delta {
                                    mouse::ScrollDelta::Lines  { y, .. } => y,
                                    mouse::ScrollDelta::Pixels { y, .. } => y / 50.0,
                                };
                                let factor = if scroll_y > 0.0 { 1.1_f32.powf(scroll_y) } else { (1.0 / 1.1_f32).powf(-scroll_y) };
                                let old_zoom = self.canvas_zoom;
                                let new_zoom = (self.canvas_zoom * factor).clamp(0.05, 10.0);
                                self.canvas_zoom = new_zoom;
                                self.dot_grid.set_zoom(new_zoom);
                                // Adjust offset so the point under the cursor stays fixed.
                                // Formula: offset_new = offset_old + (cursor - center) * (1/new_z - 1/old_z)
                                if let Some(cursor) = self.mouse_position {
                                    let cx = self.window_size.width  / 2.0;
                                    let cy = self.window_size.height / 2.0;
                                    let dx = cursor.x - cx;
                                    let dy = cursor.y - cy;
                                    self.canvas_offset.x += dx * (1.0 / new_zoom - 1.0 / old_zoom);
                                    self.canvas_offset.y += dy * (1.0 / new_zoom - 1.0 / old_zoom);
                                    self.dot_grid.set_offset(self.canvas_offset);
                                }
                                self.canvas_position_dirty = true;
                            } else {
                                let scroll_delta = match delta {
                                    mouse::ScrollDelta::Lines { x, y } => {
                                        Vector::new(x * 50.0, y * 50.0)
                                    }
                                    mouse::ScrollDelta::Pixels { x, y } => {
                                        Vector::new(x, y)
                                    }
                                };
                                self.canvas_offset.x += scroll_delta.x;
                                self.canvas_offset.y += scroll_delta.y;
                                self.dot_grid.set_offset(self.canvas_offset);
                                self.canvas_position_dirty = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Message::ShowContextMenu(pos) => {
                if !self.is_point_in_sidebar(pos) {
                    self.context_menu_position = Some(pos);
                    self.pending_card_position = Some(pos);
                }
            }
            Message::HideContextMenu => {
                self.context_menu_position = None;
                self.pending_card_position = None;
            }
            Message::AddCard => {
                if let Some(pos) = self.pending_card_position {
                    let card_id = self.dot_grid.add_card(pos, self.accent_color);
                    println!("Created card with id: {}, total cards: {}", card_id, self.dot_grid.cards().len());
                    self.workspace_dirty = true;
                }
                self.context_menu_position = None;
                self.pending_card_position = None;
            }
            Message::AddCardOfType(card_type) => {
                if let Some(pos) = self.pending_card_position {
                    let card_id = self.dot_grid.add_card(pos, self.accent_color);
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.card_type = card_type;
                    }
                    self.dot_grid.clear_cards_cache();
                    self.workspace_dirty = true;
                    // For image cards, immediately open the image picker
                    if card_type == CardType::Image {
                        let start_dir = dirs::picture_dir()
                            .or_else(dirs::home_dir)
                            .unwrap_or_else(|| std::path::PathBuf::from("."));
                        let picker = FilePickerState::new(
                            FilePickerMode::Open { filter_exts: vec![
                                "png".into(), "jpg".into(), "jpeg".into(),
                                "gif".into(), "bmp".into(), "webp".into(),
                                "svg".into(),
                            ]},
                            start_dir,
                            "Select Image",
                        );
                        self.image_picker = Some((card_id, picker));
                    }
                }
                self.context_menu_position = None;
                self.pending_card_position = None;
            }
            Message::ShowCardIconMenu(card_id) => {
                if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id) {
                    let screen_pos = Point::new(
                        card.current_position.x + self.canvas_offset.x + 25.0,
                        card.current_position.y + self.canvas_offset.y + 25.0,
                    );
                    self.card_icon_menu_position = Some(screen_pos);
                    self.card_icon_menu_card_id = Some(card_id);
                }
            }
            Message::ChangeCardIcon(card_id, icon) => {
                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                    card.icon = icon;
                    self.dot_grid.clear_cards_cache();
                }
                self.card_icon_menu_position = None;
                self.card_icon_menu_card_id = None;
                self.workspace_dirty = true;
            }
            Message::ChangeCardColor(card_id, color) => {
                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                    card.color = color;
                    self.dot_grid.clear_cards_cache();
                }
                // Keep the icon menu open so the user can see the icons update
                // with the new color. The menu will close normally on outside click.
                self.workspace_dirty = true;
            }
            Message::ChangeCardType(card_id, card_type) => {
                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                    card.card_type = card_type;
                    self.dot_grid.clear_cards_cache();
                }
                self.dot_grid.update_card_checkbox_positions(card_id);
                self.dot_grid.update_card_link_positions(card_id);
                self.card_type_menu_position = None;
                self.card_type_menu_card_id = None;
                self.workspace_dirty = true;
            }
            Message::StartEditingCard(card_id) => {
                // First, stop editing ALL cards
                for card in self.dot_grid.cards_mut().iter_mut() {
                    card.is_editing = false;
                }

                self.editing_card_id = Some(card_id);
                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                    card.is_editing = true;
                    // Move cursor to end of text
                    card.content.move_cursor_to_end();
                }
                self.dot_grid.clear_cards_cache();
            }
            Message::CardEditorAction(card_id, action) => {
                // Old text_editor action - no longer used with custom editor
            }
            Message::KeyboardInput(keyboard_event) => {
                // Block canvas keyboard shortcuts while any modal is open
                if self.workspace_modal.is_some()
                    || self.settings_open
                    || self.confirm_delete_card_id.is_some()
                {
                    return Task::none();
                }
                // Check for global shortcuts first
                if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = &keyboard_event {
                    // Ctrl+Tab to cycle forward through boards
                    if modifiers.control() && !modifiers.shift() && matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab)) {
                        if self.boards.len() > 1 {
                            let next_index = (self.active_board_index + 1) % self.boards.len();
                            // Save current board's cards
                            self.save_current_board_cards();

                            // Clear editing/selection state
                            self.editing_card_id = None;
                            self.selected_card_id = None;
                            self.card_icon_menu_position = None;
                            self.card_icon_menu_card_id = None;

                            // Switch to next board
                            self.active_board_index = next_index;

                            // Load new board's cards
                            let new_cards = self.board_cards.get(&next_index).cloned().unwrap_or_default();
                            self.load_cards_with_positions(new_cards);
                            self.save_workspace_to_file();
                        }
                        return Task::none();
                    }

                    // Ctrl+Shift+Tab to cycle backward through boards
                    if modifiers.control() && modifiers.shift() && matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab)) {
                        if self.boards.len() > 1 {
                            let prev_index = if self.active_board_index == 0 {
                                self.boards.len() - 1
                            } else {
                                self.active_board_index - 1
                            };

                            // Save current board's cards
                            self.save_current_board_cards();

                            // Clear editing/selection state
                            self.editing_card_id = None;
                            self.selected_card_id = None;
                            self.card_icon_menu_position = None;
                            self.card_icon_menu_card_id = None;

                            // Switch to previous board
                            self.active_board_index = prev_index;

                            // Load new board's cards
                            let new_cards = self.board_cards.get(&prev_index).cloned().unwrap_or_default();
                            self.load_cards_with_positions(new_cards);
                            self.save_workspace_to_file();
                        }
                        return Task::none();
                    }

                    // Ctrl+0 to recenter canvas (global, works even when not editing)
                    if modifiers.control() && matches!(key, iced::keyboard::Key::Character(c) if c.as_str() == "0") {
                        if self.config.general.enable_animations {
                            // Start animation
                            self.canvas_animating = true;
                            self.canvas_animation_start = self.canvas_offset;
                            self.canvas_animation_progress = 0.0;
                        } else {
                            // Instant recenter
                            self.canvas_offset = Vector::new(0.0, 0.0);
                            self.dot_grid.set_offset(self.canvas_offset);
                            self.canvas_position_dirty = true;
                        }
                        return Task::none();
                    }

                    // Ctrl+= / Ctrl++ → zoom in; Ctrl+- → zoom out; Ctrl+Shift+0 → reset zoom
                    if modifiers.control() && !modifiers.shift() && matches!(key, iced::keyboard::Key::Character(c) if c.as_str() == "=" || c.as_str() == "+") {
                        self.canvas_zoom = (self.canvas_zoom * 1.25).clamp(0.05, 10.0);
                        self.dot_grid.set_zoom(self.canvas_zoom);
                        self.canvas_position_dirty = true;
                        return Task::none();
                    }
                    if modifiers.control() && !modifiers.shift() && matches!(key, iced::keyboard::Key::Character(c) if c.as_str() == "-") {
                        self.canvas_zoom = (self.canvas_zoom / 1.25).clamp(0.05, 10.0);
                        self.dot_grid.set_zoom(self.canvas_zoom);
                        self.canvas_position_dirty = true;
                        return Task::none();
                    }
                    if modifiers.control() && modifiers.shift() && matches!(key, iced::keyboard::Key::Character(c) if c.as_str() == "0") {
                        self.canvas_zoom = 1.0;
                        self.dot_grid.set_zoom(self.canvas_zoom);
                        self.canvas_position_dirty = true;
                        return Task::none();
                    }

                    // Global shortcuts when NOT editing a card
                    if self.editing_card_id.is_none() {
                        // N: Add a new card at the mouse position (or viewport centre as fallback)
                        if !modifiers.control() && !modifiers.shift() && !modifiers.alt()
                            && matches!(key, iced::keyboard::Key::Character(c) if c.as_str() == "n")
                        {
                            let sidebar_right = 15.0 + SIDEBAR_WIDTH + self.sidebar_offset;
                            let pos = if let Some(mouse) = self.mouse_position {
                                if mouse.x > sidebar_right {
                                    mouse
                                } else {
                                    let cx = (self.window_size.width / 2.0) - self.canvas_offset.x - (SIDEBAR_WIDTH / 2.0);
                                    let cy = (self.window_size.height / 2.0) - self.canvas_offset.y;
                                    Point::new(cx, cy)
                                }
                            } else {
                                let cx = (self.window_size.width / 2.0) - self.canvas_offset.x - (SIDEBAR_WIDTH / 2.0);
                                let cy = (self.window_size.height / 2.0) - self.canvas_offset.y;
                                Point::new(cx, cy)
                            };
                            let card_id = self.dot_grid.add_card(pos, self.accent_color);
                            self.selected_card_ids.clear();
                            self.dot_grid.clear_selected_cards();
                            self.selected_card_id = Some(card_id);
                            self.editing_card_id = Some(card_id);
                            if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                card.is_editing = true;
                                card.content.move_cursor_to_end();
                            }
                            self.dot_grid.clear_cards_cache();
                            self.workspace_dirty = true;
                            return Task::none();
                        }

                        // Delete: delete selected card(s)
                        if !modifiers.control() && matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete)) {
                            if !self.selected_card_ids.is_empty() {
                                let ids: Vec<usize> = self.selected_card_ids.drain().collect();
                                for id in ids {
                                    self.dot_grid.delete_card(id);
                                }
                                self.dot_grid.clear_selected_cards();
                                self.selected_card_id = None;
                                self.workspace_dirty = true;
                            } else if let Some(card_id) = self.selected_card_id {
                                if self.config.general.confirm_card_delete {
                                    self.confirm_delete_card_id = Some(card_id);
                                } else {
                                    self.dot_grid.delete_card(card_id);
                                    self.selected_card_id = None;
                                    self.workspace_dirty = true;
                                }
                            }
                            return Task::none();
                        }

                        // Ctrl+D: duplicate selected card(s)
                        if modifiers.control() && !modifiers.shift()
                            && matches!(key, iced::keyboard::Key::Character(c) if c.as_str() == "d")
                        {
                            let dot_spacing = self.dot_grid.dot_spacing();
                            let ids_to_dup: Vec<usize> = if !self.selected_card_ids.is_empty() {
                                self.selected_card_ids.iter().copied().collect()
                            } else if let Some(id) = self.selected_card_id {
                                vec![id]
                            } else {
                                vec![]
                            };
                            let mut new_ids = Vec::new();
                            for card_id in ids_to_dup {
                                if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id).cloned() {
                                    let new_pos = Point::new(
                                        card.current_position.x + dot_spacing * 2.0,
                                        card.current_position.y + dot_spacing * 2.0,
                                    );
                                    let new_id = self.dot_grid.add_card_with_size(
                                        Point::new(
                                            new_pos.x + self.canvas_offset.x,
                                            new_pos.y + self.canvas_offset.y,
                                        ),
                                        &card.content.text(),
                                        card.icon,
                                        card.color,
                                        card.width,
                                        card.height,
                                    );
                                    if let Some(new_card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == new_id) {
                                        new_card.card_type = card.card_type;
                                    }
                                    new_ids.push(new_id);
                                }
                            }
                            if new_ids.len() == 1 {
                                self.selected_card_id = Some(new_ids[0]);
                                self.dot_grid.set_single_selected_card(Some(new_ids[0]));
                                self.selected_card_ids.clear();
                                self.dot_grid.clear_selected_cards();
                            } else if new_ids.len() > 1 {
                                self.selected_card_ids = new_ids.iter().copied().collect();
                                self.dot_grid.set_selected_cards(self.selected_card_ids.clone());
                                self.selected_card_id = None;
                                self.dot_grid.set_single_selected_card(None);
                            }
                            self.dot_grid.clear_cards_cache();
                            self.workspace_dirty = true;
                            return Task::none();
                        }

                        // Escape: deselect multi-selection
                        if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)) {
                            if self.confirm_delete_card_id.is_some() {
                                self.confirm_delete_card_id = None;
                                return Task::none();
                            }
                            if !self.selected_card_ids.is_empty() {
                                self.selected_card_ids.clear();
                                self.dot_grid.clear_selected_cards();
                                return Task::none();
                            }
                        }
                    }

                    if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)) {
                        // Close menus/settings/editing - but never quit the app
                        if self.confirm_delete_card_id.is_some() {
                            self.confirm_delete_card_id = None;
                            return Task::none();
                        } else if self.app_menu_open {
                            self.app_menu_open = false;
                            return Task::none();
                        } else if self.editing_board_index.is_some() {
                            // Cancel board rename
                            self.editing_board_index = None;
                            self.board_rename_value.clear();
                            return Task::none();
                        } else if self.card_icon_menu_position.is_some() {
                            self.card_icon_menu_position = None;
                            self.card_icon_menu_card_id = None;
                            return Task::none();
                        } else if self.context_menu_position.is_some() {
                            self.context_menu_position = None;
                            self.pending_card_position = None;
                            return Task::none();
                        } else if self.settings_open {
                            self.settings_open = false;
                            self.update_exclude_region();
                            return Task::none();
                        } else if self.editing_card_id.is_some() {
                            // Stop editing but keep card selected so Delete key still works
                            if let Some(card_id) = self.editing_card_id {
                                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                    card.is_editing = false;
                                }
                                self.dot_grid.update_card_checkbox_positions(card_id);
                                self.dot_grid.update_card_link_positions(card_id);
                                // Show selection border now that editing border is gone
                                self.dot_grid.set_single_selected_card(Some(card_id));
                            }
                            self.editing_card_id = None;
                            // Keep selected_card_id — toolbar stays visible, Delete key works
                            self.dot_grid.clear_cards_cache();
                            return Task::none();
                        }
                        // If nothing to close, just ignore Escape
                        return Task::none();
                    }
                }
                
                // Handle keyboard input for editing card
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        use iced::keyboard::Key;

                        match keyboard_event {
                            iced::keyboard::Event::KeyPressed { key, modifiers, text, .. } => {
                                //eprintln!("=== KEY PRESSED ===");
                                //eprintln!("Key: {:?}", key);
                                //eprintln!("Text field: {:?}", text);
                                //eprintln!("Modifiers - Shift: {}, Ctrl: {}, Alt: {}, Logo: {}",
                                //    modifiers.shift(), modifiers.control(), modifiers.alt(), modifiers.logo());

                                // Handle special Named keys FIRST (before text field)
                                // These should trigger actions, not insert characters
                                let handled_as_special = match key {
                                    Key::Named(iced::keyboard::key::Named::Enter) => {
                                        //eprintln!("-> Enter key");
                                        card.content.insert_newline();
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::Backspace) => {
                                        //eprintln!("-> Backspace (Ctrl: {})", modifiers.control());
                                        if modifiers.control() {
                                            card.content.delete_word_backward();
                                        } else {
                                            card.content.backspace();
                                        }
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::Delete) => {
                                        // eprintln!("-> Delete (Ctrl: {})", modifiers.control());
                                        if modifiers.control() {
                                            card.content.delete_word_forward();
                                        } else {
                                            card.content.delete();
                                        }
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                                        // eprintln!("-> ArrowLeft (Ctrl: {}, Shift: {})", modifiers.control(), modifiers.shift());
                                        if modifiers.control() {
                                            card.content.move_cursor_word_left(modifiers.shift());
                                        } else {
                                            card.content.move_cursor_left(modifiers.shift());
                                        }
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                                        // eprintln!("-> ArrowRight (Ctrl: {}, Shift: {})", modifiers.control(), modifiers.shift());
                                        if modifiers.control() {
                                            card.content.move_cursor_word_right(modifiers.shift());
                                        } else {
                                            card.content.move_cursor_right(modifiers.shift());
                                        }
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                                        // eprintln!("-> ArrowUp");
                                        card.content.move_cursor_up();
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                                        // eprintln!("-> ArrowDown");
                                        card.content.move_cursor_down();
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::Home) => {
                                        // eprintln!("-> Home");
                                        if modifiers.control() {
                                            card.content.move_cursor_to_start();
                                        } else {
                                            card.content.move_cursor_to_start();
                                        }
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::End) => {
                                        // eprintln!("-> End");
                                        if modifiers.control() {
                                            card.content.move_cursor_to_end();
                                        } else {
                                            card.content.move_cursor_to_end();
                                        }
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::Tab) => {
                                        // Insert 4 spaces instead of a tab character
                                        card.content.insert_text("    ");
                                        true
                                    }
                                    Key::Named(iced::keyboard::key::Named::Escape) => {
                                        // eprintln!("-> Escape - exiting edit mode");
                                        card.is_editing = false;
                                        self.editing_card_id = None;
                                        self.selected_card_id = None;
                                        true
                                    }
                                    _ => false
                                };

                                // If not handled as special key, check for Ctrl shortcuts first, then text input
                                if !handled_as_special {
                                    // Check for Ctrl shortcuts using the key field (reliable)
                                    let is_ctrl_shortcut = if modifiers.control() && !modifiers.alt() {
                                        if let Key::Character(c) = &key {
                                            match c.to_uppercase().as_str() {
                                                "A" => {
                                                    card.content.select_all();
                                                    true
                                                }
                                                "C" => {
                                                    // Copy selected text to system clipboard
                                                    if let Some(text) = card.content.get_selected_text() {
                                                        match arboard::Clipboard::new() {
                                                            Ok(mut clipboard) => {
                                                                match clipboard.set_text(text.clone()) {
                                                                    Ok(_) => {
                                                                        if self.config.general.debug_mode {
                                                                            println!("DEBUG: Successfully copied to clipboard: {:?}", text);
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        eprintln!("ERROR: Failed to copy to clipboard: {:?}", e);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("ERROR: Failed to create clipboard for copy: {:?}", e);
                                                            }
                                                        }
                                                    }
                                                    true
                                                }
                                                "X" => {
                                                    // Cut: copy to system clipboard then delete
                                                    if let Some(text) = card.content.get_selected_text() {
                                                        match arboard::Clipboard::new() {
                                                            Ok(mut clipboard) => {
                                                                match clipboard.set_text(text.clone()) {
                                                                    Ok(_) => {
                                                                        if self.config.general.debug_mode {
                                                                            println!("DEBUG: Successfully cut to clipboard: {:?}", text);
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        eprintln!("ERROR: Failed to cut to clipboard: {:?}", e);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("ERROR: Failed to create clipboard for cut: {:?}", e);
                                                            }
                                                        }
                                                    }
                                                    card.content.delete_selection();
                                                    true
                                                }
                                                "V" => {
                                                    // Paste from system clipboard
                                                    match arboard::Clipboard::new() {
                                                        Ok(mut clipboard) => {
                                                            match clipboard.get_text() {
                                                                Ok(text) => {
                                                                    if self.config.general.debug_mode {
                                                                        println!("DEBUG: Successfully read from clipboard, length: {}, content: {:?}", text.len(), &text[..text.len().min(50)]);
                                                                    }
                                                                    // Delete selection first if any
                                                                    card.content.delete_selection();
                                                                    // Insert clipboard content
                                                                    card.content.insert_text(&text);
                                                                    if self.config.general.debug_mode {
                                                                        println!("DEBUG: Text inserted into editor");
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("ERROR: Failed to read from clipboard: {:?}", e);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("ERROR: Failed to create clipboard for paste: {:?}", e);
                                                        }
                                                    }
                                                    true
                                                }
                                                _ => false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };

                                    // Only insert text if it's not a Ctrl shortcut
                                    if !is_ctrl_shortcut {
                                        // Use the 'text' field if available (OS-processed character with layout)
                                        if let Some(text_char) = text {
                                            for ch in text_char.chars() {
                                                card.content.insert_char(ch);
                                            }
                                        } else if let Key::Character(c) = &key {
                                            // No text field - use character key directly
                                            for ch in c.chars() {
                                                card.content.insert_char(ch);
                                            }
                                        } else if matches!(key, Key::Named(iced::keyboard::key::Named::Space)) {
                                            card.content.insert_char(' ');
                                        }

                                        // Auto-complete <md> tags
                                        // Check if we just typed '>' and if the text before cursor is '<md'
                                        let current_text = card.content.text();
                                        let cursor_pos = card.content.cursor_position;

                                        // Check if we just typed '>' (text ends with '<md>')
                                        if cursor_pos >= 4 && cursor_pos <= current_text.len() {
                                            let before_cursor = &current_text[..cursor_pos];
                                            if before_cursor.ends_with("<md>") {
                                                // Insert newline, empty line, closing tag
                                                card.content.insert_text("\n\n</md>");
                                                // Move cursor back to the empty line
                                                card.content.cursor_position = cursor_pos + 1;
                                            }
                                        }
                                    }
                                }

                                // CRITICAL: Update scroll and clear cache AFTER every key press
                                // This ensures characters appear immediately
                                let card_bounds = Rectangle {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 180.0,
                                    height: 110.0,
                                };
                                card.content.update_scroll(card_bounds);
                                self.dot_grid.clear_cards_cache();
                                self.workspace_dirty = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Message::StopEditingCard => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.is_editing = false;
                    }
                }
                self.editing_card_id = None;
                self.selected_card_id = None;
                self.dot_grid.set_single_selected_card(None);
                self.dot_grid.clear_cards_cache();
            }
            Message::HideCardIconMenu => {
                self.card_icon_menu_position = None;
                self.card_icon_menu_card_id = None;
            }
            Message::ShowCardTypeMenu(card_id) => {
                if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id) {
                    let screen_pos = Point::new(
                        card.current_position.x + self.canvas_offset.x + card.width - 30.0,
                        card.current_position.y + self.canvas_offset.y + 30.0,
                    );
                    self.card_type_menu_position = Some(screen_pos);
                    self.card_type_menu_card_id = Some(card_id);
                }
            }
            Message::HideCardTypeMenu => {
                self.card_type_menu_position = None;
                self.card_type_menu_card_id = None;
            }
            Message::FormatBold => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("**", "**");
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::FormatItalic => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("*", "*");
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::FormatStrikethrough => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("~~", "~~");
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::FormatCode => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("`", "`");
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::FormatCodeBlock => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("```\n", "\n```");
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::FormatHeading => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("# ", "");
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::FormatBullet => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("- ", "");
                        self.dot_grid.clear_cards_cache();
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::DuplicateCard(card_id) => {
                if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id).cloned() {
                    let new_pos = Point::new(
                        card.current_position.x + 20.0,
                        card.current_position.y + 20.0,
                    );
                    let new_card_id = self.dot_grid.add_card_with_size(
                        new_pos,
                        card.content.text(),
                        card.icon,
                        card.color,
                        card.width,
                        card.height,
                    );
                    // Copy card type to the duplicate
                    if let Some(new_card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == new_card_id) {
                        new_card.card_type = card.card_type;
                    }
                    self.selected_card_id = Some(new_card_id);
                    self.dot_grid.set_single_selected_card(Some(new_card_id));
                    self.dot_grid.clear_cards_cache();
                    self.workspace_dirty = true;
                }
            }
            Message::DeleteCard(card_id) => {
                if self.config.general.confirm_card_delete {
                    self.confirm_delete_card_id = Some(card_id);
                } else {
                    self.dot_grid.delete_card(card_id);
                    if self.selected_card_id == Some(card_id) {
                        self.selected_card_id = None;
                    }
                    if self.editing_card_id == Some(card_id) {
                        self.editing_card_id = None;
                    }
                    self.workspace_dirty = true;
                }
            }
            Message::AddNewBoard => {
                // Add a new board with a default name
                let board_number = self.boards.len() + 1;
                let board_name = format!("Board {}", board_number);
                let new_board_index = self.boards.len();
                self.boards.push(board_name);

                // Initialize empty cards vec for new board
                self.board_cards.insert(new_board_index, Vec::new());

                // Trigger add animation if animations enabled
                if self.config.general.enable_animations {
                    println!("DEBUG: Starting AddBoard animation for board {}", self.boards.len() - 1);
                    self.board_list_animating = true;
                    self.board_list_animation_progress = 0.0;
                    self.board_list_animation_type = BoardAnimationType::AddBoard;
                    self.animating_board_index = Some(self.boards.len() - 1);
                }
                self.workspace_dirty = true;
            }
            Message::SelectBoard(index) => {
                if index < self.boards.len() {
                    // If clicking the already active board, start renaming (double-click behavior)
                    if index == self.active_board_index && self.editing_board_index.is_none() {
                        self.editing_board_index = Some(index);
                        self.board_rename_value = self.boards[index].clone();
                    } else if index != self.active_board_index {
                        // Switching to a different board
                        // Save current board's cards
                        let current_cards = self.dot_grid.cards().iter().cloned().collect();
                        self.board_cards.insert(self.active_board_index, current_cards);
                        // Save current board's connections
                        let current_conns = self.dot_grid.connections().to_vec();
                        self.board_connections.insert(self.active_board_index, current_conns);

                        // Clear any editing/selection state
                        self.editing_card_id = None;
                        self.selected_card_id = None;
                        self.selected_conn = None;
                        self.card_icon_menu_position = None;
                        self.card_icon_menu_card_id = None;

                        // Switch to new board
                        self.active_board_index = index;

                        // Load new board's cards (or create empty vec if board doesn't exist in map)
                        let new_board_cards = self.board_cards.get(&index).cloned().unwrap_or_default();
                        self.load_cards_with_positions(new_board_cards);
                        // Restore new board's connections
                        let new_conns = self.board_connections.get(&index).cloned().unwrap_or_default();
                        self.dot_grid.set_connections(new_conns);

                        // Persist changes immediately on board switch
                        self.save_workspace_to_file();
                    }
                }
            }
            Message::DeleteBoard(index) => {
                // Don't allow deleting the last board
                if self.boards.len() > 1 && index < self.boards.len() {
                    // Trigger delete animation if animations enabled
                    if self.config.general.enable_animations {
                        println!("DEBUG: Starting DeleteBoard animation for board {}", index);
                        self.board_list_animating = true;
                        self.board_list_animation_progress = 0.0;
                        self.board_list_animation_type = BoardAnimationType::DeleteBoard;
                        self.animating_board_index = Some(index);
                        self.workspace_dirty = true;
                    } else {
                        // No animation, delete immediately
                        self.boards.remove(index);

                        // Adjust active board index if needed
                        if self.active_board_index >= self.boards.len() {
                            self.active_board_index = self.boards.len().saturating_sub(1);
                        } else if self.active_board_index > index {
                            self.active_board_index = self.active_board_index.saturating_sub(1);
                        }
                        self.workspace_dirty = true;
                    }
                }
            }
            Message::BoardHover(index) => {
                self.hovered_board_index = index;
            }
            Message::StartRenamingBoard(index) => {
                if index < self.boards.len() {
                    self.editing_board_index = Some(index);
                    self.board_rename_value = self.boards[index].clone();
                }
            }
            Message::BoardRenameInput(value) => {
                self.board_rename_value = value;
            }
            Message::FinishRenamingBoard => {
                if let Some(index) = self.editing_board_index {
                    if index < self.boards.len() && !self.board_rename_value.trim().is_empty() {
                        self.boards[index] = self.board_rename_value.trim().to_string();
                        self.workspace_dirty = true;
                    }
                }
                self.editing_board_index = None;
                self.board_rename_value.clear();
            }
            Message::CancelRenamingBoard => {
                self.editing_board_index = None;
                self.board_rename_value.clear();
            }
            Message::SetNewBoardButtonAtTop(at_top) => {
                // Trigger button position change animation if animations enabled
                if self.config.general.enable_animations {
                    if self.config.general.debug_mode {
                        println!("DEBUG: Starting ButtonPositionChange animation");
                    }
                    self.board_list_animating = true;
                    self.board_list_animation_progress = 0.0;
                    self.board_list_animation_type = BoardAnimationType::ButtonPositionChange;
                    self.animating_board_index = None; // Affects whole list
                }

                if let Err(e) = self.config.set_new_board_button_at_top(at_top) {
                    eprintln!("Failed to save new board button position setting: {}", e);
                }
            }
            Message::SetDebugMode(enabled) => {
                if let Err(e) = self.config.set_debug_mode(enabled) {
                    eprintln!("Failed to save debug mode setting: {}", e);
                }
                // Update DotGrid debug mode
                self.dot_grid.set_debug_mode(enabled);
            }
            Message::SetConfirmCardDelete(enabled) => {
                if let Err(e) = self.config.set_confirm_card_delete(enabled) {
                    eprintln!("Failed to save confirm card delete setting: {}", e);
                }
            }
            Message::ShowDeleteConfirmDialog(card_id) => {
                self.confirm_delete_card_id = Some(card_id);
            }
            Message::ConfirmDeleteCard => {
                if let Some(card_id) = self.confirm_delete_card_id.take() {
                    self.dot_grid.delete_card(card_id);
                    if self.selected_card_id == Some(card_id) {
                        self.selected_card_id = None;
                    }
                    if self.editing_card_id == Some(card_id) {
                        self.editing_card_id = None;
                    }
                    self.workspace_dirty = true;
                }
            }
            Message::CancelDeleteCard => {
                self.confirm_delete_card_id = None;
            }
            Message::ToggleAppMenu => {
                if self.app_menu_open {
                    // Start closing animation
                    if self.config.general.enable_animations {
                        self.app_menu_animating = true;
                        self.app_menu_opening = false;
                        self.app_menu_animation_progress = 1.0;
                    } else {
                        self.app_menu_open = false;
                    }
                } else {
                    // Open immediately, start opening animation
                    self.app_menu_open = true;
                    if self.config.general.enable_animations {
                        self.app_menu_animating = true;
                        self.app_menu_opening = true;
                        self.app_menu_animation_progress = 0.0;
                    } else {
                        self.app_menu_animation_progress = 1.0;
                    }
                }
            }
            Message::CloseAppMenu => {
                self.recent_submenu_open = false;
                if self.app_menu_open {
                    if self.config.general.enable_animations {
                        self.app_menu_animating = true;
                        self.app_menu_opening = false;
                        self.app_menu_animation_progress = 1.0;
                    } else {
                        self.app_menu_open = false;
                    }
                }
            }
            Message::MenuFileNewBoard => {
                self.app_menu_open = false;
                let board_number = self.boards.len() + 1;
                let board_name = format!("Board {}", board_number);
                let new_board_index = self.boards.len();
                self.boards.push(board_name);
                self.board_cards.insert(new_board_index, Vec::new());
                if self.config.general.enable_animations {
                    self.board_list_animating = true;
                    self.board_list_animation_progress = 0.0;
                    self.board_list_animation_type = BoardAnimationType::AddBoard;
                    self.animating_board_index = Some(self.boards.len() - 1);
                }
                self.workspace_dirty = true;
            }
            Message::MenuFileQuit => {
                self.save_workspace_to_file();
                return iced::exit();
            }
            Message::MenuViewResetCanvas => {
                self.app_menu_open = false;
                // Animate back to center (same as Ctrl+0)
                if self.config.general.enable_animations {
                    self.canvas_animating = true;
                    self.canvas_animation_start = self.canvas_offset;
                    self.canvas_animation_progress = 0.0;
                } else {
                    self.canvas_offset = Vector::new(0.0, 0.0);
                    self.dot_grid.set_offset(Vector::new(0.0, 0.0));
                    self.dot_grid.clear_cards_cache();
                    self.canvas_position_dirty = true;
                }
            }
            Message::MenuViewToggleSidebar => {
                self.app_menu_open = false;
                self.sidebar_open = !self.sidebar_open;
                if self.config.general.enable_animations {
                    self.animating = true;
                    self.animation_progress = 0.0;
                    self.animation_start_offset = self.sidebar_offset;
                } else {
                    self.sidebar_offset = if self.sidebar_open { 0.0 } else { SIDEBAR_HIDDEN_OFFSET };
                }
            }
            Message::MenuViewToggleTheme => {
                self.app_menu_open = false;
                // Re-use the same logic as ToggleTheme
                let next = match self.theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                };
                if self.config.general.enable_animations {
                    self.theme_transitioning = true;
                    self.theme_transition_progress = 0.0;
                    self.next_theme = Some(next);
                } else {
                    self.theme = next;
                    self.dot_grid.set_dot_color(self.theme.dot_color());
                    self.dot_grid.set_background_color(self.theme.background());
                    self.dot_grid.set_card_colors(
                        self.theme.card_background(),
                        self.theme.card_border(),
                        self.theme.card_text(),
                    );
                    self.dot_grid.clear_cards_cache();
                    if let Err(e) = self.config.set_theme(next.into()) {
                        eprintln!("Failed to save theme: {}", e);
                    }
                }
            }
            Message::MenuHelpAbout => {
                self.app_menu_open = false;
                self.settings_open = true;
                self.settings_category = SettingsCategory::About;
                if self.config.general.enable_animations {
                    self.settings_animating = true;
                    self.settings_opening = true;
                    self.settings_animation_progress = 0.0;
                }
            }
            Message::MenuHelpKeyboardShortcuts => {
                self.app_menu_open = false;
                self.settings_open = true;
                self.settings_category = SettingsCategory::Shortcuts;
                if self.config.general.enable_animations {
                    self.settings_animating = true;
                    self.settings_opening = true;
                    self.settings_animation_progress = 0.0;
                }
            }

            // ── Workspace messages ────────────────────────────────────────────
            Message::WorkspaceModal(modal_msg) => {
                match modal_msg {
                    WorkspaceModalMessage::Noop => {}
                    WorkspaceModalMessage::ChooseNew => {
                        let default_dir = WorkspaceFile::default_dir()
                            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name: String::new(),
                            picker: None,
                            save_dir: default_dir,
                            is_import: false,
                        });
                    }
                    WorkspaceModalMessage::ChooseOpen => {
                        let start_dir = WorkspaceFile::default_dir()
                            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                        let picker = FilePickerState::new(
                            FilePickerMode::Open { filter_exts: vec!["cards".to_string()] },
                            start_dir,
                            "Open Workspace",
                        );
                        self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                    }
                    WorkspaceModalMessage::ChooseImport => {
                        let start_dir = WorkspaceFile::default_dir()
                            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                        // Empty filter = show all files (json, cards-workspace, etc.)
                        let picker = FilePickerState::new(
                            FilePickerMode::Open { filter_exts: vec![] },
                            start_dir,
                            "Import Workspace",
                        );
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                    }
                    WorkspaceModalMessage::NewNameInput(name) => {
                        if let Some(WorkspaceModalState::CreatingNew { name: ref mut n, .. }) = self.workspace_modal {
                            *n = name;
                        }
                    }
                    WorkspaceModalMessage::BrowseSaveDir => {
                        let browse_data = if let Some(WorkspaceModalState::CreatingNew { ref save_dir, ref name, .. }) = self.workspace_modal {
                            Some((save_dir.clone(), name.clone()))
                        } else { None };
                        if let Some((save_dir, name)) = browse_data {
                            let new_picker = FilePickerState::new(
                                FilePickerMode::Save { default_name: format!("{}.cards", name.replace(' ', "_")) },
                                save_dir,
                                "Choose Save Location",
                            );
                            if let Some(WorkspaceModalState::CreatingNew { ref mut picker, .. }) = self.workspace_modal {
                                *picker = Some(new_picker);
                            }
                        }
                    }
                    WorkspaceModalMessage::ConfirmNew => {
                        if let Some(WorkspaceModalState::CreatingNew { ref name, ref save_dir, is_import, .. }) = self.workspace_modal.clone() {
                            let trimmed = name.trim().to_string();
                            if !trimmed.is_empty() {
                                let safe_name: String = trimmed.chars()
                                    .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
                                    .collect();
                                let file_name = format!("{}.cards", safe_name.replace(' ', "_"));
                                let path = save_dir.join(&file_name);
                                // If this was opened after an import, save the current
                                // in-memory boards instead of creating an empty workspace.
                                let ws = if is_import {
                                    let mut collected = self.collect_workspace();
                                    collected.name = trimmed.clone();
                                    collected
                                } else {
                                    WorkspaceFile::new_empty(&trimmed)
                                };
                                match ws.save(&path) {
                                    Ok(()) => {
                                        if is_import {
                                            // Data already in memory — just record the path
                                            self.workspace_path = Some(path.clone());
                                            if let Err(e) = self.config.push_recent_workspace(path.to_string_lossy().to_string()) {
                                                eprintln!("Failed to save recent workspace: {}", e);
                                            }
                                            self.workspace_modal = None;
                                        } else {
                                            self.apply_workspace(ws, path);
                                            self.workspace_modal = None;
                                        }
                                    }
                                    Err(e) => eprintln!("Failed to create workspace: {}", e),
                                }
                            }
                        }
                    }
                    WorkspaceModalMessage::CancelNew => {
                        if self.workspace_path.is_some() {
                            self.workspace_modal = None;
                        } else {
                            self.workspace_modal = Some(WorkspaceModalState::Idle);
                        }
                    }
                    WorkspaceModalMessage::FilePicker(fp_msg) => {
                        self.handle_file_picker_message(fp_msg);
                    }
                }
            }

            Message::MenuFileNewWorkspace => {
                self.app_menu_open = false;
                self.recent_submenu_open = false;
                let default_dir = WorkspaceFile::default_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                    name: String::new(),
                    picker: None,
                    save_dir: default_dir,
                    is_import: false,
                });
            }

            Message::MenuFileOpenWorkspace => {
                self.app_menu_open = false;
                self.recent_submenu_open = false;
                let start_dir = WorkspaceFile::default_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                let picker = FilePickerState::new(
                    FilePickerMode::Open { filter_exts: vec!["cards".to_string()] },
                    start_dir,
                    "Open Workspace",
                );
                self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
            }

            Message::MenuFileOpenRecent(path_str) => {
                self.app_menu_open = false;
                self.recent_submenu_open = false;
                let path = std::path::PathBuf::from(&path_str);
                match WorkspaceFile::load(&path) {
                    Ok(ws) => {
                        self.apply_workspace(ws, path);
                    }
                    Err(e) => {
                        eprintln!("Failed to open recent workspace '{}': {}", path_str, e);
                        // Remove the broken entry from recents
                        self.config.general.recent_workspaces.retain(|p| p != &path_str);
                        let _ = self.config.save();
                    }
                }
            }

            Message::MenuRecentSubmenuOpen => {
                self.recent_submenu_open = true;
            }

            Message::MenuRecentSubmenuClose => {
                self.recent_submenu_open = false;
            }

            Message::WorkspaceLoaded(ws) => {
                let _ = ws; // reserved for async use
            }

            Message::SaveWorkspace => {
                self.workspace_dirty = true;
            }

            // ── Import / Export ───────────────────────────────────────────────

            Message::MenuFileImportExport => {
                // Toggle the submenu open/closed when the main item is clicked
                self.import_export_submenu_open = !self.import_export_submenu_open;
            }
            Message::MenuImportExportSubmenuOpen => {
                self.import_export_submenu_open = true;
            }
            Message::MenuImportExportSubmenuClose => {
                self.import_export_submenu_open = false;
            }

            Message::MenuExportWorkspace => {
                self.app_menu_open = false;
                self.import_export_submenu_open = false;
                let dir = WorkspaceFile::default_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                let ws_name = self.workspace_path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "workspace".to_string());
                self.import_export_modal = Some(ImportExportState::new_export_workspace(&ws_name, dir));
            }
            Message::MenuExportBoard => {
                self.app_menu_open = false;
                self.import_export_submenu_open = false;
                let dir = WorkspaceFile::default_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                let board_name = self.boards.get(self.active_board_index)
                    .cloned()
                    .unwrap_or_else(|| "board".to_string());
                self.import_export_modal = Some(ImportExportState::new_export_board(&board_name, dir));
            }
            Message::MenuImportWorkspace => {
                self.app_menu_open = false;
                self.import_export_submenu_open = false;
                let dir = WorkspaceFile::default_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                self.import_export_modal = Some(ImportExportState::new_import_workspace(dir));
            }
            Message::MenuImportBoard => {
                self.app_menu_open = false;
                self.import_export_submenu_open = false;
                let dir = WorkspaceFile::default_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                self.import_export_modal = Some(ImportExportState::new_import_board(dir));
            }

            Message::ShelfDragStart(card_type, _pos) => {
                self.shelf_drag = Some(card_type);
            }

            Message::ConnSetLineStyle { from_card, from_side, to_card, to_side, style } => {
                if let Some(conn) = self.dot_grid.connections_mut().iter_mut().find(|c| {
                    c.from_card == from_card && c.from_side == from_side
                    && c.to_card == to_card && c.to_side == to_side
                }) {
                    conn.line_style = style;
                    self.workspace_dirty = true;
                    self.dot_grid.clear_cards_cache();
                }
            }
            Message::ConnToggleArrowFrom { from_card, from_side, to_card, to_side } => {
                if let Some(conn) = self.dot_grid.connections_mut().iter_mut().find(|c| {
                    c.from_card == from_card && c.from_side == from_side
                    && c.to_card == to_card && c.to_side == to_side
                }) {
                    conn.arrow_from = !conn.arrow_from;
                    self.workspace_dirty = true;
                    self.dot_grid.clear_cards_cache();
                }
            }
            Message::ConnToggleArrowTo { from_card, from_side, to_card, to_side } => {
                if let Some(conn) = self.dot_grid.connections_mut().iter_mut().find(|c| {
                    c.from_card == from_card && c.from_side == from_side
                    && c.to_card == to_card && c.to_side == to_side
                }) {
                    conn.arrow_to = !conn.arrow_to;
                    self.workspace_dirty = true;
                    self.dot_grid.clear_cards_cache();
                }
            }
            Message::ConnDelete { from_card, from_side, to_card, to_side } => {
                self.dot_grid.connections_mut().retain(|c| {
                    !(c.from_card == from_card && c.from_side == from_side
                      && c.to_card == to_card && c.to_side == to_side)
                });
                self.selected_conn = None;
                self.workspace_dirty = true;
                self.dot_grid.clear_cards_cache();
            }

            Message::ImportExport(ie_msg) => {
                self.handle_import_export_message(ie_msg);
            }

            Message::OpenImagePicker(card_id) => {
                let start_dir = dirs::picture_dir()
                    .or_else(dirs::home_dir)
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                let picker = FilePickerState::new(
                    FilePickerMode::Open { filter_exts: vec![
                        "png".into(), "jpg".into(), "jpeg".into(),
                        "gif".into(), "bmp".into(), "webp".into(),
                        "svg".into(),
                    ]},
                    start_dir,
                    "Select Image",
                );
                self.image_picker = Some((card_id, picker));
            }

            Message::ImagePickerMsg(msg) => {
                if let Some((card_id, ref mut picker)) = self.image_picker {
                    match &msg {
                        FilePickerMessage::Cancel => {
                            self.image_picker = None;
                        }
                        FilePickerMessage::Confirm => {
                            if let Some(path) = picker.confirmed_path() {
                                if let Ok(bytes) = std::fs::read(&path) {
                                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                        card.set_image(bytes);
                                    }
                                    self.workspace_dirty = true;
                                    self.dot_grid.clear_cards_cache();
                                }
                                self.image_picker = None;
                            }
                        }
                        FilePickerMessage::SelectFile(name) => {
                            // Auto-confirm on single click — no need for a separate Open button
                            let path = picker.current_dir.join(name);
                            if path.exists() {
                                if let Ok(bytes) = std::fs::read(&path) {
                                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                        card.set_image(bytes);
                                    }
                                    self.workspace_dirty = true;
                                    self.dot_grid.clear_cards_cache();
                                }
                                self.image_picker = None;
                            }
                        }
                        FilePickerMessage::EnterDir(path) => {
                            picker.enter(path.clone());
                        }
                        FilePickerMessage::GoUp => {
                            picker.go_up();
                        }
                        FilePickerMessage::FileNameInput(s) => {
                            picker.file_name = s.clone();
                        }
                        FilePickerMessage::GoToPlace(path) => {
                            picker.enter(path.clone());
                        }
                        FilePickerMessage::ToggleDrive(path) => {
                            if picker.expanded_drives.contains(path) {
                                picker.expanded_drives.remove(path);
                            } else {
                                picker.expanded_drives.insert(path.clone());
                            }
                        }
                        FilePickerMessage::Extra(_) => {}
                    }
                }
            }

            Message::CloseImagePicker => {
                self.image_picker = None;
            }
            Message::ZoomIn => {
                self.canvas_zoom = (self.canvas_zoom * 1.25).clamp(0.05, 10.0);
                self.dot_grid.set_zoom(self.canvas_zoom);
                self.canvas_position_dirty = true;
            }
            Message::ZoomOut => {
                self.canvas_zoom = (self.canvas_zoom / 1.25).clamp(0.05, 10.0);
                self.dot_grid.set_zoom(self.canvas_zoom);
                self.canvas_position_dirty = true;
            }
            Message::ZoomReset | Message::MenuViewResetZoom => {
                self.canvas_zoom = 1.0;
                self.dot_grid.set_zoom(self.canvas_zoom);
                self.canvas_position_dirty = true;
            }
        }

        // Persist any state-changing action immediately
        if self.workspace_dirty && self.workspace_path.is_some() {
            self.workspace_dirty = false;
            self.canvas_position_dirty = false; // absorbed by full save
            self.last_save_instant = Instant::now();
            self.save_workspace_to_file();
        } else if self.canvas_position_dirty && self.workspace_path.is_some() {
            // Debounce position-only saves: write at most once per second
            if self.last_save_instant.elapsed() >= Duration::from_secs(1) {
                self.canvas_position_dirty = false;
                self.last_save_instant = Instant::now();
                self.save_workspace_to_file();
            }
        }

        // Sync blocked state after all state changes so it's current for next frame.
        self.sync_grid_blocked();

        Task::none()
    }

    /// Save current board's cards to the board_cards HashMap
    fn save_current_board_cards(&mut self) {
        let current_cards = self.dot_grid.cards().iter().cloned().collect();
        self.board_cards.insert(self.active_board_index, current_cards);
        let current_conns = self.dot_grid.connections().to_vec();
        self.board_connections.insert(self.active_board_index, current_conns);
    }

    /// Load a set of cards into the canvas and rebuild checkbox + link hit-rects.
    fn load_cards_with_positions(&mut self, cards: Vec<crate::card::Card>) {
        self.dot_grid.load_cards(cards);
        self.dot_grid.clear_cards_cache();
        let ids: Vec<usize> = self.dot_grid.cards().iter().map(|c| c.id).collect();
        for id in ids {
            self.dot_grid.update_card_checkbox_positions(id);
            self.dot_grid.update_card_link_positions(id);
        }
    }

    // ── Workspace persistence ──────────────────────────────────────────────

    /// Load a `WorkspaceFile` into the application state.
    // ── File picker message dispatch ───────────────────────────────────────

    fn handle_file_picker_message(&mut self, msg: FilePickerMessage) {
        match self.workspace_modal.clone() {
            // ── Save-picker inside "Creating New" ─────────────────────────
            Some(WorkspaceModalState::CreatingNew { name, save_dir, picker: Some(mut fp), is_import }) => {
                match msg {
                    FilePickerMessage::Cancel => {
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name, picker: None, save_dir, is_import,
                        });
                    }
                    FilePickerMessage::GoUp => {
                        fp.go_up();
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name, picker: Some(fp), save_dir, is_import,
                        });
                    }
                    FilePickerMessage::EnterDir(path) => {
                        fp.enter(path);
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name, picker: Some(fp), save_dir, is_import,
                        });
                    }
                    FilePickerMessage::FileNameInput(v) => {
                        fp.file_name = v;
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name, picker: Some(fp), save_dir, is_import,
                        });
                    }
                    FilePickerMessage::SelectFile(fname) => {
                        fp.file_name = fname;
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name, picker: Some(fp), save_dir, is_import,
                        });
                    }
                    FilePickerMessage::GoToPlace(path) => {
                        fp.enter(path);
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name, picker: Some(fp), save_dir, is_import,
                        });
                    }
                    FilePickerMessage::ToggleDrive(path) => {
                        if fp.expanded_drives.contains(&path) {
                            fp.expanded_drives.remove(&path);
                        } else {
                            fp.expanded_drives.insert(path);
                        }
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name, picker: Some(fp), save_dir, is_import,
                        });
                    }
                    FilePickerMessage::Extra(_) => {}
                    FilePickerMessage::Confirm => {
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name,
                            picker: None,
                            save_dir: fp.current_dir.clone(),
                            is_import,
                        });
                    }
                }
            }

            // ── Open-picker inside "Opening Existing" ─────────────────────
            Some(WorkspaceModalState::OpeningExisting { mut picker }) => {
                match msg {
                    FilePickerMessage::Cancel => {
                        if self.workspace_path.is_some() {
                            self.workspace_modal = None;
                        } else {
                            self.workspace_modal = Some(WorkspaceModalState::Idle);
                        }
                    }
                    FilePickerMessage::GoUp => {
                        picker.go_up();
                        self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                    }
                    FilePickerMessage::EnterDir(path) => {
                        picker.enter(path);
                        self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                    }
                    FilePickerMessage::FileNameInput(v) => {
                        picker.file_name = v;
                        self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                    }
                    FilePickerMessage::SelectFile(fname) => {
                        let path = picker.current_dir.join(&fname);
                        match WorkspaceFile::load(&path) {
                            Ok(ws) => {
                                self.apply_workspace(ws, path);
                                self.workspace_modal = None;
                            }
                            Err(e) => {
                                picker.file_name = fname;
                                picker.error = Some(format!("Cannot open: {}", e));
                                self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                            }
                        }
                    }
                    FilePickerMessage::GoToPlace(path) => {
                        picker.enter(path);
                        self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                    }
                    FilePickerMessage::ToggleDrive(path) => {
                        if picker.expanded_drives.contains(&path) {
                            picker.expanded_drives.remove(&path);
                        } else {
                            picker.expanded_drives.insert(path);
                        }
                        self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                    }
                    FilePickerMessage::Extra(_) => {}
                    FilePickerMessage::Confirm => {
                        let path = picker.current_dir.join(&picker.file_name);
                        match WorkspaceFile::load(&path) {
                            Ok(ws) => {
                                self.apply_workspace(ws, path);
                                self.workspace_modal = None;
                            }
                            Err(e) => {
                                picker.error = Some(format!("Cannot open: {}", e));
                                self.workspace_modal = Some(WorkspaceModalState::OpeningExisting { picker });
                            }
                        }
                    }
                }
            }

            // ── Import-picker inside "Importing Workspace" ─────────────────
            Some(WorkspaceModalState::ImportingWorkspace { mut picker }) => {
                match msg {
                    FilePickerMessage::Cancel => {
                        // Return to welcome screen (import is only accessible from Idle)
                        self.workspace_modal = if self.workspace_path.is_some() {
                            None
                        } else {
                            Some(WorkspaceModalState::Idle)
                        };
                    }
                    FilePickerMessage::GoUp => {
                        picker.go_up();
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                    }
                    FilePickerMessage::EnterDir(path) => {
                        picker.enter(path);
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                    }
                    FilePickerMessage::FileNameInput(v) => {
                        picker.file_name = v;
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                    }
                    FilePickerMessage::SelectFile(fname) => {
                        let path = picker.current_dir.join(&fname);
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                        self.do_import_workspace_from_welcome(path);
                    }
                    FilePickerMessage::GoToPlace(path) => {
                        picker.enter(path);
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                    }
                    FilePickerMessage::ToggleDrive(path) => {
                        if picker.expanded_drives.contains(&path) {
                            picker.expanded_drives.remove(&path);
                        } else {
                            picker.expanded_drives.insert(path);
                        }
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                    }
                    FilePickerMessage::Extra(_) => {}
                    FilePickerMessage::Confirm => {
                        let path = picker.current_dir.join(&picker.file_name);
                        self.workspace_modal = Some(WorkspaceModalState::ImportingWorkspace { picker });
                        self.do_import_workspace_from_welcome(path);
                    }
                }
            }

            _ => {}
        }
    }

    /// Import a workspace from the welcome-screen import picker.
    /// On success: loads the workspace and closes the modal.
    /// On failure: shows error in the picker.
    fn do_import_workspace_from_welcome(&mut self, path: std::path::PathBuf) {
        use import_export::import_workspace;
        match import_workspace(&path) {
            Ok(r) => {
                let warnings = r.warnings.clone();
                let ws_name = r.workspace.name.clone();
                self.apply_workspace(r.workspace, path.clone());
                self.workspace_path = None;
                self.workspace_modal = None;
                // Open Save As dialog so the imported data gets a real .cards file
                let default_dir = WorkspaceFile::default_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                    name: ws_name,
                    picker: None,
                    save_dir: default_dir,
                    is_import: true,
                });
                if !warnings.is_empty() {
                    eprintln!("Import warnings: {:?}", warnings);
                }
            }
            Err(e) => {
                if let Some(WorkspaceModalState::ImportingWorkspace { ref mut picker }) = self.workspace_modal {
                    picker.error = Some(format!("Import failed: {}", e));
                }
            }
        }
    }

    fn apply_workspace(&mut self, ws: WorkspaceFile, path: std::path::PathBuf) {
        // Reset board/card state
        self.boards.clear();
        self.board_cards.clear();
        self.active_board_index = 0;
        self.editing_card_id = None;
        self.selected_card_id = None;
        self.selected_card_ids.clear();
        self.dot_grid.clear_selected_cards();
        self.last_drag_world_pos = None;
        self.card_icon_menu_position = None;
        self.card_icon_menu_card_id = None;

        // Find the highest existing card id so the DotGrid counter starts above it
        let mut max_id: usize = 0;

        for (board_idx, board) in ws.boards.iter().enumerate() {
            self.boards.push(board.name.clone());

            let mut cards: Vec<crate::card::Card> = Vec::new();
            for cd in &board.cards {
                if cd.id > max_id {
                    max_id = cd.id;
                }
                let mut card = crate::card::Card::new(
                    cd.id,
                    iced::Point::new(cd.x, cd.y),
                );
                card.content = crate::custom_text_editor::CustomTextEditor::with_text(&cd.content);
                card.content.set_font(self.dot_grid.font(), self.dot_grid.font_size());
                card.icon = cd.to_icon();
                card.color = cd.to_color();
                card.card_type = cd.to_card_type();
                card.width = cd.width;
                card.height = cd.height;
                card.target_width = cd.width;
                card.target_height = cd.height;
                card.target_position = card.current_position;
                // Restore image data and rebuild the cached render handle
                if let Some(img_bytes) = cd.image_data.clone() {
                    card.image_is_svg = cd.image_is_svg;
                    let arc = std::sync::Arc::new(img_bytes);
                    card.image_handle = Some(crate::card::build_image_handle(&arc, cd.image_is_svg));
                    card.image_data = Some(arc);
                }
                cards.push(card);
            }
            self.board_cards.insert(board_idx, cards);

            let connections = board.connections.iter().map(|c| c.to_connection()).collect();
            self.board_connections.insert(board_idx, connections);
        }

        // Ensure at least one board always exists
        if self.boards.is_empty() {
            self.boards.push("Board 1".to_string());
            self.board_cards.insert(0, Vec::new());
        }

        // Seed the DotGrid id counter so new cards won't collide
        self.dot_grid.set_next_card_id(max_id + 1);

        // Restore the last-active board (clamped in case boards were removed)
        let restored_board = ws.active_board_index.min(self.boards.len().saturating_sub(1));
        self.active_board_index = restored_board;

        // Load the restored board's cards into the canvas
        let active_cards = self.board_cards.get(&restored_board).cloned().unwrap_or_default();
        self.dot_grid.load_cards(active_cards);
        let active_conns = self.board_connections.get(&restored_board).cloned().unwrap_or_default();
        self.dot_grid.set_connections(active_conns);
        self.dot_grid.clear_cards_cache();

        // Build checkbox + link hit-rects for all loaded cards
        let card_ids: Vec<usize> = self.dot_grid.cards().iter().map(|c| c.id).collect();
        for id in card_ids {
            self.dot_grid.update_card_checkbox_positions(id);
            self.dot_grid.update_card_link_positions(id);
        }

        // Restore canvas scroll position
        self.canvas_offset = Vector::new(ws.canvas_offset_x, ws.canvas_offset_y);
        self.dot_grid.set_offset(self.canvas_offset);

        self.workspace_path = Some(path.clone());
        // Record in recents (also updates last_workspace)
        if let Err(e) = self.config.push_recent_workspace(path.to_string_lossy().to_string()) {
            eprintln!("Failed to save recent workspace: {}", e);
        }
    }

    /// Collect the current app state into a serialisable `WorkspaceFile`.
    fn collect_workspace(&mut self) -> WorkspaceFile {
        // Flush the active board first
        self.save_current_board_cards();

        let boards: Vec<BoardData> = self.boards
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let cards = self.board_cards
                    .get(&idx)
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(CardData::from_card)
                    .collect();
                let connections = self.board_connections
                    .get(&idx)
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(ConnectionData::from_connection)
                    .collect();
                BoardData { name: name.clone(), cards, connections }
            })
            .collect();

        let name = self.workspace_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Workspace".to_string());

        WorkspaceFile {
            version: 1,
            name,
            boards,
            active_board_index: self.active_board_index,
            canvas_offset_x: self.canvas_offset.x,
            canvas_offset_y: self.canvas_offset.y,
        }
    }

    /// Save the current workspace to its file (if one is set).
    fn save_workspace_to_file(&mut self) {
        let ws = self.collect_workspace();
        if let Some(ref path) = self.workspace_path.clone() {
            if let Err(e) = ws.save(path) {
                eprintln!("Auto-save failed: {}", e);
            } else if self.config.general.debug_mode {
                println!("DEBUG: Workspace auto-saved to {:?}", path);
            }
        }
    }

    fn handle_import_export_message(&mut self, msg: ImportExportMessage) {
        let Some(state) = self.import_export_modal.as_mut() else { return };

        match msg {
            ImportExportMessage::Cancel => {
                self.import_export_modal = None;
            }

            ImportExportMessage::DismissResult => {
                // On success close the whole modal; on failure return to picker
                let success = self.import_export_modal
                    .as_ref()
                    .and_then(|s| s.result.as_ref())
                    .map(|r| r.success)
                    .unwrap_or(false);
                if success {
                    self.import_export_modal = None;
                } else if let Some(s) = self.import_export_modal.as_mut() {
                    s.result = None;
                }
            }

            ImportExportMessage::SetFormat(fmt) => {
                state.format = fmt;
            }

            ImportExportMessage::Picker(fp_msg) => {
                match fp_msg {
                    FilePickerMessage::Cancel => {
                        self.import_export_modal = None;
                    }
                    FilePickerMessage::GoUp => {
                        state.picker.go_up();
                    }
                    FilePickerMessage::EnterDir(p) => {
                        state.picker.enter(p);
                    }
                    FilePickerMessage::GoToPlace(p) => {
                        state.picker.enter(p);
                    }
                    FilePickerMessage::ToggleDrive(p) => {
                        if state.picker.expanded_drives.contains(&p) {
                            state.picker.expanded_drives.remove(&p);
                        } else {
                            state.picker.expanded_drives.insert(p);
                        }
                    }
                    FilePickerMessage::FileNameInput(v) => {
                        state.picker.file_name = v;
                    }
                    FilePickerMessage::SelectFile(name) => {
                        state.picker.file_name = name.clone();
                        // In import mode, selecting a file immediately triggers confirm
                        if !state.kind.is_export() {
                            let path = state.picker.current_dir.join(&name);
                            self.execute_import(path);
                            return;
                        }
                    }
                    FilePickerMessage::Confirm => {
                        // In export Save mode the "Select Folder" / "Select" button fires Confirm.
                        // Treat it the same as ImportExportMessage::Confirm.
                        let Some(ie_state) = self.import_export_modal.clone() else { return };
                        if ie_state.kind.is_export() {
                            if let Some(path) = ie_state.resolved_path() {
                                self.execute_export(path, ie_state.format);
                            } else if let Some(s) = self.import_export_modal.as_mut() {
                                s.picker.error = Some("Please enter a file name".into());
                            }
                        }
                    }
                    FilePickerMessage::Extra(_) => {
                        // Extra is mapped to SetFormat before reaching here (in the view .map()),
                        // so this arm should never fire. Ignore if it somehow does.
                    }
                }
            }

            ImportExportMessage::Confirm => {
                let Some(state) = self.import_export_modal.clone() else { return };
                if state.kind.is_export() {
                    let Some(path) = state.resolved_path() else {
                        if let Some(s) = self.import_export_modal.as_mut() {
                            s.picker.error = Some("Please enter a file name".into());
                        }
                        return;
                    };
                    self.execute_export(path, state.format);
                } else {
                    let Some(path) = state.resolved_path() else {
                        if let Some(s) = self.import_export_modal.as_mut() {
                            s.picker.error = Some("Please select a file".into());
                        }
                        return;
                    };
                    self.execute_import(path);
                }
            }
        }
    }

    fn execute_export(&mut self, path: std::path::PathBuf, fmt: import_export::ExportFormat) {
        use import_export::{export_workspace, export_board};

        let kind = self.import_export_modal.as_ref().map(|s| s.kind.clone());
        let result = match kind {
            Some(IEKind::ExportWorkspace) => {
                let ws = self.collect_workspace();
                export_workspace(&ws, &path, fmt)
            }
            Some(IEKind::ExportBoard) => {
                self.save_current_board_cards();
                let cards = self.board_cards.get(&self.active_board_index)
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(CardData::from_card)
                    .collect();
                let board_name = self.boards.get(self.active_board_index)
                    .cloned()
                    .unwrap_or_else(|| "Board".to_string());
                let board = BoardData { name: board_name, cards, connections: Vec::new() };
                export_board(&board, &path, fmt)
            }
            _ => return,
        };

        let ie_result = match result {
            Ok(()) => ImportExportResult {
                success: true,
                message: format!("Exported successfully to {}", path.display()),
                warnings: Vec::new(),
            },
            Err(e) => ImportExportResult {
                success: false,
                message: format!("Export failed: {}", e),
                warnings: Vec::new(),
            },
        };

        if let Some(s) = self.import_export_modal.as_mut() {
            s.result = Some(ie_result);
        }
    }

    fn execute_import(&mut self, path: std::path::PathBuf) {
        use import_export::{import_workspace, import_board};

        let kind = self.import_export_modal.as_ref().map(|s| s.kind.clone());

        match kind {
            Some(IEKind::ImportWorkspace) => {
                match import_workspace(&path) {
                    Ok(r) => {
                        let warnings = r.warnings.clone();
                        let ws_name = r.workspace.name.clone();

                        // Load the workspace data into the app state.
                        // We pass the import source path temporarily; apply_workspace will
                        // set workspace_path to it, but we clear it right after so the
                        // app knows there is no persisted .cards file yet.
                        self.apply_workspace(r.workspace, path.clone());
                        self.workspace_path = None;

                        // Close the import modal
                        self.import_export_modal = None;

                        // Immediately open the "Save workspace as…" dialog so the imported
                        // data gets persisted to a real .cards file.
                        let default_dir = WorkspaceFile::default_dir()
                            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
                        self.workspace_modal = Some(WorkspaceModalState::CreatingNew {
                            name: ws_name,
                            picker: None,
                            save_dir: default_dir,
                            is_import: true,
                        });

                        if !warnings.is_empty() {
                            eprintln!("Import warnings: {:?}", warnings);
                        }
                    }
                    Err(e) => {
                        let ie_result = ImportExportResult {
                            success: false,
                            message: format!("Import failed: {}", e),
                            warnings: Vec::new(),
                        };
                        if let Some(s) = self.import_export_modal.as_mut() {
                            s.result = Some(ie_result);
                        }
                    }
                }
            }
            Some(IEKind::ImportBoard) => {
                match import_board(&path) {
                    Ok(r) => {
                        let warnings = r.warnings.clone();
                        // Add the imported board at the end
                        let new_idx = self.boards.len();
                        self.boards.push(r.board.name.clone());
                        // Re-key card IDs to avoid collisions
                        let mut next_id = self.dot_grid.next_card_id();
                        let mut cards: Vec<crate::card::Card> = Vec::new();
                        for mut cd in r.board.cards {
                            cd.id = next_id;
                            next_id += 1;
                            let mut card = crate::card::Card::new(
                                cd.id,
                                iced::Point::new(cd.x, cd.y),
                            );
                            card.width = cd.width;
                            card.height = cd.height;
                            card.content.set_text(cd.content.clone());
                            card.color = cd.to_color();
                            card.icon  = cd.to_icon();
                            cards.push(card);
                        }
                        self.dot_grid.set_next_card_id(next_id);
                        self.board_cards.insert(new_idx, cards);
                        self.workspace_dirty = true;
                        let ie_result = ImportExportResult {
                            success: true,
                            message: format!(
                                "Board \"{}\" imported successfully",
                                r.board.name
                            ),
                            warnings,
                        };
                        if let Some(s) = self.import_export_modal.as_mut() {
                            s.result = Some(ie_result);
                        }
                    }
                    Err(e) => {
                        let ie_result = ImportExportResult {
                            success: false,
                            message: format!("Import failed: {}", e),
                            warnings: Vec::new(),
                        };
                        if let Some(s) = self.import_export_modal.as_mut() {
                            s.result = Some(ie_result);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Toggle a checkbox at the specified line index in markdown text
    /// This must match exactly how the text_processor and markdown_parser work
    /// Toggle a checkbox for a plain Markdown card (content is raw markdown, no <md> tags).
    fn toggle_checkbox_markdown(text: &str, line_index: usize) -> String {
        let mut counter = 0usize;
        let mut result = String::new();
        for line in text.lines() {
            let trimmed = line.trim_start();
            let is_cb = trimmed.starts_with("- [ ]")
                || trimmed.starts_with("- [x]")
                || trimmed.starts_with("- [X]");
            if is_cb {
                if counter == line_index {
                    if trimmed.starts_with("- [ ]") {
                        result.push_str(&line.replacen("- [ ]", "- [x]", 1));
                    } else {
                        let toggled = line.replacen("- [x]", "- [ ]", 1);
                        result.push_str(&toggled.replacen("- [X]", "- [ ]", 1));
                    }
                } else {
                    result.push_str(line);
                }
                counter += 1;
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        // Remove trailing newline if original didn't end with one
        if !text.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        result
    }

    fn toggle_checkbox_in_text(text: &str, line_index: usize, debug_mode: bool) -> String {
        let mut result = String::new();
        let mut checkbox_counter = 0;  // Global counter across ALL md blocks
        let mut pos = 0;

        if debug_mode {
            println!("DEBUG: toggle_checkbox_in_text - looking for line_index: {}", line_index);
        }

        while pos < text.len() {
            // Look for <md> tag
            if let Some(md_start) = text[pos..].find("<md>") {
                let actual_md_start = pos + md_start;

                // Copy everything before <md> tag
                result.push_str(&text[pos..actual_md_start]);
                result.push_str("<md>");

                // Find closing </md> tag
                let md_content_start = actual_md_start + 4;
                if let Some(md_end) = text[md_content_start..].find("</md>") {
                    let actual_md_end = md_content_start + md_end;
                    let markdown_content = &text[md_content_start..actual_md_end];

                    if debug_mode {
                        println!("DEBUG: Found md block, content: '{}'", markdown_content);
                    }

                    // Process each line in the markdown content
                    for line in markdown_content.lines() {
                        let is_checkbox = line.trim_start().starts_with("- [ ]") ||
                                         line.trim_start().starts_with("- [x]") ||
                                         line.trim_start().starts_with("- [X]");

                        if is_checkbox {
                            if debug_mode {
                                println!("DEBUG: Found checkbox at counter {}: '{}'", checkbox_counter, line);
                            }
                            if checkbox_counter == line_index {
                                if debug_mode {
                                    println!("DEBUG: MATCH! Toggling checkbox");
                                }
                                // Toggle this checkbox
                                if line.contains("- [ ]") {
                                    result.push_str(&line.replace("- [ ]", "- [x]"));
                                } else {
                                    result.push_str(&line.replace("- [x]", "- [ ]").replace("- [X]", "- [ ]"));
                                }
                            } else {
                                result.push_str(line);
                            }
                            checkbox_counter += 1;
                        } else {
                            result.push_str(line);
                        }
                        result.push('\n');
                    }

                    // Remove trailing newline if markdown_content didn't end with one
                    if !markdown_content.ends_with('\n') && result.ends_with('\n') {
                        result.pop();
                    }

                    result.push_str("</md>");
                    pos = actual_md_end + 5; // Move past </md>
                } else {
                    // No closing tag found, copy rest as-is
                    result.push_str(&text[actual_md_start..]);
                    break;
                }
            } else {
                // No more <md> tags, copy rest
                result.push_str(&text[pos..]);
                break;
            }
        }
        
        result
    }

    fn subscription(&self) -> Subscription<Message> {
        // Always tick for card animations
        let tick = time::every(Duration::from_millis(16)).map(Message::Tick);

        let events = event::listen_with(|event, status, _id| {
            // Only process events that weren't already captured by widgets
            if status == iced::event::Status::Captured {
                // Still track mouse movement even when captured
                if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                    return Some(Message::EventOccurred(Event::Mouse(mouse::Event::CursorMoved { position })));
                }
                return None;
            }

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    Some(Message::EventOccurred(event))
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Message::EventOccurred(event))
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) => Some(Message::EventOccurred(event)),
                Event::Mouse(mouse::Event::WheelScrolled { .. }) => Some(Message::EventOccurred(event)),
                Event::Window(iced::window::Event::Resized(_)) => Some(Message::EventOccurred(event)),
                Event::Window(iced::window::Event::CloseRequested) => {
                    Some(Message::MenuFileQuit)
                }
                Event::Keyboard(keyboard_event) => Some(Message::KeyboardInput(keyboard_event)),
                _ => None,
            }
        });

        Subscription::batch([tick, events])
    }

    fn view(&self) -> Element<Message> {
        let settings_icon = self.icon_settings.clone();

        let sidebar_bg = self.theme.sidebar_background();
        let sidebar_shadow = self.theme.sidebar_shadow();
        let _separator_color = self.theme.separator_color();
        let accent = self.accent_color;
        let accent_bg = self.theme.accent_bg_from(self.accent_color);
        let accent_glow = self.theme.accent_glow_from(self.accent_color);
        let accent_separator = accent_glow;
        let icon_color = self.theme.icon_color();

        let canvas: Element<Message> = self.dot_grid.view().map(Message::DotGridMessage);

        // CardLayer renders each card in its own compositor layer (renderer.with_layer)
        // so text/SVGs composite correctly with fills — fixing the z-ordering issue.
        let card_layer: Element<Message> = CardLayer::new(
            self.dot_grid.cards(),
            self.dot_grid.offset(),
            self.dot_grid.card_background(),
            self.dot_grid.card_border(),
            self.dot_grid.card_text(),
            self.dot_grid.accent_color(),
            self.dot_grid.font(),
            self.dot_grid.font_size(),
            self.dot_grid.selected_cards(),
            self.dot_grid.single_selected_card(),
            self.dot_grid.hovered_card(),
        ).with_connections(
            self.dot_grid.connections(),
            self.dot_grid.pending_conn(),
            self.dot_grid.pending_cursor(),
            self.dot_grid.conn_anim_phase(),
        ).with_zoom(self.canvas_zoom).into();

        let mut shelf = CardShelf::new(
            self.window_size.width,
            self.window_size.height,
            |card_type, pos| Message::ShelfDragStart(card_type, pos),
            self.theme.sidebar_background(),
            self.theme.button_border(),
            self.theme.button_shadow(),
            self.theme.icon_color(),
            self.theme.accent_bg_from(self.accent_color),
        );
        // Attach ghost card data when a drag is active above the shelf zone
        if let Some(card_type) = self.shelf_drag {
            if let Some(pos) = self.mouse_position {
                if pos.y < self.window_size.height - SHELF_HEIGHT {
                    shelf = shelf.with_ghost(
                        card_type, pos,
                        self.dot_grid.card_background(),
                        self.dot_grid.card_border(),
                        self.accent_color,
                    );
                }
            }
        }
        let shelf: Element<Message> = shelf.into();

        let main_content: Element<Message> = iced::widget::stack![canvas, card_layer].into();

        // Build the base view with main content
        let mut view: Element<Message> = main_content;

        // Shelf pill always at the same overlay level — stable tree structure
        view = Overlay::new(view, shelf).into();

        // Zoom bar pill at the bottom-right corner
        let zoom_bar: Element<Message> = ZoomBar::new(
            self.window_size.width,
            self.window_size.height,
            self.canvas_zoom,
            || Message::ZoomIn,
            || Message::ZoomOut,
            || Message::ZoomReset,
            self.theme.sidebar_background(),
            self.theme.button_border(),
            self.theme.button_shadow(),
            self.theme.button_text(),
            self.theme.accent_bg_from(self.accent_color),
        ).into();
        view = Overlay::new(view, zoom_bar).into();

        // Connection toolbar — shown when a connection is selected (click-to-select)
        if let Some(conn) = self.selected_conn {
            // Look up the latest state of the connection (style/arrows may have changed)
            let live_conn = self.dot_grid.connections().iter().find(|c| {
                c.from_card == conn.from_card && c.from_side == conn.from_side
                && c.to_card == conn.to_card && c.to_side == conn.to_side
            }).copied();
            if let (Some(live), Some(toolbar_pos)) = (live_conn, self.dot_grid.conn_screen_midpoint(&conn)) {
                use connection_toolbar::ConnectionToolbar;
                let toolbar = ConnectionToolbar::new(
                    toolbar_pos,
                    self.window_size,
                    live.line_style,
                    live.arrow_from,
                    live.arrow_to,
                    self.theme.sidebar_background(),
                    self.theme.button_border(),
                    self.theme.button_shadow(),
                    self.theme.icon_color(),
                    self.theme.accent_bg_from(self.accent_color),
                    self.accent_color,
                    Color::from_rgb8(220, 60, 60),
                    {
                        let (fc, fs, tc, ts) = (live.from_card, live.from_side, live.to_card, live.to_side);
                        move |style| Message::ConnSetLineStyle { from_card: fc, from_side: fs, to_card: tc, to_side: ts, style }
                    },
                    {
                        let (fc, fs, tc, ts) = (live.from_card, live.from_side, live.to_card, live.to_side);
                        move || Message::ConnToggleArrowFrom { from_card: fc, from_side: fs, to_card: tc, to_side: ts }
                    },
                    {
                        let (fc, fs, tc, ts) = (live.from_card, live.from_side, live.to_card, live.to_side);
                        move || Message::ConnToggleArrowTo { from_card: fc, from_side: fs, to_card: tc, to_side: ts }
                    },
                    {
                        let (fc, fs, tc, ts) = (live.from_card, live.from_side, live.to_card, live.to_side);
                        move || Message::ConnDelete { from_card: fc, from_side: fs, to_card: tc, to_side: ts }
                    },
                );
                let toolbar_el: Element<Message> = toolbar.into();
                view = Overlay::new(view, toolbar_el).into();
            }
        }

        // Custom text editor is now rendered directly in canvas, no overlay needed

        // Add context menu (before sidebar)
        if let Some(pos) = self.context_menu_position {
            let context_menu_content = self.build_context_menu();
            let context_menu: Element<Message> = ContextMenu::new(
                context_menu_content,
                pos,
                sidebar_bg,
                self.theme.button_border(),
                sidebar_shadow,
            )
            .width(185.0)
            .on_close(Message::HideContextMenu)
            .into();

            view = Overlay::new(view, context_menu).into();
        }

        // Icons are now drawn directly inside the canvas in draw_cards(), so they
        // z-order correctly with overlapping cards — no widget overlays needed here.

        // Add card toolbar (before sidebar) - shown when a card is selected
        if let Some(card_id) = self.selected_card_id {
            if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id) {
                let card_type = card.card_type;

                let mut items: Vec<ToolbarItem<Message>> = Vec::new();
                if card_type == CardType::Markdown {
                    items.push(ToolbarItem::Icon { handle: self.icon_fmt_bold.clone(),          message: Message::FormatBold });
                    items.push(ToolbarItem::Icon { handle: self.icon_fmt_italic.clone(),        message: Message::FormatItalic });
                    items.push(ToolbarItem::Icon { handle: self.icon_fmt_strikethrough.clone(), message: Message::FormatStrikethrough });
                    items.push(ToolbarItem::Icon { handle: self.icon_fmt_code.clone(),          message: Message::FormatCode });
                    items.push(ToolbarItem::Icon { handle: self.icon_fmt_codeblock.clone(),     message: Message::FormatCodeBlock });
                    items.push(ToolbarItem::Icon { handle: self.icon_fmt_heading.clone(),       message: Message::FormatHeading });
                    items.push(ToolbarItem::Icon { handle: self.icon_fmt_bullet.clone(),        message: Message::FormatBullet });
                    items.push(ToolbarItem::Separator);
                }
                items.push(ToolbarItem::Icon { handle: self.icon_duplicate.clone(), message: Message::DuplicateCard(card_id) });
                items.push(ToolbarItem::Icon { handle: self.icon_delete.clone(),    message: Message::DeleteCard(card_id) });

                let pill_w = CardToolbar::<Message>::measure_width(&items);
                let pill_h = CardToolbar::<Message>::pill_height();

                // Apply zoom transform: zoom-1 screen pos → actual screen pos
                let z = self.canvas_zoom;
                let vcx = self.window_size.width  / 2.0;
                let vcy = self.window_size.height / 2.0;
                let zoom1_x = card.current_position.x + self.canvas_offset.x;
                let zoom1_y = card.current_position.y + self.canvas_offset.y;
                let card_screen_x = vcx + (zoom1_x - vcx) * z;
                let card_screen_y = vcy + (zoom1_y - vcy) * z;
                let card_width_z  = card.width * z;

                // Centre above card, clamped to window
                let mut pill_x = card_screen_x + card_width_z / 2.0 - pill_w / 2.0;
                let mut pill_y = card_screen_y - pill_h - 8.0;
                pill_x = pill_x.max(8.0).min((self.window_size.width - pill_w - 8.0).max(8.0));
                pill_y = pill_y.max(8.0);

                let toolbar: Element<Message> = CardToolbar::new(
                    items,
                    Point::new(pill_x, pill_y),
                    self.theme.sidebar_background(),
                    self.theme.button_border(),
                    self.theme.sidebar_shadow(),
                    self.theme.icon_color(),
                    self.theme.accent_bg_from(self.accent_color),
                    self.theme.button_text(),
                ).into();

                view = Overlay::new(view, toolbar).into();
            }
        }

        // Add card icon menu AFTER toolbar and card overlays so it renders on top
        if let Some(pos) = self.card_icon_menu_position {
            let card_menu_content = self.build_card_icon_menu();
            let card_menu: Element<Message> = ContextMenu::new(
                card_menu_content,
                pos,
                sidebar_bg,
                self.theme.button_border(),
                sidebar_shadow,
            )
            .width(200.0)
            .on_close(Message::HideCardIconMenu)
            .into();

            view = Overlay::new(view, card_menu).into();
        }

        // Card type menu (shown when clicking the right type icon)
        if let Some(pos) = self.card_type_menu_position {
            if let Some(card_id) = self.card_type_menu_card_id {
                let type_menu_content = self.build_card_type_menu(card_id);
                let type_menu: Element<Message> = ContextMenu::new(
                    type_menu_content,
                    pos,
                    sidebar_bg,
                    self.theme.button_border(),
                    sidebar_shadow,
                )
                .width(155.0)
                .on_close(Message::HideCardTypeMenu)
                .into();
                view = Overlay::new(view, type_menu).into();
            }
        }

        // Build sidebar content with title
        let btn_style = CardButtonStyle {
            background: self.theme.button_background(),
            background_hovered: accent_bg,
            text_color: self.theme.button_text(),
            border_color: self.theme.button_border(),
            shadow_color: self.theme.button_shadow(),
        };

        let settings_btn_style = btn_style.clone();
        let menu_btn_style = btn_style.clone();
        let floating_btn_style = btn_style.clone();

        let menu_icon = self.icon_menu.clone();
        let menu_button = button(
            container(
                svg(menu_icon)
                    .width(20)
                    .height(20)
                    .class(SvgStyle { color: icon_color })
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .height(40)
        .width(40)
        .class(menu_btn_style)
        .on_press(Message::ToggleAppMenu);

        let settings_button = button(
            container(
                svg(settings_icon)
                    .width(20)
                    .height(20)
                    .class(SvgStyle { color: icon_color })
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .height(40)
        .width(40)
        .class(settings_btn_style)
        .on_press(Message::ToggleSettings);

        let sidebar_title = row![
            svg(self.icon_app.clone())
                .width(22)
                .height(22),
            text("Cards")
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(self.theme.button_text()),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let top_row = row![
            sidebar_title,
            Space::with_width(Length::Fill),
            menu_button,
            settings_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let separator = container(Space::with_height(1))
            .width(Length::Fill)
            .height(1)
            .style(move |_theme: &IcedTheme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(accent_separator)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            });

        let top_separator = container(Space::with_height(1))
            .width(Length::Fill)
            .height(1)
            .style(move |_theme: &IcedTheme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(accent_separator)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            });

        let toggle_button = button(
            container(
                row![
                    svg(self.icon_menu_left.clone())
                        .width(20)
                        .height(20)
                        .class(SvgStyle { color: icon_color }),
                    text("Hide Sidebar").size(14),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .height(40)
        .width(Length::Fill)
        .class(btn_style)
        .on_press(Message::ToggleSidebar);

        // Build boards section with animations
        let mut board_buttons = column![].spacing(5);

        // Calculate animation values
        let animation_active = self.board_list_animating;
        let progress = self.board_list_animation_progress;
        let animation_type = self.board_list_animation_type;

        // Get the board index being animated (if any)
        let animated_board_index = self.animating_board_index;

        // Don't render button in list during position change animation
        let skip_button_in_list = animation_active && animation_type == BoardAnimationType::ButtonPositionChange;

        // Add "Add New Board" button at top if configured (and not animating position change)
        if self.config.general.new_board_button_at_top && !skip_button_in_list {
            let add_board_btn_style = CardButtonStyle {
                background: Color::TRANSPARENT,
                background_hovered: accent_bg,
                text_color: self.theme.button_text(),
                border_color: Color::TRANSPARENT,
                shadow_color: Color::TRANSPARENT,
            };

            let add_board_button = button(
                container(
                    text("+ Add New Board").size(14)
                )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::Center)
                    .padding(Padding {
                        top: 0.0,
                        right: 10.0,
                        bottom: 0.0,
                        left: 10.0,
                    })
            )
            .height(36)
            .width(Length::Fill)
            .class(add_board_btn_style)
            .on_press(Message::AddNewBoard);

            board_buttons = board_buttons.push(add_board_button);
        }

        for (index, board_name) in self.boards.iter().enumerate() {
            let is_active = index == self.active_board_index;
            let is_hovered = self.hovered_board_index == Some(index);
            let is_being_animated = animated_board_index == Some(index);
            let is_editing = self.editing_board_index == Some(index);

            let board_btn_style = if is_active {
                CardButtonStyle {
                    background: accent_bg,
                    background_hovered: accent_bg,
                    text_color: self.theme.button_text(),
                    border_color: accent,
                    shadow_color: Color::TRANSPARENT,
                }
            } else {
                CardButtonStyle {
                    background: Color::TRANSPARENT,
                    background_hovered: accent_bg,
                    text_color: self.theme.button_text(),
                    border_color: Color::TRANSPARENT,
                    shadow_color: Color::TRANSPARENT,
                }
            };

            let delete_btn_style = CardButtonStyle {
                background: Color::TRANSPARENT,
                background_hovered: accent_bg,
                text_color: self.theme.button_text(),
                border_color: Color::TRANSPARENT,
                shadow_color: Color::TRANSPARENT,
            };

            // Board content - either text input for editing or button
            let board_content: Element<Message> = if is_editing {
                // Show text input when editing - styled to match the rest of the app
                let text_input_widget = text_input("", &self.board_rename_value)
                    .on_input(Message::BoardRenameInput)
                    .on_submit(Message::FinishRenamingBoard)
                    .padding(Padding {
                        top: 0.0,
                        right: 10.0,
                        bottom: 0.0,
                        left: 10.0,
                    })
                    .size(14.0)
                    .style(|theme: &IcedTheme, _status| {
                        // Custom styling to match board button appearance
                        use iced::widget::text_input::Style;

                        Style {
                            background: iced::Background::Color(Color::TRANSPARENT),
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: 4.0.into(),
                            },
                            icon: Color::TRANSPARENT,
                            placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                            value: if theme == &IcedTheme::Dark {
                                Color::WHITE
                            } else {
                                Color::BLACK
                            },
                            selection: if theme == &IcedTheme::Dark {
                                Color::from_rgba(0.3, 0.5, 0.7, 0.5)
                            } else {
                                Color::from_rgba(0.4, 0.6, 0.8, 0.5)
                            },
                        }
                    });

                container(text_input_widget)
                    .width(Length::Fill)
                    .height(Length::Fixed(36.0))
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .align_y(Alignment::Center)
                    .style(move |theme: &IcedTheme| {
                        container::Style {
                            background: Some(iced::Background::Color(
                                if theme == &IcedTheme::Dark {
                                    Color::from_rgba(0.2, 0.2, 0.2, 1.0)
                                } else {
                                    Color::from_rgba(0.9, 0.9, 0.9, 1.0)
                                }
                            )),
                            border: Border {
                                color: if theme == &IcedTheme::Dark {
                                    Color::from_rgba(0.4, 0.4, 0.4, 1.0)
                                } else {
                                    Color::from_rgba(0.7, 0.7, 0.7, 1.0)
                                },
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            shadow: Shadow::default(),
                            text_color: None,
                        }
                    })
                    .into()
            } else if self.boards.len() > 1 && is_hovered {
                // When hovered, show delete button
                row![
                    button(
                        container(
                            text(board_name.clone()).size(14)
                        )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Alignment::Start)
                            .align_y(Alignment::Center)
                            .padding(Padding {
                                top: 0.0,
                                right: 5.0,
                                bottom: 0.0,
                                left: 10.0,
                            })
                    )
                    .height(36)
                    .width(Length::Fill)
                    .class(board_btn_style)
                    .on_press(Message::SelectBoard(index)),
                    button(
                        container(
                            svg(self.icon_delete.clone())
                                .width(28)
                                .height(28)
                                .class(SvgStyle { color: Color::from_rgb(0.8, 0.2, 0.2) })
                        )
                            .width(36)
                            .height(36)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                    )
                    .height(36)
                    .width(36)
                    .class(delete_btn_style)
                    .on_press(Message::DeleteBoard(index))
                ]
                .spacing(0)
                .into()
            } else {
                // When not hovered or only one board, full width button
                button(
                    container(
                        text(board_name.clone()).size(14)
                    )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center)
                        .padding(Padding {
                            top: 0.0,
                            right: 10.0,
                            bottom: 0.0,
                            left: 10.0,
                        })
                )
                .height(36)
                .width(Length::Fill)
                .class(board_btn_style)
                .on_press(Message::SelectBoard(index))
                .into()
            };

            // Wrap in mouse_area to detect hover
            let board_with_hover = mouse_area(board_content)
                .on_enter(Message::BoardHover(Some(index)))
                .on_exit(Message::BoardHover(None));

            // Apply animation based on type
            let final_board = if is_being_animated && animation_active {
                match animation_type {
                    BoardAnimationType::AddBoard => {
                        // Slide in from 0 to 36
                        let height = 36.0 * progress;
                        container(board_with_hover)
                            .height(Length::Fixed(height))
                    }
                    BoardAnimationType::DeleteBoard => {
                        // Slide out from 36 to 0 (reverse of add)
                        let height = 36.0 * (1.0 - progress);
                        container(board_with_hover)
                            .height(Length::Fixed(height))
                    }
                    _ => container(board_with_hover)
                }
            } else {
                container(board_with_hover)
            };

            board_buttons = board_buttons.push(final_board);
        }

        // Add "Add New Board" button at bottom if not at top (and not during animation)
        if !self.config.general.new_board_button_at_top && !skip_button_in_list {
            let add_board_btn_style = CardButtonStyle {
                background: Color::TRANSPARENT,
                background_hovered: self.theme.accent_bg(),
                text_color: self.theme.button_text(),
                border_color: Color::TRANSPARENT,
                shadow_color: Color::TRANSPARENT,
            };

            let add_board_button = button(
                container(
                    text("+ Add New Board").size(14)
                )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::Center)
                    .padding(Padding {
                        top: 0.0,
                        right: 10.0,
                        bottom: 0.0,
                        left: 10.0,
                    })
            )
            .height(36)
            .width(Length::Fill)
            .class(add_board_btn_style)
            .on_press(Message::AddNewBoard);

            board_buttons = board_buttons.push(add_board_button);
        }

        // Create a scrollable container for the boards
        // Calculate offset for entire board list during button position animation
        let board_list_offset = if animation_active && animation_type == BoardAnimationType::ButtonPositionChange {
            let button_height = 36.0 + 5.0; // height + spacing
            if self.config.general.new_board_button_at_top {
                // Button moving UP to top: boards move DOWN as a group
                button_height * progress
            } else {
                // Button moving DOWN to bottom: boards move UP (back) as a group
                button_height * (1.0 - progress)
            }
        } else {
            0.0
        };

        let boards_container = container(board_buttons)
            .width(Length::Fill)
            .padding(Padding {
                top: 5.0 + board_list_offset,
                right: 10.0,
                bottom: 5.0,
                left: 10.0,
            });

        let boards_section = scrollable(boards_container)
            .height(Length::Fill)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(3)
                    .scroller_width(3)
            ))
            .style(Self::scrollbar_style(accent));

        // Wrap boards section in stack if animating button position
        let boards_with_animation: Element<Message> = if animation_active && animation_type == BoardAnimationType::ButtonPositionChange {
            println!("DEBUG: Creating stacked animated button at progress {:.2}", progress);

            let board_count = self.boards.len();
            let spacing = 5.0;
            let button_height = 36.0;
            let item_height = button_height + spacing;

            // Calculate button Y position relative to boards section
            let start_y = if self.config.general.new_board_button_at_top {
                // Moving from bottom to top
                5.0 + (board_count as f32 * item_height) // 5.0 is top padding
            } else {
                // Moving from top to bottom
                5.0
            };

            let end_y = if self.config.general.new_board_button_at_top {
                5.0
            } else {
                5.0 + (board_count as f32 * item_height)
            };

            let current_y = start_y + (end_y - start_y) * progress;

            println!("DEBUG: Stacked button Y: {:.1} (start: {:.1}, end: {:.1})", current_y, start_y, end_y);

            let add_board_btn_style = CardButtonStyle {
                background: self.theme.accent_bg(),
                background_hovered: self.theme.accent_bg(),
                text_color: self.theme.button_text(),
                border_color: self.theme.button_border(),
                shadow_color: Color::TRANSPARENT,
            };

            let animated_button_overlay = container(
                button(
                    container(
                        text("+ Add New Board").size(14)
                    )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center)
                        .padding(Padding {
                            top: 0.0,
                            right: 10.0,
                            bottom: 0.0,
                            left: 10.0,
                        })
                )
                .height(36)
                .width(Length::Fill)
                .class(add_board_btn_style)
                .on_press(Message::AddNewBoard)
            )
            .width(Length::Fill)
            .padding(Padding {
                top: current_y,
                right: 10.0,
                bottom: 0.0,
                left: 10.0,
            });

            // Stack the boards section with the animated button on top
            stack![
                boards_section,
                animated_button_overlay
            ].into()
        } else {
            boards_section.into()
        };

        let sidebar_content = column![
            container(top_row)
                .width(Length::Fill)
                .padding(Padding::new(10.0)),
            container(top_separator)
                .width(Length::Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 0.0,
                    left: 20.0,
                }),
            boards_with_animation,
            container(separator)
                .width(Length::Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 10.0,
                    left: 20.0,
                }),
            container(toggle_button)
                .width(Length::Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 10.0,
                    bottom: 10.0,
                    left: 10.0,
                }),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Pill background is drawn by sidebar.rs; button style is transparent so it doesn't
        // draw its own background (avoids double background with wrong radius/shadow).
        let floating_transparent_style = CardButtonStyle {
            background:          Color::TRANSPARENT,
            background_hovered:  self.theme.accent_bg_from(self.accent_color),
            text_color:          self.theme.button_text(),
            border_color:        Color::TRANSPARENT,
            shadow_color:        Color::TRANSPARENT,
        };
        let floating_button = button(
            container(
                svg(self.icon_menu_right.clone())
                    .width(20)
                    .height(20)
                    .class(SvgStyle { color: icon_color })
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .height(40)
        .width(40)
        .class(floating_transparent_style)
        .on_press(Message::ToggleSidebar);

        let sidebar: Element<Message> = Sidebar::new(
            sidebar_content,
            SIDEBAR_WIDTH,
            sidebar_bg,
            accent_bg,
            sidebar_shadow,
            self.sidebar_offset,
        )
        .pill_border(self.theme.button_border())
        .floating_button(floating_button)
        .into();

        // IMPORTANT: Add sidebar overlay LAST (except settings) to ensure it renders on top of all card elements
        // The order is: base canvas -> context menu -> card menu -> toolbar -> SIDEBAR -> app menu -> settings
        view = Overlay::new(view, sidebar).into();

        // Add app menu dropdown (on top of sidebar)
        if self.app_menu_open || self.app_menu_animating {
            // Position menu aligned with the sidebar's left edge, below the top button row.
            // Sidebar is at x=15+offset, width=250. Menu button row height = 10(padding) + 40(button) + 10(padding) = 60.
            let sidebar_screen_x = 15.0 + self.sidebar_offset;
            let menu_pos = Point::new(
                sidebar_screen_x + 8.0,
                15.0 + 60.0 + 4.0, // sidebar top-y + top-row height + small gap
            );

            let toggle_theme_label = match self.theme {
                crate::theme::Theme::Light => "Switch to Dark Mode",
                crate::theme::Theme::Dark => "Switch to Light Mode",
            };

            let recents = self.config.general.recent_workspaces.clone();
            let has_recents = !recents.is_empty();

            let items: Vec<AppMenuItem<Message>> = vec![
                AppMenuItem::Label("FILE".to_string()),
                AppMenuItem::Button { label: "New Workspace".to_string(),    message: Message::MenuFileNewWorkspace },
                AppMenuItem::Button { label: "Open Workspace…".to_string(),  message: Message::MenuFileOpenWorkspace },
                AppMenuItem::SubMenu { label: "Open Recent".to_string(),     enabled: has_recents,
                    on_hover: Some(Message::MenuRecentSubmenuOpen),
                    on_close: Some(Message::MenuRecentSubmenuClose) },
                AppMenuItem::SubMenu { label: "Import / Export".to_string(), enabled: true,
                    on_hover: Some(Message::MenuImportExportSubmenuOpen),
                    on_close: Some(Message::MenuImportExportSubmenuClose) },
                AppMenuItem::Button { label: "New Board".to_string(),        message: Message::MenuFileNewBoard },
                AppMenuItem::Button { label: "Quit".to_string(),             message: Message::MenuFileQuit },
                AppMenuItem::Separator,
                AppMenuItem::Label("VIEW".to_string()),
                AppMenuItem::Button { label: "Reset Canvas".to_string(),     message: Message::MenuViewResetCanvas },
                AppMenuItem::Button { label: "Reset Zoom".to_string(),       message: Message::MenuViewResetZoom },
                AppMenuItem::Button { label: "Toggle Sidebar".to_string(),   message: Message::MenuViewToggleSidebar },
                AppMenuItem::Button { label: toggle_theme_label.to_string(), message: Message::MenuViewToggleTheme },
                AppMenuItem::Separator,
                AppMenuItem::Label("HELP".to_string()),
                AppMenuItem::Button { label: "Keyboard Shortcuts".to_string(), message: Message::MenuHelpKeyboardShortcuts },
                AppMenuItem::Button { label: "About".to_string(),            message: Message::MenuHelpAbout },
            ];

            // Use the card/button background so the menu visually pops above the sidebar bg.
            let menu_bg = self.theme.button_background();

            let app_menu: Element<Message> = AppMenu::new(items, menu_pos)
                .width(SIDEBAR_WIDTH - 16.0)
                .background(menu_bg)
                .border_color(self.theme.button_border())
                .text_color(self.theme.button_text())
                .separator_color(self.theme.accent_glow_from(self.accent_color))
                .hover_color(accent_bg)
                .shadow_color(sidebar_shadow)
                .on_close(Message::CloseAppMenu)
                .animation_progress(self.app_menu_animation_progress)
                .into();

            view = Overlay::new(view, app_menu).into();

            // Open Recent submenu — shown to the right of the main menu when hovered
            if self.recent_submenu_open && has_recents {
                // Position: right of main menu, aligned with the "Open Recent" row
                // Main menu is at sidebar_screen_x + 8, width = SIDEBAR_WIDTH-16
                // "Open Recent" is item index 3 (0-based): Label + New WS + Open WS = 3 rows before
                let item_h = 32.0;
                let label_h = 26.0;
                let submenu_x = menu_pos.x + (SIDEBAR_WIDTH - 16.0) + 4.0;
                // y: top padding 6 + Label(26) + Button(32) + Button(32) + align to this item
                let submenu_y = menu_pos.y + 6.0 + label_h + item_h + item_h;

                let submenu_items: Vec<AppMenuItem<Message>> = recents.iter()
                    .map(|p| {
                        let label = std::path::Path::new(p)
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.clone());
                        AppMenuItem::Button { label, message: Message::MenuFileOpenRecent(p.clone()) }
                    })
                    .collect();

                let submenu: Element<Message> = AppMenu::new(submenu_items, Point::new(submenu_x, submenu_y))
                    .width(200.0)
                    .background(menu_bg)
                    .border_color(self.theme.button_border())
                    .text_color(self.theme.button_text())
                    .separator_color(self.theme.accent_glow_from(self.accent_color))
                    .hover_color(accent_bg)
                    .shadow_color(sidebar_shadow)
                    .on_close(Message::MenuRecentSubmenuClose)
                    .animation_progress(1.0)
                    .into();

                view = Overlay::new(view, submenu).into();
            }

            // Import / Export submenu
            if self.import_export_submenu_open {
                let item_h = 32.0;
                let label_h = 26.0;
                let submenu_x = menu_pos.x + (SIDEBAR_WIDTH - 16.0) + 4.0;
                // "Import / Export" is 4th item (index 3): Label(26) + 3×Button(32) = 122
                let submenu_y = menu_pos.y + 6.0 + label_h + item_h * 3.0;

                let ie_items: Vec<AppMenuItem<Message>> = vec![
                    AppMenuItem::Button { label: "Import Workspace".to_string(), message: Message::MenuImportWorkspace },
                    AppMenuItem::Button { label: "Export Workspace".to_string(), message: Message::MenuExportWorkspace },
                    AppMenuItem::Separator,
                    AppMenuItem::Button { label: "Import Board".to_string(), message: Message::MenuImportBoard },
                    AppMenuItem::Button { label: "Export Board".to_string(), message: Message::MenuExportBoard },
                ];

                let ie_submenu: Element<Message> = AppMenu::new(ie_items, Point::new(submenu_x, submenu_y))
                    .width(190.0)
                    .background(menu_bg)
                    .border_color(self.theme.button_border())
                    .text_color(self.theme.button_text())
                    .separator_color(self.theme.accent_glow_from(self.accent_color))
                    .hover_color(accent_bg)
                    .shadow_color(sidebar_shadow)
                    .on_close(Message::MenuImportExportSubmenuClose)
                    .animation_progress(1.0)
                    .into();

                view = Overlay::new(view, ie_submenu).into();
            }
        }

        // Add delete confirmation dialog (on top of sidebar, below settings)
        if self.confirm_delete_card_id.is_some() {
            let confirm_dialog = self.build_delete_confirm_dialog();
            view = Overlay::new(view, confirm_dialog).modal().into();
        }

        // Add settings modal LAST (on top of everything)
        if self.settings_open || self.settings_animating {
            // Calculate scale based on animation progress
            let scale = if self.settings_animating {
                if self.settings_opening {
                    // Ease out cubic for opening
                    let t = self.settings_animation_progress;
                    let eased = 1.0 - (1.0 - t).powi(3);
                    0.8 + (eased * 0.2) // Scale from 0.8 to 1.0
                } else {
                    // Ease in cubic for closing
                    let t = self.settings_animation_progress;
                    let eased = 1.0 - t.powi(3);
                    0.8 + (eased * 0.2) // Scale from 1.0 to 0.8
                }
            } else {
                1.0
            };

            let settings_content = self.build_settings_content();
            let settings_modal: Element<Message> = SettingsModal::new(
                settings_content,
                sidebar_bg,
                accent_bg,
                sidebar_shadow,
            )
            .width(700.0)
            .height(500.0)
            .scale(scale)
            .on_close(|| Message::CloseSettings)
            .into();

            view = Overlay::new(view, settings_modal).modal().into();
        }

        // Add theme transition overlay - diagonal wipe animation
        if self.theme_transitioning {            let transition_progress = self.theme_transition_progress;

            // Apply cubic bezier easing (ease-in-out) for smooth start and end
            // Using cubic bezier curve for smooth acceleration and deceleration
            let t = transition_progress;
            let eased_progress = if t < 0.5 {
                // Ease in: slow start
                4.0 * t * t * t
            } else {
                // Ease out: slow end
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            };

            // Get the OLD theme's background color (before transition)
            // We'll wipe this away to reveal the new theme
            let wipe_color = if let Some(next_theme) = self.next_theme {
                // Show the opposite of next theme (the old theme)
                match next_theme {
                    Theme::Light => Color::from_rgb(0.1, 0.1, 0.1), // Show dark if going to light
                    Theme::Dark => Color::from_rgb(0.95, 0.95, 0.95), // Show light if going to dark
                }
            } else {
                Color::from_rgb(0.5, 0.5, 0.5)
            };

            // Create a canvas-based diagonal wipe effect
            use iced::widget::canvas::{self, Canvas, Frame, Path};
            use iced::widget::canvas::Program;

            struct DiagonalWipeOverlay {
                progress: f32,
                color: Color,
            }

            impl Program<Message> for DiagonalWipeOverlay {
                type State = ();

                fn draw(
                    &self,
                    _state: &Self::State,
                    renderer: &iced::Renderer,
                    _theme: &iced::Theme,
                    bounds: iced::Rectangle,
                    _cursor: iced::mouse::Cursor,
                ) -> Vec<canvas::Geometry> {
                    let mut frame = Frame::new(renderer, bounds.size());

                    if self.progress >= 1.0 {
                        // Wipe complete - draw nothing
                        return vec![frame.into_geometry()];
                    }

                    if self.progress <= 0.0 {
                        // No wipe yet - draw entire screen
                        let full_rect = Path::rectangle(
                            iced::Point::ORIGIN,
                            iced::Size::new(bounds.width, bounds.height)
                        );
                        frame.fill(&full_rect, self.color);
                        return vec![frame.into_geometry()];
                    }

                    // Simple approach: The wipe edge moves from top-left to bottom-right
                    // The wipe line is perpendicular to the diagonal
                    // At progress = 0.5, the wipe line goes through the center
                    // At progress = 1.0, the wipe line passes through bottom-right corner

                    // Calculate the total distance the wipe needs to travel
                    // The wipe line travels from (0,0) to beyond (width, height)
                    // The perpendicular distance it needs to cover is (width + height) / sqrt(2)
                    let total_wipe_distance = (bounds.width + bounds.height) / 1.414;
                    let current_wipe_distance = total_wipe_distance * self.progress;

                    // The wipe line is perpendicular to the diagonal (45 degrees)
                    // Points on the wipe line: it intersects top and left edges
                    // x_on_top + y_on_left = current_wipe_distance * sqrt(2)
                    let sum = current_wipe_distance * 1.414;

                    // Intersection with top edge (y=0): x = sum
                    let x_on_top = sum;
                    // Intersection with left edge (x=0): y = sum
                    let y_on_left = sum;

                    // Draw the remaining area (not yet wiped)
                    let remaining_path = Path::new(|builder| {
                        if x_on_top < bounds.width {
                            // Wipe line intersects top edge
                            builder.move_to(iced::Point::new(x_on_top, 0.0));
                            builder.line_to(iced::Point::new(bounds.width, 0.0));
                            builder.line_to(iced::Point::new(bounds.width, bounds.height));

                            if y_on_left < bounds.height {
                                // Wipe line also intersects left edge
                                builder.line_to(iced::Point::new(0.0, bounds.height));
                                builder.line_to(iced::Point::new(0.0, y_on_left));
                            } else {
                                // Wipe line intersects bottom edge instead
                                let x_on_bottom = sum - bounds.height;
                                if x_on_bottom > 0.0 && x_on_bottom < bounds.width {
                                    builder.line_to(iced::Point::new(x_on_bottom, bounds.height));
                                } else {
                                    builder.line_to(iced::Point::new(0.0, bounds.height));
                                }
                            }
                        } else {
                            // Wipe line has passed top-right corner
                            // It now intersects right and bottom (or left) edges
                            let y_on_right = sum - bounds.width;
                            if y_on_right < bounds.height {
                                builder.move_to(iced::Point::new(bounds.width, y_on_right));
                                builder.line_to(iced::Point::new(bounds.width, bounds.height));

                                let x_on_bottom = sum - bounds.height;
                                if x_on_bottom > 0.0 && x_on_bottom < bounds.width {
                                    builder.line_to(iced::Point::new(x_on_bottom, bounds.height));
                                } else if x_on_bottom <= 0.0 {
                                    builder.line_to(iced::Point::new(0.0, bounds.height));
                                }
                            }
                            // else: wipe complete
                        }

                        builder.close();
                    });

                    frame.fill(&remaining_path, self.color);

                    vec![frame.into_geometry()]
                }
            }

            let wipe_overlay = Canvas::new(DiagonalWipeOverlay {
                progress: eased_progress,
                color: wipe_color,
            })
            .width(Length::Fill)
            .height(Length::Fill);

            view = Overlay::new(view, wipe_overlay).modal().into();
        }

        // Workspace modal — rendered absolutely last (topmost layer)
        if let Some(ref modal_state) = self.workspace_modal {
            let modal_overlay = workspace_modal::view(modal_state, self.theme, self.accent_color)
                .map(Message::WorkspaceModal);
            let backdrop_msg = match modal_state {
                WorkspaceModalState::OpeningExisting { .. } |
                WorkspaceModalState::ImportingWorkspace { .. } => {
                    Message::WorkspaceModal(workspace_modal::WorkspaceModalMessage::FilePicker(
                        crate::file_picker::FilePickerMessage::Cancel
                    ))
                }
                _ => Message::WorkspaceModal(workspace_modal::WorkspaceModalMessage::CancelNew),
            };
            view = Overlay::new(view, modal_overlay)
                .modal()
                .on_backdrop_press(backdrop_msg)
                .into();
        }

        // Import / Export modal — above workspace modal
        if let Some(ref ie_state) = self.import_export_modal {
            let ie_overlay = import_export_modal::view(ie_state, self.theme, self.accent_color)
                .map(Message::ImportExport);
            view = Overlay::new(view, ie_overlay)
                .modal()
                .on_backdrop_press(Message::ImportExport(ImportExportMessage::Cancel))
                .into();
        }

        // Image picker modal — topmost
        if let Some((_, ref picker_state)) = self.image_picker {
            let img_overlay = file_picker::view(picker_state, self.theme, self.accent_color)
                .map(Message::ImagePickerMsg);
            view = Overlay::new(view, img_overlay)
                .modal()
                .on_backdrop_press(Message::CloseImagePicker)
                .into();
        }

        view
    }

    fn theme(&self) -> IcedTheme {
        match self.theme {
            Theme::Light => IcedTheme::Light,
            Theme::Dark => IcedTheme::Dark,
        }
    }

    fn update_theme_colors(&mut self) {
        self.dot_grid.set_dot_color(self.theme.dot_color());
        self.dot_grid.set_background_color(self.theme.background());
        self.dot_grid.set_card_colors(
            self.theme.card_background(),
            self.theme.card_border(),
            self.theme.card_text(),
        );
        self.dot_grid.set_accent_color(self.accent_color);
    }

    /// Keep dot_grid.blocked in sync with modal state so canvas input is
    /// suppressed at the source (canvas update()), not just in message dispatch.
    fn sync_grid_blocked(&mut self) {
        self.dot_grid.blocked = self.workspace_modal.is_some()
            || self.settings_open
            || self.confirm_delete_card_id.is_some()
            || self.import_export_modal.is_some()
            || self.image_picker.is_some();
    }

    fn update_exclude_region(&mut self) {
        if self.settings_open {
            let region = Rectangle {
                x: 0.0,
                y: 0.0,
                width: self.window_size.width,
                height: self.window_size.height,
            };
            self.dot_grid.set_exclude_region(Some(region));
            return;
        }

        let sidebar_width = SIDEBAR_WIDTH;
        let sidebar_x = 15.0 + self.sidebar_offset;

        if sidebar_x + sidebar_width < 0.0 {
            let button_x = 25.0;
            let button_y = self.window_size.height - 40.0 - 25.0;
            let region = Rectangle {
                x: button_x - 10.0,
                y: button_y - 10.0,
                width: 60.0,
                height: 60.0,
            };
            self.dot_grid.set_exclude_region(Some(region));
        } else {
            let visible_x = sidebar_x.max(0.0);

            let region = Rectangle {
                x: visible_x,
                y: 15.0,
                width: sidebar_width,
                height: self.window_size.height - 30.0,
            };
            self.dot_grid.set_exclude_region(Some(region));
        }
    }

    fn is_point_in_sidebar(&self, point: Point) -> bool {
        let sidebar_width = SIDEBAR_WIDTH;
        let sidebar_x = 15.0 + self.sidebar_offset;

        // Check if sidebar is visible
        if sidebar_x + sidebar_width < 0.0 {
            // Sidebar hidden, check floating button
            let button_x = 25.0;
            let button_y = self.window_size.height - 40.0 - 25.0;
            let button_bounds = Rectangle {
                x: button_x - 10.0,
                y: button_y - 10.0,
                width: 60.0,
                height: 60.0,
            };
            button_bounds.contains(point)
        } else {
            // Sidebar visible
            let sidebar_bounds = Rectangle {
                x: sidebar_x,
                y: 15.0,
                width: sidebar_width,
                height: self.window_size.height - 30.0,
            };
            sidebar_bounds.contains(point)
        }
    }

    fn build_app_menu(&self) -> Element<Message> {
        let icon_color = self.theme.icon_color();
        let accent_bg = self.theme.accent_bg_from(self.accent_color);
        let separator_color = self.theme.accent_glow_from(self.accent_color);

        let item_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: accent_bg,
            text_color: self.theme.button_text(),
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };

        // Helper: section label
        fn make_label<'a>(label: &'static str, color: Color) -> Element<'a, Message> {
            container(
                text(label)
                    .size(11)
                    .color(Color::from_rgba(color.r, color.g, color.b, 0.6))
                    .font(iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() })
            )
            .padding(Padding { top: 6.0, right: 12.0, bottom: 2.0, left: 12.0 })
            .into()
        }

        // Helper: separator line
        fn make_sep<'a>(sep_color: Color) -> Element<'a, Message> {
            container(Space::with_height(1))
                .width(Length::Fill)
                .height(1)
                .style(move |_: &IcedTheme| container::Style {
                    background: Some(iced::Background::Color(sep_color)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                })
                .padding(Padding { top: 0.0, right: 8.0, bottom: 0.0, left: 8.0 })
                .into()
        }

        // Helper: 4px spacer
        fn make_gap<'a>() -> Element<'a, Message> {
            Space::with_height(4).into()
        }

        let file_label = make_label("FILE", icon_color);
        let view_label = make_label("VIEW", icon_color);
        let help_label = make_label("HELP", icon_color);

        let new_board_btn: Element<Message> = button(
            container(text("New Board").size(13))
                .width(Length::Fill)
                .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 })
        )
        .width(Length::Fill)
        .class(item_style.clone())
        .on_press(Message::MenuFileNewBoard)
        .into();

        let quit_btn: Element<Message> = button(
            container(text("Quit").size(13))
                .width(Length::Fill)
                .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 })
        )
        .width(Length::Fill)
        .class(item_style.clone())
        .on_press(Message::MenuFileQuit)
        .into();

        let reset_btn: Element<Message> = button(
            container(text("Reset Canvas").size(13))
                .width(Length::Fill)
                .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 })
        )
        .width(Length::Fill)
        .class(item_style.clone())
        .on_press(Message::MenuViewResetCanvas)
        .into();

        let toggle_sidebar_btn: Element<Message> = button(
            container(text("Toggle Sidebar").size(13))
                .width(Length::Fill)
                .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 })
        )
        .width(Length::Fill)
        .class(item_style.clone())
        .on_press(Message::MenuViewToggleSidebar)
        .into();

        // Theme label changes based on current theme
        let toggle_theme_label = match self.theme {
            crate::theme::Theme::Light => "Switch to Dark Mode",
            crate::theme::Theme::Dark => "Switch to Light Mode",
        };
        let toggle_theme_btn: Element<Message> = button(
            container(text(toggle_theme_label).size(13))
                .width(Length::Fill)
                .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 })
        )
        .width(Length::Fill)
        .class(item_style.clone())
        .on_press(Message::MenuViewToggleTheme)
        .into();

        let shortcuts_btn: Element<Message> = button(
            container(text("Keyboard Shortcuts").size(13))
                .width(Length::Fill)
                .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 })
        )
        .width(Length::Fill)
        .class(item_style.clone())
        .on_press(Message::MenuHelpKeyboardShortcuts)
        .into();

        let about_btn: Element<Message> = button(
            container(text("About").size(13))
                .width(Length::Fill)
                .padding(Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 })
        )
        .width(Length::Fill)
        .class(item_style)
        .on_press(Message::MenuHelpAbout)
        .into();

        let content = column![
            file_label,
            new_board_btn,
            quit_btn,
            make_gap(),
            make_sep(separator_color),
            make_gap(),
            view_label,
            reset_btn,
            toggle_sidebar_btn,
            toggle_theme_btn,
            make_gap(),
            make_sep(separator_color),
            make_gap(),
            help_label,
            shortcuts_btn,
            about_btn,
        ]
        .spacing(0)
        .padding(Padding { top: 6.0, right: 0.0, bottom: 6.0, left: 0.0 });

        container(content)
            .into()
    }

    fn build_context_menu(&self) -> Element<Message> {
        let icon_color = self.theme.icon_color();
        let text_color = self.theme.button_text();
        let separator_color = self.theme.separator_color();

        let btn_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: self.theme.accent_bg_from(self.accent_color),
            text_color: text_color,
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };

        let item_padding = Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 };

        let add_text_btn = button(
            row![
                svg(self.icon_type_text.clone())
                    .width(15).height(15)
                    .class(SvgStyle { color: icon_color }),
                text("Text Card").size(13).color(text_color),
            ]
            .spacing(9)
            .align_y(Alignment::Center)
            .padding(item_padding)
        )
        .width(Length::Fill)
        .class(btn_style.clone())
        .on_press(Message::AddCardOfType(CardType::Text));

        let add_md_btn = button(
            row![
                svg(self.icon_type_markdown.clone())
                    .width(15).height(15)
                    .class(SvgStyle { color: icon_color }),
                text("Markdown Card").size(13).color(text_color),
            ]
            .spacing(9)
            .align_y(Alignment::Center)
            .padding(item_padding)
        )
        .width(Length::Fill)
        .class(btn_style.clone())
        .on_press(Message::AddCardOfType(CardType::Markdown));

        let sep = container(Space::with_height(1))
            .width(Length::Fill).height(1)
            .style(move |_: &IcedTheme| container::Style {
                background: Some(iced::Background::Color(separator_color)),
                ..Default::default()
            });

        let sep2 = container(Space::with_height(1))
            .width(Length::Fill).height(1)
            .style(move |_: &IcedTheme| container::Style {
                background: Some(iced::Background::Color(separator_color)),
                ..Default::default()
            });

        let add_img_btn = button(
            row![
                svg(self.icon_type_image.clone())
                    .width(15).height(15)
                    .class(SvgStyle { color: icon_color }),
                text("Image Card").size(13).color(text_color),
            ]
            .spacing(9)
            .align_y(Alignment::Center)
            .padding(item_padding)
        )
        .width(Length::Fill)
        .class(btn_style.clone())
        .on_press(Message::AddCardOfType(CardType::Image));

        container(
            column![
                add_text_btn,
                sep,
                add_md_btn,
                sep2,
                add_img_btn,
            ]
            .padding(4.0)
        )
        .into()
    }

    fn build_card_type_menu(&self, card_id: usize) -> Element<Message> {
        let icon_color = self.theme.icon_color();
        let text_color = self.theme.button_text();
        let separator_color = self.theme.separator_color();
        let accent_bg = self.theme.accent_bg_from(self.accent_color);

        let current_type = self.dot_grid.cards()
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.card_type)
            .unwrap_or(CardType::Text);

        let btn_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: accent_bg,
            text_color,
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };
        let active_style = CardButtonStyle {
            background: accent_bg,
            background_hovered: accent_bg,
            text_color,
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };

        let item_padding = Padding { top: 7.0, right: 12.0, bottom: 7.0, left: 12.0 };

        let text_style  = if current_type == CardType::Text     { active_style.clone() } else { btn_style.clone() };
        let md_style    = if current_type == CardType::Markdown { active_style.clone() } else { btn_style.clone() };
        let img_style   = if current_type == CardType::Image    { active_style.clone() } else { btn_style.clone() };

        let text_btn = button(
            row![
                svg(self.icon_type_text.clone()).width(15).height(15)
                    .class(SvgStyle { color: icon_color }),
                text("Text").size(13).color(text_color),
            ]
            .spacing(9).align_y(Alignment::Center).padding(item_padding)
        )
        .width(Length::Fill)
        .class(text_style)
        .on_press(Message::ChangeCardType(card_id, CardType::Text));

        let md_btn = button(
            row![
                svg(self.icon_type_markdown.clone()).width(15).height(15)
                    .class(SvgStyle { color: icon_color }),
                text("Markdown").size(13).color(text_color),
            ]
            .spacing(9).align_y(Alignment::Center).padding(item_padding)
        )
        .width(Length::Fill)
        .class(md_style)
        .on_press(Message::ChangeCardType(card_id, CardType::Markdown));

        let sep = container(Space::with_height(1))
            .width(Length::Fill).height(1)
            .style(move |_: &IcedTheme| container::Style {
                background: Some(iced::Background::Color(separator_color)),
                ..Default::default()
            });
        let sep2 = container(Space::with_height(1))
            .width(Length::Fill).height(1)
            .style(move |_: &IcedTheme| container::Style {
                background: Some(iced::Background::Color(separator_color)),
                ..Default::default()
            });

        let img_btn = button(
            row![
                svg(self.icon_type_image.clone()).width(15).height(15)
                    .class(SvgStyle { color: icon_color }),
                text("Image").size(13).color(text_color),
            ]
            .spacing(9).align_y(Alignment::Center).padding(item_padding)
        )
        .width(Length::Fill)
        .class(img_style)
        .on_press(Message::ChangeCardType(card_id, CardType::Image));

        container(column![text_btn, sep, md_btn, sep2, img_btn].padding(4.0)).into()
    }

    fn build_card_icon_menu(&self) -> Element<Message> {
        let separator_color = self.theme.icon_color().scale_alpha(0.2);
        let accent_bg = self.theme.accent_bg_from(self.accent_color);

        // Get the current card's color — find by ID, not by Vec index
        let card_color = if let Some(card_id) = self.card_icon_menu_card_id {
            self.dot_grid.cards().iter()
                .find(|c| c.id == card_id)
                .map(|c| c.color)
                .unwrap_or(Color::from_rgb8(124, 92, 252))
        } else {
            Color::from_rgb8(124, 92, 252)
        };

        let icon_btn_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: accent_bg,
            text_color: self.theme.button_text(),
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };

        // Build icon grid (6 icons per row)
        let mut icon_rows = column![].spacing(0);
        let icons = CardIcon::all();
        let icons_per_row = 6;

        for chunk in icons.chunks(icons_per_row) {
            let mut icon_row = row![].spacing(0);

            for icon in chunk {
                let svg_data = icon_util::icon_to_svg(icon.get_icondata());
                let icon_btn = button(
                    container(
                        svg(svg::Handle::from_memory(svg_data))
                            .width(16)
                            .height(16)
                            .class(SvgStyle { color: card_color })
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                )
                .width(32)
                .height(32)
                .class(icon_btn_style.clone())
                .on_press(Message::ChangeCardIcon(self.card_icon_menu_card_id.unwrap(), *icon));

                icon_row = icon_row.push(icon_btn);
            }

            icon_rows = icon_rows.push(icon_row);
        }

        // Custom scrollbar style - uses card color to match the card's accent
        // Scrollable icon area
        let scrollable_icons = scrollable(
            container(icon_rows)
                .padding(Padding::new(5.0))
                .width(Length::Fill)
        )
        .height(Length::Fixed(300.0))
        .width(Length::Fill)
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(3)
                .scroller_width(3)
        ))
        .style(Self::scrollbar_style(card_color));

        // Separator
        let separator = container(Space::with_height(1))
            .width(Length::Fill)
            .height(1)
            .style(move |_theme: &IcedTheme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(separator_color)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            });

        // Color selection grid (fixed at bottom) — same palette as accent color picker
        let colors: Vec<Color> = AccentColor::all().iter().map(|ac| ac.to_color()).collect();

        let border_color = self.theme.button_text();
        let mut color_rows = column![].spacing(8);
        let colors_per_row = 5;

        for chunk in colors.chunks(colors_per_row) {
            let mut color_row = row![].spacing(8).align_y(Alignment::Center);

            for &color in chunk {
                let color_btn = button(
                    container(Space::with_height(0))
                        .width(30)
                        .height(30)
                        .style(move |_theme: &IcedTheme| {
                            container::Style {
                                background: Some(iced::Background::Color(color)),
                                border: Border {
                                    radius: 6.0.into(), // Rounded square to match settings
                                    width: 2.0,
                                    color: if color == card_color {
                                        border_color
                                    } else {
                                        Color::TRANSPARENT
                                    },
                                },
                                shadow: Shadow::default(),
                                text_color: None,
                            }
                        })
                )
                .padding(0)
                .class(icon_btn_style.clone())
                .on_press(Message::ChangeCardColor(self.card_icon_menu_card_id.unwrap(), color));

                color_row = color_row.push(color_btn);
            }

            color_rows = color_rows.push(color_row);
        }

        // Main layout: scrollable icons on top, fixed separator and colors at bottom
        let content = column![
            scrollable_icons,
            container(separator)
                .width(Length::Fill)
                .padding(Padding {
                    top: 8.0,
                    right: 10.0,
                    bottom: 8.0,
                    left: 10.0,
                }),
            container(color_rows)
                .padding(Padding::new(10.0))
                .width(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill);

        container(content)
            .into()
    }

    fn build_settings_content(&self) -> Element<Message> {
        let separator_color = self.theme.separator_color();
        let text_color = self.theme.button_text();
        let icon_color = self.theme.icon_color();
        let accent = self.accent_color;
        let accent_bg = self.theme.accent_bg_from(self.accent_color);

        let settings_title = text("Settings")
            .size(18)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            });

        let close_btn_style = CardButtonStyle {
            background: self.theme.button_background(),
            background_hovered: accent_bg,
            text_color: self.theme.button_text(),
            border_color: self.theme.button_border(),
            shadow_color: self.theme.button_shadow(),
        };

        let close_button = button(
            container(
                svg(self.icon_close.clone())
                    .width(16)
                    .height(16)
                    .class(SvgStyle { color: icon_color })
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .height(32)
        .width(32)
        .class(close_btn_style)
        .on_press(Message::ToggleSettings);

        let top_bar = container(
            row![
                settings_title,
                Space::with_width(Length::Fill),
                close_button,
            ]
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(Padding {
            top: 15.0,
            right: 20.0,
            bottom: 15.0,
            left: 20.0,
        });

        let top_separator = container(Space::with_height(1))
            .width(Length::Fill)
            .height(1)
            .style(move |_theme: &IcedTheme| {
                container::Style {
                    background: Some(iced::Background::Color(separator_color)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            });

        let mut category_buttons = column![].spacing(5).width(150.0);

        for category in SettingsCategory::all() {
            let is_selected = *category == self.settings_category;

            let cat_btn_style = if is_selected {
                CardButtonStyle {
                    background: accent_bg,
                    background_hovered: accent_bg,
                    text_color: self.theme.button_text(),
                    border_color: accent,
                    shadow_color: Color::TRANSPARENT,
                }
            } else {
                CardButtonStyle {
                    background: Color::TRANSPARENT,
                    background_hovered: accent_bg,
                    text_color: self.theme.button_text(),
                    border_color: Color::TRANSPARENT,
                    shadow_color: Color::TRANSPARENT,
                }
            };

            let cat_button = button(
                container(text(category.label()).size(14))
                    .width(Length::Fill)
                    .padding(Padding {
                        top: 8.0,
                        right: 12.0,
                        bottom: 8.0,
                        left: 12.0,
                    })
            )
            .width(Length::Fill)
            .class(cat_btn_style)
            .on_press(Message::SelectSettingsCategory(*category));

            category_buttons = category_buttons.push(cat_button);
        }

        let categories_panel = container(
            scrollable(category_buttons)
        )
        .width(150.0)
        .height(Length::Fill)
        .padding(Padding::new(10.0));

        let vertical_separator = container(Space::with_width(1))
            .width(1)
            .height(Length::Fill)
            .style(move |_theme: &IcedTheme| {
                container::Style {
                    background: Some(iced::Background::Color(separator_color)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            });

        let settings_panel = self.build_category_settings();

        let content_row = row![
            categories_panel,
            vertical_separator,
            settings_panel,
        ]
        .height(Length::Fill);

        let bottom_separator = container(Space::with_height(1))
            .width(Length::Fill)
            .height(1)
            .style(move |_theme: &IcedTheme| {
                container::Style {
                    background: Some(iced::Background::Color(separator_color)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            });

        let app_name = text(APP_NAME)
            .size(12)
            .color(Color::from_rgba(
                text_color.r,
                text_color.g,
                text_color.b,
                0.6,
            ));

        let version = text(format!("v{}", APP_VERSION))
            .size(12)
            .color(Color::from_rgba(
                text_color.r,
                text_color.g,
                text_color.b,
                0.6,
            ));

        let bottom_bar = container(
            row![
                app_name,
                Space::with_width(Length::Fill),
                version,
            ]
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(Padding {
            top: 10.0,
            right: 20.0,
            bottom: 10.0,
            left: 20.0,
        });

        column![
            top_bar,
            top_separator,
            content_row,
            bottom_separator,
            bottom_bar,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn build_category_settings(&self) -> Element<Message> {
        let content: Element<Message> = match self.settings_category {
            SettingsCategory::General => {
                let sidebar_open_label = text("Open sidebar on start").size(14);
                let sidebar_open_btn = self.build_toggle_button(
                    self.config.general.sidebar_open_on_start,
                    Message::SetSidebarOpenOnStart(!self.config.general.sidebar_open_on_start),
                );

                let animations_label = text("Enable animations").size(14);
                let animations_btn = self.build_toggle_button(
                    self.config.general.enable_animations,
                    Message::SetAnimationsEnabled(!self.config.general.enable_animations),
                );

                let board_btn_label = text("New board button at top").size(14);
                let board_btn_toggle = self.build_toggle_button(
                    self.config.general.new_board_button_at_top,
                    Message::SetNewBoardButtonAtTop(!self.config.general.new_board_button_at_top),
                );

                let confirm_delete_label = text("Confirm card deletion").size(14);
                let confirm_delete_toggle = self.build_toggle_button(
                    self.config.general.confirm_card_delete,
                    Message::SetConfirmCardDelete(!self.config.general.confirm_card_delete),
                );

                column![
                    text("General Settings").size(16).font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    Space::with_height(20),
                    row![
                        sidebar_open_label,
                        Space::with_width(Length::Fill),
                        sidebar_open_btn,
                    ]
                    .align_y(Alignment::Center),
                    Space::with_height(15),
                    row![
                        animations_label,
                        Space::with_width(Length::Fill),
                        animations_btn,
                    ]
                    .align_y(Alignment::Center),
                    Space::with_height(15),
                    row![
                        board_btn_label,
                        Space::with_width(Length::Fill),
                        board_btn_toggle,
                    ]
                    .align_y(Alignment::Center),
                    Space::with_height(15),
                    row![
                        confirm_delete_label,
                        Space::with_width(Length::Fill),
                        confirm_delete_toggle,
                    ]
                    .align_y(Alignment::Center),
                    Space::with_height(5),
                    text("Show a confirmation dialog before deleting cards")
                        .size(12)
                        .color(Color::from_rgba(
                            self.theme.button_text().r,
                            self.theme.button_text().g,
                            self.theme.button_text().b,
                            0.6,
                        )),
                ]
                .spacing(10)
                .into()
            }
            SettingsCategory::Appearance => {
                let theme_label = text("Theme").size(14);
                let accent_color = self.accent_color;
                let accent_bg = self.theme.accent_bg_from(accent_color);

                let light_btn_style = CardButtonStyle {
                    background: if matches!(self.theme, Theme::Light) { accent_bg } else { Color::TRANSPARENT },
                    background_hovered: accent_bg,
                    text_color: self.theme.button_text(),
                    border_color: if matches!(self.theme, Theme::Light) { accent_color } else { self.theme.button_border() },
                    shadow_color: Color::TRANSPARENT,
                };

                let dark_btn_style = CardButtonStyle {
                    background: if matches!(self.theme, Theme::Dark) { accent_bg } else { Color::TRANSPARENT },
                    background_hovered: accent_bg,
                    text_color: self.theme.button_text(),
                    border_color: if matches!(self.theme, Theme::Dark) { accent_color } else { self.theme.button_border() },
                    shadow_color: Color::TRANSPARENT,
                };

                let light_button = button(
                    container(text("Light").size(14))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .width(80)
                .height(36)
                .class(light_btn_style)
                .on_press(Message::SetTheme(Theme::Light));

                let dark_button = button(
                    container(text("Dark").size(14))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .width(80)
                .height(36)
                .class(dark_btn_style)
                .on_press(Message::SetTheme(Theme::Dark));

                // Accent color picker
                let accent_label = text("Accent Color").size(14);
                let current_accent = self.config.appearance.accent_color;
                let border_text_color = self.theme.button_text();

                let mut accent_color_row = row![].spacing(6).align_y(Alignment::Center);
                for &ac in AccentColor::all() {
                    let color = ac.to_color();
                    let is_selected = ac == current_accent;
                    let circle_btn_style = CardButtonStyle {
                        background: color,
                        background_hovered: color,
                        text_color: Color::WHITE,
                        border_color: if is_selected { border_text_color } else { Color::TRANSPARENT },
                        shadow_color: Color::TRANSPARENT,
                    };
                    let btn = button(Space::new(0, 0))
                        .width(24)
                        .height(24)
                        .class(circle_btn_style)
                        .on_press(Message::SetAccentColor(ac));
                    accent_color_row = accent_color_row.push(btn);
                }

                // Font family dropdown
                let font_family_label = text("Font Family").size(14);

                let theme = self.theme;
                let ac = accent_color;
                let pick_list_style = move |_theme: &IcedTheme, status: pick_list::Status| {
                    let background = theme.card_background();
                    let text_color = theme.card_text();
                    let border_color = match status {
                        pick_list::Status::Opened => ac,
                        _ => theme.button_border(),
                    };

                    pick_list::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        placeholder_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        handle_color: match status {
                            pick_list::Status::Opened => ac,
                            _ => text_color,
                        },
                        border: Border {
                            color: border_color,
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                    }
                };

                let menu_style = move |_theme: &IcedTheme| {
                    let background = theme.card_background();
                    let text_color = theme.card_text();
                    iced::overlay::menu::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        selected_background: iced::Background::Color(theme.accent_bg_from(ac)),
                        selected_text_color: ac,
                        border: Border {
                            color: ac,
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                    }
                };

                let font_family_picker = pick_list(
                    FontFamily::all(),
                    Some(self.config.appearance.font.family),
                    Message::SetFontFamily,
                )
                .width(200)
                .text_size(14)
                .padding(8)
                .style(pick_list_style)
                .menu_style(menu_style);

                // Font size dropdown
                let font_size_label = text("Font Size").size(14);
                let current_size = FontSize::SIZES
                    .iter()
                    .find(|s| (s.0 - self.config.appearance.font.size).abs() < 0.1)
                    .copied();

                let pick_list_style2 = move |_theme: &IcedTheme, status: pick_list::Status| {
                    let background = theme.card_background();
                    let text_color = theme.card_text();
                    let border_color = match status {
                        pick_list::Status::Opened => ac,
                        _ => theme.button_border(),
                    };

                    pick_list::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        placeholder_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        handle_color: match status {
                            pick_list::Status::Opened => ac,
                            _ => text_color,
                        },
                        border: Border {
                            color: border_color,
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                    }
                };

                let menu_style2 = move |_theme: &IcedTheme| {
                    let background = theme.card_background();
                    let text_color = theme.card_text();
                    iced::overlay::menu::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        selected_background: iced::Background::Color(theme.accent_bg_from(ac)),
                        selected_text_color: ac,
                        border: Border {
                            color: ac,
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                    }
                };

                let font_size_picker = pick_list(
                    FontSize::SIZES,
                    current_size,
                    |size: FontSize| Message::SetFontSize(size.0),
                )
                .width(100)
                .text_size(14)
                .padding(8)
                .style(pick_list_style2)
                .menu_style(menu_style2);

                column![
                    text("Appearance").size(16).font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    Space::with_height(20),
                    row![
                        theme_label,
                        Space::with_width(Length::Fill),
                        light_button,
                        dark_button,
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    Space::with_height(15),
                    row![
                        accent_label,
                        Space::with_width(Length::Fill),
                        accent_color_row,
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    Space::with_height(20),
                    text("Fonts").size(14).font(iced::Font {
                        weight: iced::font::Weight::Semibold,
                        ..Default::default()
                    }),
                    Space::with_height(10),
                    row![
                        font_family_label,
                        Space::with_width(Length::Fill),
                        font_family_picker,
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    Space::with_height(10),
                    row![
                        font_size_label,
                        Space::with_width(Length::Fill),
                        font_size_picker,
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(10)
                .into()
            }
            SettingsCategory::Shortcuts => {
                let text_color = self.theme.button_text();
                let accent     = self.accent_color;
                let dim_color  = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.55);
                let sep_color  = Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.12);
                let key_bg     = Color::from_rgba(accent.r, accent.g, accent.b, 0.18);
                let key_bdr    = Color::from_rgba(accent.r, accent.g, accent.b, 0.40);

                // All shortcut data as plain slices — no closures with lifetimes
                // Format: ("key label", "description") — None key = section header, ("---", "") = separator
                let entries: &[(&str, &str)] = &[
                    // Canvas
                    ("SECTION", "Canvas"),
                    ("Ctrl + 0",              "Recenter canvas to origin"),
                    ("Middle Mouse",          "Pan canvas"),
                    ("Scroll",                "Pan canvas vertically / horizontally"),
                    ("Click + Drag",          "Pan canvas (on empty space)"),
                    ("SEP", ""),
                    // Zoom
                    ("SECTION", "Zoom"),
                    ("Ctrl + Scroll",         "Zoom in / out toward cursor"),
                    ("Ctrl + =  /  Ctrl + +", "Zoom in"),
                    ("Ctrl + \u{2212}",       "Zoom out"),
                    ("Ctrl + Shift + 0",      "Reset zoom to 100%"),
                    ("\u{2212}  /  %  /  +",  "Zoom bar: zoom out / reset / zoom in"),
                    ("SEP", ""),
                    // Boards
                    ("SECTION", "Boards"),
                    ("Ctrl + Tab",            "Switch to next board"),
                    ("Ctrl + Shift + Tab",    "Switch to previous board"),
                    ("Double-click board",    "Rename board"),
                    ("SEP", ""),
                    // Cards
                    ("SECTION", "Cards"),
                    ("N",                     "New card at mouse / canvas centre"),
                    ("Right-click canvas",    "Open context menu  \u{2192}  Add Card"),
                    ("Click",                 "Edit card"),
                    ("Drag header",           "Move card"),
                    ("Drag \u{2198} handle",  "Resize card"),
                    ("Delete",                "Delete selected card(s)"),
                    ("Ctrl + D",              "Duplicate selected card(s)"),
                    ("Esc",                   "Stop editing / deselect"),
                    ("SEP", ""),
                    // Multi-select
                    ("SECTION", "Multi-select"),
                    ("Drag empty canvas",     "Box-select cards"),
                    ("Drag header (multi)",   "Move all selected cards"),
                    ("Delete (multi)",        "Delete all selected cards"),
                    ("Ctrl + D (multi)",      "Duplicate all selected cards"),
                    ("SEP", ""),
                    // Text Editing
                    ("SECTION", "Text Editing"),
                    ("Tab",                   "Insert 4 spaces"),
                    ("Enter",                 "New line"),
                    ("Backspace",             "Delete previous character"),
                    ("Ctrl + Backspace",      "Delete previous word"),
                    ("Delete",                "Delete next character"),
                    ("Ctrl + Delete",         "Delete next word"),
                    ("SEP", ""),
                    // Cursor Navigation
                    ("SECTION", "Cursor Navigation"),
                    ("Arrow Keys",            "Move cursor"),
                    ("Ctrl + \u{2190} / \u{2192}", "Jump to previous / next word"),
                    ("Home",                  "Move to start of line"),
                    ("End",                   "Move to end of line"),
                    ("SEP", ""),
                    // Text Selection
                    ("SECTION", "Text Selection"),
                    ("Shift + Arrows",        "Extend selection"),
                    ("Shift + Ctrl + \u{2190} / \u{2192}", "Extend selection word by word"),
                    ("Click + Drag",          "Select text with mouse"),
                    ("Ctrl + A",              "Select all text (while editing)"),
                    ("SEP", ""),
                    // Clipboard
                    ("SECTION", "Clipboard"),
                    ("Ctrl + C",              "Copy selected text"),
                    ("Ctrl + X",              "Cut selected text"),
                    ("Ctrl + V",              "Paste from clipboard"),
                    ("SEP", ""),
                    // Toolbar
                    ("SECTION", "Toolbar  (card selected)"),
                    ("# button",              "Heading prefix"),
                    ("B button",              "Bold  **text**"),
                    ("I button",              "Italic  *text*"),
                    ("S button",              "Strikethrough  ~~text~~"),
                    ("` button",              "Inline code"),
                    ("</> button",            "Code block"),
                    ("\u{2022} button",       "Bullet point"),
                    ("Duplicate button",      "Duplicate card"),
                    ("Delete button",         "Delete card  (confirmation dialog)"),
                    ("SEP", ""),
                    // App
                    ("SECTION", "App"),
                    ("Esc",                   "Close menus / dialogs / settings"),
                ];

                let mut col: iced::widget::Column<Message> = column![
                    text("Keyboard Shortcuts").size(16).font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    Space::with_height(16),
                ].spacing(0);

                for (k, d) in entries {
                    if *k == "SECTION" {
                        col = col.push(
                            text(*d).size(13).font(iced::Font {
                                weight: iced::font::Weight::Semibold,
                                ..Default::default()
                            }).color(dim_color)
                        );
                        col = col.push(Space::with_height(6));
                    } else if *k == "SEP" {
                        col = col.push(Space::with_height(10));
                        col = col.push(
                            container(Space::with_height(1))
                                .width(Length::Fill)
                                .height(1)
                                .style(move |_: &IcedTheme| container::Style {
                                    background: Some(iced::Background::Color(sep_color)),
                                    ..Default::default()
                                })
                        );
                        col = col.push(Space::with_height(10));
                    } else {
                        col = col.push(
                            row![
                                text(*d).size(13).color(text_color),
                                Space::with_width(Length::Fill),
                                container(text(*k).size(12).color(text_color))
                                    .padding(Padding { top: 2.0, right: 7.0, bottom: 2.0, left: 7.0 })
                                    .style(move |_: &IcedTheme| container::Style {
                                        background: Some(iced::Background::Color(key_bg)),
                                        border: Border { color: key_bdr, width: 1.0, radius: 5.0.into() },
                                        ..Default::default()
                                    }),
                            ]
                            .align_y(Alignment::Center)
                        );
                        col = col.push(Space::with_height(4));
                    }
                }

                col.into()
            }
            SettingsCategory::About => {
                let config_path = Config::config_path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                column![
                    text("About").size(16).font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    Space::with_height(20),
                    text(format!("{} v{}", APP_NAME, APP_VERSION)).size(14),
                    Space::with_height(10),
                    text("A card-based application built with Iced.").size(14),
                    Space::with_height(20),
                    text("Config file location:").size(12),
                    text(config_path).size(12),
                    Space::with_height(30),
                    // Debug mode toggle
                    row![
                        text("Debug Mode").size(14),
                        Space::with_width(Length::Fill),
                        self.build_toggle_button(
                            self.config.general.debug_mode,
                            Message::SetDebugMode(!self.config.general.debug_mode)
                        ),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                    text("Enable debug output in the console").size(12)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                ]
                .spacing(10)
                .into()
            }
        };

        let accent = self.accent_color;

        container(
            scrollable(
                container(content)
                    .padding(Padding {
                        top: 15.0,
                        right: 20.0,
                        bottom: 15.0,
                        left: 20.0,
                    })
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(3)
                    .scroller_width(3)
            ))
            .style(Self::scrollbar_style(accent))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }


    /// Returns a unified scrollbar style closure.
    /// `thumb_color` is the accent/card color; the track is the same color at low alpha.
    fn scrollbar_style(thumb_color: Color) -> impl Fn(&IcedTheme, iced::widget::scrollable::Status) -> iced::widget::scrollable::Style {
        move |_theme: &IcedTheme, _status: iced::widget::scrollable::Status| {
            use iced::widget::scrollable::{Rail, Scroller};
            let track_color = Color { a: 0.10, ..thumb_color };
            let thumb = Color { a: 0.55, ..thumb_color };
            let rail = Rail {
                background: Some(iced::Background::Color(track_color)),
                border: Border { radius: 3.0.into(), ..Default::default() },
                scroller: Scroller {
                    color: thumb,
                    border: Border { radius: 3.0.into(), ..Default::default() },
                },
            };
            iced::widget::scrollable::Style {
                container: iced::widget::container::Style::default(),
                vertical_rail: rail,
                horizontal_rail: Rail {
                    background: None,
                    border: Border::default(),
                    scroller: Scroller {
                        color: thumb,
                        border: Border { radius: 3.0.into(), ..Default::default() },
                    },
                },
                gap: None,
            }
        }
    }

    fn build_toggle_button(&self, is_on: bool, message: Message) -> Element<Message> {
        let accent_bg = self.theme.accent_bg_from(self.accent_color);
        let btn_style = CardButtonStyle {
            background: if is_on { accent_bg } else { Color::TRANSPARENT },
            background_hovered: accent_bg,
            text_color: self.theme.button_text(),
            border_color: if is_on { self.accent_color } else { self.theme.button_border() },
            shadow_color: Color::TRANSPARENT,
        };

        button(
            container(text(if is_on { "On" } else { "Off" }).size(14))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(60)
        .height(32)
        .class(btn_style)
        .on_press(message)
        .into()
    }

    fn build_delete_confirm_dialog(&self) -> Element<Message> {
        let bg_color = self.theme.sidebar_background();
        let text_color = self.theme.button_text();
        let accent = self.accent_color;
        let accent_bg = self.theme.accent_bg_from(accent);
        let shadow_color = self.theme.sidebar_shadow();
        let border_color = self.theme.button_border();

        let cancel_btn_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: accent_bg,
            text_color,
            border_color,
            shadow_color: Color::TRANSPARENT,
        };

        let delete_btn_style = CardButtonStyle {
            background: Color::from_rgb(0.75, 0.18, 0.18),
            background_hovered: Color::from_rgb(0.9, 0.2, 0.2),
            text_color: Color::WHITE,
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };

        let cancel_button = button(
            container(text("Cancel").size(14))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(90)
        .height(34)
        .class(cancel_btn_style)
        .on_press(Message::CancelDeleteCard);

        let delete_button = button(
            container(text("Delete").size(14))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(90)
        .height(34)
        .class(delete_btn_style)
        .on_press(Message::ConfirmDeleteCard);

        let dialog_content = container(
            column![
                text("Delete Card")
                    .size(16)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .color(text_color),
                Space::with_height(8),
                text("Are you sure you want to delete this card?")
                    .size(14)
                    .color(text_color),
                text("This action cannot be undone.")
                    .size(13)
                    .color(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.65)),
                Space::with_height(16),
                row![
                    Space::with_width(Length::Fill),
                    cancel_button,
                    Space::with_width(8),
                    delete_button,
                ]
                .align_y(Alignment::Center),
            ]
            .padding(20)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill);

        // Use SettingsModal to get the dimmed overlay and centered positioning behavior.
        // SettingsModal already draws its own gradient background — do NOT add a background
        // to dialog_content or there will be two overlapping bodies.
        let modal_content: Element<Message> = SettingsModal::new(
            dialog_content,
            bg_color,
            accent_bg,
            shadow_color,
        )
        .width(340.0)
        .height(160.0)
        .on_close(|| Message::CancelDeleteCard)
        .into();

        modal_content
    }
}
