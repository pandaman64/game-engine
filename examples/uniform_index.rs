use byte_strings::c_str;
use cgmath::{Matrix, Matrix4, vec3, Point3};
use glfw::{Action, Context, Key};

use std::str;
use std::mem;
use std::ptr;

use game_engine::*;

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;

layout (std140) uniform Matrices {
    mat4 projection;
    mat4 view;
};

uniform mat4 model;

void main() {
    gl_Position = projection * view * model * vec4(aPos, 1.0);
}
"#;

const RED_FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(1.0, 0.3, 0.2, 1.0);
}
"#;

const GREEN_FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(0.1, 1.0, 0.2, 1.0);
}
"#;

const BLUE_FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(0.2, 0.3, 1.0, 1.0);
}
"#;

const YELLOW_FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(1.0, 0.2, 1.0, 1.0);
}
"#;

fn main() {
    env_logger::init();

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
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let red_shader = unsafe {
        Shader::from_str(VERTEX_SHADER_SOURCE, RED_FRAGMENT_SHADER)
    };
    let green_shader = unsafe {
        Shader::from_str(VERTEX_SHADER_SOURCE, GREEN_FRAGMENT_SHADER)
    };
    let blue_shader = unsafe {
        Shader::from_str(VERTEX_SHADER_SOURCE, BLUE_FRAGMENT_SHADER)
    };
    let yellow_shader = unsafe {
        Shader::from_str(VERTEX_SHADER_SOURCE, YELLOW_FRAGMENT_SHADER)
    };
    
    unsafe {
        red_shader.bind_uniform_block(c_str!("Matrices"), 0);
        green_shader.bind_uniform_block(c_str!("Matrices"), 0);
        blue_shader.bind_uniform_block(c_str!("Matrices"), 0);
        yellow_shader.bind_uniform_block(c_str!("Matrices"), 0);
    }

    let cube_vao = unsafe {
        let cube_vertices: [f32; 3 * 6 * 6] = [
             // positions     
             -0.5, -0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5,  0.5, -0.5,
              0.5,  0.5, -0.5,
             -0.5,  0.5, -0.5,
             -0.5, -0.5, -0.5,

             -0.5, -0.5,  0.5,
              0.5, -0.5,  0.5,
              0.5,  0.5,  0.5,
              0.5,  0.5,  0.5,
             -0.5,  0.5,  0.5,
             -0.5, -0.5,  0.5,

             -0.5,  0.5,  0.5,
             -0.5,  0.5, -0.5,
             -0.5, -0.5, -0.5,
             -0.5, -0.5, -0.5,
             -0.5, -0.5,  0.5,
             -0.5,  0.5,  0.5,

              0.5,  0.5,  0.5,
              0.5,  0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5, -0.5,  0.5,
              0.5,  0.5,  0.5,

             -0.5, -0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5, -0.5,  0.5,
              0.5, -0.5,  0.5,
             -0.5, -0.5,  0.5,
             -0.5, -0.5, -0.5,

             -0.5,  0.5, -0.5,
              0.5,  0.5, -0.5,
              0.5,  0.5,  0.5,
              0.5,  0.5,  0.5,
             -0.5,  0.5,  0.5,
             -0.5,  0.5, -0.5,
        ];

        let stride = conv!(mem::size_of::<f32>() * 3);

        let mut cube_vao = 0;
        let mut cube_vbo = 0;
        gl::GenVertexArrays(1, &mut cube_vao);
        gl::GenBuffers(1, &mut cube_vbo);
        gl::BindVertexArray(cube_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, cube_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mem::size_of_val(&cube_vertices)),
            cube_vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            ptr::null(),
        );
        
        cube_vao
    };

    let ubo_matrices = unsafe {
        let mut ubo_matrices = 0;

        gl::GenBuffers(1, &mut ubo_matrices);
        gl::BindBuffer(gl::UNIFORM_BUFFER, ubo_matrices);
        gl::BufferData(
            gl::UNIFORM_BUFFER,
            conv!(2 * mem::size_of::<Matrix4<f32>>()),
            ptr::null(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

        gl::BindBufferRange(
            gl::UNIFORM_BUFFER,
            0,
            ubo_matrices,
            0,
            conv!(2 * mem::size_of::<Matrix4<f32>>()),
        );

        ubo_matrices
    };

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let mut last_time = glfw.get_time() as f32;
    let mut delta_time;

    let mut camera = FPSCamera::new(
        Point3::new(0.0, 0.0, 3.0), // position
        vec3(0.0, 0.0, -1.0), // direction
        45.0, // fov
        -90.0, // yaw
        0.0, // pitch
        800.0 / 600.0, // ratio
    );

    while !window.should_close() {
        let current_time = glfw.get_time() as f32;
        delta_time = current_time - last_time;
        last_time = current_time;

        if delta_time > 0.0 {
            log::info!("FPS = {:04}", 1.0 / delta_time);
        }

        for (_, event) in glfw::flush_messages(&events) {
            match &event {
                glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                    gl::Viewport(0, 0, *width, *height);
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }

            camera.process_event(&event);
        }

        camera.process_mouse(&window, delta_time);

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let projection = camera.projection();
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo_matrices);
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, conv!(mem::size_of::<Matrix4<f32>>()), projection.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

            let view = camera.view();
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo_matrices);
            gl::BufferSubData(gl::UNIFORM_BUFFER, conv!(mem::size_of::<Matrix4<f32>>()), conv!(mem::size_of::<Matrix4<f32>>()), view.as_ptr() as *const _);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        
            // cubes
            gl::BindVertexArray(cube_vao);

            red_shader.use_program();
            red_shader.set_matrix4(c_str!("model"), &Matrix4::from_translation(vec3(-1.0, -1.0, 0.0)));
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            green_shader.use_program();
            green_shader.set_matrix4(c_str!("model"), &Matrix4::from_translation(vec3(1.0, 1.0, 0.0)));
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            blue_shader.use_program();
            blue_shader.set_matrix4(c_str!("model"), &Matrix4::from_translation(vec3(-1.0, 1.0, 0.0)));
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            yellow_shader.use_program();
            yellow_shader.set_matrix4(c_str!("model"), &Matrix4::from_translation(vec3(1.0, -1.0, 0.0)));
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
