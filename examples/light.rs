use byte_strings::c_str;
use cgmath::{Matrix4, vec3, Deg, perspective, InnerSpace, Point3};
use gl::types::*;
use glfw::{Action, Context, Key};
use image::GenericImageView;

use std::mem;
use std::ptr;
use std::str;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aNormal;

    out vec3 FragPos;
    out vec3 Normal;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    void main() {
        gl_Position = projection * view * model * vec4(aPos, 1.0);
        FragPos = vec3(model * vec4(aPos, 1.0));
        Normal = mat3(transpose(inverse(model))) * aNormal;
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    in vec3 Normal;
    in vec3 FragPos;
    out vec4 FragColor;

    uniform vec3 cameraPos;
    uniform vec3 lightPos;
    uniform vec3 objectColor;
    uniform vec3 lightColor;

    void main()
    {
        float ambientStrength = 0.1;
        vec3 ambient = ambientStrength * lightColor;

        vec3 norm = normalize(Normal);
        vec3 lightDir = normalize(lightPos - FragPos);
        float diffuseStrength = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = diffuseStrength * lightColor;

        float specularStrength = 0.5;
        vec3 viewDir = normalize(cameraPos - FragPos);
        vec3 reflectDir = reflect(-lightDir, norm);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
        vec3 specular = specularStrength * spec * lightColor;

        FragColor = vec4((ambient + diffuse + specular) * objectColor, 1.0);
    } 
"#;

const LIGHT_VERTEX_SHADER: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 projection;

    void main() {
        gl_Position = projection * view * model * vec4(aPos, 1.0);
    }
"#;

const LIGHT_FRAGMENT_SHADER: &str = r#"
    #version 330 core
    out vec4 FragColor;

    void main()
    {
        FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

fn main() {
    let tex1 = image::open("./examples/wall.jpg").expect("failed to open texture");
    let tex2 = image::open("./examples/awesomeface.png").expect("failed to open texture");
    let tex2 = tex2.flipv();

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

    let (shader_program, vao, vbo) = unsafe {
        let shader_program =
            game_engine::Shader::from_str(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

        let vertices: [f32; 6 * 6 * 6] = [
             -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
              0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
              0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
              0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
             -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
             -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,

             -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,
              0.5, -0.5,  0.5,  0.0,  0.0,  1.0,
              0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
              0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
             -0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
             -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,

             -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,
             -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,
             -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
             -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
             -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,
             -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,

              0.5,  0.5,  0.5,  1.0,  0.0,  0.0,
              0.5,  0.5, -0.5,  1.0,  0.0,  0.0,
              0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
              0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
              0.5, -0.5,  0.5,  1.0,  0.0,  0.0,
              0.5,  0.5,  0.5,  1.0,  0.0,  0.0,

             -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
              0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
              0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
              0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
             -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
             -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,

             -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
              0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
              0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
              0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
             -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
             -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
        ];
        let mut vbo = 0;
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            (3 * mem::size_of::<GLfloat>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        (shader_program, vao, vbo)
    };

    let light_shader = unsafe {
        game_engine::Shader::from_str(LIGHT_VERTEX_SHADER, LIGHT_FRAGMENT_SHADER)
    };

    let light_vao = unsafe {
        let mut light_vao = 0;
        gl::GenVertexArrays(1, &mut light_vao);
        gl::BindVertexArray(light_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        light_vao
    };

    let tex1 = unsafe {
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            tex1.width() as i32,
            tex1.height() as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            tex1.raw_pixels().as_ptr() as *const _,
        );

        gl::GenerateMipmap(gl::TEXTURE_2D);

        texture
    };
    let tex2 = unsafe {
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            tex2.width() as i32,
            tex2.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            tex2.raw_pixels().as_ptr() as *const _,
        );

        gl::GenerateMipmap(gl::TEXTURE_2D);

        texture
    };

    let positions = [
        vec3(0.0, 0.0, 0.0),
        vec3(2.0, 5.0, -15.0),
        vec3(-1.5, -2.2, -2.5),
        vec3(-3.8, -2.0, -12.3),
        vec3(2.4, -0.4, -3.5),
        vec3(-1.7, 3.0, -7.5),
        vec3(1.3, -2.0, -2.5),
        vec3(1.5, 2.0, -2.5),
        vec3(1.5, 0.2, -1.5),
        vec3(-1.3, 1.0, -1.5),
    ];

    let mut light_pos = vec3(1.2, 1.0, 2.0);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
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

        light_pos.x = 1.2 + current_time.cos() * 1.3;

        let view = Matrix4::<f32>::look_at_dir(camera_pos, camera_dir, camera_up);
        let projection = perspective(Deg(fov), 800.0 as f32 / 600.0 as f32, 0.1, 100.0);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader_program.use_program();
            shader_program.set_vec3(c_str!("objectColor"), 1.0, 0.5, 0.31);
            shader_program.set_vec3(c_str!("lightColor"), 1.0, 1.0, 1.0);
            shader_program.set_vec3(c_str!("lightPos"), light_pos.x, light_pos.y, light_pos.z);
            shader_program.set_vec3(c_str!("cameraPos"), camera_pos.x, camera_pos.y, camera_pos.z);

            shader_program.set_matrix4(c_str!("view"), &view);
            shader_program.set_matrix4(c_str!("projection"), &projection);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, tex2);

            gl::BindVertexArray(vao);
            for (i, position) in positions.iter().enumerate() {
                let mut model = Matrix4::<f32>::from_translation(*position);
                model = model * Matrix4::<f32>::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg((i as f32 + glfw.get_time() as f32) * 20.0));
                shader_program.set_matrix4(c_str!("model"), &model);
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

            light_shader.use_program();
            light_shader.set_matrix4(c_str!("view"), &view);
            light_shader.set_matrix4(c_str!("projection"), &projection);
            let mut model = Matrix4::<f32>::from_translation(light_pos);
            model = model * Matrix4::<f32>::from_scale(0.2);
            light_shader.set_matrix4(c_str!("model"), &model);

            gl::BindVertexArray(light_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
