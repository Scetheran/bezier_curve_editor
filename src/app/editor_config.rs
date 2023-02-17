
#[derive(Debug, Clone, Copy)]
pub struct EditorConfig {
    pub larp_ratio: f32,
    pub samples: i32,
    pub bezier_curve_color: [f32; 3],
    pub control_points_color: [f32; 3],
    pub control_points_strip_color: [f32; 3],
    pub larp_point_color: [f32; 3],
}

impl EditorConfig {
    pub fn default() -> Self {
        Self {
            larp_ratio: 0.5,
            samples: 100,
            bezier_curve_color: [0.1, 0.2, 0.9],
            control_points_color: [0.7, 0.7, 0.1],
            control_points_strip_color: [0.8, 0.2, 0.2],
            larp_point_color: [0.3, 0.9, 0.3],
        }
    }
}