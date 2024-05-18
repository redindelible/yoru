use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;

use tiny_skia::{PixmapMut};
use cosmic_text::{Attrs, FontSystem, Metrics, Shaping, SwashCache};

use winit::{
    event_loop::EventLoop,
    window::Window
};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};
use softbuffer::{Buffer, Surface};
use crate::style::Color;
use crate::element::{Element, Root};
use crate::{math, RenderContext};

fn timed<T>(message: &str, f: impl FnOnce() -> T) -> T {
    use std::time::Instant;

    let now = Instant::now();
    let ret = f();
    let time = Instant::now() - now;
    eprintln!("{}: {} sec", message, time.as_secs_f32());
    ret
}

struct ActiveApplication {
    window: Rc<Window>,
    context: softbuffer::Context<Rc<Window>>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

pub struct Application<A> {
    active: Option<ActiveApplication>,

    font_system: FontSystem,
    swash_cache: SwashCache,

    // scale_factor: f32,

    state: A,
    to_draw: Root<A>
}

impl<A> Application<A> {
    pub fn new(state: A, to_draw: Root<A>) -> Self {
        Application {
            active: None,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),

            // scale_factor: 1.0,

            state,
            to_draw
        }
    }

    pub fn run(&mut self) {
        env_logger::init();

        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(self).unwrap();
    }
}

impl<A> winit::application::ApplicationHandler for Application<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(event_loop.create_window(WindowAttributes::default()).unwrap());
        let context = softbuffer::Context::new(Rc::clone(&window)).unwrap();
        let surface = Surface::new(&context, Rc::clone(&window)).unwrap();

        self.to_draw.set_scale_factor(window.scale_factor() as f32);
        self.active = Some(ActiveApplication {
            window,
            context,
            surface
        })
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(ActiveApplication { window, context, surface }) = &mut self.active else { return; };

        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.to_draw.set_scale_factor(scale_factor as f32);
            }
            WindowEvent::Resized(new_size) => {
                self.to_draw.set_viewport(math::Size::new(new_size.width as f32, new_size.height as f32));
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                surface.resize(NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap()).unwrap();
                let mut buffer = surface.buffer_mut().unwrap();
                let mut pixmap = PixmapMut::from_bytes(bytemuck::must_cast_slice_mut(buffer.as_mut()), size.width, size.height).unwrap();
                pixmap.fill(Color::WHITE.into());

                timed("Update Model", || self.to_draw.update_model(&mut self.state));
                timed("Update Layout", || self.to_draw.update_layout());

                let mut render_context = RenderContext {
                    canvas: pixmap
                };
                timed("Drawing", || self.to_draw.draw(&mut render_context));

                window.pre_present_notify();
                buffer.present().unwrap();
            }
            WindowEvent::CloseRequested => {
                self.active = None;
                event_loop.exit();
            }
            _ => { }
        }
    }

    // fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    //     if self.active.is_none() {
    //         event_loop.exit();
    //     }
    // }
}


