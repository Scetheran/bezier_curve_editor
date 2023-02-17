use glfw;

pub struct Window<'a> {
    window_ref: &'a mut glfw::Window
}

impl<'a> Window<'a> {
    pub fn new(window: &'a mut glfw::Window) -> Self {
        Self {
            window_ref : window
        }
    }

    pub fn mouse_pos(&self) -> Option<(u32, u32)> {
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

    fn mouse_button_pressed(&self, button: glfw::MouseButton) -> bool {
        self.window_ref.get_mouse_button(button) != glfw::Action::Release
    }

    pub fn mouse_left_button_pressed(&self) -> bool {
        self.mouse_button_pressed(glfw::MouseButtonLeft)
    }

    pub fn mouse_right_button_pressed(&self) -> bool {
        self.mouse_button_pressed(glfw::MouseButtonRight)
    }

    fn mouse_button_released(&self, button: glfw::MouseButton) -> bool {
        self.window_ref.get_mouse_button(button) == glfw::Action::Release
    }

    pub fn mouse_left_button_released(&self) -> bool {
        self.mouse_button_released(glfw::MouseButtonLeft)
    }

    pub fn mouse_right_button_released(&self) -> bool {
        self.mouse_button_released(glfw::MouseButtonRight)
    }

    pub fn process_address(&mut self, process_name: &str) -> glfw::GLProc {
        self.window_ref.get_proc_address(process_name)
    }

    pub fn size(&self) -> (u32, u32) {
        let (w, h) = self.window_ref.get_size();
        (w as u32, h as u32)
    }
}