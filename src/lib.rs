use tiny_skia::PixmapMut;

mod app;
mod element;
mod style;
pub mod math;
mod layout;
pub mod widgets;
pub mod tracking;
mod interact;
mod utils;

pub use crate::element::{Element, Root};
pub use crate::app::Application;
pub use crate::layout::{BoxLayout, LayoutInput, ComputedLayout, Layout};
pub use crate::style::{LayoutStyle, Sizing, Justify, Direction, Color};
pub use crate::widgets::{Widget, Div, Label};
// pub use crate::tracking::{Derived, Computed, RwSignal, ReadableSignal};

pub struct RenderContext<'a> {
    pub canvas: PixmapMut<'a>,
}

