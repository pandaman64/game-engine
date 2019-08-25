use byte_strings::c_str;
use cgmath::{Matrix4, vec3, Deg, perspective, InnerSpace, Point3, MetricSpace, SquareMatrix, EuclideanSpace};
use glfw::{Action, Context, Key};

use std::str;
use std::mem;
use std::ptr;

use game_engine::*;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core

    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec2 aTexCoord;

    out vec2 TexCoords;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    void main() {
        TexCoords = aTexCoord;
        gl_Position = projection * view * model * vec4(aPos, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;

    in vec2 TexCoords;

    uniform sampler2D texture1;

    void main() {
        vec4 texColor = texture(texture1, TexCoords);
        FragColor = texColor;
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

    let shader_program = unsafe {
        let shader_program = Shader::from_str(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        shader_program.use_program();
        shader_program.set_integer(c_str!("texture1"), 0);
        shader_program
    };
    
    let (cube_vao, plane_vao, transparent_vao) = unsafe {
        let cube_vertices: [f32; 180] = [
             // positions       // texture Coords
             -0.5, -0.5, -0.5,  0.0, 0.0,
              0.5, -0.5, -0.5,  1.0, 0.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
             -0.5,  0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 0.0,

             -0.5, -0.5,  0.5,  0.0, 0.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 1.0,
              0.5,  0.5,  0.5,  1.0, 1.0,
             -0.5,  0.5,  0.5,  0.0, 1.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,

             -0.5,  0.5,  0.5,  1.0, 0.0,
             -0.5,  0.5, -0.5,  1.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,
             -0.5,  0.5,  0.5,  1.0, 0.0,

              0.5,  0.5,  0.5,  1.0, 0.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5,  0.5,  0.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 0.0,

             -0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5, -0.5,  1.0, 1.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,

             -0.5,  0.5, -0.5,  0.0, 1.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5,  0.5,  0.5,  1.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 0.0,
             -0.5,  0.5,  0.5,  0.0, 0.0,
             -0.5,  0.5, -0.5,  0.0, 1.0
        ];
        
        let plane_vertices: [f32; 30] = [
            // positions       // texture Coords (note we set these higher than 1 (together with GL_REPEAT as texture wrapping mode). this will cause the floor texture to repeat)
             5.0, -0.5,  5.0,  2.0, 0.0,
            -5.0, -0.5, -5.0,  0.0, 2.0,
            -5.0, -0.5,  5.0,  0.0, 0.0,

             5.0, -0.5,  5.0,  2.0, 0.0,
             5.0, -0.5, -5.0,  2.0, 2.0,
            -5.0, -0.5, -5.0,  0.0, 2.0,
        ];

        let transparent_vertices: [f32; 30] = [
            // positions      // texture Coords (swapped y coordinates because texture is flipped upside down)
            0.0,  0.5,  0.0,  0.0,  0.0,
            0.0, -0.5,  0.0,  0.0,  1.0,
            1.0, -0.5,  0.0,  1.0,  1.0,

            0.0,  0.5,  0.0,  0.0,  0.0,
            1.0, -0.5,  0.0,  1.0,  1.0,
            1.0,  0.5,  0.0,  1.0,  0.0
        ];

        let stride = conv!(mem::size_of::<f32>() * 5);
        let offset = (3 * mem::size_of::<f32>()) as *const _;

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
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset,
        );
        gl::BindVertexArray(0);

        let mut plane_vao = 0;
        let mut plane_vbo = 0;
        gl::GenVertexArrays(1, &mut plane_vao);
        gl::GenBuffers(1, &mut plane_vbo);
        gl::BindVertexArray(plane_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, plane_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mem::size_of_val(&plane_vertices)),
            plane_vertices.as_ptr() as *const _,
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
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset,
        );
        gl::BindVertexArray(0);

        let mut transparent_vao = 0;
        let mut transparent_vbo = 0;
        gl::GenVertexArrays(1, &mut transparent_vao);
        gl::GenBuffers(1, &mut transparent_vbo);
        gl::BindVertexArray(transparent_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, transparent_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mem::size_of_val(&transparent_vertices)),
            transparent_vertices.as_ptr() as *const _,
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
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset,
        );
        gl::BindVertexArray(0);

        (cube_vao, plane_vao, transparent_vao)
    };

    let cube_texture = unsafe { load_texture("./examples/marble.jpg") };
    let floor_texture = unsafe { load_texture("./examples/metal.png") };
    let transparent_texture = unsafe { load_texture("./examples/blending_transparent_window.png") };
    
    let mut vegetation = [
        vec3(-1.5, 0.0, -0.48),
        vec3( 1.5, 0.0, 0.51),
        vec3( 0.0, 0.0, 0.7),
        vec3(-0.3, 0.0, -2.3),
        vec3 (0.5, 0.0, -0.6)
    ];

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);
    }

    const SPEED: f32 = 5.0;
    let mut camera_pos = Point3::new(0.0, 0.0, 3.0);
    let mut camera_dir = vec3(0.0, 0.0, -1.0);
    let camera_up = vec3(0.0, 1.0, 0.0);

    let mut last_time = glfw.get_time() as f32;
    let mut delta_time;

    let mut fov: f32 = 45.0;
    let mut yaw: f32 = -90.0;
    let mut pitch: f32 = 0.0;
    let mut last_x: f32 = 400.0;
    let mut last_y: f32 = 300.0;
    let mut first_mouse = true;

    while !window.should_close() {
        let current_time = glfw.get_time() as f32;
        delta_time = current_time - last_time;
        last_time = current_time;

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                    gl::Viewport(0, 0, width, height);
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::CursorPos(xpos, ypos) => {
                    if first_mouse {
                        first_mouse = false;
                        last_x = xpos as f32;
                        last_y = ypos as f32;
                        continue;
                    }
                    let xoffset = xpos as f32 - last_x;
                    let yoffset = last_y - ypos as f32;

                    last_x = xpos as f32;
                    last_y = ypos as f32;

                    const SENSITIVITY: f32 = 0.05;
                    yaw += xoffset * SENSITIVITY;
                    pitch += yoffset * SENSITIVITY;

                    if pitch > 89.0 {
                        pitch = 89.0;
                    }
                    if pitch < -89.0 {
                        pitch = -89.0;
                    }

                    camera_dir.x = pitch.to_radians().cos() * yaw.to_radians().cos();
                    camera_dir.y = pitch.to_radians().sin();
                    camera_dir.z = pitch.to_radians().cos() * yaw.to_radians().sin();
                    camera_dir = camera_dir.normalize();
                }
                glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
                    if fov >= 1.0 && fov <= 45.0 {
                        fov -= yoffset as f32;
                    }
                    if fov <= 1.0 {
                        fov = 1.0;
                    }
                    if fov >= 45.0 {
                        fov = 45.0;
                    }
                }
                _ => {}
            }
        }

        if window.get_key(Key::W) == Action::Press {
            camera_pos += camera_dir * SPEED * delta_time;
        }
        if window.get_key(Key::S) == Action::Press {
            camera_pos -= camera_dir * SPEED * delta_time;
        }
        if window.get_key(Key::D) == Action::Press {
            camera_pos += camera_dir.cross(camera_up).normalize() * SPEED * delta_time;
        }
        if window.get_key(Key::A) == Action::Press {
            camera_pos -= camera_dir.cross(camera_up).normalize() * SPEED * delta_time;
        }

        // sort transparent objects
        vegetation.sort_by(|p1, p2| {
            let p = camera_pos.to_vec();
            p.distance2(*p1).partial_cmp(&p.distance2(*p2)).unwrap().reverse()
        });

        let view = Matrix4::<f32>::look_at_dir(camera_pos, camera_dir, camera_up);
        let projection = perspective(Deg(fov), 800.0 as f32 / 600.0 as f32, 0.1, 100.0);

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader_program.use_program();
            shader_program.set_matrix4(c_str!("view"), &view);
            shader_program.set_matrix4(c_str!("projection"), &projection);
        
            // cubes
            gl::BindVertexArray(cube_vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, cube_texture);
            shader_program.set_matrix4(c_str!("model"), &Matrix4::from_translation(vec3(-1.0, 0.0, -1.0)));
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            shader_program.set_matrix4(c_str!("model"), &Matrix4::from_translation(vec3(2.0, 0.0, 0.0)));
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // floor
            gl::BindVertexArray(plane_vao);
            gl::BindTexture(gl::TEXTURE_2D, floor_texture);
            shader_program.set_matrix4(c_str!("model"), &Matrix4::identity());
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            
            gl::BindVertexArray(transparent_vao);
            gl::BindTexture(gl::TEXTURE_2D, transparent_texture);
            for v in vegetation.iter() {
                shader_program.set_matrix4(c_str!("model"), &Matrix4::from_translation(*v));
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
