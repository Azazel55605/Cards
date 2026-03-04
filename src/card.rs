use iced::{Color, Point, Rectangle};
use crate::custom_text_editor::CustomTextEditor;
use crate::text_renderer::CheckboxPosition;

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
    pub is_dragging: bool,
    pub content: CustomTextEditor,
    pub is_editing: bool,
    pub checkbox_positions: Vec<CheckboxPosition>, // Track checkbox positions for interaction
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
            is_dragging: self.is_dragging,
            content: self.content.clone(),
            is_editing: self.is_editing,
            checkbox_positions: self.checkbox_positions.clone(),
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
            is_dragging: false,
            content: CustomTextEditor::new(),
            is_editing: false,
            checkbox_positions: Vec::new(),
        }
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

    /// Snap position to grid
    pub fn snap_to_grid(position: Point, grid_spacing: f32) -> Point {
        Point::new(
            (position.x / grid_spacing).round() * grid_spacing,
            (position.y / grid_spacing).round() * grid_spacing,
        )
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        if delta_time <= 0.0 || delta_time > 0.1 {
            // Skip if delta_time is invalid (too large or negative)
            return;
        }

        // Animate position
        let distance = ((self.target_position.x - self.current_position.x).powi(2)
                       + (self.target_position.y - self.current_position.y).powi(2)).sqrt();

        if distance > 0.5 {
            // Smooth interpolation with easing
            let speed = 10.0; // Higher = faster snap
            let t = 1.0 - (-speed * delta_time).exp();

            self.current_position.x += (self.target_position.x - self.current_position.x) * t;
            self.current_position.y += (self.target_position.y - self.current_position.y) * t;
        } else {
            self.current_position = self.target_position;
        }

        // Animate size
        let width_diff = (self.target_width - self.width).abs();
        let height_diff = (self.target_height - self.height).abs();

        if width_diff > 0.5 || height_diff > 0.5 {
            // Smooth interpolation with easing for size
            let speed = 10.0; // Same speed as position for consistency
            let t = 1.0 - (-speed * delta_time).exp();

            self.width += (self.target_width - self.width) * t;
            self.height += (self.target_height - self.height) * t;
        } else {
            // Snap to target when close enough
            self.width = self.target_width;
            self.height = self.target_height;
        }
    }
}
