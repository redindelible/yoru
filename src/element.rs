use std::convert::identity;
use crate::{math, RenderContext, Widget};
use crate::interact::{Interaction, InteractSet};
use crate::layout::{BoxLayout, LayoutInput, ComputedLayout};
use crate::tracking::OnChangeToken;


pub struct Root<A>(Element<A>);

impl<A> Root<A> {
    pub fn new(element: Element<A>) -> Root<A> {
        Root(element)
    }

    pub fn handle_interaction(&mut self, interaction: &Interaction) {
        self.0.handle_interaction(interaction)
    }

    pub fn update_model(&mut self, model: &mut A) {
        self.0.update_model(model);
    }

    pub fn compute_layout(&mut self, viewport: math::Size, scale_factor: f32) {
        self.0.compute_layout(LayoutInput::FinalLayout {
            allocated: math::Rect::from_topleft_size((0.0, 0.0).into(), viewport),
            scale_factor
        });
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
    pub fn props(&self) -> &BoxLayout<A> {
        self.0.layout_cache()
    }

    pub fn props_mut(&mut self) -> &mut BoxLayout<A> {
        self.0.layout_cache_mut()
    }

    pub fn update_model(&mut self, model: &mut A) -> OnChangeToken {
        self.0.update_model(model)
    }

    pub fn handle_interaction(&mut self, interaction: &Interaction) {
        self.0.handle_interaction(interaction)
    }

    pub fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.0.compute_layout(input)
    }

    pub fn interactions(&mut self) -> (OnChangeToken, InteractSet) {
        self.0.interactions()
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        let layout = self.0.layout_cache().get_final_layout().unwrap_or_else(identity);
        self.0.draw(context, &layout);
    }
}
