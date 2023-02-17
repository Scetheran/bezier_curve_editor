use crate::app::gl_renderer::Renderer;
use crate::app::window_proxy::Window;
use crate::app::application_event::ApplicationEvent;
use crate::app::editor_config::EditorConfig;
pub struct EditorLayer {
    side_panel_width_ratio: f32,
    control_point_radius: u32,
    working_area_top_left: (u32, u32),
    working_area_bottom_right: (u32, u32),
    window_size: (u32, u32),
    control_points_normalized: Vec<(f32, f32)>,
    control_point_dragged: Option<usize>,
    last_mouse_pos: (u32, u32),
}

impl EditorLayer {
    pub fn new(window: Window, side_panel_width_ratio: f32) -> Self {
        let control_point_radius = 5;
        let window_size = window.size();
        let (window_width, window_height) = window_size;
        let side_panel_width = (window_width as f32 * side_panel_width_ratio) as u32;
        let working_area_top_left = (side_panel_width + control_point_radius, control_point_radius);
        let working_area_bottom_right = (window_width - control_point_radius, window_height - control_point_radius);
        Self {
            side_panel_width_ratio,
            control_point_radius,
            working_area_top_left,
            working_area_bottom_right,
            window_size,
            control_points_normalized: Vec::new(),
            control_point_dragged: None,
            last_mouse_pos: (0,0)
        }

    }

    fn recalculate_canvas(&mut self, window_width: u32 , window_height: u32) {
        let side_panel_width = (window_width as f32 * self.side_panel_width_ratio) as u32;
        self.working_area_top_left = (side_panel_width + self.control_point_radius, self.control_point_radius);
        self.working_area_bottom_right = (window_width - self.control_point_radius, window_height - self.control_point_radius);
        self.window_size = (window_width, window_height);
    }

    fn is_in_working_area(&self, point: (u32, u32)) -> bool {
        self.working_area_top_left.0 < point.0 &&
        point.0 < self.working_area_bottom_right.0 &&
        self.working_area_top_left.1 < point.1 &&
        point.1 < self.working_area_bottom_right.1
    }

    fn to_normalized_control_point(point: (u32, u32), window_size: (u32, u32)) -> (f32, f32) {
        let (window_width, window_height) = window_size;
        return (point.0 as f32 / window_width as f32, -(point.1 as f32 / window_height as f32));
    }

    fn from_normalized_control_point(point: (f32, f32), window_size: (u32, u32)) -> (u32, u32) {
        let (window_width, window_height) = (window_size.0 as f32, window_size.1 as f32);
        let (top_left_x, top_left_y) = (point.0  * window_width, -point.1 * window_height);
        return (top_left_x.round() as u32, top_left_y.round() as u32);
    }

    fn handle_right_mouse_click(&mut self, mouse_pos: (u32, u32)) {
        if let Some(_) = self.control_point_dragged {
            return;
        }

        let starting_vec_size = self.control_points_normalized.len();
        let radius = self.control_point_radius;
        let window_size = self.window_size;
        self.control_points_normalized.retain_mut(|control_point| {
            let cp = EditorLayer::from_normalized_control_point(*control_point, window_size);

            !(cp.0 < mouse_pos.0 && mouse_pos.0 < cp.0 + 2*radius + 1 && cp.1 < mouse_pos.1 && mouse_pos.1 < cp.1 + 2*radius + 1)
        });
        if starting_vec_size != self.control_points_normalized.len() {
            return
        }
        self.control_points_normalized.push(
            EditorLayer::to_normalized_control_point(
                (mouse_pos.0 - self.control_point_radius, mouse_pos.1 - self.control_point_radius),
                self.window_size
            )
        );
    }

    fn handle_left_mouse_click(&mut self, mouse_pos: (u32, u32)) {
        let radius = self.control_point_radius;
        let window_size = self.window_size;
        for (idx, control_point) in self.control_points_normalized.iter().enumerate() {
            let cp = EditorLayer::from_normalized_control_point(*control_point, window_size);
            if cp.0 < mouse_pos.0 && mouse_pos.0 < cp.0 + 2*radius + 1 && cp.1 < mouse_pos.1 && mouse_pos.1 < cp.1 + 2*radius + 1 {
                self.control_point_dragged = Some(idx);
                break;
            }
        }
    }

    pub fn handle_event(&mut self, event: ApplicationEvent, window: Window) {
        match event {
            ApplicationEvent::MouseRightButtonPressed => {
                if let Some(mouse_pos) = window.mouse_pos() {
                    if self.is_in_working_area(mouse_pos) {
                        self.handle_right_mouse_click(mouse_pos);
                    }
                }
            },
            ApplicationEvent::MouseLeftButtonPressed => {
                if let Some(mouse_pos) = window.mouse_pos() {
                    if self.is_in_working_area(mouse_pos) {
                        self.handle_left_mouse_click(mouse_pos);
                    }
                }
            },
            ApplicationEvent::MouseLeftButtonReleased => {
                self.control_point_dragged = None;
            },
            ApplicationEvent::MouseMoved { x, y } => {
                if !self.is_in_working_area((x,y)) {
                    self.control_point_dragged = None;
                } else {
                    if let Some(idx) = self.control_point_dragged {
                        self.control_points_normalized[idx] = EditorLayer::to_normalized_control_point((x - self.control_point_radius,y - self.control_point_radius), self.window_size);
                    }
                }

                self.last_mouse_pos = (x,y);
            },
            ApplicationEvent::WindowResized { width, height } => {
                self.recalculate_canvas(width, height);
            },
            _ => ()
        }
    }

    fn interpolated_point(&self, t: f32) -> (f32, f32) {
        let mut buffer1 = Vec::<(f32, f32)>::with_capacity(self.control_points_normalized.len());
        for idx in 0..(self.control_points_normalized.len() - 1){
            buffer1.push(
                (
                    (1.0 - t) * self.control_points_normalized[idx].0 + t * self.control_points_normalized[idx+1].0,
                    (1.0 - t) * self.control_points_normalized[idx].1 + t * self.control_points_normalized[idx+1].1
                )
            )
        }

        let mut buffer2 = Vec::<(f32, f32)>::with_capacity(self.control_points_normalized.len());

        while buffer1.len() > 1 {
            for idx in 0..(buffer1.len() - 1){
                buffer2.push(
                    (
                        (1.0 - t) * buffer1[idx].0 + t * buffer1[idx+1].0,
                        (1.0 - t) * buffer1[idx].1 + t * buffer1[idx+1].1
                    )
                )
            }

            std::mem::swap(&mut buffer1, &mut buffer2);
            buffer2.clear();
        }

        return buffer1[0];
    }

    fn draw_bezier_curve(&self, renderer: &mut Renderer, samples: u32, color: (f32, f32, f32)) {
        if self.control_points_normalized.len() <= 2 {
            return;
        }

        let starting_point = EditorLayer::from_normalized_control_point(self.control_points_normalized[0], self.window_size);
        let starting_point = (starting_point.0 + self.control_point_radius, starting_point.1 + self.control_point_radius);
        renderer.begin_line_strip(starting_point, color, 0.1);
        let step = 1.0 / (samples as f32);
        let mut t = step;
        while t < 0.999 {
            let larped_point = self.interpolated_point(t);
            let larped_point = EditorLayer::from_normalized_control_point(larped_point, self.window_size);
            renderer.push_point((larped_point.0 + self.control_point_radius, larped_point.1 + self.control_point_radius));
            t = t + step;
        }
        let end_point = *self.control_points_normalized.last().unwrap();
        let end_point = EditorLayer::from_normalized_control_point(end_point, self.window_size);
        renderer.push_point((end_point.0 + self.control_point_radius, end_point.1 + self.control_point_radius));
        renderer.end_line_strip();
    }

    fn draw_larp_points_strip(&self, renderer: &mut Renderer, color: (f32, f32, f32)) {
        if self.control_points_normalized.len() <= 1 {
            return;
        }

        let starting_point = EditorLayer::from_normalized_control_point(self.control_points_normalized[0], self.window_size);
        let starting_point = (starting_point.0 + self.control_point_radius, starting_point.1 + self.control_point_radius);
        renderer.begin_line_strip(starting_point, color, 0.0);
        for control_point in self.control_points_normalized.iter().skip(1) {
            let cp = EditorLayer::from_normalized_control_point(*control_point, self.window_size);
            renderer.push_point((cp.0 + self.control_point_radius, cp.1 + self.control_point_radius));
        }
        renderer.end_line_strip();
    }

    fn draw_control_points(&self, renderer: &mut Renderer, color: (f32, f32, f32)) {
        if self.control_points_normalized.len() == 0 {
            return;
        }

        renderer.begin_quad_batch(color, 0.4);
        for control_point in &self.control_points_normalized {
            renderer.push_quad(
                EditorLayer::from_normalized_control_point(*control_point, self.window_size),
                (2 * self.control_point_radius + 1, 2 * self.control_point_radius + 1));
        }
        renderer.end_quad_batch();
    }

    fn draw_larp_point(&self, renderer: &mut Renderer, t: f32, color: (f32, f32, f32)) {
        if self.control_points_normalized.len() <= 2 {
            return;
        }

        let larped_point = self.interpolated_point(t);
        let larped_point = EditorLayer::from_normalized_control_point(larped_point, self.window_size);
        renderer.begin_quad_batch(color, 0.3);
        renderer.push_quad((larped_point.0, larped_point.1), (2 * self.control_point_radius + 1, 2 * self.control_point_radius + 1));
        renderer.end_quad_batch();
    }

    pub fn render(&self, renderer: &mut Renderer, config: &EditorConfig) {
        self.draw_larp_points_strip(renderer, (config.control_points_strip_color[0], config.control_points_strip_color[1], config.control_points_strip_color[2]));
        self.draw_bezier_curve(renderer, config.samples as u32, (config.bezier_curve_color[0], config.bezier_curve_color[1], config.bezier_curve_color[2]));
        self.draw_control_points(renderer, (config.control_points_color[0], config.control_points_color[1], config.control_points_color[2]));
        self.draw_larp_point(renderer,  config.larp_ratio, (config.larp_point_color[0], config.larp_point_color[1], config.larp_point_color[2]));
    }

}
