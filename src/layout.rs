use bytemuck::Zeroable;
use crate::{LayoutStyle, math};

#[derive(Copy, Clone, PartialEq, Debug, Zeroable, Default)]
pub struct PrelayoutInput {
    pub available: math::Size,
    pub scale_factor: f32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct LayoutCharacteristics<'a> {
    pub layout_style: &'a LayoutStyle,
    pub min_size: math::Size
}

#[derive(Copy, Clone, PartialEq, Debug, Default, Zeroable)]
pub struct LayoutInput {
    pub allocated: math::Rect,
    pub scale_factor: f32,
}


#[derive(Debug, Copy, Clone, PartialEq, Default, Zeroable)]
pub struct Layout {
    pub margin_box: math::Rect,
    pub border_box: math::Rect,
    pub half_border_box: math::Rect,
    pub padding_box: math::Rect,
    pub content_box: math::Rect,
    pub scale_factor: f32,
}

impl Layout {
    pub fn from_margin_box(style: &LayoutStyle, scale_factor: f32, margin_box: math::Rect) -> Layout {
        let border_box = margin_box.shrink_by(scale_factor * style.margin);
        let half_border_box = margin_box.shrink_by(scale_factor * (style.margin + 0.5 * math::SizeRect::from_border(style.border_size)));
        let padding_box = margin_box.shrink_by(scale_factor * (style.margin + math::SizeRect::from_border(style.border_size)));
        let content_box = margin_box.shrink_by(scale_factor * (style.margin + style.padding + math::SizeRect::from_border(style.border_size)));
        Layout {
            margin_box: margin_box.clamp_positive(),
            border_box: border_box.clamp_positive(),
            half_border_box: half_border_box.clamp_positive(),
            padding_box: padding_box.clamp_positive(),
            content_box: content_box.clamp_positive(),
            scale_factor
        }
    }

    pub fn from_layout_input(style: &LayoutStyle, input: LayoutInput) -> Layout {
        Layout::from_margin_box(style, input.scale_factor, input.allocated)
    }
}

pub mod leaf {
    use crate::{LayoutCharacteristics, LayoutStyle, math, PrelayoutInput};
    use crate::layout::LayoutInput;

    pub fn do_prelayout(style: &LayoutStyle, input: PrelayoutInput, measure: impl FnOnce(math::Size, f32) -> math::Size) -> LayoutCharacteristics {
        let spacing = input.scale_factor * (style.margin + style.padding + math::SizeRect::from_border(style.border_size));
        let content_box = input.available - spacing.sum_axes();
        let measured_size = measure(content_box, input.scale_factor);
        return LayoutCharacteristics { layout_style: &style, min_size: measured_size }
    }

    pub fn do_layout(_style: &LayoutStyle, _input: LayoutInput) {

    }
}


pub mod container {
    use crate::{LayoutCharacteristics, math, PrelayoutInput};
    use crate::element::Element;
    use crate::layout::LayoutInput;
    use crate::style::{ContainerLayoutStyle, Direction, Justify, Sizing};

    #[allow(dead_code)]
    struct MeasuredChildren {
        content_size: math::Size,
        child_content_sizes: Vec<(Sizing, Sizing, math::Size)>,
        total_main_space: f32,
        max_cross_space: f32,
        total_expand_factor: f32,
        max_space_per_expand: f32,
    }

    fn measure_children<'a, A: 'a>(style: &ContainerLayoutStyle, available: math::Size, scale_factor: f32, children: impl IntoIterator<Item=&'a Element<A>>) -> MeasuredChildren {
        use crate::math::Axis;

        let spacing = scale_factor * (style.layout_style.margin + style.layout_style.padding + math::SizeRect::from_border(style.layout_style.border_size));
        let main_axis = style.main_axis;
        let cross_axis = main_axis.cross();
        let (main_sizing, cross_sizing) = {
            match main_axis {
                Axis::Vertical => (style.layout_style.height, style.layout_style.width),
                Axis::Horizontal => (style.layout_style.width, style.layout_style.height)
            }
        };

        let available_content_size = available - spacing.sum_axes();
        let cross_available = available_content_size.axis(cross_axis);

        let mut child_content_sizes = Vec::new();
        let mut total_main_space: f32 = 0.0;
        let mut max_cross_space: f32 = 0.0;
        let mut total_expand_factor: f32 = 0.0;
        let mut max_space_per_expand: f32 = 0.0;
        for child in children {
            let child_characteristics = child.prelayout(PrelayoutInput {
                available: math::Size::from_axes(main_axis, f32::INFINITY, cross_available),
                scale_factor
            });

            let (child_main_sizing, child_cross_sizing) = {
                let child_style = child_characteristics.layout_style;
                match main_axis {
                    Axis::Vertical => (child_style.height, child_style.width),
                    Axis::Horizontal => (child_style.width, child_style.height)
                }
            };

            let child_main_space = child_characteristics.min_size.axis(main_axis);
            let child_cross_space = child_characteristics.min_size.axis(cross_axis);

            if let Sizing::Expand = child_main_sizing {
                total_expand_factor += 1.0;
                max_space_per_expand = max_space_per_expand.max(child_main_space / 1.0);
            } else {
                total_main_space += child_main_space;
            }

            max_cross_space = max_cross_space.max(child_cross_space);

            child_content_sizes.push((
                child_main_sizing,
                child_cross_sizing,
                math::Size::from_axes(main_axis, child_main_space, child_cross_space))
            );
        }
        total_main_space += total_expand_factor * max_space_per_expand;

        let main_content_size = main_sizing.as_definite(scale_factor).unwrap_or(total_main_space);
        let cross_content_size = cross_sizing.as_definite(scale_factor).unwrap_or(max_cross_space);
        let content_size = math::Size::from_axes(main_axis, main_content_size, cross_content_size);

        MeasuredChildren {
            content_size,
            child_content_sizes,
            total_main_space,
            max_cross_space,
            total_expand_factor,
            max_space_per_expand,
        }
    }

    pub fn do_prelayout<'a, 'b, A: 'b>(style: &'a ContainerLayoutStyle, input: PrelayoutInput, children: impl IntoIterator<Item=&'b Element<A>>) -> LayoutCharacteristics<'a> {
        let spacing = input.scale_factor * (style.layout_style.margin + style.layout_style.padding + math::SizeRect::from_border(style.layout_style.border_size));
        let measured = measure_children(style, input.available, input.scale_factor, children);
        let min_size = measured.content_size + spacing.sum_axes();

        LayoutCharacteristics { layout_style: &style.layout_style, min_size }
    }

    pub fn do_layout<'a, A: 'a>(style: &ContainerLayoutStyle, input: LayoutInput, children: impl IntoIterator<Item=&'a Element<A>>) -> Vec<LayoutInput> {
        use crate::math::Axis;

        let spacing = input.scale_factor * (style.layout_style.margin + style.layout_style.padding + math::SizeRect::from_border(style.layout_style.border_size));
        let main_axis = style.main_axis;
        let cross_axis = main_axis.cross();

        let measured = measure_children(style, input.allocated.size(), input.scale_factor, children);
        let allocated = input.allocated;

        let (allocated, space_per_expand) = {
            let remaining = allocated.shrink_by(spacing).size().axis(main_axis) - measured.content_size.axis(main_axis);
            if remaining > 0.0 {
                if measured.total_expand_factor == 0.0 {
                    let (min_shrink, max_shrink) = match style.main_justify {
                        Justify::Min => (0.0, remaining),
                        Justify::Max => (remaining, 0.0),
                        Justify::Center => (remaining / 2.0, remaining / 2.0)
                    };
                    (allocated.shrink_by(math::SizeRect::from_axis(main_axis, min_shrink, max_shrink)), 0.0)
                } else {
                    (allocated, remaining / measured.total_expand_factor)
                }
            } else {
                (allocated, 0.0)
            }
        };
        let content_box = allocated.shrink_by(spacing);

        let mut curr = match (main_axis, style.main_direction) {
            (Axis::Horizontal, Direction::Positive) => content_box.left(),
            (Axis::Horizontal, Direction::Negative) => content_box.right(),
            (Axis::Vertical, Direction::Positive) => content_box.top(),
            (Axis::Vertical, Direction::Negative) => content_box.bottom()
        };

        let mut child_layouts = Vec::new();
        for (child_main_sizing, child_cross_sizing, child_content_size) in measured.child_content_sizes.into_iter() {
            let main_amount = match child_main_sizing {
                Sizing::Expand => space_per_expand * 1.0,
                Sizing::Fixed(_) => child_content_size.axis(main_axis),
                Sizing::Fit => child_content_size.axis(main_axis)
            };
            let cross_amount = match child_cross_sizing {
                Sizing::Expand => content_box.size().axis(cross_axis),
                Sizing::Fixed(_) => child_content_size.axis(cross_axis),
                Sizing::Fit => child_content_size.axis(cross_axis)
            };
            let cross_start = match cross_axis {
                Axis::Horizontal => content_box.left(),
                Axis::Vertical => content_box.top()
            } + match style.cross_justify {
                Justify::Min => 0.0,
                Justify::Max => measured.content_size.axis(cross_axis) - cross_amount,
                Justify::Center => (measured.content_size.axis(cross_axis) - cross_amount) / 2.0,
            };

            let child_allocated = match (main_axis, style.main_direction) {
                (Axis::Horizontal, Direction::Positive) => {
                    math::Rect::from_lrtb(curr, curr + main_amount, cross_start, cross_start + cross_amount)
                }
                (Axis::Horizontal, Direction::Negative) => {
                    math::Rect::from_lrtb(curr - main_amount, curr, cross_start, cross_start + cross_amount)
                }
                (Axis::Vertical, Direction::Positive) => {
                    math::Rect::from_lrtb(cross_start, cross_start + cross_amount, curr, curr + main_amount)
                }
                (Axis::Vertical, Direction::Negative) => {
                    math::Rect::from_lrtb(cross_start, cross_start + cross_amount, curr - main_amount, curr)
                }
            };
            match style.main_direction {
                Direction::Positive => curr += main_amount,
                Direction::Negative => curr -= main_amount
            };
            child_layouts.push(LayoutInput { allocated: child_allocated, scale_factor: input.scale_factor });
        }

        child_layouts
    }
}