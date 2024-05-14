// use crate::props::{CalculatedLayout, ElementAttrs};
use crate::{RenderContext, Widget};

pub use props::{CalculatedLayout, ElementAttrs, ElementProperties};

#[derive(Debug)]
pub struct Element(Box<dyn Widget>);

impl Element {
    pub fn update_layout(&mut self, layout: CalculatedLayout) {
        self.0.props_mut().set_calculated(layout);
        self.0.update();
    }

    pub fn content_width(&self) -> f32 {
        self.0.content_width()
    }

    pub fn content_height(&self) -> f32 {
        self.0.content_height()
    }

    pub fn attrs(&self) -> &ElementAttrs {
        self.0.props().attrs()
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        self.0.draw(context);
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
    use crate::attrs::Size;

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct CalculatedLayout {
        pub content_box: Rect
    }

    #[derive(Debug)]
    pub struct ElementAttrs {
        pub width: Size,
        pub height: Size,
    }

    impl Default for ElementAttrs {
        fn default() -> Self {
            ElementAttrs {
                width: Size::Fit,
                height: Size::Fit
            }
        }
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

        pub fn get_calculated(&self) -> CalculatedLayout {
            if let Some(layout) = self.calculated_layout {
                layout
            } else {
                CalculatedLayout { content_box: Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap() }
            }
        }

        pub fn set_width(&mut self, width: Size) {
            self.invalidate();
            self.attrs.width = width;
        }

        pub fn set_height(&mut self, height: Size) {
            self.invalidate();
            self.attrs.height = height;
        }
    }
}
