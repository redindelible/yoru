use std::cell::{Cell, RefCell};
use std::ops::IndexMut;
use std::rc::{Rc, Weak};

use tiny_skia::PixmapMut;

mod app;
mod element;
mod style;
pub mod math;
mod layout;
pub mod widgets;

pub use crate::element::{Element, Root};
pub use crate::app::Application;
pub use crate::layout::{BoxLayout, LayoutInput, ComputedLayout, Layout};
pub use crate::style::{LayoutStyle, Sizing, Justify, Direction, Color};
pub use crate::widgets::{Widget, Div, Label};

pub struct RenderContext<'a> {
    pub canvas: PixmapMut<'a>,
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
