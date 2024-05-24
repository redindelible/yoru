use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};


struct Scope {
    observers: Rc<ObserverInner>
}

thread_local! {
    static SCOPE: Cell<Option<Scope>> = const { Cell::new(None) };
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Dirtiness {
    Clean,
    Dirty
}

impl Dirtiness {
    fn is_clean(&self) -> bool {
        *self == Dirtiness::Clean
    }

    fn is_dirty(&self) -> bool {
        *self == Dirtiness::Dirty
    }
}

struct ObservableInner {
    dependents: RefCell<Vec<Weak<ObserverInner>>>
}

impl ObservableInner {
    fn new() -> ObservableInner {
        ObservableInner {
            dependents: RefCell::new(Vec::new())
        }
    }

    fn register(&self) {
        SCOPE.with(|maybe_scope| {
            if let Some(scope) = maybe_scope.take() {
                let observer = Rc::downgrade(&scope.observers);
                self.dependents.borrow_mut().push(observer);
                maybe_scope.set(Some(scope));
            }
        });
    }

    fn try_register(&self) -> bool {
        SCOPE.with(|maybe_scope| {
            let Some(scope) = maybe_scope.take() else { return false; };
            let observer = Rc::downgrade(&scope.observers);
            self.dependents.borrow_mut().push(observer);
            maybe_scope.set(Some(scope));
            return true;
        })
    }

    pub fn trigger(&self) {
        let mut to_visit = Vec::new();

        fn mark_and_push_children(to_visit: &mut Vec<Rc<ObserverInner>>, observable: &ObservableInner) {
            for dependent in observable.dependents.borrow().iter() {
                if let Some(observer) = dependent.upgrade() {
                    if observer.is_dirty.get().is_clean() {
                        observer.is_dirty.set(Dirtiness::Dirty);
                        to_visit.push(observer);
                    }
                }
            }
        }

        mark_and_push_children(&mut to_visit, self);

        while let Some(next) = to_visit.pop() {
            mark_and_push_children(&mut to_visit, &next.as_observable);
        }
    }
}

struct ObserverInner {
    as_observable: ObservableInner,
    is_dirty: Cell<Dirtiness>,
}

impl ObserverInner {
    fn new(starting: Dirtiness) -> Rc<ObserverInner> {
        Rc::new(ObserverInner {
            as_observable: ObservableInner::new(),
            is_dirty: Cell::new(starting)
        })
    }

    fn run_and_track<T>(f: impl FnOnce() -> T) -> (Rc<ObserverInner>, T) {
        let observer = Rc::new(ObserverInner {
            as_observable: ObservableInner::new(),
            is_dirty: Cell::new(Dirtiness::Clean)   // todo make sure the dependents are all clean
        });
        let old_scope = SCOPE.replace(Some(Scope { observers: Rc::clone(&observer) }));
        let value = f();
        SCOPE.set(old_scope);
        (observer, value)
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty.get().is_dirty()
    }

    fn mark_dirty(&self) {
        let old_value = self.is_dirty.replace(Dirtiness::Dirty);
        if old_value.is_clean() {
            self.as_observable.trigger();
        }
    }
}


pub trait ReadableSignal<T> {
    fn as_read_signal(&self) -> ReadSignal<'_, T>;

    fn get_untracked(&self) -> &T;

    fn get(&self) -> &T;
}

pub struct ReadSignal<'a, T> {
    as_observable: Option<&'a ObservableInner>,
    value: &'a T
}

impl<'a, T> ReadSignal<'a, T> {
    pub fn from_value(value: &'a T) -> ReadSignal<'a, T> {
        ReadSignal {
            as_observable: None,
            value
        }
    }

    pub fn to_trigger(self) -> Trigger<'a> {
        Trigger {
            inner: self.as_observable,
        }
    }
}

impl<'a, T> Clone for ReadSignal<'a, T> {
    fn clone(&self) -> Self {
        ReadSignal { as_observable: self.as_observable, value: self.value }
    }
}

impl<'a, T> Copy for ReadSignal<'a, T> { }

impl<'a, T> ReadableSignal<T> for ReadSignal<'a, T> {
    fn as_read_signal(&self) -> ReadSignal<'_, T> {
        *self
    }

    fn get_untracked(&self) -> &T {
        self.value
    }

    fn get(&self) -> &T {
        if let Some(observable) = self.as_observable {
            observable.register();
        }
        self.value
    }
}

#[must_use]
#[derive(Copy, Clone)]
pub struct Trigger<'a> {
    inner: Option<&'a ObservableInner>
}

impl<'a> Trigger<'a> {
    pub fn track(&self) {
        if let Some(inner) = self.inner {
            inner.register()
        }
    }
}

impl<'a> ReadableSignal<()> for Trigger<'a> {
    fn as_read_signal(&self) -> ReadSignal<'_, ()> {
        ReadSignal {
            as_observable: self.inner,
            value: &()
        }
    }

    fn get_untracked(&self) -> &() {
        &()
    }

    fn get(&self) -> &() {
        self.track();
        &()
    }
}



struct SignalInner<T> {
    as_observable: ObservableInner,
    value: T,
}

impl<T> SignalInner<T> {
    fn new(value: T) -> SignalInner<T> {
        SignalInner {
            as_observable: ObservableInner::new(),
            value
        }
    }

    fn set_untracked(&mut self, value: T) {
        self.value = value;
    }

    fn set(&mut self, value: T) {
        self.as_observable.trigger();
        self.set_untracked(value);
    }
}

impl<T> ReadableSignal<T> for SignalInner<T> {
    fn as_read_signal(&self) -> ReadSignal<'_, T> {
        ReadSignal { as_observable: Some(&self.as_observable), value: &self.value }
    }

    fn get_untracked(&self) -> &T {
        &self.value
    }

    fn get(&self) -> &T {
        self.as_observable.register();
        &self.value
    }
}

pub struct RwSignal<T> {
    inner: SignalInner<T>
}

impl<T> RwSignal<T> {
    pub fn new(value: T) -> RwSignal<T> {
        RwSignal {
            inner: SignalInner::new(value)
        }
    }

    pub fn set_untracked(&mut self, value: T) {
        self.inner.set_untracked(value);
    }

    pub fn set(&mut self, value: T) {
        self.inner.set(value);
    }
}

impl<T> ReadableSignal<T> for RwSignal<T> {
    fn as_read_signal(&self) -> ReadSignal<'_, T> {
        self.inner.as_read_signal()
    }

    fn get_untracked(&self) -> &T {
        self.inner.get_untracked()
    }

    fn get(&self) -> &T {
        self.inner.get()
    }
}

pub struct Computed<V> {
    as_observer: Rc<ObserverInner>,
    value: V,
}

impl<V: Default> Computed<V> {
    pub fn new() -> Computed<V> {
        Computed::new_with_initial(V::default())
    }
}

impl<V> Computed<V> {
    pub fn new_with_initial(initial: V) -> Computed<V> {
        Computed {
            as_observer: ObserverInner::new(Dirtiness::Dirty),
            value: initial,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.as_observer.is_dirty()
    }

    pub fn invalidate(&self) {
        self.as_observer.mark_dirty();
    }

    pub fn maybe_update(&mut self, f: impl FnOnce(&V) -> V) -> Option<(V, &V)> {
        if self.as_observer.is_dirty() {
            let (observer, value) = ObserverInner::run_and_track(|| f(&self.value));
            let old_value = std::mem::replace(&mut self.value, value);
            self.as_observer = observer;
            Some((old_value, &self.value))
        } else {
            None
        }
    }
}

impl<T> ReadableSignal<T> for Computed<T> {
    fn as_read_signal(&self) -> ReadSignal<'_, T> {
        ReadSignal { as_observable: Some(&self.as_observer.as_observable), value: &self.value }
    }

    fn get_untracked(&self) -> &T {
        &self.value
    }

    fn get(&self) -> &T {
        self.as_observer.as_observable.register();
        self.get_untracked()
    }
}

pub struct Derived<A, V> {
    as_observer: Rc<ObserverInner>,
    value: V,
    compute: Box<dyn Fn(&mut A) -> V>
}

impl<A, V> Derived<A, V> where V: Default {
    pub fn new(compute: impl (Fn(&mut A) -> V) + 'static) -> Derived<A, V> {
        Derived::new_with_initial(V::default(), compute)
    }
}

impl<A, V> Derived<A, V> {
    pub fn new_with_initial(initial: V, compute: impl (Fn(&mut A) -> V) + 'static) -> Derived<A, V> {
        Derived {
            as_observer: ObserverInner::new(Dirtiness::Dirty),
            value: initial,
            compute: Box::new(compute)
        }
    }

    pub fn maybe_update(&mut self, model: &mut A) -> Option<(V, &V)> {
        if self.as_observer.is_dirty() {
            let (observer, value) = ObserverInner::run_and_track(|| (self.compute)(model));
            let old_value = std::mem::replace(&mut self.value, value);
            self.as_observer = observer;
            Some((old_value, &self.value))
        } else {
            None
        }
    }
}

impl<A, T> ReadableSignal<T> for Derived<A, T> {
    fn as_read_signal(&self) -> ReadSignal<'_, T> {
        ReadSignal { as_observable: Some(&self.as_observer.as_observable), value: &self.value }
    }

    fn get_untracked(&self) -> &T {
        &self.value
    }

    fn get(&self) -> &T {
        self.as_observer.as_observable.register();
        &self.value
    }
}
