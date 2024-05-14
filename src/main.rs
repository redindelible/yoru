use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use kurbo::{Point, Shape};
use crate::drawing::Application;

mod drawing;
mod element;
mod attrs;

use tiny_skia::{PixmapMut, Rect};
use attrs::Size;
use element::{Element, IntoElement, CalculatedLayout, ElementProperties};

pub struct RenderContext<'a> {
    canvas: PixmapMut<'a>
}

pub trait Widget: Debug {
    fn props(&self) -> &ElementProperties;
    fn props_mut(&mut self) -> &mut ElementProperties;

    fn update(&mut self);

    fn content_width(&self) -> f32;
    fn content_height(&self) -> f32;

    fn draw(&mut self, context: &mut RenderContext);
}


fn to_tiny_skia_path<S: Shape>(shape: S) -> tiny_skia::Path {
    let mut path_builder = tiny_skia::PathBuilder::new();
    for path_el in shape.path_elements(0.1) {
        match path_el {
            kurbo::PathEl::MoveTo(Point { x, y }) => {
                path_builder.move_to(x as f32, y as f32);
            }
            kurbo::PathEl::LineTo(Point { x, y }) => {
                path_builder.line_to(x as f32, y as f32);
            }
            kurbo::PathEl::QuadTo(Point { x: x1, y: y1 }, Point{ x, y }) => {
                path_builder.quad_to(x1 as f32, y1 as f32, x as f32, y as f32);
            }
            kurbo::PathEl::CurveTo(p1, p2, p) => {
                path_builder.cubic_to(p1.x as f32, p1.y as f32, p2.x as f32, p2.y as f32, p.x as f32, p.y as f32);
            }
            kurbo::PathEl::ClosePath => {
                path_builder.close();
            }
        }
    }
    path_builder.finish().unwrap()
}


#[derive(Debug)]
struct Div {
    props: ElementProperties,

    children: Vec<Element>,
}

impl Div {
    fn new() -> Div {
        Div {
            props: ElementProperties::new(),
            children: Vec::new()
        }
    }

    fn add_child(&mut self, element: impl IntoElement) {
        self.props.invalidate();
        self.children.push(element.into_element())
    }
}

impl Widget for Div {
    fn props(&self) -> &ElementProperties {
        &self.props
    }

    fn props_mut(&mut self) -> &mut ElementProperties {
        &mut self.props
    }

    fn update(&mut self) {
        let allocated = self.props.get_calculated().content_box;

        let mut used = 0.0;
        let mut split_ways = 0.0;
        for child in &self.children {
            match child.attrs().height {
                Size::Fit => {
                    used += child.content_height();
                }
                Size::Fixed(amount) => {
                    used += amount;
                }
                Size::Expand => {
                    split_ways += 1.0;
                }
            }
        }

        let remaining = (allocated.height() - used).max(0.0);
        let y = allocated.y();
        for child in &mut self.children {
            let width = match child.attrs().width {
                Size::Fit => child.content_width(),
                Size::Fixed(amount) => amount,
                Size::Expand => allocated.width()
            };

            match child.attrs().height {
                Size::Fit => {
                    let space = Rect::from_xywh(allocated.x(), y, width, child.content_height()).unwrap();
                    let layout = CalculatedLayout { content_box: space };
                    child.update_layout(layout);
                }
                Size::Fixed(amount) => {
                    let space = Rect::from_xywh(allocated.x(), y, width, amount).unwrap();
                    let layout = CalculatedLayout { content_box: space };
                    child.update_layout(layout);
                }
                Size::Expand => {
                    let space = Rect::from_xywh(allocated.x(), y, width, remaining / split_ways * 1.0).unwrap();
                    let layout = CalculatedLayout { content_box: space };
                    child.update_layout(layout);
                }
            }
        }
    }

    fn content_width(&self) -> f32 {
        match self.props.attrs().width {
            Size::Fixed(amount) => amount,
            Size::Fit | Size::Expand => self.children.iter().map(Element::content_width).max_by(f32::total_cmp).unwrap_or(0.0)
        }
    }

    fn content_height(&self) -> f32 {
        match self.props.attrs().height {
            Size::Fixed(amount) => amount,
            Size::Fit | Size::Expand => self.children.iter().map(Element::content_height).sum()
        }
    }

    fn draw(&mut self, mut context: &mut RenderContext) {
        let layout = self.props.get_calculated();

        let content_box = layout.content_box;
        let rect = kurbo::Rect::new(content_box.left() as f64, content_box.top() as f64, content_box.right() as f64, content_box.bottom() as f64);

        let path = to_tiny_skia_path(rect);
        let mut stroke = tiny_skia::Stroke::default();
        // stroke.width = 1.0;
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(tiny_skia::Color::WHITE);

        context.canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);

        for child in &mut self.children {
            child.draw(context);
        }
    }
}


macro_rules! div {
    // (class=$e:expr $(, $($rest:tt)*)?) => {{
    //     let mut div = div!($( $($rest)* )?);
    //     div.add_class($e);
    //     div
    // }};
    (width=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.props_mut().set_width($e);
        div
    }};
    (height=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.props_mut().set_height($e);
        div
    }};
    ([$($item:expr),*]) => {{
        let mut div = Div::new();
        $(
            div.add_child(IntoElement::into_element($item));
        )*
        div
    }};
    () => {{ Div::new() }};
}


fn main() {
    let mut b= div!(width=Size::Fit, [
        div!(width=Size::Expand, height=Size::Fixed(10.0))
    ]).into_element();
    b.update_layout(CalculatedLayout { content_box: Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap() });

    dbg!(&b);

    Application::new(b).run();
}
