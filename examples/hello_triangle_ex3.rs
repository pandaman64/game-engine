use glfw::{Action, Context, Key};

use gl::types::*;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
        gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;

    void main()
    {
        FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    } 
"#;

const FRAGMENT_SHADER_SOURCE_YELLOW: &str = r#"
    #version 330 core
    out vec4 FragColor;

    void main()
    {
        FragColor = vec4(1.0f, 1.0f, 0.2f, 1.0f);
    } 
"#;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to init GLFW");
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(800, 600, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader_program_red, shader_program_yellow, vao1, vao2) = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512);
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "failed to compile vertex shader: {}",
                str::from_utf8_unchecked(&info_log)
            );
        }

        let fragment_shader_red = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader_red, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader_red);
        gl::GetShaderiv(fragment_shader_red, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                fragment_shader_red,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "failed to compile fragment shader: {}",
                str::from_utf8_unchecked(&info_log)
            );
        }

        let fragment_shader_yellow = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE_YELLOW.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader_yellow, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader_yellow);
        gl::GetShaderiv(fragment_shader_yellow, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                fragment_shader_yellow,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "failed to compile fragment shader: {}",
                str::from_utf8_unchecked(&info_log)
            );
        }

        let shader_program_red = gl::CreateProgram();
        gl::AttachShader(shader_program_red, vertex_shader);
        gl::AttachShader(shader_program_red, fragment_shader_red);
        gl::LinkProgram(shader_program_red);
        gl::GetProgramiv(shader_program_red, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shader_program_red,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "failed to compile program: {}",
                str::from_utf8_unchecked(&info_log)
            );
        }

        let shader_program_yellow = gl::CreateProgram();
        gl::AttachShader(shader_program_yellow, vertex_shader);
        gl::AttachShader(shader_program_yellow, fragment_shader_yellow);
        gl::LinkProgram(shader_program_yellow);
        gl::GetProgramiv(shader_program_yellow, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shader_program_yellow,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "failed to compile program: {}",
                str::from_utf8_unchecked(&info_log)
            );
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader_red);
        gl::DeleteShader(fragment_shader_yellow);

        let verticies1: [f32; 9] = [
            // first triangle
            -0.5, -0.5, 0.0, // bottom left
            0.5, -0.5, 0.0, // bottom right
            0.0, 0.5, 0.0, // top
        ];
        let mut vbo = 0;
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (verticies1.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &verticies1[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        let mut vao2 = 0;
        let mut vbo2 = 0;
        gl::GenVertexArrays(1, &mut vao2);
        gl::GenBuffers(1, &mut vbo2);
        let verticies2: [f32; 9] = [
            // second triangle
            0.5, -0.5, 0.0, // bottom left
            0.8, -0.5, 0.0, // bottom right,
            0.65, 0.5, 0.0, // top
        ];
        gl::BindVertexArray(vao2);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo2);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (verticies2.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &verticies2[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        (shader_program_red, shader_program_yellow, vao, vao2)
    };

    while !window.should_close() {
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                    gl::Viewport(0, 0, width, height);
                },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program_red);
            gl::BindVertexArray(vao1);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            gl::UseProgram(shader_program_yellow);
            gl::BindVertexArray(vao2);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
