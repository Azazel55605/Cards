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
mod markdown;
mod custom_text_editor;
mod icon_util;
mod positioned;
mod text_document;
mod text_renderer;
mod markdown_parser;
mod text_processor;

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
use config::{Config, FontFamily};
use context_menu::ContextMenu;
use card::{Card, CardIcon};
use positioned::Positioned;

// Application constants (not user-configurable)
const SIDEBAR_WIDTH: f32 = 250.0;
const DOT_SPACING: f32 = 30.0;
const DOT_RADIUS: f32 = 2.0;
const ANIMATION_DURATION_MS: f32 = 250.0;

// Custom text editor style with visible cursor
struct TransparentTextEditorStyle {
    theme: Theme,
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
                value: match self.theme {
                    Theme::Light => Color::from_rgb8(0, 0, 0),
                    Theme::Dark => Color::from_rgb8(255, 255, 255),
                },
                selection: Color::from_rgba(0.4, 0.6, 1.0, 0.5),
            },
            text_editor::Status::Hovered => text_editor::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border {
                    color: Color::from_rgb8(100, 150, 255),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                icon: cursor_color,
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                value: match self.theme {
                    Theme::Light => Color::from_rgb8(0, 0, 0),
                    Theme::Dark => Color::from_rgb8(255, 255, 255),
                },
                selection: Color::from_rgba(0.4, 0.6, 1.0, 0.5),
            },
            text_editor::Status::Focused => text_editor::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border {
                    color: Color::from_rgb8(100, 150, 255),
                    width: 2.0,
                    radius: 4.0.into(),
                },
                icon: cursor_color,
                placeholder: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                value: match self.theme {
                    Theme::Light => Color::from_rgb8(0, 0, 0),
                    Theme::Dark => Color::from_rgb8(255, 255, 255),
                },
                selection: Color::from_rgba(0.4, 0.6, 1.0, 0.5),
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
                selection: Color::from_rgba(0.4, 0.6, 1.0, 0.3),
            },
        }
    }
}

const APP_NAME: &str = "Cards";
const APP_VERSION: &str = "0.1.0";

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
    Tick(Instant),
    DotGridMessage(DotGridMessage),
    EventOccurred(Event),
    // Context menu messages
    ShowContextMenu(Point),
    HideContextMenu,
    AddCard,
    // Card messages
    ShowCardIconMenu(usize),
    ChangeCardIcon(usize, CardIcon),
    ChangeCardColor(usize, Color),
    HideCardIconMenu,
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
    // Context menu state
    context_menu_position: Option<Point>,
    pending_card_position: Option<Point>,  // Store position for card creation
    // Card customization menu
    card_icon_menu_position: Option<Point>,
    card_icon_menu_card_id: Option<usize>,
    // Card editing state
    editing_card_id: Option<usize>,
    selected_card_id: Option<usize>,  // Track selected card for toolbar
    clipboard_text: String,  // Store clipboard content
    // Board management
    boards: Vec<String>,  // List of board names
    active_board_index: usize,  // Currently active board
    hovered_board_index: Option<usize>,  // Track which board is being hovered
    editing_board_index: Option<usize>,  // Track which board is being renamed
    board_rename_value: String,  // Current value during rename
    board_cards: HashMap<usize, Vec<card::Card>>,  // Store cards for each board
    // Board animations
    board_list_animating: bool,  // Animation for add/delete/reorder
    board_list_animation_progress: f32,
    board_list_animation_type: BoardAnimationType,
    animating_board_index: Option<usize>,  // Track which board is being animated
    // Configuration
    config: Config,
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
    window_size: iced::Size,
    last_tick: Instant,
}

const SIDEBAR_HIDDEN_OFFSET: f32 = -280.0;
const BOARD_ANIMATION_DURATION_MS: f32 = 150.0; // Faster animation for board changes

impl Cards {
    fn new(config: Config) -> (Self, Task<Message>) {
        let theme: Theme = config.appearance.theme.into();
        let sidebar_open = config.general.sidebar_open_on_start;
        let sidebar_offset = if sidebar_open { 0.0 } else { SIDEBAR_HIDDEN_OFFSET };

        let mut dot_grid = DotGrid::new(theme.dot_color(), theme.background());
        dot_grid.set_dot_spacing(DOT_SPACING);
        dot_grid.set_dot_radius(DOT_RADIUS);
        dot_grid.set_card_colors(
            theme.card_background(),
            theme.card_border(),
            theme.card_text(),
        );
        // Set debug mode from config
        dot_grid.set_debug_mode(config.general.debug_mode);
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
            context_menu_position: None,
            pending_card_position: None,
            card_icon_menu_position: None,
            card_icon_menu_card_id: None,
            editing_card_id: None,
            selected_card_id: None,
            clipboard_text: String::new(),
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
            board_list_animating: false,
            board_list_animation_progress: 0.0,
            board_list_animation_type: BoardAnimationType::None,
            animating_board_index: None,
            config,
            icon_menu_left: svg::Handle::from_memory(include_bytes!("icons/menu-left.svg")),
            icon_menu_right: svg::Handle::from_memory(include_bytes!("icons/menu-right.svg")),
            icon_moon: svg::Handle::from_memory(include_bytes!("icons/moon.svg")),
            icon_sun: svg::Handle::from_memory(include_bytes!("icons/sun.svg")),
            icon_settings: svg::Handle::from_memory(include_bytes!("icons/settings.svg")),
            icon_close: svg::Handle::from_memory(include_bytes!("icons/close.svg")),
            icon_add: svg::Handle::from_memory(include_bytes!("icons/add.svg")),
            icon_duplicate: svg::Handle::from_memory(include_bytes!("icons/duplicate.svg")),
            icon_delete: svg::Handle::from_memory(include_bytes!("icons/delete.svg")),
            window_size: iced::Size::new(800.0, 600.0),
            last_tick: Instant::now(),
        };
        cards.update_exclude_region();
        (cards, Task::none())
    }

    // Helper function to convert icondata to complete SVG

    fn update(&mut self, message: Message) -> Task<Message> {
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
                self.theme = self.theme.toggle();
                self.update_theme_colors();
                if let Err(e) = self.config.set_theme(self.theme) {
                    eprintln!("Failed to save theme: {}", e);
                }
            }
            Message::SetTheme(theme) => {
                if self.theme != theme {
                    self.theme = theme;
                    self.update_theme_colors();
                    if let Err(e) = self.config.set_theme(self.theme) {
                        eprintln!("Failed to save theme: {}", e);
                    }
                }
            }
            Message::ToggleSettings => {
                self.settings_open = !self.settings_open;
                self.context_menu_position = None;
                self.pending_card_position = None;
                self.dot_grid.set_effect_enabled(!self.settings_open);
                self.update_exclude_region();
            }
            Message::CloseSettings => {
                self.settings_open = false;
                self.dot_grid.set_effect_enabled(true);
                self.update_exclude_region();
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
                                            // Load cards for the new active board
                                            let new_cards = self.board_cards.get(&self.active_board_index).cloned().unwrap_or_default();
                                            self.dot_grid.load_cards(new_cards);
                                        } else if self.active_board_index > index {
                                            self.active_board_index = self.active_board_index.saturating_sub(1);
                                            // Active board shifted, reload its cards
                                            let new_cards = self.board_cards.get(&self.active_board_index).cloned().unwrap_or_default();
                                            self.dot_grid.load_cards(new_cards);
                                        } else if self.active_board_index == index {
                                            // Deleted the active board, switch to new active board
                                            let new_cards = self.board_cards.get(&self.active_board_index).cloned().unwrap_or_default();
                                            self.dot_grid.load_cards(new_cards);
                                        }

                                        self.dot_grid.clear_cards_cache();
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

                // Save current board's cards to keep them synced
                self.save_current_board_cards();

                self.update_exclude_region();
            }
            Message::DotGridMessage(msg) => {
                match msg {
                    DotGridMessage::Pan(delta) => {
                        self.context_menu_position = None;
                        self.pending_card_position = None;
                        self.card_icon_menu_position = None;
                        self.card_icon_menu_card_id = None;
                        self.canvas_offset.x += delta.x;
                        self.canvas_offset.y += delta.y;
                        self.dot_grid.set_offset(self.canvas_offset);
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
                        }
                    }
                    DotGridMessage::CardLeftClickBar(card_id, _pos) => {
                        // Start dragging - keep card selected but stop editing
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            card.is_dragging = true;
                            card.is_editing = false; // Stop editing when dragging
                        }
                        self.editing_card_id = None;
                        // Keep selected_card_id so toolbar stays visible during drag
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CardLeftClickBody(card_id) => {
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

                            // Update checkbox positions for cards that were editing
                            for id in card_ids {
                                self.dot_grid.update_card_checkbox_positions(id);
                            }

                            self.editing_card_id = None;
                            self.selected_card_id = None;
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

                            // Update checkbox positions for previously editing cards
                            for id in previously_editing {
                                self.dot_grid.update_card_checkbox_positions(id);
                            }

                            // Start editing the card and select it
                            self.editing_card_id = Some(card_id);
                            self.selected_card_id = Some(card_id);
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
                    DotGridMessage::CardDrag(card_id, pos, drag_offset) => {
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            if card.is_dragging {
                                // Convert screen pos to world pos, accounting for drag offset
                                let world_pos = Point::new(
                                    pos.x - self.canvas_offset.x - drag_offset.x,
                                    pos.y - self.canvas_offset.y - drag_offset.y,
                                );
                                card.target_position = world_pos;
                                card.current_position = world_pos;
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
                        // Final snap to grid with smooth animation
                        let dot_spacing = self.dot_grid.dot_spacing();
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            let final_width = ((card.width / dot_spacing).round() * dot_spacing).max(Card::MIN_WIDTH);
                            let final_height = ((card.height / dot_spacing).round() * dot_spacing).max(Card::MIN_HEIGHT);

                            // Set target size - animation will smooth the transition
                            card.target_width = final_width;
                            card.target_height = final_height;
                        }
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CardDrop(card_id) => {
                        let dot_spacing = self.dot_grid.dot_spacing();
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            card.is_dragging = false;
                            // Snap to grid
                            card.target_position = Card::snap_to_grid(card.current_position, dot_spacing);
                        }
                        // Clear selection after drag completes
                        self.selected_card_id = None;
                        self.dot_grid.clear_cards_cache();
                    }
                    DotGridMessage::CheckboxToggle(card_id, line_index) => {
                        // Toggle checkbox in the card's markdown text
                        if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                            let text = card.content.text();
                            if self.config.general.debug_mode {
                                println!("DEBUG: CheckboxToggle - card_id: {}, line_index: {}", card_id, line_index);
                                println!("DEBUG: Text before toggle:\n{}", text);
                            }
                            let updated_text = Self::toggle_checkbox_in_text(&text, line_index, self.config.general.debug_mode);
                            if self.config.general.debug_mode {
                                println!("DEBUG: Text after toggle:\n{}", updated_text);
                            }
                            card.content.set_text(updated_text);
                            self.dot_grid.clear_cards_cache();
                            // Update checkbox positions after content change
                            self.dot_grid.update_card_checkbox_positions(card_id);
                            self.save_state();
                        }
                    }
                }
            }
            Message::EventOccurred(event) => {
                match event {
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
                    Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                        if !self.settings_open {
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
                    let card_id = self.dot_grid.add_card(pos);
                    println!("Created card with id: {}, total cards: {}", card_id, self.dot_grid.cards().len());
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
            }
            Message::ChangeCardColor(card_id, color) => {
                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                    card.color = color;
                    self.dot_grid.clear_cards_cache();
                }
                self.card_icon_menu_position = None;
                self.card_icon_menu_card_id = None;
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
                            self.dot_grid.load_cards(new_cards);
                            self.dot_grid.clear_cards_cache();
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
                            self.dot_grid.load_cards(new_cards);
                            self.dot_grid.clear_cards_cache();
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
                        }
                        return Task::none();
                    }

                    if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)) {
                        // Close menus/settings/editing - but never quit the app
                        if self.editing_board_index.is_some() {
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
                            // Stop editing
                            if let Some(card_id) = self.editing_card_id {
                                if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                                    card.is_editing = false;
                                }
                            }
                            self.editing_card_id = None;
                            self.selected_card_id = None;
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
                                                    // Copy selected text to internal clipboard
                                                    if let Some(text) = card.content.get_selected_text() {
                                                        self.clipboard_text = text;
                                                    }
                                                    true
                                                }
                                                "X" => {
                                                    // Cut: copy to clipboard then delete
                                                    if let Some(text) = card.content.get_selected_text() {
                                                        self.clipboard_text = text;
                                                    }
                                                    card.content.delete_selection();
                                                    true
                                                }
                                                "V" => {
                                                    // Paste from internal clipboard
                                                    if !self.clipboard_text.is_empty() {
                                                        // Delete selection first if any
                                                        card.content.delete_selection();
                                                        // Insert clipboard content
                                                        for ch in self.clipboard_text.chars() {
                                                            card.content.insert_char(ch);
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
                                // eprintln!("Cache cleared\n");
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
                self.dot_grid.clear_cards_cache();
            }
            Message::HideCardIconMenu => {
                self.card_icon_menu_position = None;
                self.card_icon_menu_card_id = None;
            }
            Message::FormatBold => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("**", "**");
                        self.dot_grid.clear_cards_cache();
                    }
                }
            }
            Message::FormatItalic => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("*", "*");
                        self.dot_grid.clear_cards_cache();
                    }
                }
            }
            Message::FormatStrikethrough => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("~~", "~~");
                        self.dot_grid.clear_cards_cache();
                    }
                }
            }
            Message::FormatCode => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("`", "`");
                        self.dot_grid.clear_cards_cache();
                    }
                }
            }
            Message::FormatCodeBlock => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("```\n", "\n```");
                        self.dot_grid.clear_cards_cache();
                    }
                }
            }
            Message::FormatHeading => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("# ", "");
                        self.dot_grid.clear_cards_cache();
                    }
                }
            }
            Message::FormatBullet => {
                if let Some(card_id) = self.editing_card_id {
                    if let Some(card) = self.dot_grid.cards_mut().iter_mut().find(|c| c.id == card_id) {
                        card.content.wrap_selection("- ", "");
                        self.dot_grid.clear_cards_cache();
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
                    self.selected_card_id = Some(new_card_id);
                    self.dot_grid.clear_cards_cache();
                }
            }
            Message::DeleteCard(card_id) => {
                // Remove the card using DotGrid's method
                self.dot_grid.delete_card(card_id);

                // Clear selection if this was the selected card
                if self.selected_card_id == Some(card_id) {
                    self.selected_card_id = None;
                }
                if self.editing_card_id == Some(card_id) {
                    self.editing_card_id = None;
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

                        // Clear any editing/selection state
                        self.editing_card_id = None;
                        self.selected_card_id = None;
                        self.card_icon_menu_position = None;
                        self.card_icon_menu_card_id = None;

                        // Switch to new board
                        self.active_board_index = index;

                        // Load new board's cards (or create empty vec if board doesn't exist in map)
                        let new_board_cards = self.board_cards.get(&index).cloned().unwrap_or_default();
                        self.dot_grid.load_cards(new_board_cards);

                        // Update checkbox positions for all loaded cards
                        let card_ids: Vec<usize> = self.dot_grid.cards().iter().map(|c| c.id).collect();
                        for card_id in card_ids {
                            self.dot_grid.update_card_checkbox_positions(card_id);
                        }

                        // Clear the cards cache to force re-render
                        self.dot_grid.clear_cards_cache();
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
                    } else {
                        // No animation, delete immediately
                        self.boards.remove(index);

                        // Adjust active board index if needed
                        if self.active_board_index >= self.boards.len() {
                            self.active_board_index = self.boards.len().saturating_sub(1);
                        } else if self.active_board_index > index {
                            self.active_board_index = self.active_board_index.saturating_sub(1);
                        }
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
        }
        Task::none()
    }

    /// Save current board's cards to the board_cards HashMap
    fn save_current_board_cards(&mut self) {
        let current_cards = self.dot_grid.cards().iter().cloned().collect();
        self.board_cards.insert(self.active_board_index, current_cards);
    }

    /// Toggle a checkbox at the specified line index in markdown text
    /// This must match exactly how the text_processor and markdown_parser work
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

    /// Save application state (placeholder for future persistence)
    fn save_state(&self) {
        // TODO: Implement state persistence if needed
    }

    fn subscription(&self) -> Subscription<Message> {
        // Always tick for card animations
        let tick = time::every(Duration::from_millis(16)).map(Message::Tick);

        let events = event::listen_with(|event, status, _id| {
            // Only process events that weren't already captured by widgets
            if status == iced::event::Status::Captured {
                return None;
            }

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    // Will be handled in EventOccurred to finish board rename if clicking outside
                    Some(Message::EventOccurred(event))
                }
                Event::Mouse(mouse::Event::WheelScrolled { .. }) => Some(Message::EventOccurred(event)),
                Event::Window(iced::window::Event::Resized(_)) => Some(Message::EventOccurred(event)),
                Event::Window(iced::window::Event::CloseRequested) => {
                    std::process::exit(0);
                }
                Event::Keyboard(keyboard_event) => Some(Message::KeyboardInput(keyboard_event)),
                _ => None,
            }
        });

        Subscription::batch([tick, events])
    }

    fn view(&self) -> Element<Message> {
        let theme_icon = if matches!(self.theme, Theme::Light) {
            self.icon_moon.clone()
        } else {
            self.icon_sun.clone()
        };

        let settings_icon = self.icon_settings.clone();

        let sidebar_bg = self.theme.sidebar_background();
        let sidebar_shadow = self.theme.sidebar_shadow();
        let separator_color = self.theme.separator_color();
        let icon_color = self.theme.icon_color();

        let main_content: Element<Message> = self.dot_grid.view().map(Message::DotGridMessage);

        // Build the base view with main content
        let mut view: Element<Message> = main_content;

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
            .width(160.0)
            .on_close(Message::HideContextMenu)
            .into();

            view = Overlay::new(view, context_menu).into();
        }

        // Add card icon menu (before sidebar)
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

        // Add icon overlays for all cards - renders Bootstrap Icons properly
        for card in self.dot_grid.cards().iter() {
            let icon_size = 20.0;
            let icon_x = card.current_position.x + self.canvas_offset.x + 5.0;
            let icon_y = card.current_position.y + self.canvas_offset.y + 5.0;

            let svg_data = icon_util::icon_to_svg(card.icon.get_icondata());
            let icon_widget = svg(svg::Handle::from_memory(svg_data))
                .width(icon_size)
                .height(icon_size)
                .class(SvgStyle { color: card.color });

            let positioned_icon: Element<Message> = Positioned::new(
                icon_widget,
                Point::new(icon_x, icon_y)
            ).into();

            view = Overlay::new(view, positioned_icon).into();
        }

        // Add card toolbar (before sidebar) - shown when a card is selected
        if let Some(card_id) = self.selected_card_id {
            if let Some(card) = self.dot_grid.cards().iter().find(|c| c.id == card_id) {
                // Position toolbar above the card, centered
                // Toolbar width is 360.0, so offset by half to center it
                let toolbar_x = card.current_position.x + self.canvas_offset.x + (card.width / 2.0) - 250.0;
                let toolbar_y = card.current_position.y + self.canvas_offset.y - 70.0;
                let toolbar_pos = Point::new(toolbar_x, toolbar_y);

                let toolbar_content = self.build_card_toolbar(card_id);
                let toolbar: Element<Message> = ContextMenu::new(
                    toolbar_content,
                    toolbar_pos,
                    self.theme.sidebar_background(),
                    self.theme.button_border(),
                    self.theme.sidebar_shadow(),
                )
                .width(500.0)
                .into();

                view = Overlay::new(view, toolbar).into();
            }
        }

        // Build sidebar content with title
        let btn_style = CardButtonStyle {
            background: self.theme.button_background(),
            background_hovered: self.theme.button_background_hovered(),
            text_color: self.theme.button_text(),
            border_color: self.theme.button_border(),
            shadow_color: self.theme.button_shadow(),
        };

        let theme_btn_style = btn_style.clone();
        let settings_btn_style = btn_style.clone();
        let floating_btn_style = btn_style.clone();

        let theme_button = button(
            container(
                svg(theme_icon)
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
        .class(theme_btn_style)
        .on_press(Message::ToggleTheme);

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

        let sidebar_title = text("Cards")
            .size(18)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .color(self.theme.button_text());

        let top_row = row![
            sidebar_title,
            Space::with_width(Length::Fill),
            theme_button,
            settings_button,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

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

        let top_separator = container(Space::with_height(1))
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
                background_hovered: self.theme.button_background_hovered(),
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
                    background: self.theme.button_background_hovered(),
                    background_hovered: self.theme.button_background_hovered(),
                    text_color: self.theme.button_text(),
                    border_color: Color::TRANSPARENT,
                    shadow_color: Color::TRANSPARENT,
                }
            } else {
                CardButtonStyle {
                    background: Color::TRANSPARENT,
                    background_hovered: self.theme.button_background_hovered(),
                    text_color: self.theme.button_text(),
                    border_color: Color::TRANSPARENT,
                    shadow_color: Color::TRANSPARENT,
                }
            };

            let delete_btn_style = CardButtonStyle {
                background: Color::TRANSPARENT,
                background_hovered: self.theme.button_background_hovered(),
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
                background_hovered: self.theme.button_background_hovered(),
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
                    .width(2)
                    .scroller_width(2)
            ));

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
                background: self.theme.button_background_hovered(),
                background_hovered: self.theme.button_background_hovered(),
                text_color: self.theme.button_text(),
                border_color: self.theme.button_border(),
                shadow_color: self.theme.button_shadow(),
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
        .class(floating_btn_style)
        .on_press(Message::ToggleSidebar);

        let sidebar: Element<Message> = Sidebar::new(
            sidebar_content,
            SIDEBAR_WIDTH,
            sidebar_bg,
            sidebar_shadow,
            self.sidebar_offset,
        )
        .floating_button(floating_button)
        .into();

        // IMPORTANT: Add sidebar overlay LAST (except settings) to ensure it renders on top of all card elements
        // The order is: base canvas -> context menu -> card menu -> toolbar -> SIDEBAR -> settings
        view = Overlay::new(view, sidebar).into();

        // Add settings modal LAST (on top of everything)
        if self.settings_open {
            let settings_content = self.build_settings_content();
            let settings_modal: Element<Message> = SettingsModal::new(
                settings_content,
                sidebar_bg,
                sidebar_shadow,
            )
            .width(700.0)
            .height(500.0)
            .on_close(|| Message::CloseSettings)
            .into();

            view = Overlay::new(view, settings_modal).into();
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

    fn build_context_menu(&self) -> Element<Message> {
        let icon_color = self.theme.icon_color();
        let bg_color = self.theme.sidebar_background();

        let add_card_btn_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: self.theme.button_background_hovered(),
            text_color: self.theme.button_text(),
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };

        let add_card_button = button(
            row![
                svg(self.icon_add.clone())
                    .width(16)
                    .height(16)
                    .class(SvgStyle { color: icon_color }),
                text("Add Card").size(14),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .padding(Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            })
        )
        .width(Length::Fill)
        .class(add_card_btn_style)
        .on_press(Message::AddCard);

        container(
            column![
                add_card_button,
            ]
            .padding(5.0)
        )
        .style(move |_theme: &IcedTheme| {
            container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: None,
            }
        })
        .into()
    }

    fn build_card_icon_menu(&self) -> Element<Message> {
        let bg_color = self.theme.sidebar_background();
        let separator_color = self.theme.icon_color().scale_alpha(0.2);
        let scrollbar_color = self.theme.icon_color().scale_alpha(0.3);

        // Get the current card's color
        let card_color = if let Some(card_id) = self.card_icon_menu_card_id {
            self.dot_grid.cards().get(card_id).map(|c| c.color).unwrap_or(Color::from_rgb8(100, 150, 255))
        } else {
            Color::from_rgb8(100, 150, 255)
        };

        let icon_btn_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: self.theme.button_background_hovered(),
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

        // Custom scrollbar style
        let scrollable_style = move |_theme: &IcedTheme, status: iced::widget::scrollable::Status| {
            use iced::widget::scrollable::{Rail, Scroller};
            iced::widget::scrollable::Style {
                container: iced::widget::container::Style::default(),
                vertical_rail: Rail {
                    background: None,
                    border: Border::default(),
                    scroller: Scroller {
                        color: scrollbar_color,
                        border: Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                    },
                },
                horizontal_rail: Rail {
                    background: None,
                    border: Border::default(),
                    scroller: Scroller {
                        color: scrollbar_color,
                        border: Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                    },
                },
                gap: None,
            }
        };

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
                .width(4)
                .scroller_width(4)
        ))
        .style(scrollable_style);

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

        // Color selection grid (fixed at bottom)
        let colors = [
            Color::from_rgb8(100, 150, 255), // Blue
            Color::from_rgb8(255, 100, 100), // Red
            Color::from_rgb8(100, 255, 100), // Green
            Color::from_rgb8(255, 200, 100), // Orange
            Color::from_rgb8(200, 100, 255), // Purple
            Color::from_rgb8(255, 150, 200), // Pink
            Color::from_rgb8(100, 255, 255), // Cyan
            Color::from_rgb8(255, 255, 100), // Yellow
            Color::from_rgb8(150, 150, 150), // Gray
            Color::from_rgb8(255, 150, 100), // Coral
        ];

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
                                    radius: 15.0.into(), // Make it circular
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
            .style(move |_theme: &IcedTheme| {
                container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            })
            .into()
    }

    fn build_card_toolbar(&self, card_id: usize) -> Element<Message> {
        let btn_style = CardButtonStyle {
            background: Color::TRANSPARENT,
            background_hovered: self.theme.button_background_hovered(),
            text_color: self.theme.button_text(),
            border_color: Color::TRANSPARENT,
            shadow_color: Color::TRANSPARENT,
        };

        // Markdown formatting buttons - all square (32×32)
        let bold_btn = button(
            container(
                text("B").size(14).font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
            )
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::FormatBold);

        let italic_btn = button(
            container(
                text("I").size(14).font(iced::Font {
                    style: iced::font::Style::Italic,
                    ..Default::default()
                })
            )
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::FormatItalic);

        let strike_btn = button(
            container(text("S̶").size(14))
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::FormatStrikethrough);

        let code_btn = button(
            container(text("<>").size(12))
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::FormatCode);

        let code_block_btn = button(
            container(text("{ }").size(12))
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::FormatCodeBlock);

        let heading_btn = button(
            container(text("H").size(14).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }))
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::FormatHeading);

        let bullet_btn = button(
            container(text("•").size(16))
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::FormatBullet);

        // Vertical separator
        let separator_color = self.theme.separator_color();
        let separator = container(Space::new(Length::Fixed(1.0), Length::Fixed(24.0)))
            .width(1)
            .height(24)
            .style(move |_theme: &IcedTheme| {
                container::Style {
                    background: Some(iced::Background::Color(separator_color)),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }
            });

        // Card management buttons - square (32×32)
        let icon_color = self.theme.icon_color();

        let duplicate_btn = button(
            container(
                svg(self.icon_duplicate.clone())
                    .width(20)
                    .height(20)
                    .class(SvgStyle { color: icon_color })
            )
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::DuplicateCard(card_id));

        let delete_btn = button(
            container(
                svg(self.icon_delete.clone())
                    .width(20)
                    .height(20)
                    .class(SvgStyle { color: icon_color })
            )
            .width(32)
            .height(32)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
        )
        .class(btn_style.clone())
        .on_press(Message::DeleteCard(card_id));

        let bg_color = self.theme.sidebar_background();

        container(
            row![
                bold_btn,
                italic_btn,
                strike_btn,
                code_btn,
                code_block_btn,
                heading_btn,
                bullet_btn,
                separator,
                duplicate_btn,
                delete_btn,
            ]
            .spacing(2)
            .padding(6)
            .align_y(Alignment::Center)
        )
        .style(move |_theme: &IcedTheme| {
            container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: None,
            }
        })
        .into()
    }

    fn build_settings_content(&self) -> Element<Message> {
        let separator_color = self.theme.separator_color();
        let text_color = self.theme.button_text();
        let icon_color = self.theme.icon_color();

        let settings_title = text("Settings")
            .size(18)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            });

        let close_btn_style = CardButtonStyle {
            background: self.theme.button_background(),
            background_hovered: self.theme.button_background_hovered(),
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
                    background: self.theme.button_background_hovered(),
                    background_hovered: self.theme.button_background_hovered(),
                    text_color: self.theme.button_text(),
                    border_color: self.theme.button_border(),
                    shadow_color: Color::TRANSPARENT,
                }
            } else {
                CardButtonStyle {
                    background: Color::TRANSPARENT,
                    background_hovered: self.theme.button_background_hovered(),
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
                ]
                .spacing(10)
                .into()
            }
            SettingsCategory::Appearance => {
                let theme_label = text("Theme").size(14);

                let light_btn_style = CardButtonStyle {
                    background: if matches!(self.theme, Theme::Light) {
                        self.theme.button_background_hovered()
                    } else {
                        Color::TRANSPARENT
                    },
                    background_hovered: self.theme.button_background_hovered(),
                    text_color: self.theme.button_text(),
                    border_color: self.theme.button_border(),
                    shadow_color: if matches!(self.theme, Theme::Light) {
                        self.theme.button_shadow()
                    } else {
                        Color::TRANSPARENT
                    },
                };

                let dark_btn_style = CardButtonStyle {
                    background: if matches!(self.theme, Theme::Dark) {
                        self.theme.button_background_hovered()
                    } else {
                        Color::TRANSPARENT
                    },
                    background_hovered: self.theme.button_background_hovered(),
                    text_color: self.theme.button_text(),
                    border_color: self.theme.button_border(),
                    shadow_color: if matches!(self.theme, Theme::Dark) {
                        self.theme.button_shadow()
                    } else {
                        Color::TRANSPARENT
                    },
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

                // Font family dropdown
                let font_family_label = text("Font Family").size(14);

                let theme = self.theme;
                let pick_list_style = move |_theme: &IcedTheme, _status: pick_list::Status| {
                    let background = theme.card_background();
                    let text_color = theme.card_text();
                    let border_color = theme.button_border();

                    pick_list::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        placeholder_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        handle_color: text_color,
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
                    let border_color = theme.button_border();
                    let selected_bg = theme.button_background_hovered();

                    iced::overlay::menu::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        selected_background: iced::Background::Color(selected_bg),
                        selected_text_color: text_color,
                        border: Border {
                            color: border_color,
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

                let pick_list_style2 = move |_theme: &IcedTheme, _status: pick_list::Status| {
                    let background = theme.card_background();
                    let text_color = theme.card_text();
                    let border_color = theme.button_border();

                    pick_list::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        placeholder_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        handle_color: text_color,
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
                    let border_color = theme.button_border();
                    let selected_bg = theme.button_background_hovered();

                    iced::overlay::menu::Style {
                        background: iced::Background::Color(background),
                        text_color,
                        selected_background: iced::Background::Color(selected_bg),
                        selected_text_color: text_color,
                        border: Border {
                            color: border_color,
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
                column![
                    text("Keyboard Shortcuts").size(16).font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                    Space::with_height(20),
                    text("Keyboard shortcuts will be configured here.").size(14),
                ]
                .spacing(10)
                .into()
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
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }


    fn build_toggle_button(&self, is_on: bool, message: Message) -> Element<Message> {
        let btn_style = CardButtonStyle {
            background: if is_on {
                self.theme.button_background_hovered()
            } else {
                Color::TRANSPARENT
            },
            background_hovered: self.theme.button_background_hovered(),
            text_color: self.theme.button_text(),
            border_color: self.theme.button_border(),
            shadow_color: if is_on {
                self.theme.button_shadow()
            } else {
                Color::TRANSPARENT
            },
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
}
