use std::cell::RefCell;
use std::collections::HashMap;
use crate::{Color, LayoutCharacteristics, Element, PrelayoutInput, LayoutStyle, math, RenderContext, Sizing, layout, Layout};
use crate::interact::{Interaction, InteractSet};
use crate::layout::LayoutInput;
use crate::tracking::{Computed2, Derived, ReadableSignal, RwSignal};
use crate::widgets::Widget;

thread_local! {
    static FONTS: RefCell<cosmic_text::FontSystem> = RefCell::new(cosmic_text::FontSystem::new());
    static GLYPH_CACHE: RefCell<GlyphCache> = RefCell::new(GlyphCache::new());
}


struct CachedGlyph {
    offset: (i32, i32),
    image: Option<tiny_skia::Pixmap>
}

struct GlyphCache {
    swash_cache: cosmic_text::SwashCache,
    cached_glyphs: HashMap<cosmic_text::CacheKey, CachedGlyph>
}

impl GlyphCache {
    fn new() -> GlyphCache {
        GlyphCache {
            swash_cache: cosmic_text::SwashCache::new(),
            cached_glyphs: HashMap::new()
        }
    }

    fn get_glyph(&mut self, fonts: &mut cosmic_text::FontSystem, key: cosmic_text::CacheKey) -> &CachedGlyph {
        self.cached_glyphs.entry(key)
            .or_insert_with_key(|&key| Self::render(fonts, &mut self.swash_cache, key))
    }

    fn render(fonts: &mut cosmic_text::FontSystem, swash_cache: &mut cosmic_text::SwashCache, key: cosmic_text::CacheKey) -> CachedGlyph {
        if let Some(swash_image) = swash_cache.get_image_uncached(fonts, key) {
            if let Some(mut image) = tiny_skia::Pixmap::new(swash_image.placement.width, swash_image.placement.height) {
                let mask = tiny_skia::Mask::from_vec(swash_image.data, tiny_skia::IntSize::from_wh(swash_image.placement.width, swash_image.placement.height).unwrap()).unwrap();
                let mut paint = tiny_skia::Paint::default();
                paint.set_color(Color::BLACK.into());

                image.fill_rect(tiny_skia::Rect::from_xywh(
                    0.0, 0.0,
                    swash_image.placement.width as f32,
                    swash_image.placement.height as f32
                ).unwrap(), &paint, tiny_skia::Transform::identity(), Some(&mask));

                CachedGlyph {
                    offset: (swash_image.placement.left, swash_image.placement.top),
                    image: Some(image)
                }
            } else {
                CachedGlyph {
                    offset: (swash_image.placement.left, swash_image.placement.top),
                    image: None
                }
            }
        } else {
            CachedGlyph {
                offset: (0, 0),
                image: None
            }
        }
    }
}


pub struct Label<A> {
    style: LayoutStyle,
    font_size: f32,

    text: Derived<A, String>,

    sizing_buffer: RwSignal<cosmic_text::Buffer>,
    buffer: RefCell<cosmic_text::Buffer>,

    prelayout_cache: Computed2<PrelayoutInput, math::Size>,
    layout_cache: Computed2<LayoutInput, Layout>
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
            style: LayoutStyle {
                border_size: 0.0,
                padding: 0.0.into(),
                margin: 0.0.into(),
                width: Sizing::Fit,
                height: Sizing::Fit
            },
            font_size,
            text: Derived::new(compute),
            sizing_buffer: RwSignal::new(sizing_buffer),
            buffer: RefCell::new(FONTS.with_borrow_mut(|fonts| {
                cosmic_text::Buffer::new(fonts, default_metrics)
            })),

            prelayout_cache: Computed2::new(),
            layout_cache: Computed2::new()
        }
    }
}

impl<A> Widget<A> for Label<A> {
    fn update(&self, model: &mut A) {
        if self.text.maybe_update(model) {
            let new_value = self.text.get();
            FONTS.with_borrow_mut(|fonts| {
                self.buffer.borrow_mut().set_text(fonts, &new_value, cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced);
                self.sizing_buffer.update(|buffer| buffer.set_text(fonts, &new_value, cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced));
            });
        }
        self.text.track()
    }

    fn prelayout(&self, input: PrelayoutInput) -> LayoutCharacteristics {
        self.prelayout_cache.maybe_update(input, |&input| {
            self.text.track();
            let characteristics = layout::leaf::do_prelayout(&self.style, input, |available, scale_factor| {
                FONTS.with_borrow_mut(|fonts| {
                    self.sizing_buffer.update(|buffer| buffer.set_metrics_and_size(
                        fonts,
                        cosmic_text::Metrics::new(self.font_size * scale_factor, self.font_size * scale_factor),
                        available.width(), available.height()
                    ));
                    let min_size = self.sizing_buffer.with(|buffer| {
                        let max_width = buffer.layout_runs().map(|run| run.line_w).max_by(f32::total_cmp).unwrap_or(0.0);
                        let total_height = buffer.layout_runs().len() as f32 * buffer.metrics().line_height;
                        math::Size::new(max_width, total_height)
                    });
                    min_size
                })
            });
            characteristics.min_size
        });

        LayoutCharacteristics { layout_style: &self.style, min_size: self.prelayout_cache.get() }
    }

    fn layout(&self, input: LayoutInput) {
        self.layout_cache.maybe_update(input, |&input| {
            self.prelayout_cache.track();
            layout::leaf::do_layout(&self.style, input);
            Layout::from_layout_input(&self.style, input)
        });
        self.layout_cache.track();
    }

    fn interactions(&self) -> InteractSet {
        InteractSet::empty()
    }

    fn handle_interaction(&mut self, _interaction: &Interaction, _model: &mut A) {

    }

    fn draw(&mut self, context: &mut RenderContext) {
        let layout = self.layout_cache.get_untracked();
        FONTS.with_borrow_mut(|fonts| {
            self.buffer.borrow_mut().set_metrics_and_size(
                fonts,
                cosmic_text::Metrics::new(self.font_size * layout.scale_factor, self.font_size * layout.scale_factor),
                layout.content_box.width(), layout.content_box.height()
            );

            GLYPH_CACHE.with_borrow_mut(|glyph_cache| {
                let mut paint = tiny_skia::Paint::default();
                paint.set_color(Color::BLACK.into());
                let content_top_left = layout.content_box.top_left();

                for run in self.buffer.borrow().layout_runs() {
                    for glyph in run.glyphs {
                        let physical_glyph = glyph.physical((content_top_left.x, content_top_left.y), 1.0);

                        let rendered_glyph = glyph_cache.get_glyph(fonts, physical_glyph.cache_key);
                        if let Some(glyph_image) = &rendered_glyph.image {
                            let x_off = content_top_left.x + glyph.x + glyph.x_offset;
                            let y_off = content_top_left.y + glyph.y_offset + run.line_y;

                            context.canvas.draw_pixmap(
                                rendered_glyph.offset.0 + x_off as i32,
                                -rendered_glyph.offset.1 + y_off as i32,
                                glyph_image.as_ref(),
                                &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None
                            );
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
