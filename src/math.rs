use std::ops::{Add, Sub};


#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Axis {
    Horizontal,
    #[default]
    Vertical,
}

impl Axis {
    pub fn cross(&self) -> Axis {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Size {
    pub vertical: f32,
    pub horizontal: f32
}

impl Size {
    pub fn new(horizontal: f32, vertical: f32) -> Size {
        Size { vertical, horizontal }
    }

    pub fn from_axes(main_axis: Axis, main_size: f32, cross_size: f32) -> Size {
        match main_axis {
            Axis::Horizontal => Size::new(main_size, cross_size),
            Axis::Vertical => Size::new(cross_size, main_size)
        }
    }

    pub fn from_border(size: f32) -> Size {
        Size::new(2.0 * size, 2.0 * size)
    }

    pub fn axis(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Vertical => self.vertical,
            Axis::Horizontal => self.horizontal
        }
    }

    pub fn clamp_positive(&self) -> Size {
        Size {
            vertical: self.vertical.max(0.0),
            horizontal: self.horizontal.max(0.0)
        }
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        Size {
            vertical: self.vertical + rhs.vertical,
            horizontal: self.horizontal + rhs.horizontal
        }
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Self) -> Self::Output {
        Size {
            vertical: self.vertical - rhs.vertical,
            horizontal: self.horizontal - rhs.horizontal
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn from_lrtb(left: f32, right: f32, top: f32, bottom: f32) -> Rect {
        Rect::from_xywh(left, top, right - left, bottom - top)
    }

    pub fn from_topleft_size(topleft: Point, size: Size) -> Rect {
        Rect {
            x: topleft.x,
            y: topleft.y,
            w: size.horizontal,
            h: size.vertical
        }
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn width(&self) -> f32 {
        self.w
    }

    pub fn height(&self) -> f32 {
        self.h
    }

    pub fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }

    pub fn clamp_positive(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.w.max(0.0),
            h: self.h.max(0.0)
        }
    }

    pub fn grow_by(&self, size: SizeRect) -> Rect {
        Rect::from_lrtb(self.left() - size.left, self.right() + size.right, self.top() - size.top, self.bottom() + size.bottom)
    }

    pub fn shrink_by(&self, size: SizeRect) -> Rect {
        Rect::from_lrtb(self.left() + size.left, self.right() - size.right, self.top() + size.top, self.bottom() - size.bottom)
    }
}

impl From<Rect> for tiny_skia::Rect {
    fn from(value: Rect) -> Self {
        tiny_skia::Rect::from_xywh(value.x, value.y, value.w, value.h).unwrap()
    }
}

impl From<Rect> for kurbo::Rect {
    fn from(value: Rect) -> Self {
        kurbo::Rect::new(value.left() as f64, value.top() as f64, value.right() as f64, value.bottom() as f64)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SizeRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32
}


impl SizeRect {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> SizeRect {
        SizeRect { left, right, top, bottom }
    }

    pub fn from_axis(axis: Axis, start: f32, end: f32) -> SizeRect {
        match axis {
            Axis::Horizontal => SizeRect::new(start, end, 0.0, 0.0),
            Axis::Vertical => SizeRect::new(0.0, 0.0, start, end),
        }
    }

    pub fn from_axes(horizontal: f32, vertical: f32) -> SizeRect {
        SizeRect::new(horizontal, horizontal, vertical, vertical)
    }

    pub fn from_border(size: f32) -> SizeRect {
        SizeRect::new(size, size, size, size)
    }

    pub fn sum_axis(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.left + self.right,
            Axis::Vertical => self.top + self.bottom
        }
    }

    pub fn sum_axes(&self) -> Size {
        Size {
            horizontal: self.left + self.right,
            vertical: self.top + self.bottom
        }
    }
}

impl From<f32> for SizeRect {
    fn from(value: f32) -> Self {
        SizeRect::new(value, value, value, value)
    }
}

impl From<(f32, f32)> for SizeRect {
    fn from(value: (f32, f32)) -> Self {
        SizeRect::new(value.0, value.0, value.1, value.1)
    }
}

impl From<(f32, f32, f32, f32)> for SizeRect {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        SizeRect::new(value.0, value.1, value.2, value.3)
    }
}

impl Add for SizeRect {
    type Output = SizeRect;

    fn add(self, rhs: Self) -> Self::Output {
        SizeRect {
            left: self.left + rhs.left,
            right: self.right + rhs.right,
            top: self.top + rhs.top,
            bottom: self.bottom + rhs.bottom,
        }
    }
}

impl Sub for SizeRect {
    type Output = SizeRect;

    fn sub(self, rhs: Self) -> Self::Output {
        SizeRect {
            left: self.left - rhs.left,
            right: self.right - rhs.right,
            top: self.top - rhs.top,
            bottom: self.bottom - rhs.bottom,
        }
    }
}
