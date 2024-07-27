use crate::{math, RenderContext, Widget};
use crate::interact::{Interaction, InteractSet};
use crate::layout::{PrelayoutInput, LayoutCharacteristics, LayoutInput};
use crate::tracking::{Computed};


pub struct Root<A>(Element<A>, Computed<()>);

impl<A> Root<A> {
    pub fn new(element: Element<A>) -> Root<A> {
        Root(element, Computed::new())
    }

    pub fn needs_redraw(&self) -> bool {
        self.1.is_dirty()
    }

    pub fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A) {
        self.0.handle_interaction(interaction, model)
    }

    pub fn update(&mut self, model: &mut A) {
        self.1.maybe_update(|| {
            self.0.update(model)
        });
    }

    pub fn layout(&mut self, viewport: math::Size, scale_factor: f32) {
        let _ = self.0.layout(LayoutInput {
            allocated: math::Rect::from_topleft_size((0.0, 0.0).into(), viewport),
            scale_factor
        });
    }

    // todo does this really need to be called from the loop?
    pub fn interactions(&mut self) {
        self.0.interactions();
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        self.0.draw(context);
    }
}


pub struct Element<A>(Box<dyn Widget<A>>);

impl<A> Element<A> {
    pub fn new<W: Widget<A> + 'static>(widget: W) -> Element<A> {
        Element(Box::new(widget))
    }
}

impl<A> Element<A> {
    pub fn update(&self, model: &mut A) {
        self.0.update(model)
    }

    pub fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A) {
        self.0.handle_interaction(interaction, model)
    }

    pub fn prelayout(&self, input: PrelayoutInput) -> LayoutCharacteristics {
        self.0.prelayout(input)
    }

    pub fn layout(&self, input: LayoutInput) {
        self.0.layout(input)
    }

    pub fn interactions(&self) -> InteractSet {
        self.0.interactions()
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        self.0.draw(context);
    }
}
