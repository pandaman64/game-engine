use cgmath::vec2;
use glfw::{Action, Context, Key};

use std::str;
use std::mem;
use std::ptr;

use game_engine::*;

const VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aOffset;

out vec3 fColor;

void main() {
    vec2 pos = aPos * (gl_InstanceID / 100.0);
    gl_Position = vec4(pos + aOffset, 0.0, 1.0);
    fColor = aColor;
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
        Shader::from_str(VERTEX_SHADER, FRAGMENT_SHADER)
    };

    let vao = unsafe {
        let points: [f32; 5 * 6] = [
             // positions // colors
             -0.05,  0.05,  1.0, 0.0, 0.0,
              0.05, -0.05,  0.0, 1.0, 0.0,
             -0.05, -0.05,  0.0, 0.0, 1.0,

             -0.05,  0.05,  1.0, 0.0, 0.0,
              0.05, -0.05,  0.0, 1.0, 0.0,
              0.05,  0.05,  0.0, 0.0, 1.0,
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

    let translations = {
        let mut translations = [vec2(0.0, 0.0); 100];

        const OFFSET: f32 = 0.1;
        let mut index = 0;

        for y in 0..10 {
            for x in 0..10 {
                let y = y * 2 - 10;
                let x = x * 2 - 10;

                translations[index].x += x as f32 / 10.0 + OFFSET;
                translations[index].y += y as f32 / 10.0 + OFFSET;

                index += 1; 
            }
        }

        translations
    };

    let instance_vbo = unsafe {
        let mut instance_vbo = 0;
        gl::GenBuffers(1, &mut instance_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mem::size_of_val(&translations)),
            translations.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        
        instance_vbo
    };

    unsafe {
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(2);
        gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            conv!(2 * mem::size_of::<f32>()),
            ptr::null(),
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::VertexAttribDivisor(2, 1);
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
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, 100);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
