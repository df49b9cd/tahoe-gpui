use gpui::{Hsla, hsla};

/// Standard 16-color ANSI terminal palette.
#[derive(Debug, Clone)]
pub struct AnsiColors {
    pub black: Hsla,
    pub red: Hsla,
    pub green: Hsla,
    pub yellow: Hsla,
    pub blue: Hsla,
    pub magenta: Hsla,
    pub cyan: Hsla,
    pub white: Hsla,
    pub bright_black: Hsla,
    pub bright_red: Hsla,
    pub bright_green: Hsla,
    pub bright_yellow: Hsla,
    pub bright_blue: Hsla,
    pub bright_magenta: Hsla,
    pub bright_cyan: Hsla,
    pub bright_white: Hsla,
}

impl AnsiColors {
    /// Create ANSI colors for the given appearance mode.
    pub fn new(is_dark: bool) -> Self {
        if is_dark {
            AnsiColors {
                black: hsla(0.0, 0.0, 0.10, 1.0),
                red: hsla(0.0, 0.70, 0.60, 1.0),
                green: hsla(0.35, 0.65, 0.55, 1.0),
                yellow: hsla(0.12, 0.75, 0.65, 1.0),
                blue: hsla(0.58, 0.70, 0.60, 1.0),
                magenta: hsla(0.83, 0.55, 0.65, 1.0),
                cyan: hsla(0.50, 0.60, 0.65, 1.0),
                white: hsla(0.0, 0.0, 0.80, 1.0),
                bright_black: hsla(0.0, 0.0, 0.35, 1.0),
                bright_red: hsla(0.0, 0.75, 0.70, 1.0),
                bright_green: hsla(0.35, 0.70, 0.65, 1.0),
                bright_yellow: hsla(0.12, 0.80, 0.75, 1.0),
                bright_blue: hsla(0.58, 0.75, 0.70, 1.0),
                bright_magenta: hsla(0.83, 0.60, 0.75, 1.0),
                bright_cyan: hsla(0.50, 0.65, 0.75, 1.0),
                bright_white: hsla(0.0, 0.0, 0.95, 1.0),
            }
        } else {
            AnsiColors {
                black: hsla(0.0, 0.0, 0.15, 1.0),
                red: hsla(0.0, 0.70, 0.45, 1.0),
                green: hsla(0.35, 0.65, 0.35, 1.0),
                yellow: hsla(0.12, 0.75, 0.45, 1.0),
                blue: hsla(0.58, 0.70, 0.45, 1.0),
                magenta: hsla(0.83, 0.55, 0.45, 1.0),
                cyan: hsla(0.50, 0.60, 0.40, 1.0),
                white: hsla(0.0, 0.0, 0.95, 1.0),
                bright_black: hsla(0.0, 0.0, 0.40, 1.0),
                bright_red: hsla(0.0, 0.75, 0.55, 1.0),
                bright_green: hsla(0.35, 0.70, 0.45, 1.0),
                bright_yellow: hsla(0.12, 0.80, 0.55, 1.0),
                bright_blue: hsla(0.58, 0.75, 0.55, 1.0),
                bright_magenta: hsla(0.83, 0.60, 0.55, 1.0),
                bright_cyan: hsla(0.50, 0.65, 0.50, 1.0),
                bright_white: hsla(0.0, 0.0, 0.98, 1.0),
            }
        }
    }

    /// Return `color` with its alpha reduced to match SGR 2 (dim / faint).
    ///
    /// xterm and VTE render dim as a ~50 % opacity draw of the original
    /// foreground. We mirror that so dim text visibly degrades intensity
    /// while retaining the underlying hue — bold+dim then produces the
    /// "medium" weight that some CLIs exploit for de-emphasised output.
    pub fn dim(&self, color: Hsla) -> Hsla {
        Hsla {
            a: color.a * 0.6,
            ..color
        }
    }
}
