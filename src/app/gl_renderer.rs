use std::ptr;
use std::ffi::CStr;
use std::ffi::CString;
use gl;
use nalgebra_glm as glm;

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Program
}

#[derive(Debug)]
pub enum ShaderError {
    FailedToCompile{ shader_type: ShaderType, error: String }
}
struct ShaderProgram {
    shader: u32
}

impl ShaderProgram {
    pub fn new() -> Result<Self, ShaderError>{
        let shader = unsafe { gl::CreateProgram()} ;
        const VERTEX_SHADER: &[u8] = b"
            #version 330

            layout (location = 0) in vec2 a_pos;
            layout (location = 1) in vec3 a_col;
            layout (location = 2) in float a_depth;
            uniform mat4 u_VP;
            out vec3 col;

            void main()
            {
                gl_Position = u_VP * vec4(a_pos, a_depth, 1.0f);
                col = a_col;
            };\0";
        let vertex = unsafe {
            ShaderProgram::compile_shader_from_source(ShaderType::Vertex, &CStr::from_bytes_with_nul(VERTEX_SHADER).unwrap())
        }?;

        const FRAGMENT_SHADER: &[u8] = b"
            #version 330

            out vec4 color;
            in vec3 col;

            void main()
            {
                color = vec4(col, 1.0f);
            };\0";
        let fragment = unsafe {
            ShaderProgram::compile_shader_from_source(ShaderType::Fragment, &CStr::from_bytes_with_nul(FRAGMENT_SHADER).unwrap())
        }?;
        unsafe {
            gl::AttachShader(shader, vertex);
            gl::AttachShader(shader, fragment);
            gl::LinkProgram(shader);
            let mut result = 0i32;
            gl::GetProgramiv(shader, gl::LINK_STATUS, &mut result);
            if result == 0 {
                return Err(ShaderError::FailedToCompile { shader_type: ShaderType::Program, error: ShaderProgram::decode_shader_error_msg(shader) })
            }

            gl::ValidateProgram(shader);
            gl::GetProgramiv(shader, gl::VALIDATE_STATUS, &mut result);
            if result == 0 {
                return Err(ShaderError::FailedToCompile { shader_type: ShaderType::Program, error: ShaderProgram::decode_shader_error_msg(shader) })
            }
        }

        Ok(Self {
            shader,
        })
    }

    unsafe fn compile_shader_from_source(shader_type: ShaderType, source: &CStr) -> Result<u32, ShaderError>{
        let gl_shader_type = match shader_type {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            _ => 0
        };
        let vertex_shader_id = gl::CreateShader(gl_shader_type);
        gl::ShaderSource(vertex_shader_id, 1, &source.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader_id);
        let mut result = 0i32;
        gl::GetShaderiv(vertex_shader_id, gl::COMPILE_STATUS, &mut result);

        if result == 0 {
            return Err(ShaderError::FailedToCompile {shader_type: shader_type, error: ShaderProgram::decode_shader_error_msg(vertex_shader_id)});
        }

        Ok(vertex_shader_id)
    }

    unsafe fn decode_shader_error_msg(shader_id: u32) -> String {
        let mut len: gl::types::GLint = 0;
        gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
        // fill it with len spaces
        buffer.extend([b' '].iter().cycle().take(len as usize));
        // convert buffer to CString
        let error: CString = CString::from_vec_unchecked(buffer);
        gl::GetShaderInfoLog(
            shader_id,
            len,
            std::ptr::null_mut(),
            error.as_ptr() as *mut gl::types::GLchar
        );
        error.to_string_lossy().into_owned()
    }

    pub fn get_program_gl_id(&self) -> u32 {
        self.shader
    }
}
struct QuadRenderer {
    vertex_buffer_id: u32,
    vertex_array_id: u32,
    index_buffer_id: u32,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    max_buffer_size: usize,
}

impl QuadRenderer {
    pub fn new(max_quads: usize) -> Self {
        unsafe {
            let mut vertex_buffer_id = 0;
            gl::GenBuffers(1, &mut vertex_buffer_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER, (max_quads * 2 * 4) as isize, ptr::null(), gl::DYNAMIC_DRAW);

            let mut vertex_array_id = 0;
            gl::GenVertexArrays(1, &mut vertex_array_id);
            gl::BindVertexArray(vertex_array_id);

            let mut index_buffer_id = 0;
            gl::GenBuffers(1, &mut index_buffer_id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer_id);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (max_quads * 6) as isize, ptr::null(), gl::DYNAMIC_DRAW);

            gl::BindVertexArray( 0 );
            gl::BindBuffer( gl::ARRAY_BUFFER, 0 );
            gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, 0 );


            Self {
                vertex_buffer_id,
                vertex_array_id,
                index_buffer_id,
                vertices: Vec::new(),
                indices: Vec::new(),
                max_buffer_size: (max_quads * 2 * 4)
            }
        }
    }

    pub fn begin_batch(&mut self, shader: &ShaderProgram, ortho_matrix: glm::Mat4, color: (f32, f32, f32), depth: f32) {
        unsafe {
            gl::UseProgram(shader.get_program_gl_id());

            gl::BindVertexArray(self.vertex_array_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer_id);

            let col_location = gl::GetAttribLocation(shader.get_program_gl_id(), CStr::from_bytes_with_nul(b"a_col\0").unwrap().as_ptr());
            gl::VertexAttrib3f(col_location as u32, color.0, color.1, color.2);
            let col_location = gl::GetAttribLocation(shader.get_program_gl_id(), CStr::from_bytes_with_nul(b"a_depth\0").unwrap().as_ptr());
            gl::VertexAttrib1f(col_location as u32, depth);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 8, ptr::null());


            let location = gl::GetUniformLocation( shader.get_program_gl_id(), CStr::from_bytes_with_nul(b"u_VP\0").unwrap().as_ptr());
	        gl::UniformMatrix4fv( location, 1, gl::FALSE, ortho_matrix.as_ptr());
        }

        self.vertices.clear();
        self.indices.clear();
    }

    pub fn end_batch(&mut self) {
        self.flush();
        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::UseProgram(0);
        }
    }

    pub fn push_quad(&mut self, pos: (u32, u32), size: (u32, u32)) {
        if self.vertices.len() > self.max_buffer_size {
            self.flush();
        }

        let top_left = pos;
        let bottom_left = (pos.0, pos.1 + size.1);
        let top_right = (pos.0 + size.0, pos.1);
        let bottom_right = (pos.0 + size.0, pos.1 + size.1);

        let top_left_index = self.vertices.len() as u32 / 2;
        self.vertices.push(top_left.0 as f32);
        self.vertices.push(top_left.1 as f32);

        self.vertices.push(bottom_left.0 as f32);
        self.vertices.push(bottom_left.1 as f32);
        let bottom_left_index = top_left_index + 1;

        self.vertices.push(bottom_right.0 as f32);
        self.vertices.push(bottom_right.1 as f32);
        let bottom_right_index = bottom_left_index + 1;

        self.vertices.push(top_right.0 as f32);
        self.vertices.push(top_right.1 as f32);
        let top_right_index = bottom_right_index + 1;

        self.indices.push(top_left_index);
        self.indices.push(bottom_left_index);
        self.indices.push(bottom_right_index);
        self.indices.push(top_left_index);
        self.indices.push(top_right_index);
        self.indices.push(bottom_right_index);

    }

    fn flush(&mut self) {
        if self.vertices.len() == 0 {
            return;
        }

        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER, (self.vertices.len() * 4) as isize, self.vertices.as_slice().as_ptr().cast(), gl::DYNAMIC_DRAW);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (self.indices.len() * 4) as isize, self.indices.as_slice().as_ptr().cast(), gl::DYNAMIC_DRAW);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
        }
        self.vertices.clear();
        self.indices.clear();
    }
}

struct LineRenderer {
    vertex_buffer_id: u32,
    vertex_array_id: u32,
    vertices: Vec<f32>,
    max_vertices: usize,
}

impl LineRenderer {
    pub fn new(max_lines: usize) -> Self {
        unsafe {
            gl::LineWidth(2.0);
            let mut vertex_buffer_id = 0;
            gl::GenBuffers(1, &mut vertex_buffer_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER, ((max_lines + 1) * 2 * 4) as isize, ptr::null(), gl::DYNAMIC_DRAW);

            let mut vertex_array_id = 0;
            gl::GenVertexArrays(1, &mut vertex_array_id);
            gl::BindVertexArray(vertex_array_id);

            Self {
                vertex_buffer_id,
                vertex_array_id,
                vertices: Vec::new(),
                max_vertices: max_lines + 1,
            }
        }
    }

    pub fn begin_line_strip(&mut self, shader: &ShaderProgram, ortho_matrix: glm::Mat4, starting_point: (u32, u32), color: (f32, f32, f32), depth: f32) {
        unsafe {
            gl::UseProgram(shader.get_program_gl_id());

            gl::BindVertexArray(self.vertex_array_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_id);

            let col_location = gl::GetAttribLocation(shader.get_program_gl_id(), CStr::from_bytes_with_nul(b"a_col\0").unwrap().as_ptr());
            gl::VertexAttrib3f(col_location as u32, color.0, color.1, color.2);
            let col_location = gl::GetAttribLocation(shader.get_program_gl_id(), CStr::from_bytes_with_nul(b"a_depth\0").unwrap().as_ptr());
            gl::VertexAttrib1f(col_location as u32, depth);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 8, ptr::null());

            let location = gl::GetUniformLocation( shader.get_program_gl_id(), CStr::from_bytes_with_nul(b"u_VP\0").unwrap().as_ptr());
	        gl::UniformMatrix4fv( location, 1, gl::FALSE, ortho_matrix.as_ptr());
        }

        self.vertices.clear();
        self.vertices.push(starting_point.0 as f32);
        self.vertices.push(starting_point.1 as f32);
    }

    pub fn end_line_strip(&mut self) {
        self.flush();
        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::UseProgram(0);
        }
    }

    pub fn push_point(&mut self, point: (u32, u32)) {
        if self.vertices.len() == self.max_vertices {
            let last_point_y = *self.vertices.last().unwrap();
            self.vertices.pop();
            let last_point_x = *self.vertices.last().unwrap();
            self.vertices.push(last_point_y);
            self.flush();
            self.vertices.push(last_point_x);
            self.vertices.push(last_point_y);
            self.vertices.push(point.0 as f32);
            self.vertices.push(point.1 as f32);
        } else {
            self.vertices.push(point.0 as f32);
            self.vertices.push(point.1 as f32);
        }
    }

    fn flush(&mut self) {

        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER, (self.vertices.len() * 4) as isize, self.vertices.as_slice().as_ptr().cast(), gl::DYNAMIC_DRAW);
            gl::DrawArrays(gl::LINE_STRIP, 0, (self.vertices.len() / 2) as i32)
        }
        self.vertices.clear();
    }

}

pub struct Renderer {
    shader_program: ShaderProgram,
    quad_renderer: QuadRenderer,
    line_renderer: LineRenderer,
    ortho_matrix: glm::Mat4,
}

impl Renderer {
    pub fn new(viewport_size: (u32, u32), max_quads_per_batch: usize, max_lines_per_batch: usize) -> Self {
        Self {
            quad_renderer: QuadRenderer::new(max_quads_per_batch),
            line_renderer: LineRenderer::new(max_lines_per_batch),
            shader_program: ShaderProgram::new().unwrap(),
            ortho_matrix: glm::ortho(0.0, viewport_size.0 as f32, viewport_size.1 as f32, 0.0, -5.0, 5.0)
        }
    }

    pub fn set_viewport(&mut self, size: (u32, u32)) {
        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
        }
        self.ortho_matrix = glm::ortho(0.0, size.0 as f32, size.1 as f32, 0.0, -5.0, 5.0);
    }

    pub fn begin_quad_batch(&mut self, color: (f32, f32, f32), depth: f32) {
        self.quad_renderer.begin_batch(&self.shader_program, self.ortho_matrix, color, depth);
    }

    pub fn end_quad_batch(&mut self) {
        self.quad_renderer.end_batch();
    }

    pub fn push_quad(&mut self, position: (u32, u32), size: (u32, u32)) {
        self.quad_renderer.push_quad(position, size);
    }

    pub fn begin_line_strip(&mut self, starting_point: (u32, u32), color: (f32, f32, f32), depth: f32) {
        self.line_renderer.begin_line_strip(&self.shader_program, self.ortho_matrix, starting_point, color, depth);
    }

    pub fn push_point(&mut self, point: (u32, u32)) {
        self.line_renderer.push_point(point);
    }

    pub fn end_line_strip(&mut self) {
        self.line_renderer.end_line_strip();
    }
}