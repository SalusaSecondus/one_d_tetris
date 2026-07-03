use crossterm::{
    Command,
    style::{Color, Colored},
};

pub const CSI: &str = "\x1b[";

pub const BLACK: Color = Color::Rgb { r: 0, g: 0, b: 0 };
pub const RED: Color = Color::Rgb { r: 255, g: 0, b: 0 };
pub const GREEN: Color = Color::Rgb { r: 0, g: 255, b: 0 };
pub const BLUE: Color = Color::Rgb { r: 0, g: 0, b: 255 };
pub const WHITE: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 255,
};
pub const GRAY: Color = Color::Rgb {
    r: 228,
    g: 228,
    b: 228,
};
pub const RED_GREEN: crossterm::style::Color = blend(RED, GREEN);
pub const RED_BLUE: crossterm::style::Color = blend(RED, BLUE);
pub const BLUE_GREEN: crossterm::style::Color = blend(BLUE, GREEN);

pub const fn blend(a: Color, b: Color) -> Color {
    let a_parts = match a {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => panic!("Unsupported color"),
    };
    let b_parts = match b {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => panic!("Unsupported color"),
    };
    Color::Rgb {
        r: a_parts.0 + b_parts.0,
        g: a_parts.1 + b_parts.1,
        b: a_parts.2 + b_parts.2,
    }
}

pub fn collides(a: Color, b: Color) -> bool {
    let a_parts = match a {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => panic!("Unsupported color"),
    };
    let b_parts = match b {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => panic!("Unsupported color"),
    };
    (a_parts.0 > 0 && b_parts.0 > 0)
        || (a_parts.1 > 0 && b_parts.1 > 0)
        || (a_parts.2 > 0 && b_parts.2 > 0)
}

pub struct RenderCell(pub Color, pub bool);

impl Command for RenderCell {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        if self.0 == BLACK {
            return write!(f, " ");
        }
        if self.1 {
            // Color blind mode
            let char = match self.0 {
                RED => '▅',
                GREEN => '▌',
                BLUE => '▞',
                RED_GREEN => '▙',
                RED_BLUE => '▟',
                BLUE_GREEN => '▛',
                BLACK => ' ',
                _ => '▓',
            };
            write!(f, "{}{}m{}", CSI, Colored::ForegroundColor(self.0), char)
        } else {
            write!(f, "{}{}m▓", CSI, Colored::ForegroundColor(self.0))
        }
    }
}
