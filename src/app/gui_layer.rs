use imgui;
use std::time::Instant;
use imgui_opengl_renderer::Renderer;

use crate::app::editor_config::EditorConfig;
use crate::app::window_proxy::Window;
pub struct GUILayer {
    last_frame_time: Instant,
    imgui_renderer: Renderer,
    imgui_context: imgui::Context,
    editor_config: EditorConfig,
    side_panel_width_ratio: f32,
}

impl GUILayer {
    pub fn new(mut window_proxy: Window, side_panel_width_ratio: f32) -> Self {
        let mut imgui_context = imgui::Context::create();
        let imgui_renderer = Renderer::new(&mut imgui_context, |s| window_proxy.process_address(s) as _);
        imgui_context.style_mut().window_rounding = 0.0;
        imgui_context.io_mut().font_global_scale = 1.2;
        Self {
            last_frame_time: Instant::now(),
            imgui_renderer,
            imgui_context,
            editor_config: EditorConfig::default(),
            side_panel_width_ratio
        }
    }

    pub fn handle_user_input(&mut self, window_proxy: Window) {
        let mut imgui_io = self.imgui_context.io_mut();
        let mut button_indeces: [bool; 5] = [false, false, false, false, false];
        button_indeces[0] = window_proxy.mouse_left_button_pressed();
        button_indeces[0] = !window_proxy.mouse_left_button_released();
        button_indeces[1] = window_proxy.mouse_right_button_pressed();
        button_indeces[1] = !window_proxy.mouse_right_button_released();
        if let Some((x, y)) = window_proxy.mouse_pos() {
            imgui_io.mouse_pos = [x as f32, y as f32];
        }
        imgui_io.mouse_down = button_indeces;
    }

    pub fn editor_config(&self) -> &EditorConfig {
        return &self.editor_config;
    }

    pub fn render(&mut self, window_proxy: Window) {
        let imgui_io = self.imgui_context.io_mut();

        let now = Instant::now();
        let delta = now - self.last_frame_time;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame_time = now;
        imgui_io.delta_time = delta_s;

        let window_size = window_proxy.size();
        imgui_io.display_size = [window_size.0 as f32, window_size.1 as f32];
        let side_panel_size = [window_size.0 as f32 * self.side_panel_width_ratio, window_size.1 as f32];
        let ui = self.imgui_context.frame();

        ui.window(imgui::im_str!("Side panel")  )
        .size(side_panel_size, imgui::Condition::Always)
        .position([0.0, 0.0], imgui::Condition::Always)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .opened(&mut true)
        .build(|| {
            let [window_width, window_height] = ui.get_window_content_region_max();
            let _token = ui.push_item_width(window_width);
            ui.dummy([window_width, window_height * 0.01]);
            ui.text(format!("Frame time: {:.3}s [{} FPS]", delta_s, (1.0 / delta_s) as u32 ));
            ui.dummy([window_width, window_height * 0.02]);
            ui.text("LARP Ratio:");
            ui.slider_float(imgui::im_str!("LARP"), &mut self.editor_config.larp_ratio, 0.0, 1.0).build();
            ui.dummy([window_width, window_height * 0.02]);
            ui.text("Curve sample points:");
            ui.slider_int(imgui::im_str!("Samples"), &mut self.editor_config.samples, 10, 1000).build();
            ui.dummy([window_width, window_height * 0.02]);
            ui.text("Curve color:");
            ui.color_edit(imgui::im_str!("beier curve color"), &mut self.editor_config.bezier_curve_color).flags(imgui::ImGuiColorEditFlags::NoSmallPreview).build();
            ui.dummy([window_width, window_height * 0.02]);
            ui.text("Control points color:");
            ui.color_edit(imgui::im_str!("control points color"), &mut self.editor_config.control_points_color).flags(imgui::ImGuiColorEditFlags::NoSmallPreview).build();
            ui.dummy([window_width, window_height * 0.02]);
            ui.text("Lines color:");
            ui.color_edit(imgui::im_str!("larp lines color"), &mut self.editor_config.control_points_strip_color).flags(imgui::ImGuiColorEditFlags::NoSmallPreview).build();
            ui.dummy([window_width, window_height * 0.02]);
            ui.text("LARP point color:");
            ui.color_edit(imgui::im_str!("larp point color"), &mut self.editor_config.larp_point_color).flags(imgui::ImGuiColorEditFlags::NoSmallPreview).build();

        });
        self.imgui_renderer.render(ui);
    }
}