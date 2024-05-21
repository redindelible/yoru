use std::cell::{Cell, RefCell};
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
    pub fn untracked(starts_dirty: bool) -> Changed {
        Changed { dirty: Rc::new(Cell::new(starts_dirty)) }
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
