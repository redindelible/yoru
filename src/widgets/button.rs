use std::convert::identity;
use crate::{BoxLayout, Changed, Color, ComputedLayout, Direction, Element, Justify, Label, Layout, LayoutInput, LayoutStyle, math, RenderContext, Sizing};
use crate::interact::{Interaction, InteractSet};
use crate::math::Axis;
use crate::tracking::{Computed, Derived, OnChangeToken};
use crate::widgets::div::to_tiny_skia_path;
use crate::widgets::Widget;


pub struct Button<A> {
    layout_cache: BoxLayout<A>,

    interactions: Computed<InteractSet>,

    inner: Element<A>,
    on_click: Box<dyn Fn(&mut A)>
}

impl<A: 'static> Button<A> {
    pub fn new(inner: Label<A>, on_click: impl Fn(&mut A) + 'static) -> Button<A> {
        Button {
            layout_cache: BoxLayout::new(LayoutStyle {
                border_size: 2.0,
                padding: 2.0.into(),
                margin: 1.0.into(),
                width: Sizing::Fit,
                height: Sizing::Fit,
                // todo make a ContainerLayoutCache so that leaf elements don't need this?
                main_axis: Axis::Vertical,
                main_direction: Direction::Positive,
                main_justify: Justify::Center,
                cross_justify: Justify::Center
            }),

            interactions: Computed::new(),

            inner: inner.into(),
            on_click: Box::new(on_click)
        }
    }
}

impl<A> Widget<A> for Button<A> {
    fn layout_cache(&self) -> &BoxLayout<A> {
        &self.layout_cache
    }

    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A> {
        &mut self.layout_cache
    }

    fn handle_interaction(&mut self, interaction: &Interaction, model: &mut A) {
        if self.interactions.get_untracked().accepts(interaction) {
            match interaction {
                Interaction::Click(point) => {
                    let layout = self.layout_cache.get_final_layout().unwrap_or_else(identity);
                    if layout.border_box.contains(*point) {
                        (self.on_click)(model);
                    }
                }
                _ => ()
            }

            self.inner.handle_interaction(interaction, model);
        }
    }

    fn update_model(&mut self, model: &mut A) -> OnChangeToken {
        self.inner.update_model(model)
    }

    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.layout_cache.compute_layout_with_children(input, std::slice::from_mut(&mut self.inner))
    }

    fn interactions(&mut self, layout: &Layout) -> (OnChangeToken, InteractSet) {
        self.interactions.maybe_update(|_| {
            let (token, set) = self.inner.interactions();
            token.notify_read();
            let this_set = InteractSet {
                click: true,
                click_area: layout.border_box
            };
            this_set | set
        });
        (self.interactions.token(), *self.interactions.get_untracked())
    }

    fn draw(&mut self, context: &mut RenderContext, layout: &Layout) {
        let border_size = self.layout_cache.attrs().border_size * layout.scale_factor;
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
