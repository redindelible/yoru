mod label;
mod div;

use crate::{BoxLayout, ComputedLayout, Layout, LayoutInput, RenderContext};

pub use div::{Div};
pub use label::Label;

pub trait Widget<A> {
    fn layout_cache(&self) -> &BoxLayout<A>;
    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A>;
    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout;

    fn update_model(&mut self, model: &mut A);

    fn draw(&mut self, context: &mut RenderContext, layout: &Layout);
}