use std::sync::mpsc::Receiver;

use gl;
use glfw;
use glfw::Context;
use glfw::WindowEvent;

use imgui;
use std::time::Instant;
use imgui_opengl_renderer::Renderer;


#[derive(Debug)]
pub enum ApplicationError {
    InitializationError
}
struct WindowProxy<'a> {
    window_ref: &'a mut glfw::Window
}

impl<'a> WindowProxy<'a> {
    pub fn new(window: &'a mut glfw::Window) -> Self {
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

    pub fn get_process_address(&mut self, process_name: &str) -> glfw::GLProc {
        self.window_ref.get_proc_address(process_name)
    }

    pub fn get_size(&self) -> (u32, u32) {
        let (w, h) = self.window_ref.get_size();
        (w as u32, h as u32)
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
        self.gui_layer.handle_application_event(event)
    }
}

struct GUILayer {
    last_frame_time: Instant,
    imgui_renderer: Renderer,
    imgui_context: imgui::Context,
}

impl GUILayer {
    pub fn new(mut window_proxy: WindowProxy) -> Self {
        let mut imgui_context = imgui::Context::create();
        let imgui_renderer = Renderer::new(&mut imgui_context, |s| window_proxy.get_process_address(s) as _);
        imgui_context.style_mut().window_rounding = 0.0;
        Self {
            last_frame_time: Instant::now(),
            imgui_renderer,
            imgui_context,
        }
    }

    pub fn handle_application_event(&mut self, app_event: ApplicationEvent) {
        let mut imgui_io = self.imgui_context.io_mut();
        let mut button_indeces: [bool; 5] = [false, false, false, false, false];

        match app_event {
            ApplicationEvent::MouseLeftButtonPressed => {
                button_indeces[0] = true
            },
            ApplicationEvent::MouseLeftButtonReleased => {
                button_indeces[0] = false
            },
            ApplicationEvent::MouseRightButtonPressed => {
                button_indeces[1] = true
            },
            ApplicationEvent::MouseRightButtonReleased => {
                button_indeces[1] = false
            },
            ApplicationEvent::MouseMoved{ x, y } => {
                imgui_io.mouse_pos = [x as f32, y as f32];
            },
            _ => ()
        };

        imgui_io.mouse_down = button_indeces;
    }

    pub fn render(&mut self, window_proxy: WindowProxy) {
        let imgui_io = self.imgui_context.io_mut();

        let now = Instant::now();
        let delta = now - self.last_frame_time;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame_time = now;
        imgui_io.delta_time = delta_s;

        let window_size = window_proxy.get_size();
        imgui_io.display_size = [window_size.0 as f32, window_size.1 as f32];
        let ui_pane_size = [window_size.0 as f32 / 4.0, window_size.1 as f32];
        let ui = self.imgui_context.frame();
        ui.window(imgui::im_str!("Big complex window")  )
        .size(ui_pane_size, imgui::Condition::Always)
        .position([0.0, 0.0], imgui::Condition::Always)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .opened(&mut true)
        .build(|| {
            ui.text("Imagine something complicated here..");
            ui.slider_float(imgui::im_str!("LARP Ratio"), &mut 0.5, 0.0, 1.0).build();

        });

        self.imgui_renderer.render(ui);
    }
}

fn main() {
    let mut app = Application::new(1280, 720, "test_window").unwrap();
    app.run()
}