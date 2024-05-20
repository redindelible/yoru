use std::cell::{Cell, RefCell};
use std::ops::IndexMut;
use std::rc::{Rc, Weak};
use kurbo::{Point, Shape};
use tiny_skia::PixmapMut;

mod app;
mod element;
mod style;
pub mod math;
mod layout;

use crate::math::Axis;

pub use crate::element::{Element, Root};
pub use crate::app::Application;
pub use crate::layout::{BoxLayout, LayoutInput, ComputedLayout, Layout};
pub use crate::style::{LayoutStyle, Sizing, Justify, Direction, Color};

pub struct RenderContext<'a> {
    pub canvas: PixmapMut<'a>,
}

pub trait Widget<A> {
    fn layout_cache(&self) -> &BoxLayout<A>;
    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A>;
    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout;

    fn update_model(&mut self, model: &mut A);

    fn draw(&mut self, context: &mut RenderContext, layout: &Layout);
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


thread_local! {
    static FONTS: RefCell<cosmic_text::FontSystem> = RefCell::new(cosmic_text::FontSystem::new());
    static SWASH_CACHE: RefCell<cosmic_text::SwashCache> = RefCell::new(cosmic_text::SwashCache::new());
}


pub struct Label<A> {
    layout_cache: BoxLayout<A>,

    font_size: f32,

    changed: Changed,
    text: String,
    compute: Box<dyn Fn(&mut A) -> String>,

    sizing_buffer: cosmic_text::Buffer,
    buffer: cosmic_text::Buffer
}

impl<A> Label<A> {
    pub fn new(compute: impl (Fn(&mut A) -> String) + 'static) -> Label<A> {
        let font_size = 15.0;
        let default_metrics = cosmic_text::Metrics { font_size, line_height: font_size };

        let sizing_buffer = FONTS.with_borrow_mut(|fonts| {
            let mut buffer = cosmic_text::Buffer::new(fonts, default_metrics);
            buffer.set_size(fonts, f32::INFINITY, f32::INFINITY);
            buffer
        });

        Label {
            layout_cache: BoxLayout::new(LayoutStyle {
                border_size: 0.0,
                padding: 0.0.into(),
                margin: 0.0.into(),
                width: Sizing::Fit,
                height: Sizing::Fit,
                // todo make a ContainerLayoutCache so that leaf elements don't need this?
                main_axis: Axis::Vertical,
                main_direction: Direction::Positive,
                main_justify: Justify::Min,
                cross_justify: Justify::Min
            }),

            font_size,

            changed: Changed::untracked(true),
            text: String::new(),
            compute: Box::new(compute),

            sizing_buffer,
            buffer: FONTS.with_borrow_mut(|fonts| {
                cosmic_text::Buffer::new(fonts, default_metrics)
            })
        }
    }
}

impl<A> Widget<A> for Label<A> {
    fn layout_cache(&self) -> &BoxLayout<A> {
        &self.layout_cache
    }

    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A> {
        &mut self.layout_cache
    }

    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.layout_cache.compute_layout_leaf(input, |available_size, scale_factor| {
            FONTS.with_borrow_mut(|fonts| {
                self.sizing_buffer.set_metrics_and_size(
                    fonts,
                    cosmic_text::Metrics::new(self.font_size * scale_factor, self.font_size * scale_factor),
                    available_size.width(), available_size.height()
                );
                let max_width = self.sizing_buffer.layout_runs().map(|run| run.line_w).max_by(f32::total_cmp).unwrap_or(0.0);
                let total_height = self.sizing_buffer.layout_runs().len() as f32 * self.sizing_buffer.metrics().line_height;
                math::Size::new(max_width, total_height)
            })
        })
    }

    fn update_model(&mut self, model: &mut A) {
        if self.changed.is_changed() {
            let (changed, text) = Changed::run_and_track(|| (self.compute)(model));
            self.text = text;
            FONTS.with_borrow_mut(|fonts| {
                self.buffer.set_text(fonts, &self.text, cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced);
                self.sizing_buffer.set_text(fonts, &self.text, cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced);
            });

            self.layout_cache.invalidate();
            self.changed = changed;
        }
    }

    fn draw(&mut self, context: &mut RenderContext, layout: &Layout) {
        FONTS.with_borrow_mut(|fonts| {
            self.buffer.set_metrics_and_size(
                fonts,
                cosmic_text::Metrics::new(self.font_size * layout.scale_factor, self.font_size * layout.scale_factor),
                layout.content_box.width(), layout.content_box.height()
            );

            SWASH_CACHE.with_borrow_mut(|swash_cache| {
                let mut paint = tiny_skia::Paint::default();
                paint.set_color(Color::BLACK.into());
                let content_top_left = layout.content_box.top_left();

                for run in self.buffer.layout_runs() {
                    for glyph in run.glyphs {
                        let physical_glyph = glyph.physical((content_top_left.x, content_top_left.y), 1.0);

                        // todo first try get_image
                        // todo add with pixel fallback
                        if let Some(commands) = swash_cache.get_outline_commands(fonts, physical_glyph.cache_key) {
                            use cosmic_text::Command;

                            let x_off = content_top_left.x + glyph.x + glyph.x_offset;
                            let y_off = content_top_left.y + glyph.y_offset + run.line_y;

                            let mut path_builder = tiny_skia::PathBuilder::new();
                            for command in commands {
                                match command {
                                    Command::MoveTo(point) =>
                                        path_builder.move_to(point.x + x_off, -point.y + y_off),
                                    Command::LineTo(point) =>
                                        path_builder.line_to(point.x + x_off, -point.y + y_off),
                                    Command::CurveTo(p1, p2, p3) =>
                                        path_builder.cubic_to(p1.x + x_off, -p1.y + y_off, p2.x + x_off, -p2.y + y_off, p3.x + x_off, -p3.y + y_off),
                                    Command::QuadTo(p1, p2) =>
                                        path_builder.quad_to(p1.x + x_off, -p1.y + y_off, p2.x + x_off, -p2.y + y_off),
                                    Command::Close => path_builder.close()
                                }
                            }
                            if let Some(path) = path_builder.finish() {
                                context.canvas.fill_path(
                                    &path,
                                    &paint,
                                    tiny_skia::FillRule::EvenOdd,
                                    tiny_skia::Transform::identity(),
                                    None
                                )
                            }
                        }
                    }
                }
            });
        });
    }
}

impl<A: 'static> From<Label<A>> for Element<A> {
    fn from(value: Label<A>) -> Self {
        Element::new(value)
    }
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
    // (class=$e:expr $(, $($rest:tt)*)?) => {{
    //     let mut div = div!($( $($rest)* )?);
    //     div.add_class($e);
    //     div
    // }};
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


thread_local! {
    static TRACKER: Cell<Option<Rc<Cell<bool>>>> = const { Cell::new(None) };
}

pub struct Signal<T> {
    value: T,
    trackers: RefCell<Vec<Weak<Cell<bool>>>>
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Signal<T> {
        Signal {
            value,
            trackers: RefCell::new(vec![])
        }
    }

    pub fn get_untracked(&self) -> &T {
        &self.value
    }

    pub fn set_untracked(&mut self, value: T) {
        self.value = value;
    }

    pub fn get(&self) -> &T {
        if let Some(tracker) = TRACKER.take() {
            self.trackers.borrow_mut().push(Rc::downgrade(&tracker));
            TRACKER.set(Some(tracker));
        }
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.trackers.borrow_mut().retain(|observer| {
            if let Some(observer) = observer.upgrade() {
                observer.set(true);
                true
            } else {
                false
            }
        });
    }
}

pub struct Changed {
    dirty: Rc<Cell<bool>>
}

impl Changed {
    pub fn untracked(initial: bool) -> Changed {
        Changed { dirty: Rc::new(Cell::new(initial)) }
    }

    pub fn is_changed(&self) -> bool {
        self.dirty.get()
    }

    pub fn reset(&self) {
        self.dirty.set(false);
    }

    pub fn run_and_track<T>(f: impl FnOnce() -> T) -> (Changed, T) {
        let old = TRACKER.replace(Some(Rc::new(Cell::new(false))));
        let value = f();
        let new = TRACKER.replace(old).unwrap();
        (Changed { dirty: new }, value)
    }
}

pub struct Select<A, S, O> {
    dirty: Changed,
    cached: Cell<S>,

    options: O,
    selector: Box<dyn Fn(&mut A) -> S>,
}

impl<A, S, O> Select<A, S, O> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    pub fn new(starting: S, options: O, selector: impl (Fn(&mut A) -> S) + 'static) -> Self {
        Select {
            dirty: Changed::untracked(true),
            cached: Cell::new(starting),
            options,
            selector: Box::new(selector),
        }
    }

    pub fn element(&self) -> &Element<A> {
        &self.options[self.cached.get()]
    }

    pub fn element_mut(&mut self) -> &mut Element<A> {
        &mut self.options[self.cached.get()]
    }
}

impl<A: 'static, S, O> From<Select<A, S, O>> for Element<A> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    fn from(value: Select<A, S, O>) -> Self {
        Element::new(value)
    }
}

impl<A, S, O> Widget<A> for Select<A, S, O> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    fn layout_cache(&self) -> &BoxLayout<A> {
        self.element().props()
    }

    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A> {
        self.element_mut().props_mut()
    }

    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.element_mut().compute_layout(input)
    }

    fn update_model(&mut self, model: &mut A) {
        if self.dirty.is_changed() {
            let (dirty, index) = Changed::run_and_track(|| (self.selector)(model));
            let old_index = self.cached.replace(index);
            if let Some(parent) = self.options[old_index].props().remove_parent() {
                self.options[index].props().set_parent(&parent);
            }

            self.dirty = dirty;
        }
    }

    fn draw(&mut self, context: &mut RenderContext, _layout: &Layout) {
        self.element_mut().draw(context)
    }
}
