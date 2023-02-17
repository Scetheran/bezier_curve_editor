use crate::app::gl_renderer::Renderer;

pub struct EditorLayer {

}

impl EditorLayer {
    pub fn new() -> Self {
        return Self {}

    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        renderer.begin_quad_batch((0.8, 0.4, 0.3));
        renderer.push_quad((-0.8, -0.3), (0.2, 0.3));
        renderer.push_quad((0.7, -0.3), (0.2, 0.3));
        renderer.end_quad_batch();
        renderer.begin_line_strip((0.0, 0.0), (1.0, 0.0, 0.0));
        renderer.push_point((0.5, 0.5));
        renderer.push_point((1.0, 0.0));
        renderer.push_point((0.5, -0.5));
        renderer.end_line_strip();

    }

}
