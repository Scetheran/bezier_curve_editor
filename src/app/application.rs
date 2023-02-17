use std::sync::mpsc::Receiver;


use gl;
use glfw;
use glfw::Context;
use glfw::WindowEvent;



use crate::app::application_event::ApplicationEvent;
use crate::app::window_proxy::WindowProxy;
use crate::app::gui_layer::GUILayer;

#[derive(Debug)]
pub enum ApplicationError {
    InitializationError
}

pub struct Application {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    gui_layer: GUILayer
}

impl Application {
    pub fn new(window_width: u32, window_height: u32, window_title: &str) -> Result<Self, ApplicationError> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));

        let creation_result = glfw.create_window(
            window_width,
            window_height,
            window_title,
            glfw::WindowMode::Windowed,
        );

        let (mut window, events) = if let Some(res) = creation_result {
            res
        } else {
            return Err(ApplicationError::InitializationError);
        };


        window.make_current();
        window.set_all_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        }

        let gui_layer = GUILayer::new(WindowProxy::new(&mut window));

        Ok(
            Self { glfw, window, events, gui_layer }
        )
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            self.gui_layer.handle_user_input(WindowProxy::new(&mut self.window));
            self.gui_layer.transform_mouse_coordinates_for_editor(WindowProxy::new(&mut self.window));
            self.gui_layer.render(WindowProxy::new(&mut self.window));

            self.window.swap_buffers();

            self.glfw.poll_events();
            let mut event_queue = Vec::<WindowEvent>::new();
            for (_, event) in glfw::flush_messages(& self.events) {
                event_queue.push(event);
            }

            for event in event_queue {
                if let Some(application_event) = self.event_from_glfw_event(event) {
                    self.handle_event(application_event)
                }
            }
        }
    }

    fn event_from_glfw_event(&self, event: WindowEvent) -> Option<ApplicationEvent> {
        match event {
            WindowEvent::Size(width, height) => Some(ApplicationEvent::WindowResized{width: width as u32, height: height as u32}),
            WindowEvent::CursorPos(x, y) => {
                let (window_width, window_height) = self.window.get_size();
                let x_int = x as i32;
                let y_int = y as i32;
                if x_int >= 0 && x_int < window_width &&
                   y_int >= 0 && y_int < window_height &&
                   self.window.is_focused() {
                    Some(ApplicationEvent::MouseMoved { x: x_int as u32, y: y_int as u32 })
                } else {
                    None
                }
            },
            WindowEvent::MouseButton(mb, action, _) => {
                if action != glfw::Action::Release {
                    match mb {
                        glfw::MouseButtonLeft => Some(ApplicationEvent::MouseLeftButtonPressed),
                        glfw::MouseButtonRight => Some(ApplicationEvent::MouseRightButtonPressed),
                        _ => None
                    }
                } else {
                    match mb {
                        glfw::MouseButtonLeft => Some(ApplicationEvent::MouseLeftButtonReleased),
                        glfw::MouseButtonRight => Some(ApplicationEvent::MouseRightButtonReleased),
                        _ => None
                    }
                }
            },
            WindowEvent::FramebufferSize(width, height) => Some(ApplicationEvent::FramebufferResized { width: width as u32, height: height as u32 }),
            _ => None
        }
    }

    fn handle_event(&mut self, event: ApplicationEvent) {
    }
}