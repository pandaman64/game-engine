use glfw::{Action, Context, Key};

use std::str;
use std::mem;
use std::ptr;

use game_engine::*;

const VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec3 aColor;

out VS_OUT {
    vec3 color;
} vs_out;

void main() {
    gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
    vs_out.color = aColor;
}
"#;

const GEOMETRY_SHADER: &str = r#"
#version 330 core
layout (points) in;
layout (triangle_strip, max_vertices = 5) out;

in VS_OUT {
    vec3 color;
} gs_in[];

out vec3 fColor;

void build_house(vec4 position)
{    
    fColor = gs_in[0].color;

    gl_Position = position + vec4(-0.2, -0.2, 0.0, 0.0);    // 1:bottom-left
    EmitVertex();   
    gl_Position = position + vec4( 0.2, -0.2, 0.0, 0.0);    // 2:bottom-right
    EmitVertex();
    gl_Position = position + vec4(-0.2,  0.2, 0.0, 0.0);    // 3:top-left
    EmitVertex();
    gl_Position = position + vec4( 0.2,  0.2, 0.0, 0.0);    // 4:top-right
    EmitVertex();
    gl_Position = position + vec4( 0.0,  0.4, 0.0, 0.0);    // 5:top
    fColor = vec3(1.0, 1.0, 1.0);
    EmitVertex();
    EndPrimitive();
}

void main() {    
    build_house(gl_in[0].gl_Position);
}  
"#;

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec3 fColor;

void main() {
    FragColor = vec4(fColor, 1.0);
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

    let shader = unsafe {
        Shader::with_geometry_shader(VERTEX_SHADER, GEOMETRY_SHADER, FRAGMENT_SHADER)
    };

    let vao = unsafe {
        let points: [f32; 5 * 4] = [
             // positions // colors
             -0.5,  0.5,  1.0, 0.0, 0.0,
              0.5,  0.5,  0.0, 1.0, 0.0,
              0.5, -0.5,  0.0, 0.0, 1.0,
             -0.5, -0.5,  1.0, 1.0, 0.0,
        ];

        let stride = conv!(mem::size_of::<f32>() * 5);

        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mem::size_of_val(&points)),
            points.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (mem::size_of::<f32>() * 2) as *const _,
        );
        
        vao
    };

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let mut last_time = glfw.get_time() as f32;
    let mut delta_time;

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
        }

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader.use_program();
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::POINTS, 0, 4);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
