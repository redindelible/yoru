use std::cell::Cell;
use std::ops::IndexMut;
use crate::{BoxLayout, Changed, ComputedLayout, Element, Layout, LayoutInput, RenderContext, Widget};

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