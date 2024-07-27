use std::ops::IndexMut;

use crate::{Widget, RenderContext};
use crate::element::Element;
use crate::interact::{Interaction, InteractSet};
use crate::layout::{LayoutCharacteristics, PrelayoutInput, LayoutInput};
use crate::tracking::{Computed, Computed2, Derived, ReadableSignal};

pub struct Select<A, S, O> {
    selector: Derived<A, S>,
    options: O,

    update_cache: Computed<()>,
    layout_cache: Computed2<LayoutInput, ()>,
    interactions: Computed<InteractSet>,
}

impl<A, S, O> Select<A, S, O> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    pub fn new(starting: S, options: O, selector: impl (Fn(&mut A) -> S) + 'static) -> Self {
        Select {
            options,
            selector: Derived::new_with_initial(starting, selector),

            update_cache: Computed::new(),
            layout_cache: Computed2::new(),
            interactions: Computed::new(),
        }
    }
}

impl<A: 'static, S, O> From<Select<A, S, O>> for Element<A> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    fn from(value: Select<A, S, O>) -> Self {
        Element::new(value)
    }
}

impl<A, S, O> Widget<A> for Select<A, S, O> where O: IndexMut<S, Output=Element<A>> + 'static, S: Copy + 'static {
    fn update(&self, model: &mut A) {
        self.update_cache.maybe_update(|| {
            self.selector.maybe_update(model);
            self.options[self.selector.get_untracked()].update(model);
        });
        self.update_cache.track()
    }

    fn prelayout(&self, input: PrelayoutInput) -> LayoutCharacteristics {
        // todo cache?
        self.options[self.selector.get_untracked()].prelayout(input)
    }

    fn layout(&self, input: LayoutInput) {
        self.layout_cache.maybe_update(input, |&input| {
            self.options[self.selector.get()].layout(input);
        });
        self.layout_cache.track()
    }

    fn interactions(&self) -> InteractSet {
        self.interactions.maybe_update(|| self.options[self.selector.get()].interactions());
        self.interactions.get()
    }

    fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A) {
        self.options[self.selector.get_untracked()].handle_interaction(interaction, model)
    }

    fn draw(&mut self, context: &mut RenderContext) {
        self.options[self.selector.get_untracked()].draw(context)
    }
}