use gpui::{Hsla, hsla};

/// Syntax highlighting colors for code blocks.
#[derive(Debug, Clone)]
pub struct SyntaxColors {
    pub keyword: Hsla,
    pub string: Hsla,
    pub comment: Hsla,
    pub function: Hsla,
    pub r#type: Hsla,
    pub variable: Hsla,
    pub number: Hsla,
    pub operator: Hsla,
    pub punctuation: Hsla,
    pub constant: Hsla,
    pub attribute: Hsla,
    pub tag: Hsla,
}

impl SyntaxColors {
    /// Create syntax highlighting colors for the given appearance mode.
    pub fn new(is_dark: bool) -> Self {
        if is_dark {
            SyntaxColors {
                keyword: hsla(0.83, 0.60, 0.75, 1.0),
                string: hsla(0.30, 0.60, 0.70, 1.0),
                comment: hsla(0.0, 0.0, 0.52, 1.0),
                function: hsla(0.75, 0.50, 0.75, 1.0),
                r#type: hsla(0.55, 0.70, 0.75, 1.0),
                variable: hsla(0.0, 0.0, 0.90, 1.0),
                number: hsla(0.10, 0.80, 0.70, 1.0),
                operator: hsla(0.55, 0.60, 0.70, 1.0),
                punctuation: hsla(0.0, 0.0, 0.60, 1.0),
                constant: hsla(0.55, 0.70, 0.75, 1.0),
                attribute: hsla(0.10, 0.70, 0.70, 1.0),
                tag: hsla(0.35, 0.60, 0.65, 1.0),
            }
        } else {
            SyntaxColors {
                keyword: hsla(0.83, 0.70, 0.45, 1.0),
                string: hsla(0.58, 0.60, 0.35, 1.0),
                comment: hsla(0.0, 0.0, 0.48, 1.0),
                function: hsla(0.75, 0.60, 0.40, 1.0),
                r#type: hsla(0.55, 0.70, 0.40, 1.0),
                variable: hsla(0.0, 0.0, 0.15, 1.0),
                number: hsla(0.10, 0.80, 0.45, 1.0),
                operator: hsla(0.0, 0.0, 0.30, 1.0),
                punctuation: hsla(0.0, 0.0, 0.40, 1.0),
                constant: hsla(0.55, 0.70, 0.40, 1.0),
                attribute: hsla(0.10, 0.70, 0.45, 1.0),
                tag: hsla(0.35, 0.60, 0.40, 1.0),
            }
        }
    }
}
