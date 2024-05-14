use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use kurbo::{Point, Shape};
use crate::drawing::Application;

mod drawing;
mod element;
mod attrs;

use tiny_skia::{PixmapMut, Rect};
use attrs::Size;
use element::{Element, IntoElement, ElementProperties};
use crate::attrs::{Border, Color};
use crate::element::BoxSize;

pub struct RenderContext<'a> {
    canvas: PixmapMut<'a>,
    transform: tiny_skia::Transform
}

pub trait Widget: Debug {
    fn props(&self) -> &ElementProperties;
    fn props_mut(&mut self) -> &mut ElementProperties;

    fn update(&mut self, margin_box: Rect);

    fn min_content_size(&self) -> BoxSize;

    fn draw(&mut self, context: &mut RenderContext, margin_box: Rect);

    fn border_box(&self, margin_box: Rect) -> Rect {
        let attrs = self.props().attrs();

        let left = margin_box.left() + attrs.margin.left + attrs.border.thickness / 2.0;
        let right = left.max(margin_box.right() - attrs.margin.right - attrs.border.thickness / 2.0);
        let top = margin_box.top() + attrs.margin.top + attrs.border.thickness / 2.0;
        let bottom = top.max(margin_box.bottom() - attrs.margin.bottom - attrs.border.thickness / 2.0);

        Rect::from_ltrb(left, top, right, bottom).unwrap()
    }

    fn content_box(&self, margin_box: Rect) -> Rect {
        let attrs = self.props().attrs();

        let left = margin_box.left() + attrs.margin.left + attrs.border.thickness + attrs.padding.left;
        let right = left.max(margin_box.right() - attrs.margin.right - attrs.border.thickness - attrs.padding.right);
        let top = margin_box.top() + attrs.margin.top + attrs.border.thickness + attrs.padding.top;
        let bottom = top.max(margin_box.bottom() - attrs.margin.bottom - attrs.border.thickness - attrs.padding.bottom);

        Rect::from_ltrb(left, top, right, bottom).unwrap()
    }
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

    fn update(&mut self, margin_box: Rect) {
        let content_box = self.content_box(margin_box);

        let mut used = 0.0;
        let mut split_ways = 0.0;
        for child in &self.children {
            match child.attrs().height {
                Size::Fit => {
                    used += child.min_margin_box_size().vert;
                }
                Size::Fixed(amount) => {
                    used += amount;
                }
                Size::Expand => {
                    used += child.min_margin_box_size().vert;
                    split_ways += 1.0;
                }
            }
        }

        let remaining = (content_box.height() - used).max(0.0);
        let y = content_box.y();
        for child in &mut self.children {
            let width = match child.attrs().width {
                Size::Fit => child.min_margin_box_size().horiz,
                Size::Fixed(amount) => amount,
                Size::Expand => content_box.width()
            };

            match child.attrs().height {
                Size::Fit => {
                    let space = Rect::from_xywh(content_box.x(), y, width, child.min_margin_box_size().vert).unwrap();
                    child.update(space);
                }
                Size::Fixed(amount) => {
                    let space = Rect::from_xywh(content_box.x(), y, width, amount).unwrap();
                    child.update(space);
                }
                Size::Expand => {
                    let space = Rect::from_xywh(content_box.x(), y, width, child.min_margin_box_size().vert + remaining / split_ways * 1.0).unwrap();
                    child.update(space);
                }
            }
        }
    }

    fn min_content_size(&self) -> BoxSize {
        let mut max_width = 0.0;
        let mut total_height = 0.0;

        for child in &self.children {
            let child_min_margin_box_size = child.min_margin_box_size();
            if child_min_margin_box_size.horiz > max_width {
                max_width = child_min_margin_box_size.horiz;
            }
            total_height += child_min_margin_box_size.vert;
        }
        BoxSize { vert: total_height, horiz: max_width }
    }

    fn draw(&mut self, mut context: &mut RenderContext, margin_box: Rect) {
        let border = self.props.attrs().border;

        if border.thickness > 0.0 {
            let border_box = self.border_box(margin_box);
            let path = to_tiny_skia_path(kurbo::Rect::new(
                border_box.left() as f64, border_box.top() as f64, border_box.right() as f64, border_box.bottom() as f64
            ));
            let mut stroke = tiny_skia::Stroke::default();
            stroke.width = border.thickness;
            let mut paint = tiny_skia::Paint::default();
            paint.set_color(border.color.into());
            context.canvas.stroke_path(&path, &paint, &stroke, context.transform, None);
        }

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
    (margin=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.props_mut().set_margin($e);
        div
    }};
    (padding=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.props_mut().set_padding($e);
        div
    }};
    (border=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.props_mut().set_border($e);
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
    let mut b= div!(width=Size::Fit, margin=0.0, border=Border::default().with_color(Color::GREEN), [
        div!(width=Size::Expand, height=Size::Fixed(10.0))
    ]).into_element();
    b.update(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap());

    Application::new(b).run();
}
