use byte_strings::c_str;
use cgmath::{Matrix4, Point3, SquareMatrix, vec3};
use glfw::{Action, Context, Key};

use std::str;

use game_engine::*;

const VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

out VS_OUT {
    vec3 normal;
} vs_out;

out vec2 texCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    mat3 normalMatrix = mat3(transpose(inverse(view * model)));
    vs_out.normal = normalize(vec3(projection * vec4(normalMatrix * aNormal, 0.0)));
    texCoords = aTexCoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

struct Material {
    sampler2D texture_diffuse1;
    sampler2D texture_specular1;
};

in vec2 texCoords;

uniform Material material;

void main()
{
    FragColor = texture(material.texture_diffuse1, texCoords);
}
"#;

const GEOMETRY_SHADER: &str = r#"
#version 330 core
layout (triangles) in;
layout (line_strip, max_vertices = 6) out;

in VS_OUT {
    vec3 normal;
} gs_in[];

void GenerateLine(int index) {
    float magnitude = 0.4;

    gl_Position = gl_in[index].gl_Position;
    EmitVertex();
    gl_Position = gl_in[index].gl_Position + vec4(gs_in[index].normal, 0.0) * magnitude;
    EmitVertex();
    
    EndPrimitive();
}

void main() {    
    GenerateLine(0);
    GenerateLine(1);
    GenerateLine(2);
}
"#;

const NORMAL_FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

void main()
{
    FragColor = vec4(1.0, 1.0, 0.0, 1.0);
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

    let model_shader = unsafe {
        Shader::from_str(VERTEX_SHADER, FRAGMENT_SHADER)
    };

    let normal_shader = unsafe {
        Shader::with_geometry_shader(VERTEX_SHADER, GEOMETRY_SHADER, NORMAL_FRAGMENT_SHADER)
    };

    let model_obj = unsafe { 
        Model::load_obj("./examples/nanosuit/nanosuit.obj").unwrap()
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
            gl::ClearColor(0.9, 0.9, 0.9, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            model_shader.use_program();
            model_shader.set_matrix4(c_str!("model"), &Matrix4::identity());
            model_shader.set_matrix4(c_str!("view"), &camera.view());
            model_shader.set_matrix4(c_str!("projection"), &camera.projection());
            model_obj.draw(model_shader);

            normal_shader.use_program();
            normal_shader.set_matrix4(c_str!("model"), &Matrix4::identity());
            normal_shader.set_matrix4(c_str!("view"), &camera.view());
            normal_shader.set_matrix4(c_str!("projection"), &camera.projection());
            model_obj.draw(normal_shader);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
