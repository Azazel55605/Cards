use iced::{Color, Point, Rectangle};
use std::sync::Arc;
use crate::custom_text_editor::CustomTextEditor;
use crate::text_renderer::{CheckboxPosition, LinkPosition};

/// A decoded/ready-to-render image handle (raster or SVG).
/// The handle is cached inside the card to avoid re-allocating every frame.
#[derive(Clone)]
pub enum CardImageHandle {
    Raster(iced::advanced::image::Handle),
    Svg(iced::advanced::svg::Handle),
}

impl std::fmt::Debug for CardImageHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raster(_) => write!(f, "Raster(...)"),
            Self::Svg(_)    => write!(f, "Svg(...)"),
        }
    }
}

/// Returns true if `bytes` look like an SVG document.
pub fn bytes_are_svg(bytes: &[u8]) -> bool {
    // Skip BOM / leading whitespace, then look for XML/SVG markers
    let trimmed = bytes.iter()
        .position(|&b| !b.is_ascii_whitespace())
        .map(|i| &bytes[i..])
        .unwrap_or(bytes);
    trimmed.starts_with(b"<svg") || trimmed.starts_with(b"<?xml") || trimmed.starts_with(b"<!DOCTYPE svg")
}

/// Build a cached image handle from raw bytes.
pub fn build_image_handle(bytes: &[u8], is_svg: bool) -> CardImageHandle {
    if is_svg {
        CardImageHandle::Svg(iced::advanced::svg::Handle::from_memory(bytes.to_vec()))
    } else {
        CardImageHandle::Raster(iced::advanced::image::Handle::from_bytes(bytes.to_vec()))
    }
}

/// Style of a connection line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle { Solid, Dashed, Dotted }
impl LineStyle {
    pub fn as_str(self) -> &'static str {
        match self { LineStyle::Solid => "Solid", LineStyle::Dashed => "Dashed", LineStyle::Dotted => "Dotted" }
    }
    pub fn from_str(s: &str) -> Self {
        match s { "Dashed" => LineStyle::Dashed, "Dotted" => LineStyle::Dotted, _ => LineStyle::Solid }
    }
}

/// Which side of a card a connection attaches to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSide { Top, Bottom, Left, Right }

impl CardSide {
    pub fn as_str(self) -> &'static str {
        match self {
            CardSide::Top    => "Top",
            CardSide::Bottom => "Bottom",
            CardSide::Left   => "Left",
            CardSide::Right  => "Right",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Bottom" => CardSide::Bottom,
            "Left"   => CardSide::Left,
            "Right"  => CardSide::Right,
            _        => CardSide::Top,
        }
    }
    pub fn all() -> &'static [CardSide] {
        &[CardSide::Top, CardSide::Bottom, CardSide::Left, CardSide::Right]
    }
    /// Unit vector pointing outward from this side.
    pub fn outward(self) -> (f32, f32) {
        match self {
            CardSide::Top    => (0.0, -1.0),
            CardSide::Bottom => (0.0,  1.0),
            CardSide::Left   => (-1.0, 0.0),
            CardSide::Right  => (1.0,  0.0),
        }
    }
}

/// A directed connection between two card sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connection {
    pub from_card:  usize,
    pub from_side:  CardSide,
    pub to_card:    usize,
    pub to_side:    CardSide,
    pub line_style: LineStyle,
    pub arrow_from: bool,   // arrowhead at from_card's attachment point
    pub arrow_to:   bool,   // arrowhead at to_card's attachment point
}

impl Connection {
    pub fn new(from_card: usize, from_side: CardSide, to_card: usize, to_side: CardSide) -> Self {
        Self { from_card, from_side, to_card, to_side, line_style: LineStyle::Solid, arrow_from: false, arrow_to: false }
    }
}

/// The display / editing mode of a card.
/// Keep this as a flat enum so it's easy to add new types later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardType {
    /// Plain text — no markdown rendering, just raw text.
    #[default]
    Text,
    /// Full markdown — rendered with the markdown parser.
    Markdown,
    /// Image card — holds raster (PNG/JPEG/GIF/BMP/WebP) or SVG image data.
    Image,
}

impl CardType {
    pub fn label(&self) -> &'static str {
        match self {
            CardType::Text     => "Text",
            CardType::Markdown => "Markdown",
            CardType::Image    => "Image",
        }
    }

    /// Serialize to a stable string (for workspace files).
    pub fn as_str(&self) -> &'static str {
        match self {
            CardType::Text     => "Text",
            CardType::Markdown => "Markdown",
            CardType::Image    => "Image",
        }
    }

    /// Parse from the serialized string; unknown values fall back to Text.
    pub fn from_str(s: &str) -> Self {
        match s {
            "Markdown" => CardType::Markdown,
            "Image"    => CardType::Image,
            _          => CardType::Text,
        }
    }
}

#[derive(Debug)]
pub struct Card {
    pub id: usize,
    pub current_position: Point,
    pub target_position: Point,
    pub width: f32,
    pub height: f32,
    pub target_width: f32,
    pub target_height: f32,
    pub icon: CardIcon,
    pub color: Color,
    pub card_type: CardType,
    pub is_dragging: bool,
    pub content: CustomTextEditor,
    pub is_editing: bool,
    pub checkbox_positions: Vec<CheckboxPosition>,
    pub link_positions: Vec<LinkPosition>,
    // Image card fields
    pub image_data:   Option<Arc<Vec<u8>>>,
    pub image_is_svg: bool,
    pub image_handle: Option<CardImageHandle>,
}

impl Clone for Card {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            current_position: self.current_position,
            target_position: self.target_position,
            width: self.width,
            height: self.height,
            target_width: self.target_width,
            target_height: self.target_height,
            icon: self.icon,
            color: self.color,
            card_type: self.card_type,
            is_dragging: self.is_dragging,
            content: self.content.clone(),
            is_editing: self.is_editing,
            checkbox_positions: self.checkbox_positions.clone(),
            link_positions: self.link_positions.clone(),
            image_data:   self.image_data.clone(),
            image_is_svg: self.image_is_svg,
            image_handle: self.image_handle.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardIcon {
    // Original icons
    Default,
    Star,
    Heart,
    Circle,
    Square,
    Triangle,
    Check,
    Cross,
    Question,
    Exclamation,
    Plus,
    Minus,
    // New Bootstrap Icons - Shapes & Symbols
    StarFill,
    HeartFill,
    CircleFill,
    SquareFill,
    TriangleFill,
    Diamond,
    DiamondFill,
    Hexagon,
    HexagonFill,
    Octagon,
    OctagonFill,
    Pentagon,
    PentagonFill,
    // Common Actions
    CheckCircle,
    CheckCircleFill,
    XCircle,
    XCircleFill,
    InfoCircle,
    InfoCircleFill,
    ExclamationTriangle,
    ExclamationTriangleFill,
    ExclamationCircle,
    ExclamationCircleFill,
    QuestionCircle,
    QuestionCircleFill,
    PlusCircle,
    PlusCircleFill,
    DashCircle,
    DashCircleFill,
    // Arrows
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUpCircle,
    ArrowDownCircle,
    ArrowLeftCircle,
    ArrowRightCircle,
    // Common Objects
    Book,
    BookFill,
    Bookmark,
    BookmarkFill,
    Calendar,
    CalendarFill,
    Clock,
    ClockFill,
    Flag,
    FlagFill,
    Folder,
    FolderFill,
    Gear,
    GearFill,
    House,
    HouseFill,
    Lightning,
    LightningFill,
    Lightbulb,
    LightbulbFill,
    Link,
    Lock,
    LockFill,
    Unlock,
    UnlockFill,
    Pencil,
    PencilFill,
    Pin,
    PinFill,
    Tag,
    TagFill,
    Trophy,
    TrophyFill,
    // Nature & Weather
    Cloud,
    CloudFill,
    Sun,
    Moon,
    Umbrella,
    UmbrellaFill,
    Fire,
    Flower1,
    Tree,
    // Communication
    Bell,
    BellFill,
    Chat,
    ChatFill,
    Envelope,
    EnvelopeFill,
    Telephone,
    TelephoneFill,
    // Technology
    Camera,
    CameraFill,
    Laptop,
    Phone,
    Wifi,
    // People & Body
    Person,
    PersonFill,
    People,
    PeopleFill,
    Eye,
    EyeFill,
    Hand,
    // Food & Drink
    Cup,
    CupFill,
    // Business & Finance
    Cart,
    CartFill,
    CreditCard,
    CreditCardFill,
    Graph,
    GraphUp,
    GraphDown,
    Briefcase,
    BriefcaseFill,
    // Media
    Play,
    PlayFill,
    Pause,
    PauseFill,
    Stop,
    StopFill,
    Music,
    MusicFill,
    Image,
    ImageFill,
    Film,
    // Misc
    Gift,
    GiftFill,
    Balloon,
    BalloonFill,
    Gem,
    Puzzle,
    PuzzleFill,
    Clipboard,
    ClipboardFill,
}

impl CardIcon {
    // Get the icondata Icon for this CardIcon
    pub fn get_icondata(&self) -> icondata_core::Icon {
        use icondata_bs::*;
        match self {
            Self::Default => BsCircle,
            Self::Star => BsStar,
            Self::Heart => BsHeart,
            Self::Circle => BsCircle,
            Self::Square => BsSquare,
            Self::Triangle => BsTriangle,
            Self::Check => BsCheck,
            Self::Cross => BsX,
            Self::Question => BsQuestion,
            Self::Exclamation => BsExclamation,
            Self::Plus => BsPlus,
            Self::Minus => BsDash,
            Self::StarFill => BsStarFill,
            Self::HeartFill => BsHeartFill,
            Self::CircleFill => BsCircleFill,
            Self::SquareFill => BsSquareFill,
            Self::TriangleFill => BsTriangleFill,
            Self::Diamond => BsDiamond,
            Self::DiamondFill => BsDiamondFill,
            Self::Hexagon => BsHexagon,
            Self::HexagonFill => BsHexagonFill,
            Self::Octagon => BsOctagon,
            Self::OctagonFill => BsOctagonFill,
            Self::Pentagon => BsPentagon,
            Self::PentagonFill => BsPentagonFill,
            Self::CheckCircle => BsCheckCircle,
            Self::CheckCircleFill => BsCheckCircleFill,
            Self::XCircle => BsXCircle,
            Self::XCircleFill => BsXCircleFill,
            Self::InfoCircle => BsInfoCircle,
            Self::InfoCircleFill => BsInfoCircleFill,
            Self::ExclamationTriangle => BsExclamationTriangle,
            Self::ExclamationTriangleFill => BsExclamationTriangleFill,
            Self::ExclamationCircle => BsExclamationCircle,
            Self::ExclamationCircleFill => BsExclamationCircleFill,
            Self::QuestionCircle => BsQuestionCircle,
            Self::QuestionCircleFill => BsQuestionCircleFill,
            Self::PlusCircle => BsPlusCircle,
            Self::PlusCircleFill => BsPlusCircleFill,
            Self::DashCircle => BsDashCircle,
            Self::DashCircleFill => BsDashCircleFill,
            Self::ArrowUp => BsArrowUp,
            Self::ArrowDown => BsArrowDown,
            Self::ArrowLeft => BsArrowLeft,
            Self::ArrowRight => BsArrowRight,
            Self::ArrowUpCircle => BsArrowUpCircle,
            Self::ArrowDownCircle => BsArrowDownCircle,
            Self::ArrowLeftCircle => BsArrowLeftCircle,
            Self::ArrowRightCircle => BsArrowRightCircle,
            Self::Book => BsBook,
            Self::BookFill => BsBookFill,
            Self::Bookmark => BsBookmark,
            Self::BookmarkFill => BsBookmarkFill,
            Self::Calendar => BsCalendar,
            Self::CalendarFill => BsCalendarFill,
            Self::Clock => BsClock,
            Self::ClockFill => BsClockFill,
            Self::Flag => BsFlag,
            Self::FlagFill => BsFlagFill,
            Self::Folder => BsFolder,
            Self::FolderFill => BsFolderFill,
            Self::Gear => BsGear,
            Self::GearFill => BsGearFill,
            Self::House => BsHouse,
            Self::HouseFill => BsHouseFill,
            Self::Lightning => BsLightning,
            Self::LightningFill => BsLightningFill,
            Self::Lightbulb => BsLightbulb,
            Self::LightbulbFill => BsLightbulbFill,
            Self::Link => BsLink,
            Self::Lock => BsLock,
            Self::LockFill => BsLockFill,
            Self::Unlock => BsUnlock,
            Self::UnlockFill => BsUnlockFill,
            Self::Pencil => BsPencil,
            Self::PencilFill => BsPencilFill,
            Self::Pin => BsPin,
            Self::PinFill => BsPinFill,
            Self::Tag => BsTag,
            Self::TagFill => BsTagFill,
            Self::Trophy => BsTrophy,
            Self::TrophyFill => BsTrophyFill,
            Self::Cloud => BsCloud,
            Self::CloudFill => BsCloudFill,
            Self::Sun => BsSun,
            Self::Moon => BsMoon,
            Self::Umbrella => BsUmbrella,
            Self::UmbrellaFill => BsUmbrellaFill,
            Self::Fire => BsFire,
            Self::Flower1 => BsFlower1,
            Self::Tree => BsTree,
            Self::Bell => BsBell,
            Self::BellFill => BsBellFill,
            Self::Chat => BsChat,
            Self::ChatFill => BsChatFill,
            Self::Envelope => BsEnvelope,
            Self::EnvelopeFill => BsEnvelopeFill,
            Self::Telephone => BsTelephone,
            Self::TelephoneFill => BsTelephoneFill,
            Self::Camera => BsCamera,
            Self::CameraFill => BsCameraFill,
            Self::Laptop => BsLaptop,
            Self::Phone => BsPhone,
            Self::Wifi => BsWifi,
            Self::Person => BsPerson,
            Self::PersonFill => BsPersonFill,
            Self::People => BsPeople,
            Self::PeopleFill => BsPeopleFill,
            Self::Eye => BsEye,
            Self::EyeFill => BsEyeFill,
            Self::Hand => BsHandThumbsUp,
            Self::Cup => BsCup,
            Self::CupFill => BsCupFill,
            Self::Cart => BsCart,
            Self::CartFill => BsCartFill,
            Self::CreditCard => BsCreditCard,
            Self::CreditCardFill => BsCreditCardFill,
            Self::Graph => BsGraphUp,
            Self::GraphUp => BsGraphUp,
            Self::GraphDown => BsGraphDown,
            Self::Briefcase => BsBriefcase,
            Self::BriefcaseFill => BsBriefcaseFill,
            Self::Play => BsPlay,
            Self::PlayFill => BsPlayFill,
            Self::Pause => BsPause,
            Self::PauseFill => BsPauseFill,
            Self::Stop => BsStop,
            Self::StopFill => BsStopFill,
            Self::Music => BsMusicNote,
            Self::MusicFill => BsMusicNoteBeamed,
            Self::Image => BsImage,
            Self::ImageFill => BsImageFill,
            Self::Film => BsFilm,
            Self::Gift => BsGift,
            Self::GiftFill => BsGiftFill,
            Self::Balloon => BsBalloon,
            Self::BalloonFill => BsBalloonFill,
            Self::Gem => BsGem,
            Self::Puzzle => BsPuzzle,
            Self::PuzzleFill => BsPuzzleFill,
            Self::Clipboard => BsClipboard,
            Self::ClipboardFill => BsClipboardFill,
        }
    }

    pub fn all() -> &'static [CardIcon] {
        &[
            // Basic Shapes
            Self::Circle,
            Self::CircleFill,
            Self::Square,
            Self::SquareFill,
            Self::Triangle,
            Self::TriangleFill,
            Self::Diamond,
            Self::DiamondFill,
            Self::Hexagon,
            Self::HexagonFill,
            Self::Octagon,
            Self::OctagonFill,
            Self::Pentagon,
            Self::PentagonFill,
            // Stars & Hearts
            Self::Star,
            Self::StarFill,
            Self::Heart,
            Self::HeartFill,
            // Status Icons
            Self::Check,
            Self::CheckCircle,
            Self::CheckCircleFill,
            Self::Cross,
            Self::XCircle,
            Self::XCircleFill,
            Self::Question,
            Self::QuestionCircle,
            Self::QuestionCircleFill,
            Self::Exclamation,
            Self::ExclamationCircle,
            Self::ExclamationCircleFill,
            Self::ExclamationTriangle,
            Self::ExclamationTriangleFill,
            Self::InfoCircle,
            Self::InfoCircleFill,
            // Plus/Minus
            Self::Plus,
            Self::PlusCircle,
            Self::PlusCircleFill,
            Self::Minus,
            Self::DashCircle,
            Self::DashCircleFill,
            // Arrows
            Self::ArrowUp,
            Self::ArrowDown,
            Self::ArrowLeft,
            Self::ArrowRight,
            Self::ArrowUpCircle,
            Self::ArrowDownCircle,
            Self::ArrowLeftCircle,
            Self::ArrowRightCircle,
            // Common Objects
            Self::Book,
            Self::BookFill,
            Self::Bookmark,
            Self::BookmarkFill,
            Self::Calendar,
            Self::CalendarFill,
            Self::Clock,
            Self::ClockFill,
            Self::Flag,
            Self::FlagFill,
            Self::Folder,
            Self::FolderFill,
            Self::Gear,
            Self::GearFill,
            Self::House,
            Self::HouseFill,
            Self::Lightning,
            Self::LightningFill,
            Self::Lightbulb,
            Self::LightbulbFill,
            Self::Link,
            Self::Lock,
            Self::LockFill,
            Self::Unlock,
            Self::UnlockFill,
            Self::Pencil,
            Self::PencilFill,
            Self::Pin,
            Self::PinFill,
            Self::Tag,
            Self::TagFill,
            Self::Trophy,
            Self::TrophyFill,
            // Nature & Weather
            Self::Cloud,
            Self::CloudFill,
            Self::Sun,
            Self::Moon,
            Self::Umbrella,
            Self::UmbrellaFill,
            Self::Fire,
            Self::Flower1,
            Self::Tree,
            // Communication
            Self::Bell,
            Self::BellFill,
            Self::Chat,
            Self::ChatFill,
            Self::Envelope,
            Self::EnvelopeFill,
            Self::Telephone,
            Self::TelephoneFill,
            // Technology
            Self::Camera,
            Self::CameraFill,
            Self::Laptop,
            Self::Phone,
            Self::Wifi,
            // People
            Self::Person,
            Self::PersonFill,
            Self::People,
            Self::PeopleFill,
            Self::Eye,
            Self::EyeFill,
            Self::Hand,
            // Food
            Self::Cup,
            Self::CupFill,
            // Business
            Self::Cart,
            Self::CartFill,
            Self::CreditCard,
            Self::CreditCardFill,
            Self::Graph,
            Self::GraphUp,
            Self::GraphDown,
            Self::Briefcase,
            Self::BriefcaseFill,
            // Media
            Self::Play,
            Self::PlayFill,
            Self::Pause,
            Self::PauseFill,
            Self::Stop,
            Self::StopFill,
            Self::Music,
            Self::MusicFill,
            Self::Image,
            Self::ImageFill,
            Self::Film,
            // Misc
            Self::Gift,
            Self::GiftFill,
            Self::Balloon,
            Self::BalloonFill,
            Self::Gem,
            Self::Puzzle,
            Self::PuzzleFill,
            Self::Clipboard,
            Self::ClipboardFill,
        ]
    }
}


impl Card {
    pub const MIN_WIDTH: f32 = 200.0;
    pub const MIN_HEIGHT: f32 = 150.0;

    pub fn new(id: usize, position: Point) -> Self {
        Self {
            id,
            current_position: position,
            target_position: position,
            width: Self::MIN_WIDTH,
            height: Self::MIN_HEIGHT,
            target_width: Self::MIN_WIDTH,
            target_height: Self::MIN_HEIGHT,
            icon: CardIcon::Default,
            color: Color::from_rgb8(124, 92, 252), // Default purple (matches app accent)
            card_type: CardType::Text,
            is_dragging: false,
            content: CustomTextEditor::new(),
            is_editing: false,
            checkbox_positions: Vec::new(),
            link_positions: Vec::new(),
            image_data:   None,
            image_is_svg: false,
            image_handle: None,
        }
    }

    /// Load image bytes into this card, building a cached render handle.
    pub fn set_image(&mut self, bytes: Vec<u8>) {
        let is_svg = bytes_are_svg(&bytes);
        let arc = Arc::new(bytes);
        self.image_handle = Some(build_image_handle(&arc, is_svg));
        self.image_data   = Some(arc);
        self.image_is_svg = is_svg;
    }

    pub fn bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x,
            y: self.current_position.y,
            width: self.width,
            height: self.height,
        }
    }

    pub fn top_bar_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x,
            y: self.current_position.y,
            width: self.width,
            height: 30.0, // Top bar height
        }
    }

    pub fn icon_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x + 5.0,
            y: self.current_position.y + 5.0,
            width: 20.0,
            height: 20.0,
        }
    }

    pub fn content_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.current_position.x,
            y: self.current_position.y + 30.0, // Below the top bar
            width: self.width,
            height: self.height - 30.0,
        }
    }

    pub fn resize_handle_bounds(&self) -> Rectangle {
        let handle_size = 16.0;
        Rectangle {
            x: self.current_position.x + self.width - handle_size,
            y: self.current_position.y + self.height - handle_size,
            width: handle_size,
            height: handle_size,
        }
    }

    /// Returns the world-space center point on the given side of this card.
    pub fn side_world_pos(&self, side: CardSide) -> Point {
        let x = self.current_position.x;
        let y = self.current_position.y;
        match side {
            CardSide::Top    => Point::new(x + self.width / 2.0, y),
            CardSide::Bottom => Point::new(x + self.width / 2.0, y + self.height),
            CardSide::Left   => Point::new(x, y + self.height / 2.0),
            CardSide::Right  => Point::new(x + self.width, y + self.height / 2.0),
        }
    }

    /// Snap position to grid
    pub fn snap_to_grid(position: Point, grid_spacing: f32) -> Point {
        Point::new(
            (position.x / grid_spacing).round() * grid_spacing,
            (position.y / grid_spacing).round() * grid_spacing,
        )
    }

    /// Returns `true` if the card's visual state changed (position or size moved).
    pub fn update_animation(&mut self, delta_time: f32) -> bool {
        if delta_time <= 0.0 || delta_time > 0.1 {
            return false;
        }

        let mut changed = false;

        // Animate position
        let distance = ((self.target_position.x - self.current_position.x).powi(2)
                       + (self.target_position.y - self.current_position.y).powi(2)).sqrt();

        if distance > 0.5 {
            let speed = 10.0;
            let t = 1.0 - (-speed * delta_time).exp();
            self.current_position.x += (self.target_position.x - self.current_position.x) * t;
            self.current_position.y += (self.target_position.y - self.current_position.y) * t;
            changed = true;
        } else if self.current_position != self.target_position {
            self.current_position = self.target_position;
            changed = true;
        }

        // Animate size
        let width_diff = (self.target_width - self.width).abs();
        let height_diff = (self.target_height - self.height).abs();

        if width_diff > 0.5 || height_diff > 0.5 {
            let speed = 10.0;
            let t = 1.0 - (-speed * delta_time).exp();
            self.width += (self.target_width - self.width) * t;
            self.height += (self.target_height - self.height) * t;
            changed = true;
        } else if self.width != self.target_width || self.height != self.target_height {
            self.width = self.target_width;
            self.height = self.target_height;
            changed = true;
        }

        changed
    }
}
