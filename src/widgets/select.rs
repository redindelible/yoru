use std::ops::IndexMut;

use crate::{Widget, RenderContext};
use crate::element::Element;
use crate::interact::{Interaction, InteractSet};
use crate::layout::{BoxLayout, ComputedLayout, Layout, LayoutInput};
use crate::tracking::{Derived, ReadableSignal, ReadSignal, Trigger};

pub struct Select<A, S, O> {
    selector: Derived<A, S>,
    options: O,
}

impl<A, S, O> Select<A, S, O> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    pub fn new(starting: S, options: O, selector: impl (Fn(&mut A) -> S) + 'static) -> Self {
        Select {
            options,
            selector: Derived::new_with_initial(starting, selector),
        }
    }

    pub fn element(&self) -> &Element<A> {
        &self.options[*self.selector.get_untracked()]
    }

    pub fn element_mut(&mut self) -> &mut Element<A> {
        &mut self.options[*self.selector.get_untracked()]
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

    fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A) {
        self.element_mut().handle_interaction(interaction, model)
    }

    fn update_model(&mut self, model: &mut A) -> Trigger {
        if let Some((old, new)) = self.selector.maybe_update(model) {
            if let Some(parent) = self.options[old].props().remove_parent() {
                self.options[*new].props().set_parent(parent);
            }
            self.layout_cache_mut().invalidate();
        }
        self.selector.as_read_signal().to_trigger()
    }

    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.element_mut().compute_layout(input)
    }

    fn interactions(&mut self, _layout: &Layout) -> ReadSignal<InteractSet> {
        self.element_mut().interactions()
    }

    fn draw(&mut self, context: &mut RenderContext, _layout: &Layout) {
        self.element_mut().draw(context)
    }
}