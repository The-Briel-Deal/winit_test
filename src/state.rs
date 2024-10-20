use core::panic;
use std::{collections::HashSet, num::NonZeroU32};

use glutin::{
    config::{Config, ConfigTemplateBuilder, GetGlConfig, GlConfig},
    context::PossiblyCurrentContext,
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window},
};

use crate::{gl::create_gl_context, renderer::Renderer, timer::Timer};

pub struct App {
    window: Option<Window>,
    surface: Option<Surface<WindowSurface>>,
    template: ConfigTemplateBuilder,
    gl_display: GlDisplayCreationState,
    gl_context: Option<PossiblyCurrentContext>,
    timer: Timer,
    renderer: Option<Renderer>,
    keys_down: HashSet<PhysicalKey>,
}

impl App {
    pub fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        App {
            template,
            gl_display: GlDisplayCreationState::Builder(display_builder),
            window: None,
            surface: None,
            gl_context: None,
            timer: Timer::new(),
            renderer: None,
            keys_down: HashSet::new(),
        }
    }
}

fn config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            if config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (window, gl_config) = match &self.gl_display {
            GlDisplayCreationState::Builder(display_builder) => {
                let (window, gl_config) = match display_builder.clone().build(
                    event_loop,
                    self.template.clone(),
                    config_picker,
                ) {
                    Ok((window, gl_config)) => (window.unwrap(), gl_config),
                    Err(err) => {
                        panic!("Builder returned error {err}")
                    }
                };

                self.gl_display = GlDisplayCreationState::Init;
                self.gl_context =
                    Some(create_gl_context(&window, &gl_config).treat_as_possibly_current());

                (window, gl_config)
            }
            GlDisplayCreationState::Init => {
                let gl_config = self.gl_context.as_ref().unwrap().config();

                match glutin_winit::finalize_window(
                    event_loop,
                    Window::default_attributes()
                        .with_transparent(true)
                        .with_title("Glutin triangle gradient example (press Escape to exit)"),
                    &gl_config,
                ) {
                    Ok(window) => (window, gl_config),
                    Err(err) => panic!("Window finalization failed {err}"),
                }
            }
        };
        window.set_cursor_grab(CursorGrabMode::Locked).unwrap();

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_context = self.gl_context.as_ref().unwrap();

        gl_context.make_current(&gl_surface).unwrap();

        self.renderer
            .get_or_insert_with(|| Renderer::new(&gl_config.display()));

        // Try setting vsync.
        if let Err(res) = gl_surface
            .set_swap_interval(gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            eprintln!("Error setting vsync: {res:?}");
        }
        self.surface = Some(gl_surface);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                let gl_surface = self.surface.as_ref().unwrap();
                let gl_context = self.gl_context.as_ref().unwrap();
                let renderer = self.renderer.as_mut().unwrap();
                let delta_time = self.timer.delta_time();

                let GroupedKeys {
                    movement_keys,
                    texture_blend_keys,
                } = group_keys(&self.keys_down);
                renderer.handle_movement_keys(movement_keys, delta_time);
                renderer.handle_texture_blends_keys(texture_blend_keys);
                renderer.draw(&self.timer);
                window.request_redraw();

                gl_surface.swap_buffers(gl_context).unwrap();
                self.timer.reset();
            }
            WindowEvent::Resized(size) => {
                if let Some(gl_surface) = &self.surface {
                    let gl_context = self.gl_context.as_ref().unwrap();

                    gl_surface.resize(
                        gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );

                    let renderer = self.renderer.as_ref().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event.state {
                ElementState::Pressed => {
                    self.keys_down.insert(event.physical_key);
                }
                ElementState::Released => {
                    self.keys_down.remove(&event.physical_key);
                }
            },
            WindowEvent::MouseWheel {
                delta,
                device_id: _,
                phase: _,
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                    self.renderer.as_mut().unwrap().adjust_zoom(-y);
                }
                winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { x: _, y }) => {
                    self.renderer.as_mut().unwrap().adjust_zoom(-y as f32);
                }
            },
            _ => (),
        }
    }
    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let (winit::event::DeviceEvent::MouseMotion { delta }, Some(renderer)) =
            (event, self.renderer.as_mut())
        {
            renderer.handle_mouse_input(delta)
        }
    }
}

struct GroupedKeys {
    movement_keys: Vec<KeyCode>,
    texture_blend_keys: Vec<KeyCode>,
}
fn group_keys(keys_down: &HashSet<PhysicalKey>) -> GroupedKeys {
    let mut movement_keys = vec![];
    let mut texture_blend_keys = vec![];
    for key in keys_down {
        match key {
            PhysicalKey::Code(key @ (KeyCode::KeyJ | KeyCode::KeyK)) => {
                texture_blend_keys.push(*key);
            }
            PhysicalKey::Code(
                key @ (KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD),
            ) => {
                movement_keys.push(*key);
            }
            _ => (),
        }
    }
    GroupedKeys {
        movement_keys,
        texture_blend_keys,
    }
}

enum GlDisplayCreationState {
    /// The display was not built yet.
    Builder(DisplayBuilder),
    /// The display was already created for the application.
    Init,
}
