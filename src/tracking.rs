use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};


struct TrackedInner {
    dirty: Cell<bool>,
    parent: RefCell<Option<Weak<TrackedInner>>>
}

impl TrackedInner {
    fn set(&self) {
        self.dirty.set(true);

        let mut curr = self.parent.borrow().as_ref().and_then(Weak::upgrade);
        while let Some(tracked) = curr {
            tracked.dirty.set(true);

            curr = tracked.parent.borrow().as_ref().and_then(Weak::upgrade);
        }
    }
}


thread_local! {
    static TRACKER: Cell<Option<Rc<TrackedInner>>> = const { Cell::new(None) };
}

pub struct Signal<T> {
    value: T,
    trackers: RefCell<Vec<Weak<TrackedInner>>>,
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
                observer.set();
                true
            } else {
                false
            }
        });
    }
}

pub struct OnChangeToken(Weak<TrackedInner>);

pub struct Changed(Rc<TrackedInner>);

impl Changed {
    pub fn untracked(starts_dirty: bool) -> Changed {
        Changed(Rc::new(TrackedInner {
            dirty: Cell::new(starts_dirty),
            parent: RefCell::new(None)
        }))
    }

    pub fn run_and_track<T>(f: impl FnOnce() -> T) -> (Changed, T) {
        let old = TRACKER.replace(Some(Rc::new(TrackedInner {
            dirty: Cell::new(false),
            parent: RefCell::new(None)
        })));
        let value = f();
        let new = TRACKER.replace(old).unwrap();
        (Changed(new), value)
    }

    pub fn any_changed(dependencies: impl IntoIterator<Item=OnChangeToken>) -> Changed {
        let this = Changed(Rc::new(TrackedInner {
            dirty: Cell::new(false),
            parent: RefCell::new(None)
        }));

        for dependency in dependencies {
            if let Some(dependency) = Weak::upgrade(&dependency.0) {
                dependency.parent.replace(Some(Rc::downgrade(&this.0)));
            }
        }

        this
    }

    // pub fn add_dependency(&mut self, dependency: OnChangeToken) {
    //
    // }

    pub fn token(&self) -> OnChangeToken {
        OnChangeToken(Rc::downgrade(&self.0))
    }

    pub fn is_changed(&self) -> bool {
        self.0.dirty.get()
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

    pub fn get_uncached(&self) -> &V {
        &self.value
    }

    pub fn token(&self) -> OnChangeToken {
        self.changed.token()
    }

    pub fn maybe_update(&mut self, model: &mut A) -> Option<(V, &V)> {
        if self.changed.is_changed() {
            let (changed, value) = Changed::run_and_track(|| (self.compute)(model));
            let old_value = std::mem::replace(&mut self.value, value);
            self.changed = changed;
            Some((old_value, &self.value))
        } else {
            None
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
}