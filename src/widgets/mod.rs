mod label;
mod div;
mod select;
mod button;

use crate::RenderContext;
use crate::layout::{BoxLayout, ComputedLayout, Layout, LayoutInput};
use crate::tracking::{Trigger, ReadSignal};
use crate::interact::{Interaction, InteractSet};

pub use div::Div;
pub use select::Select;
pub use label::Label;
pub use button::Button;

pub trait Widget<A> {
    fn layout_cache(&self) -> &BoxLayout<A>;
    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A>;

    fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A);
    #[must_use]
    fn update_model(&mut self, model: &mut A) -> Trigger;
    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout;
    fn interactions(&mut self, layout: &Layout) -> ReadSignal<InteractSet>;
    fn draw(&mut self, context: &mut RenderContext, layout: &Layout);
}