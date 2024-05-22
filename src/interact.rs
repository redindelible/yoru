use bytemuck::Zeroable;

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

