/// import_export.rs — Import / Export logic for workspaces and boards
///
/// Supported formats:
///
///   .cards-workspace  — Full workspace  (msgpack, same envelope as .cards)
///   .cards-board      — Single board    (msgpack)
///   .json             — Human-readable; workspace or board depending on context
///
/// Every exported file starts with a metadata header so the reader can detect
/// version mismatches and schema incompatibilities before attempting to decode.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

use crate::workspace::{BoardData, CardData, WorkspaceFile};

// ── Format version ────────────────────────────────────────────────────────────

/// Bump when the exported schema changes in a way that breaks old importers.
pub const EXPORT_FORMAT_VERSION: u32 = 1;

/// Magic prefix for binary (.cards-workspace / .cards-board) exports
const WS_MAGIC:    &[u8; 8] = b"CRDWS\x01\x00\x00";
const BOARD_MAGIC: &[u8; 8] = b"CRDBRD\x01\x00";

// ── Export format selector ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    CardsWorkspace,
    CardsBoard,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json           => "json",
            ExportFormat::CardsWorkspace => "cards-workspace",
            ExportFormat::CardsBoard     => "cards-board",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Json           => "JSON (.json)",
            ExportFormat::CardsWorkspace => "Cards Workspace (.cards-workspace)",
            ExportFormat::CardsBoard     => "Cards Board (.cards-board)",
        }
    }
}

// ── On-disk envelope structs ──────────────────────────────────────────────────

/// Metadata block written into every exported file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMeta {
    /// Format-version of this export (== EXPORT_FORMAT_VERSION when written)
    pub format_version: u32,
    /// App version string at export time
    pub app_version: String,
    /// Human-readable timestamp (RFC 3339)
    pub exported_at: String,
    /// "workspace" or "board"
    pub kind: String,
}

impl ExportMeta {
    fn new(kind: impl Into<String>) -> Self {
        // Simple timestamp from system time (no external crate needed)
        let ts = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            // Format as ISO-8601-ish manually
            let s = secs;
            let sec = s % 60;
            let min = (s / 60) % 60;
            let hour = (s / 3600) % 24;
            let days = s / 86400;
            // Approximate year/month/day (good enough for a label, not calendar-correct)
            let year = 1970 + days / 365;
            let doy  = days % 365;
            let month = doy / 30 + 1;
            let day   = doy % 30 + 1;
            format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hour, min, sec)
        };
        Self {
            format_version: EXPORT_FORMAT_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: ts,
            kind: kind.into(),
        }
    }
}

/// Full workspace export envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExport {
    pub meta: ExportMeta,
    pub workspace: WorkspaceFile,
}

/// Single-board export envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardExport {
    pub meta: ExportMeta,
    pub board: BoardData,
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ImportExportError {
    Io(String),
    Encode(String),
    /// File format / magic bytes wrong
    WrongFormat(String),
    /// Version newer than this build understands
    VersionMismatch { file_version: u32, our_version: u32 },
    /// Decoded OK but content is incompatible (missing required fields, etc.)
    IncompatibleData(String),
    /// File claims to be a board but we expected a workspace (or vice-versa)
    WrongKind { expected: String, got: String },
}

impl std::fmt::Display for ImportExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportExportError::Io(e) =>
                write!(f, "I/O error: {}", e),
            ImportExportError::Encode(e) =>
                write!(f, "Encoding error: {}", e),
            ImportExportError::WrongFormat(e) =>
                write!(f, "Not a valid export file: {}", e),
            ImportExportError::VersionMismatch { file_version, our_version } =>
                write!(f, "Version mismatch: file is v{} but this build only understands up to v{}", file_version, our_version),
            ImportExportError::IncompatibleData(e) =>
                write!(f, "Data is not compatible with this version: {}", e),
            ImportExportError::WrongKind { expected, got } =>
                write!(f, "Expected a {} export but got a {} export", expected, got),
        }
    }
}

// ── Export ────────────────────────────────────────────────────────────────────

/// Export a full workspace to `path` in the given format.
pub fn export_workspace(
    ws: &WorkspaceFile,
    path: &Path,
    fmt: ExportFormat,
) -> Result<(), ImportExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ImportExportError::Io(e.to_string()))?;
    }

    let envelope = WorkspaceExport {
        meta: ExportMeta::new("workspace"),
        workspace: ws.clone(),
    };

    match fmt {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&envelope)
                .map_err(|e| ImportExportError::Encode(e.to_string()))?;
            write_file(path, json.as_bytes())?;
        }
        ExportFormat::CardsWorkspace => {
            let payload = rmp_serde::to_vec_named(&envelope)
                .map_err(|e| ImportExportError::Encode(e.to_string()))?;
            let mut data = Vec::with_capacity(WS_MAGIC.len() + payload.len());
            data.extend_from_slice(WS_MAGIC);
            data.extend_from_slice(&payload);
            write_file(path, &data)?;
        }
        ExportFormat::CardsBoard => {
            return Err(ImportExportError::WrongKind {
                expected: "workspace".into(),
                got: "board".into(),
            });
        }
    }
    Ok(())
}

/// Export a single board to `path` in the given format.
pub fn export_board(
    board: &BoardData,
    path: &Path,
    fmt: ExportFormat,
) -> Result<(), ImportExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ImportExportError::Io(e.to_string()))?;
    }

    let envelope = BoardExport {
        meta: ExportMeta::new("board"),
        board: board.clone(),
    };

    match fmt {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&envelope)
                .map_err(|e| ImportExportError::Encode(e.to_string()))?;
            write_file(path, json.as_bytes())?;
        }
        ExportFormat::CardsBoard => {
            let payload = rmp_serde::to_vec_named(&envelope)
                .map_err(|e| ImportExportError::Encode(e.to_string()))?;
            let mut data = Vec::with_capacity(BOARD_MAGIC.len() + payload.len());
            data.extend_from_slice(BOARD_MAGIC);
            data.extend_from_slice(&payload);
            write_file(path, &data)?;
        }
        ExportFormat::CardsWorkspace => {
            return Err(ImportExportError::WrongKind {
                expected: "board".into(),
                got: "workspace".into(),
            });
        }
    }
    Ok(())
}

// ── Import ────────────────────────────────────────────────────────────────────

/// Result of a successful workspace import
pub struct WorkspaceImportResult {
    pub workspace: WorkspaceFile,
    /// Non-empty if the file version differs from ours but was imported anyway
    pub warnings: Vec<String>,
}

/// Result of a successful board import
pub struct BoardImportResult {
    pub board: BoardData,
    pub warnings: Vec<String>,
}

/// Import a workspace from `path`, auto-detecting format.
pub fn import_workspace(path: &Path) -> Result<WorkspaceImportResult, ImportExportError> {
    let bytes = std::fs::read(path)
        .map_err(|e| ImportExportError::Io(e.to_string()))?;

    let envelope: WorkspaceExport = if bytes.starts_with(WS_MAGIC) {
        // Binary .cards-workspace
        let payload = &bytes[WS_MAGIC.len()..];
        rmp_serde::from_slice(payload)
            .map_err(|e| ImportExportError::IncompatibleData(e.to_string()))?
    } else if bytes.starts_with(b"{") || bytes.starts_with(b" ") {
        // JSON
        serde_json::from_slice(&bytes)
            .map_err(|e| ImportExportError::IncompatibleData(
                format!("JSON parse failed: {}", e)
            ))?
    } else {
        return Err(ImportExportError::WrongFormat(
            "File does not start with a recognised workspace signature".into()
        ));
    };

    let mut warnings = Vec::new();
    check_meta(&envelope.meta, "workspace", &mut warnings)?;
    validate_workspace(&envelope.workspace)?;

    Ok(WorkspaceImportResult { workspace: envelope.workspace, warnings })
}

/// Import a board from `path`, auto-detecting format.
pub fn import_board(path: &Path) -> Result<BoardImportResult, ImportExportError> {
    let bytes = std::fs::read(path)
        .map_err(|e| ImportExportError::Io(e.to_string()))?;

    let envelope: BoardExport = if bytes.starts_with(BOARD_MAGIC) {
        let payload = &bytes[BOARD_MAGIC.len()..];
        rmp_serde::from_slice(payload)
            .map_err(|e| ImportExportError::IncompatibleData(e.to_string()))?
    } else if bytes.starts_with(b"{") || bytes.starts_with(b" ") {
        serde_json::from_slice(&bytes)
            .map_err(|e| ImportExportError::IncompatibleData(
                format!("JSON parse failed: {}", e)
            ))?
    } else {
        return Err(ImportExportError::WrongFormat(
            "File does not start with a recognised board signature".into()
        ));
    };

    let mut warnings = Vec::new();
    check_meta(&envelope.meta, "board", &mut warnings)?;
    validate_board(&envelope.board)?;

    Ok(BoardImportResult { board: envelope.board, warnings })
}

// ── Validation helpers ────────────────────────────────────────────────────────

fn check_meta(
    meta: &ExportMeta,
    expected_kind: &str,
    warnings: &mut Vec<String>,
) -> Result<(), ImportExportError> {
    // Kind mismatch is a hard error
    if meta.kind != expected_kind {
        return Err(ImportExportError::WrongKind {
            expected: expected_kind.into(),
            got: meta.kind.clone(),
        });
    }
    // Version newer than we know → hard error
    if meta.format_version > EXPORT_FORMAT_VERSION {
        return Err(ImportExportError::VersionMismatch {
            file_version: meta.format_version,
            our_version: EXPORT_FORMAT_VERSION,
        });
    }
    // Version older → soft warning, still try
    if meta.format_version < EXPORT_FORMAT_VERSION {
        warnings.push(format!(
            "File was created with an older format (v{}). Some data may be missing.",
            meta.format_version
        ));
    }
    // App version mismatch → just a warning
    let our_ver = env!("CARGO_PKG_VERSION");
    if meta.app_version != our_ver {
        warnings.push(format!(
            "File was exported by app v{}, you are running v{}.",
            meta.app_version, our_ver
        ));
    }
    Ok(())
}

fn validate_workspace(ws: &WorkspaceFile) -> Result<(), ImportExportError> {
    if ws.boards.is_empty() {
        return Err(ImportExportError::IncompatibleData(
            "Workspace contains no boards".into()
        ));
    }
    for (i, board) in ws.boards.iter().enumerate() {
        validate_board(board).map_err(|e| ImportExportError::IncompatibleData(
            format!("Board {}: {}", i, e)
        ))?;
    }
    Ok(())
}

fn validate_board(board: &BoardData) -> Result<(), ImportExportError> {
    if board.name.is_empty() {
        return Err(ImportExportError::IncompatibleData(
            "Board has an empty name".into()
        ));
    }
    for (i, card) in board.cards.iter().enumerate() {
        validate_card(card).map_err(|e| ImportExportError::IncompatibleData(
            format!("Card {}: {}", i, e)
        ))?;
    }
    Ok(())
}

fn validate_card(card: &CardData) -> Result<(), ImportExportError> {
    if card.width < 1.0 || card.height < 1.0 {
        return Err(ImportExportError::IncompatibleData(
            format!("Card {} has invalid size {}×{}", card.id, card.width, card.height)
        ));
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn write_file(path: &Path, data: &[u8]) -> Result<(), ImportExportError> {
    let mut f = std::fs::File::create(path)
        .map_err(|e| ImportExportError::Io(e.to_string()))?;
    f.write_all(data)
        .map_err(|e| ImportExportError::Io(e.to_string()))
}

