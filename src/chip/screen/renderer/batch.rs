use gl::types::GLsizei;
use crate::chip::screen::renderer::Disposable;
use crate::chip::screen::renderer::Shader;
use gl::types::GLfloat;
use gl::types::GLsizeiptr;
use gl::types::GLuint;

use std::{ mem, ptr};
use std::os::raw::c_void;

pub struct BatchConfig{ render_type: GLuint }
impl BatchConfig{
    pub unsafe fn new()->Self{ BatchConfig{ render_type: gl::POINTS } }
}

pub struct Batch{ vao: GLuint, vbo: GLuint, config: Box<BatchConfig>, shader: Box<Shader> }
impl Batch{
    pub unsafe fn new(max: usize)->Self{
        let (mut vao, mut vbo, shader):(GLuint, GLuint, Shader) = ( 0, 0, Shader::new());

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (max * mem::size_of::<GLfloat>()) as GLsizeiptr, ptr::null(), gl::DYNAMIC_DRAW);

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * mem::size_of::<GLfloat>() as GLsizei, ptr::null(), );

        gl::BindVertexArray(0);

        Batch{vao, vbo, config: Box::new(BatchConfig::new()), shader: Box::new(shader) }
    }

    pub unsafe fn draw(&self, row_count: f32, column_count: f32, data: &[f32]){
        if !data.is_empty(){
            gl::BindVertexArray(self.vao);
            self.shader.bind();
            self.shader.set_uniform_value("rowCount", row_count);
            self.shader.set_uniform_value("columnCount", column_count);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, &data[0] as *const f32 as *const c_void);
            gl::DrawArrays(self.config.render_type, 0, (data.len() / 2) as GLsizei);
            gl::BindVertexArray(0);
        }
    }
}

impl Disposable for Batch{
    unsafe fn dispose(&mut self) {
        if self.vbo != 0{
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::DeleteBuffers( 1, &self.vbo);
        }

        if self.vao != 0{
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}