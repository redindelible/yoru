mod label;
mod div;
mod select;

use crate::RenderContext;
use crate::layout::{BoxLayout, ComputedLayout, Layout, LayoutInput};
use crate::tracking::OnChangeToken;
use crate::interact::{Interaction, InteractSet};

pub use div::Div;
pub use select::Select;
pub use label::Label;

pub trait Widget<A> {
    fn layout_cache(&self) -> &BoxLayout<A>;
    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A>;

    fn handle_interaction(&mut self, interaction: &Interaction);
    fn update_model(&mut self, model: &mut A) -> OnChangeToken;
    fn interactions(&mut self) -> (OnChangeToken, InteractSet);
    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout;
    fn draw(&mut self, context: &mut RenderContext, layout: &Layout);
}