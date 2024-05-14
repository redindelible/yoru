use std::ops::Add;
use tiny_skia::Rect;
use crate::{RenderContext, Widget};

pub use props::{ElementProperties};
use props::CalculatedLayout;
use crate::attrs::{ElementAttrs, Size};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoxSize {
    pub vert: f32,
    pub horiz: f32
}

impl Add for BoxSize {
    type Output = BoxSize;

    fn add(self, rhs: Self) -> Self::Output {
        BoxSize { vert: self.vert + rhs.vert, horiz: self.horiz + rhs.horiz }
    }
}

#[derive(Debug)]
pub struct Element(Box<dyn Widget>);

impl Element {
    pub fn update(&mut self, margin_box: Rect) {
        let calculated = self.0.props().get_calculated();
        if !calculated.is_some_and(|cached| cached.margin_box == margin_box) {
            self.0.props_mut().set_calculated(CalculatedLayout { margin_box });
            self.0.update(margin_box);
        }
    }

    pub fn min_margin_box_size(&self) -> BoxSize {
        // todo cache this, and possibly cache min_content_size separately
        let mut min_content_size = self.0.min_content_size();
        let attrs = self.attrs();
        match attrs.width {
            Size::Fit => (),
            Size::Fixed(amount) => {
                min_content_size.horiz = amount;
            }
            Size::Expand => ()  // todo should this be zero?
        }
        match attrs.height {
            Size::Fit => (),
            Size::Fixed(amount) => {
                min_content_size.vert = amount;
            }
            Size::Expand => ()  // todo should this be zero?
        }
        return attrs.margin.size() + attrs.border.size() + attrs.padding.size() + min_content_size;
    }

    pub fn attrs(&self) -> &ElementAttrs {
        self.0.props().attrs()
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        let layout = self.0.props().get_calculated().unwrap();  // todo avoid panics, somehow
        self.0.draw(context, layout.margin_box);
    }
}

pub trait IntoElement {
    fn into_element(self) -> Element;
}

impl<W> IntoElement for W where W: Widget + 'static {
    fn into_element(self) -> Element {
        Element(Box::new(self))
    }
}

impl IntoElement for Element {
    fn into_element(self) -> Element {
        self
    }
}

mod props {
    use tiny_skia::Rect;
    use crate::attrs::{Border, ElementAttrs, Margin, Padding, Size};

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub(super) struct CalculatedLayout {
        pub margin_box: Rect
    }

    #[derive(Debug)]
    pub struct ElementProperties {
        attrs: ElementAttrs,
        calculated_layout: Option<CalculatedLayout>,
    }

    impl ElementProperties {
        pub fn new() -> ElementProperties {
            ElementProperties {
                attrs: ElementAttrs::default(),
                calculated_layout: None,
            }
        }

        pub fn attrs(&self) -> &ElementAttrs {
            &self.attrs
        }

        pub fn invalidate(&mut self) {
            self.calculated_layout = None;
        }

        pub(super) fn set_calculated(&mut self, layout: CalculatedLayout) {
            self.calculated_layout = Some(layout);
        }

        pub(super) fn get_calculated(&self) -> Option<CalculatedLayout> {
            self.calculated_layout
        }

        pub fn set_width(&mut self, width: Size) {
            self.invalidate();
            self.attrs.width = width;
        }

        pub fn set_height(&mut self, height: Size) {
            self.invalidate();
            self.attrs.height = height;
        }

        pub fn set_margin(&mut self, margin: impl Into<Margin>) {
            self.invalidate();
            self.attrs.margin = margin.into();
        }

        pub fn set_padding(&mut self, padding: impl Into<Padding>) {
            self.invalidate();
            self.attrs.padding = padding.into();
        }

        pub fn set_border(&mut self, border: impl Into<Border>) {
            self.invalidate();
            self.attrs.border = border.into();
        }
    }
}
