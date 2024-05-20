use std::cell::RefCell;
use crate::{BoxLayout, Changed, Color, ComputedLayout, Direction, Element, Justify, Layout, LayoutInput, LayoutStyle, math, RenderContext, Sizing};
use crate::math::Axis;
use crate::widgets::Widget;

thread_local! {
    static FONTS: RefCell<cosmic_text::FontSystem> = RefCell::new(cosmic_text::FontSystem::new());
    static SWASH_CACHE: RefCell<cosmic_text::SwashCache> = RefCell::new(cosmic_text::SwashCache::new());
}


pub struct Label<A> {
    layout_cache: BoxLayout<A>,

    font_size: f32,

    changed: Changed,
    text: String,
    compute: Box<dyn Fn(&mut A) -> String>,

    sizing_buffer: cosmic_text::Buffer,
    buffer: cosmic_text::Buffer
}

impl<A> Label<A> {
    pub fn new(compute: impl (Fn(&mut A) -> String) + 'static) -> Label<A> {
        let font_size = 15.0;
        let default_metrics = cosmic_text::Metrics { font_size, line_height: font_size };

        let sizing_buffer = FONTS.with_borrow_mut(|fonts| {
            let mut buffer = cosmic_text::Buffer::new(fonts, default_metrics);
            buffer.set_size(fonts, f32::INFINITY, f32::INFINITY);
            buffer
        });

        Label {
            layout_cache: BoxLayout::new(LayoutStyle {
                border_size: 0.0,
                padding: 0.0.into(),
                margin: 0.0.into(),
                width: Sizing::Fit,
                height: Sizing::Fit,
                // todo make a ContainerLayoutCache so that leaf elements don't need this?
                main_axis: Axis::Vertical,
                main_direction: Direction::Positive,
                main_justify: Justify::Min,
                cross_justify: Justify::Min
            }),

            font_size,

            changed: Changed::untracked(true),
            text: String::new(),
            compute: Box::new(compute),

            sizing_buffer,
            buffer: FONTS.with_borrow_mut(|fonts| {
                cosmic_text::Buffer::new(fonts, default_metrics)
            })
        }
    }
}

impl<A> Widget<A> for Label<A> {
    fn layout_cache(&self) -> &BoxLayout<A> {
        &self.layout_cache
    }

    fn layout_cache_mut(&mut self) -> &mut BoxLayout<A> {
        &mut self.layout_cache
    }

    fn compute_layout(&mut self, input: LayoutInput) -> ComputedLayout {
        self.layout_cache.compute_layout_leaf(input, |available_size, scale_factor| {
            FONTS.with_borrow_mut(|fonts| {
                self.sizing_buffer.set_metrics_and_size(
                    fonts,
                    cosmic_text::Metrics::new(self.font_size * scale_factor, self.font_size * scale_factor),
                    available_size.width(), available_size.height()
                );
                let max_width = self.sizing_buffer.layout_runs().map(|run| run.line_w).max_by(f32::total_cmp).unwrap_or(0.0);
                let total_height = self.sizing_buffer.layout_runs().len() as f32 * self.sizing_buffer.metrics().line_height;
                math::Size::new(max_width, total_height)
            })
        })
    }

    fn update_model(&mut self, model: &mut A) {
        if self.changed.is_changed() {
            let (changed, text) = Changed::run_and_track(|| (self.compute)(model));
            self.text = text;
            FONTS.with_borrow_mut(|fonts| {
                self.buffer.set_text(fonts, &self.text, cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced);
                self.sizing_buffer.set_text(fonts, &self.text, cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced);
            });

            self.layout_cache.invalidate();
            self.changed = changed;
        }
    }

    fn draw(&mut self, context: &mut RenderContext, layout: &Layout) {
        FONTS.with_borrow_mut(|fonts| {
            self.buffer.set_metrics_and_size(
                fonts,
                cosmic_text::Metrics::new(self.font_size * layout.scale_factor, self.font_size * layout.scale_factor),
                layout.content_box.width(), layout.content_box.height()
            );

            SWASH_CACHE.with_borrow_mut(|swash_cache| {
                let mut paint = tiny_skia::Paint::default();
                paint.set_color(Color::BLACK.into());
                let content_top_left = layout.content_box.top_left();

                for run in self.buffer.layout_runs() {
                    for glyph in run.glyphs {
                        let physical_glyph = glyph.physical((content_top_left.x, content_top_left.y), 1.0);

                        // todo first try get_image
                        // todo add with pixel fallback
                        if let Some(commands) = swash_cache.get_outline_commands(fonts, physical_glyph.cache_key) {
                            use cosmic_text::Command;

                            let x_off = content_top_left.x + glyph.x + glyph.x_offset;
                            let y_off = content_top_left.y + glyph.y_offset + run.line_y;

                            let mut path_builder = tiny_skia::PathBuilder::new();
                            for command in commands {
                                match command {
                                    Command::MoveTo(point) =>
                                        path_builder.move_to(point.x + x_off, -point.y + y_off),
                                    Command::LineTo(point) =>
                                        path_builder.line_to(point.x + x_off, -point.y + y_off),
                                    Command::CurveTo(p1, p2, p3) =>
                                        path_builder.cubic_to(p1.x + x_off, -p1.y + y_off, p2.x + x_off, -p2.y + y_off, p3.x + x_off, -p3.y + y_off),
                                    Command::QuadTo(p1, p2) =>
                                        path_builder.quad_to(p1.x + x_off, -p1.y + y_off, p2.x + x_off, -p2.y + y_off),
                                    Command::Close => path_builder.close()
                                }
                            }
                            if let Some(path) = path_builder.finish() {
                                context.canvas.fill_path(
                                    &path,
                                    &paint,
                                    tiny_skia::FillRule::EvenOdd,
                                    tiny_skia::Transform::identity(),
                                    None
                                )
                            }
                        }
                    }
                }
            });
        });
    }
}

impl<A: 'static> From<Label<A>> for Element<A> {
    fn from(value: Label<A>) -> Self {
        Element::new(value)
    }
}
