use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::{self, EventLoop},
    window::{Window, WindowAttributes},
};

#[derive(Debug)]
pub enum Exception {
    EventLoopCreatedFailed,
}

#[derive(Default)]
pub struct Application {
    main_window: Option<Arc<Window>>,
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(WindowAttributes::default());
        let window = match window {
            Ok(w) => w,
            Err(_) => return,
        };
        self.main_window = Some(Arc::new(window))
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
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
    fn device_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
    }
    fn about_to_wait(&mut self, event_loop: &event_loop::ActiveEventLoop) {}
    fn user_event(&mut self, event_loop: &event_loop::ActiveEventLoop, event: ()) {}
    fn memory_warning(&mut self, event_loop: &event_loop::ActiveEventLoop) {}
    fn new_events(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
    }
    fn suspended(&mut self, event_loop: &event_loop::ActiveEventLoop) {}
    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}
}
impl Application {
    pub fn initialize(&mut self) -> Result<(), Exception> {
        let event_loop = EventLoop::new();
        let event_loop = match event_loop {
            Ok(e) => e,
            Err(_) => return Err(Exception::EventLoopCreatedFailed),
        };

        event_loop.set_control_flow(event_loop::ControlFlow::Poll);

        let result = event_loop.run_app(self);
        match result {
            Ok(_) => return Ok(()),
            Err(_) => return Err(Exception::EventLoopCreatedFailed),
        }
    }
}
