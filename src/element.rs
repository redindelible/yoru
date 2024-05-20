use std::cell::{Cell, RefCell};
use std::convert::identity;
use std::rc::{Rc, Weak};
use bytemuck::{Pod, Zeroable};
use crate::{math, RenderContext, Widget};

// pub use props::{LayoutCache};
pub use props::{BoxLayout, ContentInfo, Layout};


pub struct Root<A>(Element<A>);

impl<A> Root<A> {
    pub fn new(element: Element<A>) -> Root<A> {
        Root(element)
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
    pub fn props(&self) -> &BoxLayout<A> {
        self.0.layout_cache()
    }

    pub fn props_mut(&mut self) -> &mut BoxLayout<A> {
        self.0.layout_cache_mut()
    }

    pub fn update_model(&mut self, model: &mut A) {
        self.0.update_model(model)
    }

    pub fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.0.compute_layout(input)
    }

    pub fn draw(&mut self, context: &mut RenderContext) {
        let layout = self.0.layout_cache().get_final_layout().unwrap_or_else(identity);
        self.0.draw(context, &layout);
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct ComputedLayout {
    margin_box: math::Rect
}

pub struct LayoutCache(Rc<LayoutCacheInner>);

struct CachedLayout {
    is_valid: Cell<bool>,
    with_input: Cell<LayoutInput>,
    cached: Cell<ComputedLayout>
}

impl CachedLayout {
    pub fn new_invalid() -> CachedLayout {
        CachedLayout {
            is_valid: Cell::new(false),
            with_input: Cell::new(LayoutInput::ComputeSize {
                available: Zeroable::zeroed(),
                scale_factor: Zeroable::zeroed(),
            }),
            cached: Zeroable::zeroed()
        }
    }
}

struct LayoutCacheInner {
    parent: RefCell<Option<Weak<LayoutCacheInner>>>,
    cached_unknown: CachedLayout,
    cached_known: CachedLayout,
    cached_final: CachedLayout,
}

impl LayoutCache {
    fn new() -> LayoutCache {
        LayoutCache(Rc::new(LayoutCacheInner {
            parent: RefCell::new(None),
            cached_unknown: CachedLayout::new_invalid(),
            cached_known: CachedLayout::new_invalid(),
            cached_final: CachedLayout::new_invalid()
        }))
    }

    fn invalidate(&self) {
        let mut curr = Some(Rc::clone(&self.0));
        while let Some(strong_curr) = curr {
            strong_curr.cached_unknown.is_valid.set(false);
            strong_curr.cached_known.is_valid.set(false);
            strong_curr.cached_final.is_valid.set(false);
            curr = strong_curr.parent.borrow().as_ref().and_then(Weak::upgrade);
        }
    }

    fn get_or_update(&self, input: LayoutInput, f: impl FnOnce(LayoutInput) -> ComputedLayout) -> ComputedLayout {
        let cache = match input {
            LayoutInput::ComputeSize { available, .. } => {
                if available.width() == f32::INFINITY || available.height() == f32::INFINITY {
                    &self.0.cached_unknown
                } else {
                    &self.0.cached_known
                }
            },
            LayoutInput::FinalLayout { .. } => {
                &self.0.cached_final
            }
        };

        if cache.is_valid.get() && cache.with_input.get() == input {
            cache.cached.get()
        } else {
            let inner = f(input);
            cache.cached.set(inner);
            cache.with_input.set(input);
            cache.is_valid.set(true);
            inner
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LayoutInput {
    ComputeSize {
        available: math::Size,
        scale_factor: f32,
    },
    FinalLayout {
        allocated: math::Rect,
        scale_factor: f32,
    }
}

impl LayoutInput {
    fn available(&self) -> math::Size {
        match self {
            LayoutInput::ComputeSize { available, .. } => *available,
            LayoutInput::FinalLayout { allocated, .. } => allocated.size()
        }
    }

    pub fn scale_factor(&self) -> f32 {
        match self {
            LayoutInput::ComputeSize { scale_factor, .. } => *scale_factor,
            LayoutInput::FinalLayout { scale_factor, .. } => *scale_factor,
        }
    }
}

mod props {
    use std::marker::PhantomData;
    use std::rc::{Rc, Weak};
    use bytemuck::Zeroable;
    use crate::style::{Direction, Justify, LayoutStyle, Sizing};
    use crate::element::{Element, LayoutInput, LayoutCache, ComputedLayout};
    use crate::math;
    use crate::math::{Axis};

    #[derive(Debug, Copy, Clone, PartialEq, Zeroable)]
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

    pub struct BoxLayout<A> {
        attrs: LayoutStyle,
        cache: LayoutCache,

        _phantom: PhantomData<fn(A)>
    }

    impl<A> BoxLayout<A> {
        pub fn new(layout_style: LayoutStyle) -> BoxLayout<A> {
            BoxLayout {
                attrs: layout_style,
                cache: LayoutCache::new(),
                _phantom: PhantomData
            }
        }

        pub fn attrs(&self) -> &LayoutStyle {
            &self.attrs
        }

        pub fn remove_parent(&self) -> Option<LayoutCache> {
            self.cache.0.parent.borrow_mut().take().as_ref().and_then(Weak::upgrade).map(LayoutCache)
        }

        pub fn set_parent(&self, parent: &LayoutCache) {
            let old_parent = self.cache.0.parent.borrow_mut().replace(Rc::downgrade(&parent.0));
            assert!(old_parent.is_none());
        }

        pub fn invalidate(&mut self) {
            self.cache.invalidate();
        }

        pub(super) fn get_final_layout(&self) -> Result<Layout, Layout> {
            let cache = &self.cache.0.cached_final;
            let margin_box = cache.cached.get().margin_box;
            let border_box = margin_box.shrink_by(self.attrs.margin + 0.5 * math::SizeRect::from_border(self.attrs.border_size));
            let padding_box = margin_box.shrink_by(self.attrs.margin + math::SizeRect::from_border(self.attrs.border_size));
            let content_box = margin_box.shrink_by(self.attrs.margin + self.attrs.padding + math::SizeRect::from_border(self.attrs.border_size));
            let scale_factor = cache.with_input.get().scale_factor();
            let layout = Layout { margin_box, border_box, padding_box, content_box, scale_factor };
            if cache.is_valid.get() {
                Ok(layout)
            } else {
                Err(layout)
            }
        }

        pub fn compute_layout_leaf(&self, input: LayoutInput, measure: impl FnOnce(math::Size, f32) -> math::Size) -> ComputedLayout {
            self.cache.get_or_update(input, |input| {
                let scale_factor = input.scale_factor();
                let spacing = scale_factor * (self.attrs.margin + self.attrs.padding + math::SizeRect::from_border(self.attrs.border_size));
                let content_box = input.available() - spacing.sum_axes();

                let measured_size = measure(content_box, scale_factor);
                match input {
                    LayoutInput::ComputeSize { .. } => {
                        ComputedLayout {
                            margin_box: math::Rect::from_topleft_size((0.0, 0.0).into(), measured_size)
                        }
                    }
                    LayoutInput::FinalLayout { allocated, .. } => {
                        let top_left = allocated.top_left();
                        ComputedLayout {
                            margin_box: math::Rect::from_topleft_size(top_left, measured_size)
                        }
                    }
                }
            })
        }

        pub fn compute_layout_with_children(&mut self, input: LayoutInput, children: &mut [Element<A>]) -> ComputedLayout {
            self.cache.get_or_update(input, |input| {
                let scale_factor = input.scale_factor();
                let spacing = scale_factor * (self.attrs.margin + self.attrs.padding + math::SizeRect::from_border(self.attrs.border_size));
                let main_axis = self.attrs.main_axis;
                let cross_axis = main_axis.cross();
                let (main_sizing, cross_sizing) = {
                    let attrs = &self.attrs;
                    match main_axis {
                        Axis::Vertical => (attrs.height, attrs.width),
                        Axis::Horizontal => (attrs.width, attrs.height)
                    }
                };
                let main_justify = self.attrs.main_justify;
                let main_direction = self.attrs.main_direction;
                let cross_justify = self.attrs.cross_justify;

                let available_content_size = input.available() - spacing.sum_axes();
                let main_available = available_content_size.axis(main_axis);
                let cross_available = available_content_size.axis(cross_axis);

                let mut child_content_sizes = Vec::new();
                let mut total_main_space: f32 = 0.0;
                let mut max_cross_space: f32 = 0.0;
                let mut total_expand_factor: f32 = 0.0;
                let mut max_space_per_expand: f32 = 0.0;
                for child in children.iter_mut() {
                    let (child_main_sizing, child_cross_sizing) = {
                        let attrs = &child.0.layout_cache().attrs;
                        match main_axis {
                            Axis::Vertical => (attrs.height, attrs.width),
                            Axis::Horizontal => (attrs.width, attrs.height)
                        }
                    };

                    let child_computed = child.compute_layout(LayoutInput::ComputeSize {
                        available: math::Size::from_axes(main_axis, f32::INFINITY, cross_available),
                        scale_factor: input.scale_factor()
                    });

                    let child_main_space = child_main_sizing.as_definite(scale_factor).unwrap_or(child_computed.margin_box.size().axis(main_axis));
                    let child_cross_space = child_cross_sizing.as_definite(scale_factor).unwrap_or(child_computed.margin_box.size().axis(cross_axis));

                    if let Sizing::Expand = child_main_sizing {
                        total_expand_factor += 1.0;
                        max_space_per_expand = max_space_per_expand.max(child_main_space / 1.0);
                    } else {
                        total_main_space += child_main_space;
                    }

                    max_cross_space = max_cross_space.max(child_cross_space);

                    child_content_sizes.push((child_main_sizing, child_cross_sizing, math::Size::from_axes(main_axis, child_main_space, child_cross_space)));
                }
                total_main_space += total_expand_factor * max_space_per_expand;

                let main_content_size = main_sizing.as_definite(scale_factor).unwrap_or(total_main_space);
                let cross_content_size = cross_sizing.as_definite(scale_factor).unwrap_or(max_cross_space);
                let content_size = math::Size::from_axes(main_axis, main_content_size, cross_content_size);

                let allocated = match input {
                    LayoutInput::ComputeSize { .. } =>
                        return ComputedLayout {
                            margin_box: math::Rect::from_topleft_size((0.0, 0.0).into(), content_size + spacing.sum_axes())
                        },
                    LayoutInput::FinalLayout { allocated, .. } => allocated
                };
                assert_ne!(main_available, f32::INFINITY);

                let (allocated, space_per_expand) = {
                    let remaining = allocated.shrink_by(spacing).size().axis(main_axis) - content_size.axis(main_axis);
                    if remaining > 0.0 {
                        if total_expand_factor == 0.0 {
                            let (min_shrink, max_shrink) = match main_justify {
                                Justify::Min => (0.0, remaining),
                                Justify::Max => (remaining, 0.0),
                                Justify::Center => (remaining / 2.0, remaining / 2.0)
                            };
                            (allocated.shrink_by(math::SizeRect::from_axis(main_axis, min_shrink, max_shrink)), 0.0)
                        } else {
                            (allocated, remaining / total_expand_factor)
                        }
                    } else {
                        (allocated, 0.0)
                    }
                };
                let content_box = allocated.shrink_by(spacing);

                let mut curr = match (main_axis, main_direction) {
                    (Axis::Horizontal, Direction::Positive) => content_box.left(),
                    (Axis::Horizontal, Direction::Negative) => content_box.right(),
                    (Axis::Vertical, Direction::Positive) => content_box.top(),
                    (Axis::Vertical, Direction::Negative) => content_box.bottom()
                };

                for ((child_main_sizing, child_cross_sizing, child_content_size), child) in child_content_sizes.into_iter().zip(children) {
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
                    } + match cross_justify {
                        Justify::Min => 0.0,
                        Justify::Max => content_size.axis(cross_axis) - cross_amount,
                        Justify::Center => (content_size.axis(cross_axis) - cross_amount) / 2.0,
                    };

                    let child_allocated = match (main_axis, main_direction) {
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
                            math::Rect::from_lrtb(cross_start, cross_start + cross_amount,curr - main_amount, curr)
                        }
                    };
                    match main_direction {
                        Direction::Positive => curr += main_amount,
                        Direction::Negative => curr -= main_amount
                    };
                    child.compute_layout(LayoutInput::FinalLayout { allocated: child_allocated, scale_factor });
                }

                ComputedLayout {
                    margin_box: allocated
                }
            })
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
