use std::num::NonZeroU32;
use std::rc::Rc;

use tiny_skia::{Pixmap, PixmapMut};
use cosmic_text::{Attrs, FontSystem, Metrics, Shaping, SwashCache};

use winit::{
    event_loop::EventLoop,
    window::Window
};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};
use softbuffer::{Buffer, Surface};
use crate::element::Element;
use crate::RenderContext;


// piet and softbuffer store color in different orders
struct Color {
    b: u8,
    g: u8,
    r: u8,
    a: u8,
}

impl Color {
    fn from_rgb8(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba8(r, g, b, 255)
    }

    fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { b, g, r, a }
    }
}

impl From<Color> for cosmic_text::Color {
    fn from(value: Color) -> Self {
        cosmic_text::Color::rgba(value.b, value.g, value.r, value.a)
    }
}

impl From<Color> for tiny_skia::Color {
    fn from(value: Color) -> Self {
        tiny_skia::Color::from_rgba8(value.b, value.g, value.r, value.a)
    }
}

struct ActiveApplication {
    window: Rc<Window>,
    context: softbuffer::Context<Rc<Window>>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

pub struct Application {
    active: Option<ActiveApplication>,

    font_system: FontSystem,
    swash_cache: SwashCache,

    to_draw: Element
}

impl Application {
    pub fn new(to_draw: Element) -> Self {
        Application {
            active: None,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),

            to_draw
        }
    }

    pub fn run(&mut self) {
        env_logger::init();

        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(self).unwrap();
    }
}

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(event_loop.create_window(WindowAttributes::default()).unwrap());
        let context = softbuffer::Context::new(Rc::clone(&window)).unwrap();
        let surface = Surface::new(&context, Rc::clone(&window)).unwrap();

        self.active = Some(ActiveApplication {
            window,
            context,
            surface
        })
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(ActiveApplication { window, context, surface }) = &mut self.active else { return; };

        match event {
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                surface.resize(NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap()).unwrap();
                let mut buffer = surface.buffer_mut().unwrap();
                let mut paint = tiny_skia::Paint::default();
                let mut pixmap = PixmapMut::from_bytes(bytemuck::cast_slice_mut(buffer.as_mut()), size.width, size.height).unwrap();
                pixmap.fill(Color::from_rgb8(0, 0, 0).into());

                // {
                //     let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, Metrics::new(48.0, 60.0));
                //     buffer.set_size(&mut self.font_system, size.width as f32, size.height as f32);
                //     buffer.set_text(&mut self.font_system, "Much Text", Attrs::new(), Shaping::Advanced);
                //     buffer.draw(&mut self.font_system, &mut self.swash_cache, Color::from_rgb8(255, 255, 0).into(), |x, y, w, h, color| {
                //         paint.set_color(tiny_skia::Color::from_rgba8(color.r(), color.g(), color.b(), color.a()));
                //         pixmap.fill_rect(tiny_skia::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).unwrap(), &paint, tiny_skia::Transform::identity(), None);
                //     });
                // }
                let mut render_context = RenderContext {
                    canvas: pixmap
                };
                self.to_draw.draw(&mut render_context);

                buffer.present().unwrap();
            }
            WindowEvent::CloseRequested => {
                self.active = None;
            }
            _ => { }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.active.is_none() {
            event_loop.exit();
        }
    }
}


