use std::sync::mpsc::Receiver;
use gl;
use glfw;
use glfw::Context;
use glfw::WindowEvent;


#[derive(Debug)]
pub enum ApplicationError {
    InitializationError
}
pub struct Application {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
}

struct UserInput<'a> {
    window_ref: &'a glfw::Window
}

impl<'a> UserInput<'a> {
    pub fn new(window: &'a glfw::Window) -> Self {
        Self {
            window_ref : window
        }
    }

    pub fn get_mouse_pos(&self) -> Option<(u32, u32)> {
        let (x, y) = self.window_ref.get_cursor_pos();
        let (window_width, window_height) = self.window_ref.get_size();
        let x_int = x as i32;
        let y_int = y as i32;
        if x_int >= 0 && x_int < window_width &&
            y_int >= 0 && y_int < window_height &&
            self.window_ref.is_focused() {
            Some((x_int as u32, y_int as u32))
        } else {
            None
        }
    }

    fn get_mouse_button_pressed(&self, button: glfw::MouseButton) -> bool {
        self.window_ref.get_mouse_button(button) != glfw::Action::Release
    }

    pub fn get_mouse_left_button_pressed(&self) -> bool {
        self.get_mouse_button_pressed(glfw::MouseButtonLeft)
    }

    pub fn get_mouse_right_button_pressed(&self) -> bool {
        self.get_mouse_button_pressed(glfw::MouseButtonRight)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ApplicationEvent {
    WindowResized { width: u32, height: u32 },
    MouseMoved { x: u32, y: u32 },
    MouseLeftButtonPressed,
    MouseLeftButtonReleased,
    MouseRightButtonPressed,
    MouseRightButtonReleased,
    FramebufferResized { width: u32, height: u32},
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

        Ok(
            Self { glfw, window, events }
        )
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            self.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&self.events) {
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

    fn handle_event(&self, event: ApplicationEvent) {
        match event {
            ApplicationEvent::MouseMoved { x, y } => {
                if UserInput::new(&self.window).get_mouse_left_button_pressed() {
                    println!("Left mouse button pressed at ({}, {})", x, y)
                }
            },
            _ => ()
        };
    }
}

fn main() {
    let mut app = Application::new(1280, 720, "test_window").unwrap();
    app.run()
}