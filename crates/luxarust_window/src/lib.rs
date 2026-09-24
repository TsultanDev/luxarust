use std::time::Duration;

use luxarust::prelude::*;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Clone)]
pub struct WindowSettings {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub resizable: bool,
}

/// Persistent application-level host. winit only allows one `EventLoop` per
/// process, so it is stored here and outlives the `WindowPlugin` itself.
pub struct WindowHost {
    event_loop: Option<EventLoop<()>>,
}

pub struct WindowPlugin {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub resizable: bool,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        WindowPlugin {
            title: "Luxarust".to_string(),
            width: 1280.0,
            height: 720.0,
            resizable: true,
        }
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        if app.resource::<WindowHost>().is_none() {
            match EventLoop::new() {
                Ok(event_loop) => {
                    event_loop.set_control_flow(ControlFlow::Wait);
                    app.insert_persistent_resource(WindowHost {
                        event_loop: Some(event_loop),
                    });
                }
                Err(error) => {
                    eprintln!("[luxarust_window] EventLoop::new failed: {error}");
                    app.insert_persistent_resource(WindowHost { event_loop: None });
                }
            }
        }
        app.insert_resource(WindowSettings {
            title: self.title.clone(),
            width: self.width,
            height: self.height,
            resizable: self.resizable,
        });
        app.add_systems(Schedule::Update, pump_window);
    }

    fn name(&self) -> &'static str {
        "WindowPlugin"
    }
}

struct WindowRuntime {
    window: Option<Window>,
    settings: WindowSettings,
    should_close: bool,
}

impl ApplicationHandler for WindowRuntime {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.should_close = true;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[allow(deprecated)]
fn pump_window(app: &mut App) {
    let Some(mut host) = app.remove_resource::<WindowHost>() else {
        return;
    };
    let Some(mut event_loop) = host.event_loop.take() else {
        app.insert_persistent_resource(host);
        return;
    };
    let mut runtime = match app.remove_resource::<WindowRuntime>() {
        Some(runtime) => runtime,
        None => match app.resource::<WindowSettings>() {
            Some(settings) => WindowRuntime {
                window: None,
                settings: settings.clone(),
                should_close: false,
            },
            None => {
                host.event_loop = Some(event_loop);
                app.insert_persistent_resource(host);
                return;
            }
        },
    };

    if runtime.window.is_none() {
        let attributes = WindowAttributes::default()
            .with_title(runtime.settings.title.clone())
            .with_inner_size(LogicalSize::new(
                runtime.settings.width,
                runtime.settings.height,
            ))
            .with_resizable(runtime.settings.resizable);
        match event_loop.create_window(attributes) {
            Ok(window) => {
                runtime.window = Some(window);
                println!("[luxarust_window] window created");
            }
            Err(error) => {
                eprintln!("[luxarust_window] create_window failed: {error}");
            }
        }
    }

    let status = event_loop.pump_app_events(Some(Duration::ZERO), &mut runtime);
    if matches!(status, PumpStatus::Exit(_)) {
        runtime.should_close = true;
    }
    let should_close = runtime.should_close;

    host.event_loop = Some(event_loop);
    app.insert_persistent_resource(host);
    app.insert_resource(runtime);
    if should_close {
        app.exit();
    }
}