/// Math rendering via Typst — converts Typst math expressions to SVG handles
/// that can be drawn directly into an Iced canvas frame.
///
/// Typst math syntax is used (not LaTeX). Key differences from LaTeX:
///   \frac{a}{b}  →  a/b  (or frac(a,b))
///   \sqrt{x}     →  sqrt(x)
///   \alpha       →  alpha  (no backslash for Greek letters)
///   \sum_{i}^{n} →  sum_(i)^n
///   \int_0^\inf  →  integral_0^infinity
///   \pm          →  plus.minus
///   \infty       →  infinity
use std::collections::HashMap;

use typst::LibraryExt;
use typst::diag::FileError;
use typst::foundations::Bytes;
use typst::layout::PagedDocument;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::syntax::{FileId, Source, VirtualPath};
use typst_kit::fonts::{FontSearcher, FontSlot};

use iced::widget::svg::Handle as SvgHandle;

// ── MathWorld ─────────────────────────────────────────────────────────────────

struct MathWorld {
    source:  Source,
    library: LazyHash<typst::Library>,
    book:    LazyHash<FontBook>,
    fonts:   Vec<FontSlot>,
}

impl MathWorld {
    fn new(typst_src: String) -> Self {
        let fonts = FontSearcher::new()
            .include_system_fonts(false)
            .search();

        let file_id = FileId::new(None, VirtualPath::new("/math.typ"));
        let source  = Source::new(file_id, typst_src);

        Self {
            source,
            library: LazyHash::new(typst::Library::default()),
            book:    LazyHash::new(fonts.book),
            fonts:   fonts.fonts,
        }
    }
}

impl typst::World for MathWorld {
    fn library(&self) -> &LazyHash<typst::Library> { &self.library }
    fn book(&self)    -> &LazyHash<FontBook>        { &self.book }
    fn main(&self)    -> FileId                     { self.source.id() }

    fn source(&self, id: FileId) -> typst::diag::FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().to_owned()))
        }
    }

    fn file(&self, id: FileId) -> typst::diag::FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rootless_path().to_owned()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index)?.get()
    }

    fn today(&self, _offset: Option<i64>) -> Option<typst::foundations::Datetime> {
        None
    }
}

// ── SVG rendering ─────────────────────────────────────────────────────────────

/// Build a Typst source document that renders a single math expression.
/// `is_display` controls whether to use display or inline style.
fn build_typst_src(expr: &str, is_display: bool, font_size_pt: f32) -> String {
    let math = if is_display {
        // Display math: spaces inside $ delimiters → centred, full-size
        format!("$ {} $", expr)
    } else {
        // Inline math: no surrounding spaces
        format!("${}$", expr)
    };
    format!(
        "#set page(width: auto, height: auto, margin: 3pt, fill: none)\n\
         #set text(size: {:.1}pt)\n\
         {}",
        font_size_pt, math
    )
}

/// Render a Typst math expression to SVG bytes.  Returns `None` on error.
/// `formula` may be LaTeX or Typst — it is converted to Typst first.
pub fn render_math_to_svg(formula: &str, is_display: bool, font_size_pt: f32) -> Option<Vec<u8>> {
    let typst_expr = crate::latex_to_typst::convert(formula);
    let src   = build_typst_src(&typst_expr, is_display, font_size_pt);
    let world = MathWorld::new(src);

    let result: typst::diag::Warned<typst::diag::SourceResult<PagedDocument>> =
        typst::compile(&world);

    let doc = result.output.ok()?;
    let svg = typst_svg::svg_merged(&doc, typst::layout::Abs::zero());
    Some(svg.into_bytes())
}

/// Parse the `width="Xpt"` and `height="Ypt"` from a typst SVG string.
/// Returns `(width_px, height_px)` treating 1pt as 4/3 logical pixels.
pub fn parse_svg_size(svg_bytes: &[u8]) -> (f32, f32) {
    let s = std::str::from_utf8(svg_bytes).unwrap_or("");
    let w = parse_attr(s, "width");
    let h = parse_attr(s, "height");
    // 1pt = 96/72 px = 1.333...
    (w * 4.0 / 3.0, h * 4.0 / 3.0)
}

fn parse_attr(svg: &str, attr: &str) -> f32 {
    let needle = format!("{}=\"", attr);
    if let Some(start) = svg.find(&needle) {
        let rest = &svg[start + needle.len()..];
        if let Some(end) = rest.find('"') {
            let val = &rest[..end];
            // strip unit suffix (pt, px, etc.)
            let digits: String = val.chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            return digits.parse::<f32>().unwrap_or(0.0);
        }
    }
    0.0
}

// ── MathCache ─────────────────────────────────────────────────────────────────

/// Cached math SVG: handle + natural width/height in logical pixels.
#[derive(Clone)]
pub struct MathEntry {
    pub handle: SvgHandle,
    pub width:  f32,
    pub height: f32,
}

/// In-memory cache from formula → rendered SVG.
/// Keyed by `(formula_string, is_display)`.
pub struct MathCache {
    entries: HashMap<(String, bool), MathEntry>,
    /// Base font size used for rendering (pt).  Changing this invalidates the cache.
    font_size: f32,
}

impl MathCache {
    pub fn new(font_size: f32) -> Self {
        Self { entries: HashMap::new(), font_size }
    }

    pub fn set_font_size(&mut self, size: f32) {
        if (size - self.font_size).abs() > 0.1 {
            self.entries.clear();
            self.font_size = size;
        }
    }

    /// Return the cached entry, rendering it on first access.
    pub fn get_or_render(&mut self, formula: &str, is_display: bool) -> Option<&MathEntry> {
        let key = (formula.to_string(), is_display);
        if !self.entries.contains_key(&key) {
            let bytes = render_math_to_svg(formula, is_display, self.font_size)?;
            let (w, h) = parse_svg_size(&bytes);
            let entry = MathEntry {
                handle: SvgHandle::from_memory(bytes),
                width:  w.max(4.0),
                height: h.max(4.0),
            };
            self.entries.insert(key.clone(), entry);
        }
        self.entries.get(&key)
    }
}
