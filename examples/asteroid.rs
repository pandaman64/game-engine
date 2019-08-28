use byte_strings::c_str;
use cgmath::{InnerSpace, Matrix4, Point3, Rad, vec3};
use glfw::{Action, Context, Key};
use rand::Rng;

use std::ptr;
use std::mem;
use std::str;

use game_engine::*;

const PLANET_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;
    TexCoords = aTexCoord;
}
"#;

const ROCK_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;
// this matrix occupies location 3, 4, 5, 6
layout (location = 3) in mat4 instanceMatrix;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords;

uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * instanceMatrix * vec4(aPos, 1.0);
    FragPos = vec3(instanceMatrix * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(instanceMatrix))) * aNormal;
    TexCoords = aTexCoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 330 core

struct Material {
    sampler2D texture_diffuse1;
    sampler2D texture_specular1;
    float shininess;
};

in vec2 TexCoords;

out vec4 FragColor;

uniform Material material;

void main() {
    FragColor = texture(material.texture_diffuse1, TexCoords);
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

    let planet_shader = unsafe {
        Shader::from_str(PLANET_VERTEX_SHADER, FRAGMENT_SHADER)
    };

    let rock_shader = unsafe {
        Shader::from_str(ROCK_VERTEX_SHADER, FRAGMENT_SHADER)
    };

    let planet = unsafe {
        Model::load_obj("./examples/planet/planet.obj").unwrap()
    };

    let rock = unsafe {
        Model::load_obj("./examples/rock/rock.obj").unwrap()
    };

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let model_matrices = {
        const LEN: usize = 100000;
        const RADIUS: f32 = 25.0;
        const OFFSET: f32 = 5.5;

        let mut model_matrices = Vec::with_capacity(LEN);

        let mut rng = rand::thread_rng();
        for i in 0..LEN {
            // translate
            let angle = (i as f32 / (LEN as f32) * 360.0).to_radians();
            let x = RADIUS * angle.sin() + rng.gen_range(-OFFSET, OFFSET);
            let z = RADIUS * angle.cos() + rng.gen_range(-OFFSET, OFFSET);
            let y = rng.gen_range(-OFFSET, OFFSET) * 0.4;
            let mut matrix = Matrix4::from_translation(vec3(x, y, z));

            // scale
            let scale = rng.gen_range(0.05, 0.25);
            matrix = matrix * Matrix4::from_scale(scale);

            // rotation
            let angle = rng.gen_range(0.0, std::f32::consts::PI * 2.0);
            matrix = matrix * Matrix4::from_axis_angle(vec3(0.4, 0.6, 0.8).normalize(), Rad(angle));
            model_matrices.push(matrix);
        }

        model_matrices
    };

    unsafe {
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mem::size_of::<Matrix4<f32>>() * model_matrices.len()),
            model_matrices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        for mesh in rock.meshes() {
            let vao = mesh.vao();
            gl::BindVertexArray(vao);

            let mat4size: gl::types::GLsizei = conv!(mem::size_of::<Matrix4<f32>>());
            let vec4size: gl::types::GLsizei = conv!(mem::size_of::<f32>() * 4);

            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE, mat4size, ptr::null());
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(4, 4, gl::FLOAT, gl::FALSE, mat4size, vec4size as *const _);
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribPointer(5, 4, gl::FLOAT, gl::FALSE, mat4size, (2 * vec4size) as *const _);
            gl::EnableVertexAttribArray(6);
            gl::VertexAttribPointer(6, 4, gl::FLOAT, gl::FALSE, mat4size, (3 * vec4size) as *const _);

            gl::VertexAttribDivisor(3, 1);
            gl::VertexAttribDivisor(4, 1);
            gl::VertexAttribDivisor(5, 1);
            gl::VertexAttribDivisor(6, 1);

            gl::BindVertexArray(0);
        }

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
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
        unsafe {
            let error = gl::GetError();
            if error != 0 {
                log::error!("GL error: {}", error);
            }
        }

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

            planet_shader.use_program();
            planet_shader.set_matrix4(c_str!("model"), &Matrix4::from_scale(4.0));
            planet_shader.set_matrix4(c_str!("projection"), &camera.projection());
            planet_shader.set_matrix4(c_str!("view"), &camera.view());
            planet.draw(planet_shader);

            rock_shader.set_matrix4(c_str!("projection"), &camera.projection());
            rock_shader.set_matrix4(c_str!("view"), &camera.view());
            
            for mesh in rock.meshes() {
                mesh.draw_instanced(rock_shader, conv!(model_matrices.len()));
            }
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
