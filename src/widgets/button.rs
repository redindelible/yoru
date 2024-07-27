use crate::style::{Color, Direction, LayoutStyle, Justify, Sizing, ContainerLayoutStyle};
use crate::layout::{LayoutCharacteristics, Layout, PrelayoutInput, LayoutInput};
use crate::{Element, Label, layout, math, RenderContext};
use crate::interact::{Interaction, InteractSet};
use crate::math::{Axis};
use crate::tracking::{Computed, Computed2, ReadableSignal};
use crate::widgets::div::to_tiny_skia_path;
use crate::widgets::Widget;


pub struct Button<A> {
    style: ContainerLayoutStyle,

    prelayout_cache: Computed2<PrelayoutInput, math::Size>,
    layout_cache: Computed2<LayoutInput, Layout>,
    interactions: Computed<InteractSet>,

    inner: Element<A>,
    on_click: Box<dyn Fn(&mut A)>
}

impl<A: 'static> Button<A> {
    pub fn new(inner: Label<A>, on_click: impl Fn(&mut A) + 'static) -> Button<A> {
        let layout_style = ContainerLayoutStyle {
            layout_style: LayoutStyle {
                border_size: 2.0,
                padding: 2.0.into(),
                margin: 1.0.into(),
                width: Sizing::Fit,
                height: Sizing::Fit,
            },
            main_axis: Axis::Vertical,
            main_direction: Direction::Positive,
            main_justify: Justify::Center,
            cross_justify: Justify::Center
        };

        Button {
            style: layout_style,

            prelayout_cache: Computed2::new(),
            layout_cache: Computed2::new(),
            interactions: Computed::new(),

            inner: inner.into(),
            on_click: Box::new(on_click)
        }
    }
}

impl<A> Widget<A> for Button<A> {
    fn update(&self, model: &mut A) {
        self.inner.update(model)
    }

    fn prelayout(&self, input: PrelayoutInput) -> LayoutCharacteristics {
        self.prelayout_cache.maybe_update(input, |&input| {
            let characteristics = layout::container::do_prelayout(&self.style, input, std::slice::from_ref(&self.inner));
            characteristics.min_size
        });
        LayoutCharacteristics { layout_style: &self.style.layout_style, min_size: self.prelayout_cache.get() }
    }

    fn layout(&self, input: LayoutInput) {
        self.layout_cache.maybe_update(input, |&input| {
            self.prelayout_cache.track();
            let children_layout = layout::container::do_layout(&self.style, input, std::slice::from_ref(&self.inner));
            self.inner.layout(children_layout[0]);
            Layout::from_layout_input(&self.style.layout_style, input)
        });
        self.layout_cache.track();
    }

    fn interactions(&self) -> InteractSet {
        self.interactions.maybe_update(|| {
            let set = self.inner.interactions();
            let this_set = InteractSet {
                click: true,
                click_area: self.layout_cache.get().border_box
            };
            this_set | set
        });
        self.interactions.get()
    }

    fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A) {
        if self.interactions.get_untracked().accepts(interaction) {
            match interaction {
                Interaction::Click(point) => {
                    let layout = self.layout_cache.get_untracked();
                    if layout.border_box.contains(*point) {
                        (self.on_click)(model);
                    }
                }
            }

            self.inner.handle_interaction(interaction, model);
        }
    }

    fn draw(&mut self, context: &mut RenderContext) {
        let layout = self.layout_cache.get_untracked();
        let border_size = self.style.layout_style.border_size * layout.scale_factor;
        if let Some(border_color) = Some(Color::BLACK) {
            if border_size > 0.0 {
                let border_box = layout.half_border_box;
                let path = to_tiny_skia_path(kurbo::Rect::from(border_box));
                let mut stroke = tiny_skia::Stroke::default();
                stroke.width = border_size;
                let mut paint = tiny_skia::Paint::default();
                paint.set_color(border_color.into());
                context.canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
            }
        }

        if let Some(background) = Some(Color::LIGHT_GRAY) {
            let padding_box = layout.padding_box;

            let mut paint = tiny_skia::Paint::default();
            paint.set_color(background.into());
            context.canvas.fill_rect(padding_box.into(), &paint, tiny_skia::Transform::identity(), None);
        }

        self.inner.draw(context);
    }
}

impl<A: 'static> From<Button<A>> for Element<A> {
    fn from(value: Button<A>) -> Self {
        Element::new(value)
    }
}
