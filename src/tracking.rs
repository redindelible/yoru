use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
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


pub struct Derived<A, V> {
    value: V,
    changed: Changed,
    compute: Box<dyn Fn(&mut A) -> V>
}

impl<A, V> Derived<A, V> {
    pub fn new_with_initial(initial: V, compute: impl (Fn(&mut A) -> V) + 'static) -> Derived<A, V> {
        Derived {
            value: initial,
            changed: Changed::untracked(true),
            compute: Box::new(compute)
        }
    }
}

impl<A, V> Derived<A, V> where V: Default {
    pub fn new(compute: impl (Fn(&mut A) -> V) + 'static) -> Derived<A, V> {
        Derived {
            value: V::default(),
            changed: Changed::untracked(true),
            compute: Box::new(compute)
        }
    }

    pub fn maybe_update(&mut self, model: &mut A) -> Option<&V> {
        if self.changed.is_changed() {
            self.value = (self.compute)(model);
            Some(&self.value)
        } else {
            None
        }
    }
}