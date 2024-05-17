use std::cell::{Cell, RefCell};
use std::ops::{IndexMut};
use std::rc::{Rc, Weak};
use kurbo::{Point, Shape};
use crate::drawing::Application;

mod drawing;
mod element;
mod style;
mod math;

use tiny_skia::{PixmapMut};
use element::{Element, LayoutCache, ContentInfo};
use crate::element::{Layout, Root};
use crate::math::Axis;
use crate::style::{Color, Direction, Justify, LayoutStyle, Sizing};

pub struct RenderContext<'a> {
    canvas: PixmapMut<'a>,
    transform: tiny_skia::Transform
}

// struct Layout {
//     top_left: (f32, f32),
//     margin_size: (f32, f32)
// }

pub trait Widget<A> {
    fn layout_cache(&self) -> &LayoutCache<A>;
    fn layout_cache_mut(&mut self) -> &mut LayoutCache<A>;
    fn intrinsic_size(&self) -> ContentInfo;

    fn update_model(&mut self, model: &mut A);

    fn update_layout(&mut self);

    // fn update(&mut self, layout: &Layout);

    // fn min_content_size(&self) -> BoxSize;

    fn draw(&mut self, context: &mut RenderContext, layout: &Layout);

    // fn border_box(&self, margin_box: Rect) -> Rect {
    //     let attrs = self.props().attrs();
    //
    //     let left = margin_box.left() + attrs.margin.left + attrs.border.thickness / 2.0;
    //     let right = left.max(margin_box.right() - attrs.margin.right - attrs.border.thickness / 2.0);
    //     let top = margin_box.top() + attrs.margin.top + attrs.border.thickness / 2.0;
    //     let bottom = top.max(margin_box.bottom() - attrs.margin.bottom - attrs.border.thickness / 2.0);
    //
    //     Rect::from_ltrb(left, top, right, bottom).unwrap()
    // }
    //
    // fn padding_box(&self, margin_box: Rect) -> Rect {
    //     let attrs = self.props().attrs();
    //
    //     let left = margin_box.left() + attrs.margin.left + attrs.border.thickness;
    //     let right = left.max(margin_box.right() - attrs.margin.right - attrs.border.thickness);
    //     let top = margin_box.top() + attrs.margin.top + attrs.border.thickness;
    //     let bottom = top.max(margin_box.bottom() - attrs.margin.bottom - attrs.border.thickness);
    //
    //     Rect::from_ltrb(left, top, right, bottom).unwrap()
    // }
    //
    // fn content_box(&self, margin_box: Rect) -> Rect {
    //     let attrs = self.props().attrs();
    //
    //     let left = margin_box.left() + attrs.margin.left + attrs.border.thickness + attrs.padding.left;
    //     let right = left.max(margin_box.right() - attrs.margin.right - attrs.border.thickness - attrs.padding.right);
    //     let top = margin_box.top() + attrs.margin.top + attrs.border.thickness + attrs.padding.top;
    //     let bottom = top.max(margin_box.bottom() - attrs.margin.bottom - attrs.border.thickness - attrs.padding.bottom);
    //
    //     Rect::from_ltrb(left, top, right, bottom).unwrap()
    // }
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


// #[derive(Debug)]
struct Div<A> {
    layout_cache: LayoutCache<A>,
    children: Vec<Element<A>>,

    border_color: Option<Color>,
    background_color: Option<Color>,
}

impl<A> Div<A> {
    fn new() -> Div<A> {
        Div {
            layout_cache: LayoutCache::new(LayoutStyle {
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

    fn add_child(&mut self, element: impl Into<Element<A>>) {
        let element = element.into();
        self.layout_cache.invalidate();
        self.children.push(element);
    }

    fn set_background_color(&mut self, color: impl Into<Option<Color>>) {
        self.background_color = color.into();
    }
}

impl<A: 'static> From<Div<A>> for Element<A> {
    fn from(value: Div<A>) -> Self {
        Element::new(value)
        // unsafe {
        //     Element(Box::new(std::mem::transmute::<Div<A>, Div<A>>(value)))
        // }
    }
}

impl<A> Widget<A> for Div<A> {
    fn layout_cache(&self) -> &LayoutCache<A> { &self.layout_cache }
    fn layout_cache_mut(&mut self) -> &mut LayoutCache<A> { &mut self.layout_cache }

    fn intrinsic_size(&self) -> ContentInfo {
        self.layout_cache.get_intrinsic_size(&self.children)
    }

    fn update_model(&mut self, model: &mut A) {
        // todo the caching needs to be tracked somehow
        //   similar to layout, but the current doesn't need to be rerun if a child is out of date
        // for child in &mut self.children {
        //     child.
        // }
    }

    fn update_layout(&mut self) {
        self.layout_cache.update_layout(&self.children);
        for child in &mut self.children {
            child.update_layout();
        }
    }

    fn draw(&mut self, mut context: &mut RenderContext, layout: &Layout) {
        let border = self.layout_cache.attrs().border_size;
        if let Some(border_color) = self.border_color {
            if border > 0.0 {
                let border_box = layout.border_box;
                let path = to_tiny_skia_path(kurbo::Rect::from(border_box));
                let mut stroke = tiny_skia::Stroke::default();
                stroke.width = border;
                let mut paint = tiny_skia::Paint::default();
                paint.set_color(border_color.into());
                context.canvas.stroke_path(&path, &paint, &stroke, context.transform, None);
            }
        }

        if let Some(background) = self.background_color {
            let padding_box = layout.padding_box;

            let mut paint = tiny_skia::Paint::default();
            paint.set_color(background.into());
            context.canvas.fill_rect(padding_box.into(), &paint, context.transform, None);
        }

        dbg!(layout);

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
        div.layout_cache_mut().set_width($e);
        div
    }};
    (height=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.layout_cache_mut().set_height($e);
        div
    }};
    (margin=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.layout_cache_mut().set_margin($e);
        div
    }};
    (padding=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.layout_cache_mut().set_padding($e);
        div
    }};
    (border=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.props_mut().set_border($e);
        div
    }};
    (background=$e:expr $(, $($rest:tt)*)?) => {{
        let mut div = div!($( $($rest)* )?);
        div.set_background_color($e);
        div
    }};
    ([$($item:expr),*]) => {{
        let mut div = Div::new();
        $(
            div.add_child($item);
        )*
        div
    }};
    () => {{ Div::new() }};
}


thread_local! {
    static TRACKER: Cell<Option<Rc<Cell<bool>>>> = const { Cell::new(None) };
}

struct Signal<T> {
    value: T,
    trackers: RefCell<Vec<Weak<Cell<bool>>>>
}

impl<T> Signal<T> {
    fn new(value: T) -> Signal<T> {
        Signal {
            value,
            trackers: RefCell::new(vec![])
        }
    }

    fn get_untracked(&self) -> &T {
        &self.value
    }

    fn set_untracked(&mut self, value: T) {
        self.value = value;
    }

    fn get(&self) -> &T {
        if let Some(tracker) = TRACKER.take() {
            self.trackers.borrow_mut().push(Rc::downgrade(&tracker));
            TRACKER.set(Some(tracker));
        }
        &self.value
    }

    fn set(&mut self, value: T) {
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

struct Changed {
    dirty: Rc<Cell<bool>>
}

impl Changed {
    fn untracked() -> Changed {
        Changed { dirty: Rc::new(Cell::new(false)) }
    }

    fn is_changed(&self) -> bool {
        self.dirty.get()
    }

    fn reset(&self) {
        self.dirty.set(false);
    }

    fn run_and_track<T>(f: impl FnOnce() -> T) -> (Changed, T) {
        let old = TRACKER.replace(Some(Rc::new(Cell::new(false))));
        let value = f();
        let new = TRACKER.replace(old).unwrap();
        (Changed { dirty: new }, value)
    }
}

struct Select<A, S, O> {
    dirty: Changed,
    cached: Cell<S>,

    options: O,
    selector: Box<dyn Fn(&mut A) -> S>,
}

impl<A, S, O> Select<A, S, O> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    fn new(starting: S, options: O, selector: impl (Fn(&mut A) -> S) + 'static) -> Self {
        Select {
            dirty: Changed::untracked(),
            cached: Cell::new(starting),
            options,
            selector: Box::new(selector),
        }
    }

    fn element(&self) -> &Element<A> {
        &self.options[self.cached.get()]
    }

    fn element_mut(&mut self) -> &mut Element<A> {
        &mut self.options[self.cached.get()]
    }
}

impl<A: 'static, S, O> From<Select<A, S, O>> for Element<A> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    fn from(value: Select<A, S, O>) -> Self {
        Element::new(value)
    }
}

impl<A, S, O> Widget<A> for Select<A, S, O> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    fn layout_cache(&self) -> &LayoutCache<A> {
        self.element().props()
    }

    fn layout_cache_mut(&mut self) -> &mut LayoutCache<A> {
        self.element_mut().props_mut()
    }

    fn intrinsic_size(&self) -> ContentInfo {
        self.element().intrinsic_size()
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

    fn update_layout(&mut self) {
        self.element_mut().update_layout()
    }

    fn draw(&mut self, context: &mut RenderContext, _layout: &Layout) {
        self.element_mut().draw(context)
    }
}

// struct Do<F> {
//     f: F
// }

// impl<A, F> ViewWidget<A> for Do<F> where F: Fn(&mut A) + 'static {
//     fn click(&self, app: &mut A) {
//         (self.f)(app)
//     }
// }

// trait ViewWidget<A> {
//     fn click(&self, app: &mut A);
// }
//
// struct View<A>(Box<dyn ViewWidget<A>>);
//
// impl<A> View<A> {
//     fn new<T: ViewWidget<A> + 'static>(value: T) -> Self {
//         View(Box::new(value))
//     }
// }


// struct Project<F, A1, A2> {
//     f: F,
//     view: View<A2>,
//     _phantom: PhantomData<A1>
// }
//
// impl<A1, A2, F> Project<F, A1, A2> where F: Fn(&mut A1) -> &mut A2 {
//     fn new(view: View<A2>, projection: F) -> Project<F, A1, A2> {
//         Project { f: projection, view, _phantom: PhantomData }
//     }
// }
//
// impl<A1, A2, F> ViewWidget<A1> for Project<F, A1, A2> where F: Fn(&mut A1) -> &mut A2 {
//     fn click(&self, app: &mut A1) {
//         self.view.0.click((self.f)(app))
//     }
// }

// struct ExampleAppState {
//     num: Signal<u8>,
// }
//
// impl ExampleAppState {
//     fn new() -> Self {
//         let mut num = Signal::new(0);
//         ExampleAppState {
//             num,
//         }
//     }
//
//     fn create_view() -> View<Self> {
//         View::new(Select::new(0, vec![
//             View::new(Do {
//                 f: |app: &mut ExampleAppState| {
//                     println!("Clicked");
//                 }
//             })
//         ], |app| {
//             0
//         }))
//     }
// }


fn main() {
    // let mut example_app = ExampleAppState::new();
    // let mut view = ExampleAppState::create_view();
    // given(&mut view, &mut example_app);

    let mut model = 7;

    let mut b: Element<i32> = div!(width=Sizing::Fit, margin=10.0, background=Color::LIGHT_GRAY, [
        div!(width=Sizing::Expand, height=Sizing::Fixed(10.0))
    ]).into();

    // b.update_model(&mut model);
    // b.update(Rect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap());
    //
    Application::new(model, Root::new(b)).run();
}
