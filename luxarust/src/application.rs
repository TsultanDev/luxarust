use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{self, EventLoop},
    window::{Window, WindowAttributes, WindowButtons},
};

use crate::config::{Config, Exception};

use winit::dpi::LogicalSize;

pub struct Application {
    config: Config,
    main_window: Option<Arc<Window>>,
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let buttons = if self.config.window.resizable {
            WindowButtons::all()
        } else {
            WindowButtons::all() - WindowButtons::MAXIMIZE
        };
        let window_attrs = WindowAttributes::default()
            .with_title(&self.config.window.title)
            .with_inner_size(LogicalSize::new(
                self.config.window.width,
                self.config.window.height,
            ))
            .with_resizable(self.config.window.resizable)
            .with_maximized(self.config.window.maximized)
            .with_enabled_buttons(buttons);
        let window = match event_loop.create_window(window_attrs) {
            Ok(w) => w,
            Err(_) => return,
        };
        window.set_resizable(self.config.window.resizable);
        window.set_enabled_buttons(buttons);
        if self.config.window.maximized {
            window.set_maximized(true);
        }
        self.main_window = Some(Arc::new(window))
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.main_window.as_mut().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
impl Application {
    pub fn load(config_file: &str) -> Result<Application, Exception> {
        let config = Config::load(config_file)?;
        Ok(Application {
            config,
            main_window: None,
        })
    }
    pub fn initialize(&mut self) -> Result<(), Exception> {
        let event_loop = EventLoop::new().map_err(|_| Exception::EventLoopCreatedFailed)?;
        event_loop.set_control_flow(event_loop::ControlFlow::Poll);

        event_loop
            .run_app(self)
            .map_err(|_| Exception::InternalAppError)?;
        Ok(())
    }
}
