use crate::{BoxLayout, Color, ComputedLayout, Direction, Element, Justify, Layout, LayoutInput, LayoutStyle, RenderContext, Sizing};
use crate::math::Axis;
use crate::widgets::Widget;

fn to_tiny_skia_path<S: kurbo::Shape>(shape: S) -> tiny_skia::Path {
    use kurbo::Point;

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


pub struct Div<A> {
    layout_cache: BoxLayout<A>,
    children: Vec<Element<A>>,

    border_color: Option<Color>,
    background_color: Option<Color>,
}

impl<A> Div<A> {
    pub fn new() -> Div<A> {
        Div {
            layout_cache: BoxLayout::new(LayoutStyle {
                border_size: 2.0,
                padding: 2.0.into(),
                margin: 1.0.into(),
                width: Sizing::Fit,
                height: Sizing::Fit,
                main_axis: Axis::Vertical,
                main_direction: Direction::Positive,
                main_justify: Justify::Min,
                cross_justify: Justify::Min
            }),
            children: Vec::new(),
            border_color: Some(Color::BLACK),
            background_color: None
        }
    }

    pub fn add_child(&mut self, element: impl Into<Element<A>>) {
        let element = element.into();
        self.layout_cache.invalidate();
        self.children.push(element);
    }

    pub fn set_background_color(&mut self, color: impl Into<Option<Color>>) {
        self.background_color = color.into();
    }
}

impl<A: 'static> From<Div<A>> for Element<A> {
    fn from(value: Div<A>) -> Self {
        Element::new(value)
    }
}

impl<A> Widget<A> for Div<A> {
    fn layout_cache(&self) -> &BoxLayout<A> { &self.layout_cache }
    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A> { &mut self.layout_cache }

    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.layout_cache.compute_layout_with_children(input, &mut self.children)
    }

    fn update_model(&mut self, model: &mut A) {
        // todo the caching needs to be tracked somehow
        //   similar to layout, but the current doesn't need to be rerun if a child is out of date
        for child in &mut self.children {
            child.update_model(model);
        }
    }

    fn draw(&mut self, context: &mut RenderContext, layout: &Layout) {
        let border_size = self.layout_cache.attrs().border_size * layout.scale_factor;
        if let Some(border_color) = self.border_color {
            if border_size > 0.0 {
                let border_box = layout.border_box;
                let path = to_tiny_skia_path(kurbo::Rect::from(border_box));
                let mut stroke = tiny_skia::Stroke::default();
                stroke.width = border_size;
                let mut paint = tiny_skia::Paint::default();
                paint.set_color(border_color.into());
                context.canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
            }
        }

        if let Some(background) = self.background_color {
            let padding_box = layout.padding_box;

            let mut paint = tiny_skia::Paint::default();
            paint.set_color(background.into());
            context.canvas.fill_rect(padding_box.into(), &paint, tiny_skia::Transform::identity(), None);
        }

        for child in &mut self.children {
            child.draw(context);
        }
    }
}


#[macro_export]
macro_rules! div {
    (width=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.layout_cache_mut().set_width($e);
        div
    }};
    (height=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.layout_cache_mut().set_height($e);
        div
    }};
    (margin=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.layout_cache_mut().set_margin($e);
        div
    }};
    (padding=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.layout_cache_mut().set_padding($e);
        div
    }};
    (border=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.props_mut().set_border($e);
        div
    }};
    (background=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.set_background_color($e);
        div
    }};
    ([$($item:expr),* $(,)?]) => {{
        let mut div = $crate::Div::new();
        $(
            div.add_child($item);
        )*
        div
    }};
    () => {{ $crate::Div::new() }};
}