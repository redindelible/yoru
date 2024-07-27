use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
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
        observer.is_dirty.set(Dirtiness::Clean);
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
    fn get(&self) -> T;
    fn get_untracked(&self) -> T;
    fn track(&self);
}

pub trait WritableSignal<T> {
    fn set_untracked(&self, value: T);
    fn set(&self, value: T);

    fn update_untracked<O>(&self, f: impl FnOnce(&mut T) -> O) -> O;
    fn update<O>(&self, f: impl FnOnce(&mut T) -> O) -> O;
}

struct SignalInner<T> {
    as_observable: ObservableInner,
    value: RefCell<T>,
}

impl<T> SignalInner<T> {
    fn new(value: T) -> SignalInner<T> {
        SignalInner {
            as_observable: ObservableInner::new(),
            value: RefCell::new(value)
        }
    }
    //
    // fn get_mut(&mut self) -> &mut T {
    //     self.as_observable.trigger();
    //     self.as_observable.register();
    //     &mut self.value
    // }
    //
    // fn get_mut_for_read(&mut self) -> &mut T {
    //     self.as_observable.register();
    //     &mut self.value
    // }
    //
    // fn set_untracked(&mut self, value: T) {
    //     self.value = value;
    // }
    //
    // fn set(&mut self, value: T) {
    //     self.as_observable.trigger();
    //     self.set_untracked(value);
    // }
}

impl<T> ReadableSignal<T> for SignalInner<T> where T: Clone {
    fn get(&self) -> T {
        self.as_observable.register();
        self.value.borrow().clone()
    }

    fn get_untracked(&self) -> T {
        self.value.borrow().clone()
    }

    fn track(&self) {
        self.as_observable.register();
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

    pub fn with<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        let value = f(&*self.inner.value.borrow());
        self.inner.as_observable.register();
        value
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut *self.inner.value.borrow_mut());
        self.inner.as_observable.trigger();
    }

    // pub fn get_mut(&mut self) -> &mut T {
    //     self.inner.get_mut()
    // }
    //
    // pub fn get_mut_for_read(&mut self) -> &mut T {
    //     self.inner.get_mut_for_read()
    // }
    //
    // pub fn set_untracked(&mut self, value: T) {
    //     self.inner.set_untracked(value);
    // }
    //
    // pub fn set(&mut self, value: T) {
    //     self.inner.set(value);
    // }
}

impl<T> ReadableSignal<T> for RwSignal<T> where T: Clone {
    fn get(&self) -> T {
        self.inner.get()
    }

    fn get_untracked(&self) -> T {
        self.inner.get_untracked()
    }

    fn track(&self) {
        self.inner.as_observable.register();
    }
}

pub struct Computed<V> {
    as_observer: RefCell<Rc<ObserverInner>>,
    value: RefCell<V>,
}

impl<V: Default> Computed<V> {
    pub fn new() -> Computed<V> {
        Computed::new_with_initial(V::default())
    }
}

impl<V> Computed<V> {
    pub fn new_with_initial(initial: V) -> Computed<V> {
        Computed {
            as_observer: RefCell::new(ObserverInner::new(Dirtiness::Dirty)),
            value: RefCell::new(initial),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.as_observer.borrow().is_dirty()
    }

    pub fn invalidate(&self) {
        self.as_observer.borrow().mark_dirty();
    }

    pub fn maybe_update(&self, f: impl FnOnce() -> V) {
        if self.is_dirty() {
            self.as_observer.borrow().mark_dirty();
            let (observer, value) = ObserverInner::run_and_track(f);
            *self.value.borrow_mut() = value;
            *self.as_observer.borrow_mut() = observer;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn count_observers(&self) -> usize {
        Rc::weak_count(&*self.as_observer.borrow())
    }
}

impl<T> ReadableSignal<T> for Computed<T> where T: Clone {
    fn get(&self) -> T {
        self.as_observer.borrow().as_observable.register();
        self.get_untracked()
    }

    fn get_untracked(&self) -> T {
        self.value.borrow().clone()
    }

    fn track(&self) {
        self.as_observer.borrow().as_observable.register();
    }
}

pub struct Derived<A, V> {
    as_observer: RefCell<Rc<ObserverInner>>,
    value: RefCell<V>,
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
            as_observer: RefCell::new(ObserverInner::new(Dirtiness::Dirty)),
            value: RefCell::new(initial),
            compute: Box::new(compute)
        }
    }

    pub fn maybe_update(&self, model: &mut A) -> bool {
        if self.as_observer.borrow().is_dirty() {
            self.as_observer.borrow().mark_dirty();
            let (observer, value) = ObserverInner::run_and_track(|| (self.compute)(model));
            *self.value.borrow_mut() = value;
            *self.as_observer.borrow_mut() = observer;
            true
        } else {
            false
        }
    }
}

impl<A, T> ReadableSignal<T> for Derived<A, T> where T: Clone {
    fn get(&self) -> T {
        self.as_observer.borrow().as_observable.register();
        self.value.borrow().clone()
    }

    fn get_untracked(&self) -> T {
        self.value.borrow().clone()
    }

    fn track(&self) {
        self.as_observer.borrow().as_observable.register();
    }
}


pub struct Computed2<I, V> {
    as_observer: RefCell<Rc<ObserverInner>>,
    input: RefCell<I>,
    value: RefCell<V>,
    phantom: PhantomData<fn(I)>
}

impl<I, V> Computed2<I, V> where for<'a> &'a I: PartialEq {
    pub fn new() -> Computed2<I, V> where I: Default, V: Default {
        Computed2 {
            as_observer: RefCell::new(ObserverInner::new(Dirtiness::Dirty)),
            input: RefCell::new(I::default()),
            value: RefCell::new(V::default()),
            phantom: PhantomData
        }
    }

    pub fn new_with_initial(input: I, value: V) -> Computed2<I, V> {
        Computed2 {
            as_observer: RefCell::new(ObserverInner::new(Dirtiness::Dirty)),
            input: RefCell::new(input),
            value: RefCell::new(value),
            phantom: PhantomData
        }
    }

    pub fn maybe_update(&self, input: I, f: impl FnOnce(&I) -> V) {
        if self.as_observer.borrow().is_dirty() || &input != &*self.input.borrow() {
            let (observer, value) = ObserverInner::run_and_track(|| f(&input));
            self.as_observer.borrow().mark_dirty();
            *self.input.borrow_mut() = input;
            *self.as_observer.borrow_mut() = observer;
            *self.value.borrow_mut() = value;
        }
    }
}

impl<I, V> ReadableSignal<V> for Computed2<I, V> where V: Clone {
    fn get(&self) -> V {
        self.as_observer.borrow().as_observable.register();
        self.value.borrow().clone()
    }

    fn get_untracked(&self) -> V {
        self.value.borrow().clone()
    }

    fn track(&self) {
        self.as_observer.borrow().as_observable.register();
    }
}


pub struct TrackedVec<T> {
    inner: RwSignal<Vec<T>>
}

impl<T> TrackedVec<T> {
    pub fn new() -> TrackedVec<T> {
        TrackedVec {
            inner: RwSignal::new(Vec::new()),
        }
    }

    // pub fn maybe_for_each(&self, mut f: impl FnMut(&T)) {
    //     self.items.maybe_update(|| {
    //         self.inner.with(|items| {
    //             for item in items {
    //                 f(item);
    //             }
    //         });
    //     });
    // }

    pub fn with<O>(&self, f: impl FnOnce(&[T]) -> O) -> O {
        self.inner.with(|items| f(items))
    }

    pub fn with_mut_untracked<O>(&mut self, f: impl FnOnce(&mut [T]) -> O) -> O {
        f(self.inner.inner.value.borrow_mut().as_mut_slice())
    }

    pub fn push(&mut self, item: T) {
        self.inner.update(|items| items.push(item));
    }
}
