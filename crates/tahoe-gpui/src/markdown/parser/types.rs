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
///
/// Marked `#[non_exhaustive]` so future HIG or GFM block additions can
/// land without forcing downstream consumers to update exhaustive
/// matches. Add a `_ => …` arm when pattern-matching this enum.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MarkdownBlock {
    /// A paragraph of inline content.
    Paragraph(Vec<InlineContent>),
    /// A heading with level (1-6) and inline content.
    ///
    /// `anchor_id` is the slug used as the heading's URL fragment — either
    /// the explicit `{#id}` attribute (when the author wrote one) or a
    /// GitHub-style auto-slug derived from the heading's plain text. `None`
    /// means the heading produced no usable slug (empty or all-punctuation
    /// text) and should not be addressed by a `#fragment` link. Slugs are
    /// deduplicated across the document by appending `-2`, `-3`, … to the
    /// later occurrences, matching GitHub's rendering.
    Heading {
        level: u8,
        content: Vec<InlineContent>,
        anchor_id: Option<String>,
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
    DisplayMath(gpui::SharedString),
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
