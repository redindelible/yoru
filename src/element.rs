use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use crate::{math, RenderContext, Widget};

// pub use props::{LayoutCache};
pub use props::{LayoutCache, ContentInfo, Layout};


pub struct Root<A>(Element<A>);

impl<A> Root<A> {
    pub fn new(element: Element<A>) -> Root<A> {
        Root(element)
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.0.0.layout_cache().set_scale_factor(scale_factor);
    }

    pub fn set_viewport(&mut self, viewport: math::Size) {
        self.0.0.layout_cache().set_allocated(math::Rect::from_topleft_size((0.0, 0.0).into(), viewport));
    }

    pub fn update_model(&mut self, model: &mut A) {
        self.0.update_model(model);
    }

    pub fn update_layout(&mut self) {
        self.0.update_layout();
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        self.0.draw(context);
    }
}


// #[derive(Debug)]
pub struct Element<A>(Box<dyn Widget<A>>);

impl<A> Element<A> {
    pub fn new<W: Widget<A> + 'static>(widget: W) -> Element<A> {
        Element(Box::new(widget))
    }
}

// impl<A, W: Widget<A>> From<W> for Element<A> {
//     fn from(value: W) -> Self {
//         Element::new(value)
//     }
// }

impl<A> Element<A> {
    pub fn props(&self) -> &LayoutCache<A> {
        self.0.layout_cache()
    }

    pub fn props_mut(&mut self) -> &mut LayoutCache<A> {
        self.0.layout_cache_mut()
    }

    pub fn update_model(&mut self, model: &mut A) {
        self.0.update_model(model)
    }

    pub fn intrinsic_size(&self) -> ContentInfo {
        self.0.intrinsic_size()
    }

    pub fn update_layout(&mut self) {
        self.0.update_layout();
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        let layout = self.0.layout_cache().get_cached_layout();
        self.0.draw(context, &layout);
    }
}

pub struct LayoutInvalidator(Rc<LayoutInvalidatorInner>);

struct LayoutInvalidatorInner {
    dirty: Cell<(bool, bool)>,
    parent: RefCell<Option<Weak<LayoutInvalidatorInner>>>
}

impl LayoutInvalidator {
    fn new(dirty: bool) -> LayoutInvalidator {
        LayoutInvalidator(Rc::new(LayoutInvalidatorInner {
            dirty: Cell::new((dirty, dirty)),
            parent: RefCell::new(None)
        }))
    }

    fn is_size_dirty(&self) -> bool {
        self.0.dirty.get().0
    }

    fn is_layout_dirty(&self) -> bool {
        self.0.dirty.get().1
    }

    fn set_layout_dirty_untracked(&self) {
        self.0.dirty.set((self.0.dirty.get().0, true));
    }

    fn reset_size(&self) {
        self.0.dirty.set((false, self.0.dirty.get().1));
    }

    fn reset_layout_dirty(&self) {
        self.0.dirty.set((self.0.dirty.get().0, false));
    }

    fn invalidate(&self) {
        // todo this can be folded into the loop
        self.0.dirty.set((true, true));

        let mut curr = self.0.parent.borrow().as_ref().and_then(Weak::upgrade);
        while let Some(strong_curr) = curr {
            strong_curr.dirty.set((true, true));
            curr = strong_curr.parent.borrow().as_ref().and_then(Weak::upgrade);
        }
    }
}

mod props {
    use std::cell::Cell;
    use std::marker::PhantomData;
    use std::rc::{Rc, Weak};
    use crate::style::{Direction, Justify, LayoutStyle, Sizing};
    use crate::element::{Element, LayoutInvalidator};
    use crate::math;
    use crate::math::{Axis};

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Layout {
        pub margin_box: math::Rect,
        pub border_box: math::Rect,
        pub padding_box: math::Rect,
        pub content_box: math::Rect,
        pub scale_factor: f32,
    }

    #[derive(Debug, Copy, Clone)]
    pub struct ContentInfo {
        total_size: math::Size,
        content_size: math::Size,

        total_expand_factor: f32,
        total_expand_size: f32
    }

    impl ContentInfo {
        pub fn new_for_leaf(layout_style: &LayoutStyle, content_size: math::Size) -> ContentInfo {
            ContentInfo {
                total_size: content_size + layout_style.spacing_size().sum_axes(),
                content_size,
                total_expand_size: 0.0,
                total_expand_factor: 0.0,
            }
        }
    }

    pub struct LayoutCache<A> {
        attrs: LayoutStyle,
        invalidator: LayoutInvalidator,

        scale_factor: Cell<f32>,
        allocated: Cell<math::Rect>,

        cached_intrinsic_size: Cell<ContentInfo>,
        cached_layout: Cell<Layout>,
        _phantom: PhantomData<fn(A)>
    }

    impl<A> LayoutCache<A> {
        pub fn new(layout_style: LayoutStyle) -> LayoutCache<A> {
            LayoutCache {
                attrs: layout_style,
                invalidator: LayoutInvalidator::new(true),
                scale_factor: Cell::new(1.0),
                allocated: Cell::new(math::Rect::from_xywh(0.0, 0.0, 0.0, 0.0)),
                cached_intrinsic_size: Cell::new(ContentInfo {
                    total_size: math::Size::new(0.0, 0.0),
                    content_size: math::Size::new(0.0, 0.0),
                    total_expand_factor: 0.0,
                    total_expand_size: 0.0,
                }),
                cached_layout: Cell::new(Layout {
                    margin_box: math::Rect::from_xywh(0.0, 0.0, 0.0, 0.0),
                    border_box: math::Rect::from_xywh(0.0, 0.0, 0.0, 0.0),
                    padding_box: math::Rect::from_xywh(0.0, 0.0, 0.0, 0.0),
                    content_box: math::Rect::from_xywh(0.0, 0.0, 0.0, 0.0),
                    scale_factor: 1.0,
                }),
                _phantom: PhantomData
            }
        }

        pub fn attrs(&self) -> &LayoutStyle {
            &self.attrs
        }

        fn content_box(&self) -> math::Rect {
            self.allocated.get().shrink_by(
                self.scale_factor.get() * (self.attrs.margin + self.attrs.padding + math::SizeRect::from_border(self.attrs.border_size))
            ).clamp_positive()
        }

        pub fn remove_parent(&self) -> Option<LayoutInvalidator> {
            self.invalidator.0.parent.borrow_mut().take().as_ref().and_then(Weak::upgrade).map(LayoutInvalidator)
        }

        pub fn set_parent(&self, parent: &LayoutInvalidator) {
            let old_parent = self.invalidator.0.parent.borrow_mut().replace(Rc::downgrade(&parent.0));
            assert!(old_parent.is_none());
        }

        pub(super) fn set_allocated(&self, allocated: math::Rect) {
            if allocated != self.allocated.get() {
                self.allocated.set(allocated);
                self.invalidator.set_layout_dirty_untracked();
            }
        }

        pub(super) fn set_scale_factor(&self, scale_factor: f32) {
            if scale_factor != self.scale_factor.get() {
                self.scale_factor.set(scale_factor);
                self.invalidator.set_layout_dirty_untracked();
            }
        }

        pub fn invalidate(&mut self) {
            self.invalidator.invalidate();
        }

        fn calculate_intrinsic_size(&self, children: &[Element<A>]) -> ContentInfo {
            let main_axis = self.attrs.main_axis;
            let cross_axis = main_axis.cross();

            let (known_width, known_height) = (
                self.attrs.width.as_definite().map(|width| width * self.scale_factor.get()),
                self.attrs.height.as_definite().map(|width| width * self.scale_factor.get())
            );

            let mut total_main_size: f32 = 0.0;
            let mut max_cross_size: f32 = 0.0;
            let mut total_expand_factor: f32 = 0.0;
            let mut total_expand_size: f32 = 0.0;
            for child in children {
                let child_size = child.0.intrinsic_size();

                total_main_size += child_size.total_size.axis(main_axis);
                max_cross_size = max_cross_size.max(child_size.total_size.axis(cross_axis));

                let child_sizing = match main_axis {
                    Axis::Horizontal => child.props().attrs.width,
                    Axis::Vertical => child.props().attrs.height
                };
                match child_sizing {
                    Sizing::Expand => {
                        total_expand_factor += 1.0;
                        total_expand_size += child_size.total_size.axis(main_axis);
                    },
                    Sizing::Fit => (),
                    Sizing::Fixed(_) => ()
                }
            }

            let extra_size = self.scale_factor.get() * (math::Size::from_border(self.attrs.border_size) + self.attrs.padding.sum_axes() + self.attrs.margin.sum_axes()).clamp_positive();
            let initial_content_size = math::Size::from_axes(main_axis, total_main_size, max_cross_size);
            let content_size = math::Size::new(
                known_width.unwrap_or(initial_content_size.horizontal),
                known_height.unwrap_or(initial_content_size.vertical),
            ).clamp_positive();
            let total_size = (content_size + extra_size).clamp_positive();

            ContentInfo {
                total_size,
                content_size,
                total_expand_factor,
                total_expand_size
            }
        }

        pub fn get_intrinsic_size_with_children(&self, children: &[Element<A>]) -> ContentInfo {
            if self.invalidator.is_size_dirty() {
                let new_size = self.calculate_intrinsic_size(children);
                self.cached_intrinsic_size.replace(new_size);
                self.invalidator.reset_size();
                new_size
            } else {
                self.cached_intrinsic_size.get()
            }
        }

        pub fn get_cached_layout(&self) -> Layout {
            self.cached_layout.get()
        }

        pub fn update_layout_leaf(&self) -> (Layout, bool) {
            if self.invalidator.is_layout_dirty() {
                let layout = Layout {
                    margin_box: self.allocated.get(),
                    border_box: self.allocated.get().shrink_by(self.attrs.margin + math::SizeRect::from_border(self.attrs.border_size / 2.0)).clamp_positive(),
                    padding_box: self.allocated.get().shrink_by(self.attrs.margin + math::SizeRect::from_border(self.attrs.border_size)).clamp_positive(),
                    content_box: self.allocated.get().shrink_by(self.attrs.margin + self.attrs.padding + math::SizeRect::from_border(self.attrs.border_size)).clamp_positive(),
                    scale_factor: self.scale_factor.get()
                };
                self.cached_layout.set(layout);
                self.invalidator.reset_layout_dirty();
                (self.cached_layout.get(), true)
            } else {
                (self.cached_layout.get(), false)
            }
        }

        pub fn update_layout_with_children(&self, children: &[Element<A>]) -> Layout {
            if self.invalidator.is_layout_dirty() {
                let main_axis = self.attrs.main_axis;
                let cross_axis = main_axis.cross();

                let intrinsic_size = self.get_intrinsic_size_with_children(children);
                let content_size = intrinsic_size.content_size;
                let mut content_box = self.content_box();
                let mut remaining = (content_box.size().axis(main_axis) - content_size.axis(main_axis)).max(0.0);
                let mut per_expand_space = 0.0;
                if intrinsic_size.total_expand_factor == 0.0 {
                    let trim_amounts = match self.attrs.main_justify {
                        Justify::Min => (0.0, remaining),
                        Justify::Max => (remaining, 0.0),
                        Justify::Center => (remaining / 2.0, remaining / 2.0)
                    };
                    content_box = content_box.shrink_by(math::SizeRect::from_axis(main_axis, trim_amounts.0, trim_amounts.1));
                    remaining = 0.0;
                } else {
                    per_expand_space = (remaining + intrinsic_size.total_expand_size) / intrinsic_size.total_expand_factor;
                }
                let mut curr = match (main_axis, self.attrs.main_direction) {
                    (Axis::Vertical, Direction::Positive) => content_box.top(),
                    (Axis::Vertical, Direction::Negative) => content_box.bottom(),
                    (Axis::Horizontal, Direction::Positive) => content_box.left(),
                    (Axis::Horizontal, Direction::Negative) => content_box.right()
                };

                for child in children {
                    // todo make a helper and optimize using Rc::ptr_eq
                    child.props().remove_parent();
                    child.props().set_parent(&self.invalidator);

                    let child_size = child.0.intrinsic_size();
                    let (child_main_sizing, child_cross_sizing) = match main_axis {
                        Axis::Horizontal => (child.props().attrs.width, child.props().attrs.height),
                        Axis::Vertical => (child.props().attrs.height, child.props().attrs.width)
                    };
                    let main_axis_amount = match child_main_sizing {
                        Sizing::Expand => (per_expand_space * 1.0).max(child_size.total_size.axis(main_axis)),
                        Sizing::Fit => child_size.total_size.axis(main_axis),
                        Sizing::Fixed(_) => child_size.total_size.axis(main_axis),
                    };
                    let cross_axis_amount = content_box.width().min(match child_cross_sizing {
                        Sizing::Expand => f32::INFINITY,
                        Sizing::Fit => child_size.total_size.axis(cross_axis),
                        Sizing::Fixed(_) => child_size.total_size.axis(cross_axis),
                    });

                    // todo use cross justify
                    let allocated;
                    match (main_axis, self.attrs.main_direction) {
                        (Axis::Vertical, Direction::Positive) => {
                            allocated = math::Rect::from_lrtb(content_box.left(), content_box.left() + cross_axis_amount, curr, curr + main_axis_amount);
                            curr += main_axis_amount;
                        },
                        (Axis::Vertical, Direction::Negative) => {
                            allocated = math::Rect::from_lrtb(content_box.left(), content_box.left() + cross_axis_amount, curr - main_axis_amount, curr);
                            curr -= main_axis_amount;
                        },
                        (Axis::Horizontal, Direction::Positive) => {
                            allocated = math::Rect::from_lrtb(curr, curr + main_axis_amount, content_box.top(), content_box.top() + cross_axis_amount);
                            curr += main_axis_amount;
                        },
                        (Axis::Horizontal, Direction::Negative) => {
                            allocated = math::Rect::from_lrtb(curr - main_axis_amount, curr, content_box.top(), content_box.top() + cross_axis_amount);
                            curr -= main_axis_amount;
                        }
                    };
                    child.0.layout_cache().set_scale_factor(self.scale_factor.get());
                    child.0.layout_cache().set_allocated(allocated);
                }

                self.cached_layout.set(Layout {
                    margin_box: self.allocated.get(),
                    border_box: self.allocated.get().shrink_by(
                        self.scale_factor.get() * (self.attrs.margin + math::SizeRect::from_border(self.attrs.border_size / 2.0))
                    ).clamp_positive(),
                    padding_box: self.allocated.get().shrink_by(
                        self.scale_factor.get() * (self.attrs.margin + math::SizeRect::from_border(self.attrs.border_size))
                    ).clamp_positive(),
                    content_box: self.allocated.get().shrink_by(
                        self.scale_factor.get() * (self.attrs.margin + self.attrs.padding + math::SizeRect::from_border(self.attrs.border_size))
                    ).clamp_positive(),
                    scale_factor: self.scale_factor.get(),
                });
                self.invalidator.reset_layout_dirty();
            }
            self.cached_layout.get()
        }

        pub fn set_width(&mut self, width: Sizing) {
            self.invalidate();
            self.attrs.width = width;
        }

        pub fn set_height(&mut self, height: Sizing) {
            self.invalidate();
            self.attrs.height = height;
        }

        pub fn set_margin(&mut self, margin: impl Into<math::SizeRect>) {
            self.invalidate();
            self.attrs.margin = margin.into();
        }

        pub fn set_padding(&mut self, padding: impl Into<math::SizeRect>) {
            self.invalidate();
            self.attrs.padding = padding.into();
        }

        pub fn set_border_size(&mut self, border_size: f32) {
            self.invalidate();
            self.attrs.border_size = border_size;
        }
    }
}
