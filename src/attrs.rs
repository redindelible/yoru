use crate::element::BoxSize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color::from_rgb8(0, 0, 0);
    pub const DARK_GRAY: Color = Color::from_rgb8(105, 105, 105);
    pub const GRAY: Color = Color::from_rgb8(128, 128, 128);
    pub const LIGHT_GRAY: Color = Color::from_rgb8(211, 211, 211);
    pub const WHITE: Color = Color::from_rgb8(255, 255, 255);
    pub const RED: Color = Color::from_rgb8(255, 0, 0);
    pub const GREEN: Color = Color::from_rgb8(0, 255, 0);
    pub const BLUE: Color = Color::from_rgb8(0, 0, 255);

    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba8(r, g, b, 255)
    }

    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
}

impl From<Color> for cosmic_text::Color {
    fn from(value: Color) -> Self {
        cosmic_text::Color::rgba(value.r, value.g, value.b, value.a)
    }
}

impl From<Color> for tiny_skia::Color {
    fn from(value: Color) -> Self {
        // when writing to a buffer we need to swap b and r
        tiny_skia::Color::from_rgba8(value.b, value.g, value.r, value.a)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub enum Size {
    Expand,
    // Split(f32),
    #[default]
    Fit,
    Fixed(f32)
}

#[derive(Debug)]
enum Justify {
    Min,
    Max,
    Center
}

#[derive(Debug, Copy, Clone)]
pub struct Margin {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

impl Margin {
    pub fn size(&self) -> BoxSize {
        BoxSize { vert: self.top + self.bottom, horiz: self.left + self.right }
    }
}

impl Default for Margin {
    fn default() -> Self {
        Margin::from(2.0)
    }
}

impl From<f32> for Margin {
    fn from(value: f32) -> Self {
        Margin {
            top: value, left: value, bottom: value, right: value
        }
    }
}

impl From<(f32, f32)> for Margin {
    fn from(value: (f32, f32)) -> Self {
        Margin {
            top: value.0, left: value.1, bottom: value.0, right: value.1
        }
    }
}

impl From<(f32, f32, f32, f32)> for Margin {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Margin {
            top: value.0, left: value.1, bottom: value.2, right: value.3
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Padding {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

impl Padding {
    pub fn size(&self) -> BoxSize {
        BoxSize { vert: self.top + self.bottom, horiz: self.left + self.right }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Padding::from(2.0)
    }
}

impl From<f32> for Padding {
    fn from(value: f32) -> Self {
        Padding {
            top: value, left: value, bottom: value, right: value
        }
    }
}

impl From<(f32, f32)> for Padding {
    fn from(value: (f32, f32)) -> Self {
        Padding {
            top: value.0, left: value.1, bottom: value.0, right: value.1
        }
    }
}

impl From<(f32, f32, f32, f32)> for Padding {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Padding {
            top: value.0, left: value.1, bottom: value.2, right: value.3
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Border {
    pub thickness: f32,
    pub radius: f32,
    pub color: Color,
}

impl Border {
    pub fn with_color(&self, color: Color) -> Border {
        Border { color, ..*self }
    }

    pub fn with_thickness(&self, thickness: f32) -> Border {
        Border { thickness, ..*self }
    }

    pub fn with_radius(&self, radius: f32) -> Border {
        Border { radius, ..*self }
    }

    pub fn size(&self) -> BoxSize {
        BoxSize { vert: self.thickness + self.thickness, horiz: self.thickness + self.thickness }
    }
}

impl Default for Border {
    fn default() -> Self {
        Border {
            thickness: 2.0,
            radius: 0.0,
            color: Color::BLACK
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ElementAttrs {
    pub border: Border,
    pub padding: Padding,
    pub margin: Margin,
    pub background: Option<Color>,

    pub width: Size,
    pub height: Size,
}