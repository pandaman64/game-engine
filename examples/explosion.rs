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
        vec2 texCoords;
    } gs_out;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    void main() {
        gl_Position = projection * view * model * vec4(aPos, 1.0);
        gs_out.texCoords = aTexCoord;
    }
"#;

const GEOMETRY_SHADER: &str = r#"
#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in VS_OUT {
    vec2 texCoords;
} gs_in[];

out vec2 texCoords;

uniform float time;

vec3 GetNormal() {
    vec3 a = vec3(gl_in[0].gl_Position) - vec3(gl_in[1].gl_Position);
    vec3 b = vec3(gl_in[2].gl_Position) - vec3(gl_in[1].gl_Position);

    return normalize(cross(a, b));
}

vec4 Explode(vec4 position, vec3 normal) {
    float magnitude = 2.0;
    vec3 diff = normal * ((sin(time) + 1.0) / 2.0) * magnitude;
    return position + vec4(diff, 0.0);
}

void main() {    
    vec3 normal = GetNormal();

    gl_Position = Explode(gl_in[0].gl_Position, normal);
    texCoords = gs_in[0].texCoords;
    EmitVertex();

    gl_Position = Explode(gl_in[1].gl_Position, normal);
    texCoords = gs_in[1].texCoords;
    EmitVertex();

    gl_Position = Explode(gl_in[2].gl_Position, normal);
    texCoords = gs_in[2].texCoords;
    EmitVertex();

    EndPrimitive();
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

            shader.use_program();
            shader.set_matrix4(c_str!("model"), &Matrix4::identity());
            shader.set_matrix4(c_str!("view"), &camera.view());
            shader.set_matrix4(c_str!("projection"), &camera.projection());
            shader.set_float(c_str!("time"), current_time);
            model_obj.draw(shader);
            gl::DrawArrays(gl::POINTS, 0, 4);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
