use bytemuck::Zeroable;
use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::math;

#[derive(Copy, Clone, Debug)]
pub struct InteractState {
    pub mouse_position: (f32, f32),
    pub focused_item: ()
}


#[derive(Copy, Clone, Debug)]
pub struct InteractSet {
    pub click: bool,

    pub click_area: math::Rect
}

impl Default for InteractSet {
    fn default() -> Self {
        InteractSet {
            click: false,
            click_area: math::Rect::zeroed()
        }
    }
}

impl std::ops::BitOr for InteractSet {
    type Output = InteractSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        InteractSet {
            click: self.click | rhs.click,
            click_area: math::Rect::bounding_box([self.click_area, rhs.click_area]).unwrap()
        }
    }
}


#[derive(Debug)]
pub enum Interaction {
    Click(math::Point)
}


pub(crate) struct InteractionState {
    cursor_position: math::Point
}

impl InteractionState {
    pub fn new() -> InteractionState {
        InteractionState {
            cursor_position: math::Point::zeroed()
        }
    }

    pub fn handle_window_event(&mut self, event: WindowEvent, send_interaction: impl FnOnce(Interaction)) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = math::Point::new(position.x as f32, position.y as f32)
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if button == MouseButton::Left && state == ElementState::Released {
                    send_interaction(Interaction::Click(self.cursor_position))
                }
            }
            _ => ()
        }
    }
}
