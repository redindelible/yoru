use crate::math;

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
        // when writing to a `softbuffer::Buffer` we need to swap b and r
        tiny_skia::Color::from_rgba8(value.b, value.g, value.r, value.a)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Sizing {
    Expand,
    // Split(f32),
    Fit,
    Fixed(f32)
}

impl Sizing {
    pub fn as_definite(&self, scale_factor: f32) -> Option<f32> {
        match self {
            Sizing::Expand => None,
            Sizing::Fit => None,
            Sizing::Fixed(size) => Some(*size * scale_factor)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Justify {
    Min,
    Max,
    Center
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    Positive,
    Negative,
}

#[derive(Debug, Copy, Clone)]
pub struct LayoutStyle {
    pub border_size: f32,
    pub padding: math::SizeRect,
    pub margin: math::SizeRect,

    pub width: Sizing,
    pub height: Sizing
}

impl LayoutStyle {
    pub fn spacing_size(&self) -> math::SizeRect {
        self.margin + math::SizeRect::from_border(self.border_size) + self.padding
    }
}


#[derive(Debug, Copy, Clone)]
pub struct ContainerLayoutStyle {
    pub layout_style: LayoutStyle,

    pub main_axis: math::Axis,
    pub main_direction: Direction,
    pub main_justify: Justify,
    pub cross_justify: Justify
}
