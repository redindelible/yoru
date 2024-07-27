mod label;
mod div;
mod select;
mod button;

use crate::RenderContext;
use crate::layout::{LayoutCharacteristics, PrelayoutInput, LayoutInput};
use crate::interact::{Interaction, InteractSet};

pub use div::Div;
pub use select::Select;
pub use label::Label;
pub use button::Button;

pub trait Widget<A> {
    fn update(&self, model: &mut A);
    fn prelayout(&self, input: PrelayoutInput) -> LayoutCharacteristics;
    fn layout(&self, input: LayoutInput);
    fn interactions(&self) -> InteractSet;

    fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A);
    fn draw(&mut self, context: &mut RenderContext);
}