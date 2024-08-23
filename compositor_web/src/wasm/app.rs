use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

static mut APP_SESSION: Option<AppSession> = None;

pub struct App {
    window: Option<Arc<Window>>,
    window_ready: crossbeam_channel::Sender<Arc<Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        self.window_ready
            .send(self.window.clone().unwrap())
            .unwrap();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

impl App {
    pub fn spawn() -> Result<AppSession, EventLoopError> {
        use winit::platform::web::EventLoopExtWebSys;
        // Caching app session is necessary because Event Loop is global and can be created only once
        // This is safe because browsers run code on a single thread
        unsafe {
            if let Some(app_session) = APP_SESSION.clone() {
                return Ok(app_session);
            }
        }

        let (window_sender, window_receiver) = crossbeam_channel::bounded(1);
        let app = Self {
            window: None,
            window_ready: window_sender,
        };

        let event_loop = EventLoop::new()?;
        event_loop.spawn_app(app);
        let window = window_receiver.recv().unwrap();

        let app_session = AppSession { window };
        unsafe {
            APP_SESSION = Some(app_session.clone());
        };

        Ok(app_session)
    }
}

#[derive(Clone)]
pub struct AppSession {
    pub window: Arc<Window>,
}
