use std::io::Read;

use gl::types::*;
use std::{ptr, str};
use std::ffi::CString;


pub struct Shader{ shader_program: u32}

#[allow(dead_code)]
impl Shader{
    pub unsafe fn new() -> Self{
        let vertex_data = Shader::read("shaders/simple.vs");
        let geometry_data = Shader::read("shaders/simple.gs");
        let fragment_data = Shader::read("shaders/simple.fs");
        // Setup shader compilation checks
        let vertex_shader = Shader::compile(gl::VERTEX_SHADER, vertex_data.as_ref());
        let geometry_shader = Shader::compile(gl::GEOMETRY_SHADER, geometry_data.as_ref());
        let fragment_shader = Shader::compile(gl::FRAGMENT_SHADER, fragment_data.as_ref());

        let shader_program = Shader::link(vertex_shader, geometry_shader, fragment_shader);
        Shader { shader_program }
    }

    fn read(shader: &str)->String{
        let  mut content = String::new();
        match &mut std::fs::File::open(shader){
            Ok(file) =>{
                if let Result::Err(error) = file.read_to_string(&mut content){
                    print!("{}", error);
                }
            },
            Err(error)=>{ print!("{}", error); }
        }
        return content;
    }

    unsafe fn compile(shader_type: u32, shader_source:&str) -> u32{
        // Setup shader compilation checks
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // -1 to skip trialing null character

        // Vertex shader
        let shader = gl::CreateShader(shader_type);
        let c_str_vert = CString::new(shader_source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for shader compilation errors
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetShaderInfoLog(shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar, );
            match str::from_utf8(&info_log){
                Ok(message) => println!("ERROR::SHADER::COMPILATION_FAILED\n{}, {}", message, shader_source),
                Err(error) => println!("ERROR::SHADER::COMPILATION_FAILED\n{}, {}", error, shader_source)
            }
        }
        shader
    }

    unsafe fn link(vertex_shader:u32, geometry_shader:u32, fragment_shader:u32) ->u32{
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // -1 to skip trialing null character

        // Link Shaders
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, geometry_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        // Check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetProgramInfoLog(shader_program, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar, );
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&info_log).unwrap());
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        shader_program
    }

    pub unsafe fn set_uniform_value(&self, name:&str, value: f32){
        let c_name = CString::new(name).unwrap();
        let ptr = c_name.as_ptr();
        let uniform = gl::GetUniformLocation(self.shader_program, ptr);
        gl::ProgramUniform1f(self.shader_program, uniform, value);
    }

    pub unsafe fn set_uniform_matrix4(&self, name:&str, matrix: &[[f32; 4]; 4]){
        let c_name = CString::new(name).unwrap();

       let uniform = gl::GetUniformLocation(self.shader_program, c_name.as_ptr());
        gl::ProgramUniformMatrix4fv(self.shader_program, uniform, 1, gl::TRUE, std::mem::transmute(matrix));
    }

    pub unsafe  fn bind(&self){
        gl::UseProgram(self.shader_program);
    }

    pub unsafe fn dispose(&self){
        gl::DeleteProgram(self.shader_program);
    }
}