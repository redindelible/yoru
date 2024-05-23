use std::num::NonZeroU32;
use std::rc::Rc;

use tiny_skia::PixmapMut;

use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowAttributes, WindowId, Window};
use softbuffer::Surface;

use crate::style::Color;
use crate::element::Root;
use crate::{math, RenderContext};
use crate::interact::InteractionState;

fn timed<T>(message: &str, f: impl FnOnce() -> T) -> T {
    // f()
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

    viewport: math::Size,
    scale_factor: f32,

    state: A,
    to_draw: Root<A>,

    interaction_state: InteractionState
}

impl<A> Application<A> {
    pub fn new(state: A, to_draw: Root<A>) -> Self {
        Application {
            active: None,

            viewport: math::Size::new(0.0, 0.0),
            scale_factor: 1.0,

            state,
            to_draw,

            interaction_state: InteractionState::new()
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

        // self.viewport =
        self.scale_factor = window.scale_factor() as f32;
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
                self.scale_factor = scale_factor as f32;
            }
            WindowEvent::Resized(new_size) => {
                self.viewport = math::Size::new(new_size.width as f32, new_size.height as f32);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return; };
                surface.resize(width, height).unwrap();
                let mut buffer = surface.buffer_mut().unwrap();
                let mut pixmap = PixmapMut::from_bytes(bytemuck::must_cast_slice_mut(buffer.as_mut()), size.width, size.height).unwrap();
                pixmap.fill(Color::WHITE.into());

                timed("Update Model", || self.to_draw.update_model(&mut self.state));
                timed("Update Layout", || self.to_draw.compute_layout(self.viewport, self.scale_factor));

                timed("Update Interactions", || self.to_draw.interactions());

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
            event => {
                self.interaction_state.handle_window_event(event, |interact| self.to_draw.handle_interaction(&interact, &mut self.state))
            }
        }
    }

    // fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    //     if self.active.is_none() {
    //         event_loop.exit();
    //     }
    // }
}


