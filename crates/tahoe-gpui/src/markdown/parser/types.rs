//! Data types for the markdown parser.

/// Per-column alignment for a GFM pipe table.
///
/// Mirrors `pulldown_cmark::Alignment` so downstream consumers do not
/// need to depend on the parser crate directly. Applied at render time
/// via flex-axis positioning until GPUI exposes a `text_align` API for
/// text inside a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlignment {
    /// No explicit alignment; defaults to leading-edge (left in LTR).
    #[default]
    None,
    /// Left-align cell content.
    Left,
    /// Center-align cell content.
    Center,
    /// Right-align cell content.
    Right,
}

impl From<pulldown_cmark::Alignment> for TableAlignment {
    fn from(value: pulldown_cmark::Alignment) -> Self {
        match value {
            pulldown_cmark::Alignment::None => Self::None,
            pulldown_cmark::Alignment::Left => Self::Left,
            pulldown_cmark::Alignment::Center => Self::Center,
            pulldown_cmark::Alignment::Right => Self::Right,
        }
    }
}

/// A parsed block of markdown content.
#[derive(Debug, Clone)]
pub enum MarkdownBlock {
    /// A paragraph of inline content.
    Paragraph(Vec<InlineContent>),
    /// A heading with level (1-6) and inline content.
    Heading {
        level: u8,
        content: Vec<InlineContent>,
    },
    /// A fenced code block with optional language.
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    /// An unordered or ordered list.
    List {
        ordered: bool,
        start: Option<u64>,
        items: Vec<Vec<MarkdownBlock>>,
    },
    /// A blockquote containing nested blocks.
    BlockQuote(Vec<MarkdownBlock>),
    /// A thematic break / horizontal rule.
    ThematicBreak,
    /// A table with optional per-column alignment.
    Table {
        headers: Vec<Vec<InlineContent>>,
        rows: Vec<Vec<Vec<InlineContent>>>,
        /// One entry per column, mirroring the GFM `| :---: |` row.
        /// Empty when the document omitted an alignment row.
        alignments: Vec<TableAlignment>,
    },
    /// A display math block (e.g. `$$x^2 + y^2 = z^2$$`).
    DisplayMath(String),
    /// A GFM task-list item at top level. Nested inside a `List` the
    /// parent renders the checkbox glyph; this variant is used for
    /// stray task items the pulldown-cmark event stream emits outside
    /// a list container.
    TaskItem {
        checked: bool,
        content: Vec<InlineContent>,
    },
}

/// Inline content within a paragraph or heading.
#[derive(Debug, Clone)]
pub enum InlineContent {
    Text(String),
    Code(String),
    Bold(Vec<InlineContent>),
    Italic(Vec<InlineContent>),
    Strikethrough(Vec<InlineContent>),
    Link {
        url: String,
        content: Vec<InlineContent>,
    },
    /// An inline citation reference, e.g. `[1]`. The number is 1-based.
    Citation(usize),
    /// An inline image.
    Image {
        url: String,
        alt: String,
    },
    /// Inline math expression (e.g. `$x^2$`).
    InlineMath(String),
    /// GFM task-list marker (`[ ]` / `[x]`) emitted at the start of a
    /// task-list item. The list renderer consumes this to draw a
    /// checkbox glyph before the item's text content.
    TaskMarker(bool),
    SoftBreak,
    HardBreak,
}
