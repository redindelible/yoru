use crate::{Element, layout, Layout, math, RenderContext};
use crate::interact::{Interaction, InteractSet};
use crate::layout::{PrelayoutInput, LayoutCharacteristics, LayoutInput};
use crate::math::Axis;
use crate::style::{LayoutStyle, ContainerLayoutStyle, Justify, Sizing, Direction, Color};
use crate::tracking::{Computed, Computed2, ReadableSignal, TrackedVec};
use crate::widgets::Widget;


// todo move somewhere reasonable
pub fn to_tiny_skia_path<S: kurbo::Shape>(shape: S) -> tiny_skia::Path {
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
    style: ContainerLayoutStyle,
    children: TrackedVec<Element<A>>,

    update_cache: Computed<()>,
    prelayout_cache: Computed2<PrelayoutInput, math::Size>,
    layout_cache: Computed2<LayoutInput, Layout>,
    interactions_cache: Computed<InteractSet>,

    border_color: Option<Color>,
    background_color: Option<Color>,
}

impl<A> Div<A> {
    pub fn new() -> Div<A> {
        Div {
            style: ContainerLayoutStyle {
                layout_style: LayoutStyle {
                    border_size: 2.0,
                    padding: 2.0.into(),
                    margin: 1.0.into(),
                    width: Sizing::Fit,
                    height: Sizing::Fit
                },
                main_axis: Axis::Vertical,
                main_direction: Direction::Positive,
                main_justify: Justify::Min,
                cross_justify: Justify::Min
            },
            children: TrackedVec::new(),
            update_cache: Computed::new(),
            prelayout_cache: Computed2::new(),
            layout_cache: Computed2::new(),
            interactions_cache: Computed::new(),
            border_color: Some(Color::BLACK),
            background_color: None
        }
    }

    pub fn add_child(&mut self, element: impl Into<Element<A>>) {
        self.children.push(element.into());
    }

    pub fn set_width(&mut self, width: Sizing) {
        self.style.layout_style.width = width;
    }

    pub fn set_height(&mut self, height: Sizing) {
        self.style.layout_style.height = height;
    }

    pub fn set_margin(&mut self, margin: math::SizeRect) {
        self.style.layout_style.margin = margin;
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
    fn update(&self, model: &mut A) {
        self.update_cache.maybe_update(|| {
            self.children.with(|children| {
                for child in children {
                    child.update(model);
                }
            });
        });
        self.update_cache.track();
    }

    fn prelayout(&self, input: PrelayoutInput) -> LayoutCharacteristics {
        self.prelayout_cache.maybe_update(input, |&input| {
            let characteristics = self.children.with(|items| layout::container::do_prelayout(&self.style, input, items));
            characteristics.min_size
        });
        LayoutCharacteristics {
            layout_style: &self.style.layout_style,
            min_size: self.prelayout_cache.get_untracked()
        }
    }

    fn layout(&self, input: LayoutInput) {
        self.layout_cache.maybe_update(input, |&input| {
            self.prelayout_cache.track();
            self.children.with(|children| {
                let children_layouts = layout::container::do_layout(&self.style, input, children);
                for (child, child_layout) in children.iter().zip(children_layouts) {
                    child.layout(child_layout);
                }
            });
            Layout::from_layout_input(&self.style.layout_style, input)
        });

        self.layout_cache.track()
    }

    fn interactions(&self) -> InteractSet {
        self.interactions_cache.maybe_update(|| {
            let mut set = InteractSet::default();
            self.children.with(|children| {
                for child in children {
                    set = set | child.interactions();
                }
            });
            set
        });
        self.interactions_cache.get()
    }

    fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A) {
        if self.interactions_cache.get_untracked().accepts(interaction) {
            self.children.with_mut_untracked(|children| {
                for child in children.iter_mut() {
                    child.handle_interaction(interaction, model)
                }
            });
        }
    }

    fn draw(&mut self, context: &mut RenderContext) {
        let layout = self.layout_cache.get_untracked();
        let border_size = self.style.layout_style.border_size * layout.scale_factor;
        if let Some(border_color) = self.border_color {
            if border_size > 0.0 {
                let border_box = layout.half_border_box;
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

        self.children.with_mut_untracked(|children| {
            for child in children {
                child.draw(context);
            }
        })
    }
}


#[macro_export]
macro_rules! div {
    (width=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.set_width(($e).into());
        div
    }};
    (height=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.set_height(($e).into());
        div
    }};
    (margin=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
        let mut div = div!($( $($rest)* )?);
        div.set_margin(($e).into());
        div
    }};
    (background=$e:expr $(, $($rest:tt)*)?) => {{
        use $crate::Widget;
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
