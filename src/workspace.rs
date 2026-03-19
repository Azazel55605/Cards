/// workspace.rs — Workspace persistence layer
///
/// A workspace file (.cards) is a MessagePack-encoded binary containing:
///   WorkspaceFile
///     └─ Vec<BoardData>
///          └─ Vec<CardData>
///
/// MessagePack was chosen because:
///   - Already a dependency (rmp-serde)
///   - Compact binary (2-4× smaller than JSON/TOML)
///   - Fast encode/decode
///   - Schema-versioned for forward-compatibility

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

/// Magic bytes at the start of every .cards file
const FILE_MAGIC: &[u8; 6] = b"CARDS\x01";
/// Current format version — bump when the schema changes in a breaking way
const FORMAT_VERSION: u8 = 1;

/// Top-level file container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFile {
    /// Format version so future readers can handle old files gracefully
    pub version: u8,
    /// Human-readable workspace name (defaults to the file stem)
    pub name: String,
    /// Ordered list of boards
    pub boards: Vec<BoardData>,
    /// Which board was active when the workspace was last saved
    #[serde(default)]
    pub active_board_index: usize,
    /// Canvas scroll offset X when last saved
    #[serde(default)]
    pub canvas_offset_x: f32,
    /// Canvas scroll offset Y when last saved
    #[serde(default)]
    pub canvas_offset_y: f32,
}

fn default_line_style() -> String { "Solid".to_string() }

/// Serialisable snapshot of a card connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionData {
    pub from_card:  usize,
    pub from_side:  String,
    pub to_card:    usize,
    pub to_side:    String,
    #[serde(default = "default_line_style")]
    pub line_style: String,
    #[serde(default)]
    pub arrow_from: bool,
    #[serde(default)]
    pub arrow_to:   bool,
}

impl ConnectionData {
    pub fn from_connection(c: &crate::card::Connection) -> Self {
        Self {
            from_card:  c.from_card,
            from_side:  c.from_side.as_str().to_string(),
            to_card:    c.to_card,
            to_side:    c.to_side.as_str().to_string(),
            line_style: c.line_style.as_str().to_string(),
            arrow_from: c.arrow_from,
            arrow_to:   c.arrow_to,
        }
    }
    pub fn to_connection(&self) -> crate::card::Connection {
        use crate::card::{CardSide, LineStyle};
        crate::card::Connection {
            from_card:  self.from_card,
            from_side:  CardSide::from_str(&self.from_side),
            to_card:    self.to_card,
            to_side:    CardSide::from_str(&self.to_side),
            line_style: LineStyle::from_str(&self.line_style),
            arrow_from: self.arrow_from,
            arrow_to:   self.arrow_to,
        }
    }
}

/// One board = a named collection of cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardData {
    pub name: String,
    pub cards: Vec<CardData>,
    #[serde(default)]
    pub connections: Vec<ConnectionData>,
}

/// Serialisable snapshot of a single card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardData {
    /// Unique id within the workspace (monotonically increasing counter)
    pub id: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub content: String,
    /// `CardIcon` variant name — stored as string so adding icons never breaks old files
    pub icon: String,
    /// RGBA stored as four u8 bytes
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub color_a: u8,
    /// Card type: "Text", "Markdown", or "Image". Defaults to "Text" when missing (old files).
    #[serde(default = "default_card_type")]
    pub card_type: String,
    /// Raw image bytes — only present for Image cards. Stored as MessagePack bin (no base64).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_data: Option<Vec<u8>>,
    /// True when image_data is an SVG document.
    #[serde(default)]
    pub image_is_svg: bool,
    /// Whether the card is pinned (cannot be moved). Defaults to false for old files.
    #[serde(default)]
    pub pinned: bool,
    /// Whether the card is collapsed (content hidden). Defaults to false for old files.
    #[serde(default)]
    pub collapsed: bool,
    /// The expanded height to restore when uncollapsing. Defaults to 150.0 for old files.
    #[serde(default = "default_natural_height")]
    pub natural_height: f32,
}

fn default_card_type() -> String {
    "Text".to_string()
}

fn default_natural_height() -> f32 {
    150.0
}

// ── Conversion helpers ──────────────────────────────────────────────────────

impl CardData {
    pub fn from_card(card: &crate::card::Card) -> Self {
        let icon_name = format!("{:?}", card.icon);
        let c = card.color;
        Self {
            id: card.id,
            x: card.current_position.x,
            y: card.current_position.y,
            width: card.width,
            height: card.height,
            content: card.content.text().to_string(),
            icon: icon_name,
            color_r: (c.r * 255.0) as u8,
            color_g: (c.g * 255.0) as u8,
            color_b: (c.b * 255.0) as u8,
            color_a: (c.a * 255.0) as u8,
            card_type: card.card_type.as_str().to_string(),
            image_data:   card.image_data.as_ref().map(|arc| arc.as_ref().clone()),
            image_is_svg: card.image_is_svg,
            pinned: card.pinned,
            collapsed: card.collapsed,
            natural_height: card.natural_height,
        }
    }

    pub fn to_color(&self) -> iced::Color {
        iced::Color::from_rgba8(self.color_r, self.color_g, self.color_b, self.color_a as f32 / 255.0)
    }

    pub fn to_icon(&self) -> crate::card::CardIcon {
        // Parse the Debug-repr string back into a CardIcon.
        // If unknown (icon added in a newer version), fall back to Default.
        parse_card_icon(&self.icon)
    }

    pub fn to_card_type(&self) -> crate::card::CardType {
        crate::card::CardType::from_str(&self.card_type)
    }
}

/// Parse a `CardIcon` variant name (produced by `{:?}`) back to the enum value.
fn parse_card_icon(name: &str) -> crate::card::CardIcon {
    use crate::card::CardIcon::*;
    match name {
        "Default" => Default,
        "Star" => Star,
        "Heart" => Heart,
        "Circle" => Circle,
        "Square" => Square,
        "Triangle" => Triangle,
        "Check" => Check,
        "Cross" => Cross,
        "Question" => Question,
        "Exclamation" => Exclamation,
        "Plus" => Plus,
        "Minus" => Minus,
        "StarFill" => StarFill,
        "HeartFill" => HeartFill,
        "CircleFill" => CircleFill,
        "SquareFill" => SquareFill,
        "TriangleFill" => TriangleFill,
        "Diamond" => Diamond,
        "DiamondFill" => DiamondFill,
        "Hexagon" => Hexagon,
        "HexagonFill" => HexagonFill,
        "Octagon" => Octagon,
        "OctagonFill" => OctagonFill,
        "Pentagon" => Pentagon,
        "PentagonFill" => PentagonFill,
        "CheckCircle" => CheckCircle,
        "CheckCircleFill" => CheckCircleFill,
        "XCircle" => XCircle,
        "XCircleFill" => XCircleFill,
        "InfoCircle" => InfoCircle,
        "InfoCircleFill" => InfoCircleFill,
        "ExclamationTriangle" => ExclamationTriangle,
        "ExclamationTriangleFill" => ExclamationTriangleFill,
        "ExclamationCircle" => ExclamationCircle,
        "ExclamationCircleFill" => ExclamationCircleFill,
        "QuestionCircle" => QuestionCircle,
        "QuestionCircleFill" => QuestionCircleFill,
        "PlusCircle" => PlusCircle,
        "PlusCircleFill" => PlusCircleFill,
        "DashCircle" => DashCircle,
        "DashCircleFill" => DashCircleFill,
        "ArrowUp" => ArrowUp,
        "ArrowDown" => ArrowDown,
        "ArrowLeft" => ArrowLeft,
        "ArrowRight" => ArrowRight,
        "ArrowUpCircle" => ArrowUpCircle,
        "ArrowDownCircle" => ArrowDownCircle,
        "ArrowLeftCircle" => ArrowLeftCircle,
        "ArrowRightCircle" => ArrowRightCircle,
        "Book" => Book,
        "BookFill" => BookFill,
        "Bookmark" => Bookmark,
        "BookmarkFill" => BookmarkFill,
        "Calendar" => Calendar,
        "CalendarFill" => CalendarFill,
        "Clock" => Clock,
        "ClockFill" => ClockFill,
        "Flag" => Flag,
        "FlagFill" => FlagFill,
        "Folder" => Folder,
        "FolderFill" => FolderFill,
        "Gear" => Gear,
        "GearFill" => GearFill,
        "House" => House,
        "HouseFill" => HouseFill,
        "Lightning" => Lightning,
        "LightningFill" => LightningFill,
        "Lightbulb" => Lightbulb,
        "LightbulbFill" => LightbulbFill,
        "Link" => Link,
        "Lock" => Lock,
        "LockFill" => LockFill,
        "Unlock" => Unlock,
        "UnlockFill" => UnlockFill,
        "Pencil" => Pencil,
        "PencilFill" => PencilFill,
        "Pin" => Pin,
        "PinFill" => PinFill,
        "Tag" => Tag,
        "TagFill" => TagFill,
        "Trophy" => Trophy,
        "TrophyFill" => TrophyFill,
        "Cloud" => Cloud,
        "CloudFill" => CloudFill,
        "Sun" => Sun,
        "Moon" => Moon,
        "Umbrella" => Umbrella,
        "UmbrellaFill" => UmbrellaFill,
        "Fire" => Fire,
        "Flower1" => Flower1,
        "Tree" => Tree,
        "Bell" => Bell,
        "BellFill" => BellFill,
        "Chat" => Chat,
        "ChatFill" => ChatFill,
        "Envelope" => Envelope,
        "EnvelopeFill" => EnvelopeFill,
        "Telephone" => Telephone,
        "TelephoneFill" => TelephoneFill,
        "Camera" => Camera,
        "CameraFill" => CameraFill,
        "Laptop" => Laptop,
        "Phone" => Phone,
        "Wifi" => Wifi,
        "Person" => Person,
        "PersonFill" => PersonFill,
        "People" => People,
        "PeopleFill" => PeopleFill,
        "Eye" => Eye,
        "EyeFill" => EyeFill,
        "Hand" => Hand,
        "Cup" => Cup,
        "CupFill" => CupFill,
        "Cart" => Cart,
        "CartFill" => CartFill,
        "CreditCard" => CreditCard,
        "CreditCardFill" => CreditCardFill,
        "Graph" => Graph,
        "GraphUp" => GraphUp,
        "GraphDown" => GraphDown,
        "Briefcase" => Briefcase,
        "BriefcaseFill" => BriefcaseFill,
        "Play" => Play,
        "PlayFill" => PlayFill,
        "Pause" => Pause,
        "PauseFill" => PauseFill,
        "Stop" => Stop,
        "StopFill" => StopFill,
        "Music" => Music,
        "MusicFill" => MusicFill,
        "Image" => Image,
        "ImageFill" => ImageFill,
        "Film" => Film,
        "Gift" => Gift,
        "GiftFill" => GiftFill,
        "Balloon" => Balloon,
        "BalloonFill" => BalloonFill,
        "Gem" => Gem,
        "Puzzle" => Puzzle,
        "PuzzleFill" => PuzzleFill,
        "Clipboard" => Clipboard,
        "ClipboardFill" => ClipboardFill,
        _ => Default,
    }
}

// ── File I/O ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum WorkspaceError {
    Io(String),
    Encode(String),
    Decode(String),
    BadMagic,
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::Io(e) => write!(f, "I/O error: {}", e),
            WorkspaceError::Encode(e) => write!(f, "Encode error: {}", e),
            WorkspaceError::Decode(e) => write!(f, "Decode error: {}", e),
            WorkspaceError::BadMagic => write!(f, "Not a valid .cards workspace file"),
        }
    }
}

impl WorkspaceFile {
    /// Create an empty workspace with one default board
    pub fn new_empty(name: impl Into<String>) -> Self {
        Self {
            version: FORMAT_VERSION,
            name: name.into(),
            boards: vec![BoardData {
                name: "Board 1".to_string(),
                cards: Vec::new(),
                connections: Vec::new(),
            }],
            active_board_index: 0,
            canvas_offset_x: 0.0,
            canvas_offset_y: 0.0,
        }
    }

    /// Save to a `.cards` file.
    /// Format: 6-byte magic | 1-byte version | rmp-encoded payload
    pub fn save(&self, path: &Path) -> Result<(), WorkspaceError> {
        // Create parent dirs if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| WorkspaceError::Io(e.to_string()))?;
        }

        let payload = rmp_serde::to_vec_named(self)
            .map_err(|e| WorkspaceError::Encode(e.to_string()))?;

        let mut file = fs::File::create(path)
            .map_err(|e| WorkspaceError::Io(e.to_string()))?;

        file.write_all(FILE_MAGIC)
            .map_err(|e| WorkspaceError::Io(e.to_string()))?;
        file.write_all(&[FORMAT_VERSION])
            .map_err(|e| WorkspaceError::Io(e.to_string()))?;
        file.write_all(&payload)
            .map_err(|e| WorkspaceError::Io(e.to_string()))?;

        Ok(())
    }

    /// Load from a `.cards` file.
    pub fn load(path: &Path) -> Result<Self, WorkspaceError> {
        let bytes = fs::read(path)
            .map_err(|e| WorkspaceError::Io(e.to_string()))?;

        // Verify magic
        if bytes.len() < FILE_MAGIC.len() + 1 || &bytes[..FILE_MAGIC.len()] != FILE_MAGIC {
            return Err(WorkspaceError::BadMagic);
        }

        let payload = &bytes[FILE_MAGIC.len() + 1..]; // skip magic + version byte
        let workspace: WorkspaceFile = rmp_serde::from_slice(payload)
            .map_err(|e| WorkspaceError::Decode(e.to_string()))?;

        Ok(workspace)
    }

    /// Return the default directory where new workspaces are saved
    /// (~/.local/share/cards/ on Linux, ~/AppData/Roaming/cards/ on Windows, etc.)
    pub fn default_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("cards"))
    }
}




