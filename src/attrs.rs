use crate::element::BoxSize;

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
    pub radius: f32
}

impl Border {
    pub fn size(&self) -> BoxSize {
        BoxSize { vert: self.thickness + self.thickness, horiz: self.thickness + self.thickness }
    }
}

impl Default for Border {
    fn default() -> Self {
        Border {
            thickness: 2.0,
            radius: 0.0
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ElementAttrs {
    pub border: Border,
    pub padding: Padding,
    pub margin: Margin,

    pub width: Size,
    pub height: Size,
}