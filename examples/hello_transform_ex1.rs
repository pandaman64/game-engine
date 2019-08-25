use cgmath::{Matrix4, Rad, SquareMatrix, vec3};
use gl::types::*;
use glfw::{Action, Context, Key};
use image::GenericImageView;

use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aColor;
    layout (location = 2) in vec2 aTexCoord;

    out vec3 ourColor;
    out vec2 TexCoord;

    uniform mat4 transform;

    void main() {
        gl_Position = transform * vec4(aPos, 1.0);
        ourColor = aColor;
        TexCoord = aTexCoord;
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    in vec3 ourColor;
    in vec2 TexCoord;

    uniform sampler2D texture1;
    uniform sampler2D texture2;

    void main()
    {
        FragColor = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), 0.2);
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
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader_program, vao) = unsafe {
        let shader_program =
            game_engine::Shader::from_str(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

        let verticies: [f32; (3 + 3 + 2) * 4] = [
            // position      // color       // texture
             0.5,  0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
             0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, // bottom right
            -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, // bottom left
            -0.5,  0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, // bottom right
        ];
        let indices: [u32; 6] = [
            0, 1, 3, // first triangle
            1, 2, 3, // second triangle
        ];
        let mut vbo = 0;
        let mut vao = 0;
        let mut ebo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (verticies.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            verticies.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
            indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * mem::size_of::<GLfloat>() as GLsizei,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            8 * mem::size_of::<GLfloat>() as GLsizei,
            (6 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        (shader_program, vao)
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

        let mut transform1 = Matrix4::<f32>::identity();
        transform1 = transform1 * Matrix4::<f32>::from_translation(vec3(0.5, -0.5, 0.5));
        transform1 = transform1 * Matrix4::<f32>::from_angle_z(Rad((glfw.get_time().sin() * std::f64::consts::PI) as f32));

        let mut transform2 = Matrix4::<f32>::identity();
        transform2 = transform2 * Matrix4::<f32>::from_translation(vec3(-0.5, 0.5, 0.5));
        transform2 = transform2 * Matrix4::<f32>::from_scale(glfw.get_time().sin() as f32 + 1.0);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            shader_program.use_program();
            shader_program.set_integer("texture1", 0);
            shader_program.set_integer("texture2", 1);
            shader_program.set_matrix4("transform", &transform1);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, tex2);

            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

            shader_program.set_matrix4("transform", &transform2);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
