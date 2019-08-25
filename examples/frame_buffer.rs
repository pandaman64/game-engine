use byte_strings::c_str;
use cgmath::{Matrix4, vec3, Deg, perspective, InnerSpace, Point3, SquareMatrix};
use glfw::{Action, Context, Key};

use std::str;
use std::mem;
use std::ptr;

use game_engine::*;

const VERTEX_SHADER_SOURCE: &str = r#"
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

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    struct Material {
        sampler2D texture_diffuse1;
        sampler2D texture_specular1;
        float shininess;
    };

    struct DirectionalLight {
        vec3 direction;

        vec3 ambient;
        vec3 diffuse;
        vec3 specular;
    };

    struct PointLight {
        vec3 position;

        vec3 ambient;
        vec3 diffuse;
        vec3 specular;

        float constant;
        float linear;
        float quadratic;
    };

    struct SpotLight {
        vec3 position;
        vec3 direction;

        // in cosine
        float cutOff;
        float outerCutOff;

        vec3 ambient;
        vec3 diffuse;
        vec3 specular;
    };

    in vec2 TexCoords;
    in vec3 Normal;
    in vec3 FragPos;
    out vec4 FragColor;

    uniform vec3 cameraPos;

    uniform Material material;

    uniform DirectionalLight dirLight;
    uniform PointLight pointLights[4];
    uniform SpotLight spotLight;

    vec3 CalcDirectionalLight(DirectionalLight light, vec3 normal, vec3 viewDir) {
        vec3 ambient = light.ambient * texture(material.texture_diffuse1, TexCoords).rgb;

        vec3 lightDir = normalize(-light.direction);
        float diff = max(dot(normal, lightDir), 0.0);
        vec3 diffuse = light.diffuse * diff * texture(material.texture_diffuse1, TexCoords).rgb;

        vec3 reflectDir = reflect(-lightDir, normal);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
        vec3 specular = light.specular * spec * texture(material.texture_specular1, TexCoords).rgb;

        return ambient + diffuse + specular;
    }

    vec3 CalcPointLight(PointLight light, vec3 normal, vec3 viewDir) {
        float distance = length(light.position - FragPos);
        float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * distance * distance);

        vec3 ambient = attenuation * light.ambient * texture(material.texture_diffuse1, TexCoords).rgb;

        vec3 lightDir = normalize(FragPos - light.position);
        float diff = max(dot(normal, lightDir), 0.0);
        vec3 diffuse = attenuation * light.diffuse * diff * texture(material.texture_diffuse1, TexCoords).rgb;

        vec3 reflectDir = reflect(-lightDir, normal);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
        vec3 specular = attenuation * light.specular * spec * texture(material.texture_specular1, TexCoords).rgb;

        return ambient + diffuse + specular;
    }

    vec3 CalcSpotLight(SpotLight light, vec3 normal, vec3 viewDir) {
        vec3 lightDir = normalize(light.position - FragPos);
        float theta = dot(-lightDir, normalize(light.direction));
        float epsilon = light.cutOff - light.outerCutOff;
        float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);

        vec3 ambient = light.ambient * texture(material.texture_diffuse1, TexCoords).rgb;

        float diff = max(dot(normal, lightDir), 0.0);
        vec3 diffuse = intensity * light.diffuse * diff * texture(material.texture_diffuse1, TexCoords).rgb;

        vec3 reflectDir = reflect(-lightDir, normal);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
        vec3 specular = intensity * light.specular * spec * texture(material.texture_specular1, TexCoords).rgb;

        return ambient + diffuse + specular;
    }

    void main()
    {
        vec3 norm = normalize(Normal);
        vec3 viewDir = normalize(cameraPos - FragPos);

        vec3 result = CalcDirectionalLight(dirLight, norm, viewDir);
        for (int i = 0; i < 4; i++) {
            result += CalcPointLight(pointLights[i], norm, viewDir);
        }
        result += CalcSpotLight(spotLight, norm, viewDir);

        FragColor = vec4(result, 1.0);
    } 
"#;

const QUAD_VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoords;

out vec2 TexCoords;

void main() {
    gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
    TexCoords = aTexCoords;
}
"#;

const QUAD_FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;

const float offset = 1.0 / 300.0;

void main() {
    vec2 offsets[9] = vec2[](
        vec2(-offset,  offset), // top-left
        vec2( 0.0f,    offset), // top-center
        vec2( offset,  offset), // top-right
        vec2(-offset,  0.0f),   // center-left
        vec2( 0.0f,    0.0f),   // center-center
        vec2( offset,  0.0f),   // center-right
        vec2(-offset, -offset), // bottom-left
        vec2( 0.0f,   -offset), // bottom-center
        vec2( offset, -offset)  // bottom-right    
    );

    float kernel[9] = float[](
        1.0,  1.0,  1.0,
        1.0, -8.0,  1.0,
        1.0,  1.0,  1.0
    );
    vec3 sampleTex[9];
    for (int i = 0; i < 9; i++) {
        sampleTex[i] = vec3(texture(screenTexture, TexCoords + offsets[i]));
    }

    vec3 color = vec3(0.0, 0.0, 0.0);
    for (int i = 0; i < 9; i++) {
        color += sampleTex[i] * kernel[i];
    }

    FragColor = vec4(color, 1.0);
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
        Shader::from_str(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)
    };

    let screen_shader = unsafe {
        Shader::from_str(QUAD_VERTEX_SHADER, QUAD_FRAGMENT_SHADER)
    };

    let model_obj = unsafe { 
        Model::load_obj("./examples/nanosuit/nanosuit.obj").unwrap()
    };

    let quad_vao = unsafe {
        let quad_vertices: [f32; 4 * 6] = [
            -1.0,  1.0,  0.0, 1.0,
            -1.0, -1.0,  0.0, 0.0,
             1.0, -1.0,  1.0, 0.0,
            
            -1.0,  1.0,  0.0, 1.0,
             1.0, -1.0,  1.0, 0.0,
             1.0,  1.0,  1.0, 1.0,
        ];

        let stride = conv!(4 * mem::size_of::<f32>());

        let mut quad_vao = 0;
        let mut quad_vbo = 0;
        gl::GenVertexArrays(1, &mut quad_vao);
        gl::GenBuffers(1, &mut quad_vbo);
        gl::BindVertexArray(quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mem::size_of_val(&quad_vertices)),
            quad_vertices.as_ptr() as *const _,
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
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (2 * mem::size_of::<f32>()) as *const _,
        );

        quad_vao
    };

    let (frame_buffer, tex_color_buffer) = unsafe {
        let mut frame_buffer = 0;
        gl::GenFramebuffers(1, &mut frame_buffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer);

        let mut tex_color_buffer = 0;
        gl::GenTextures(1, &mut tex_color_buffer);
        gl::BindTexture(gl::TEXTURE_2D, tex_color_buffer);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            conv!(gl::RGB),
            800,
            600,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, conv!(gl::LINEAR));
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, conv!(gl::LINEAR));
        gl::BindTexture(gl::TEXTURE_2D, 0);

        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_color_buffer, 0);

        let mut rbo = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, 800, 600);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            log::warn!("frame buffer is not complete");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        (frame_buffer, tex_color_buffer)
    };

    let point_light_positions = [
        vec3(0.7, 0.2, 2.0),
        vec3(2.3, -3.3, -4.0),
        vec3(-4.0, 2.0, -12.0),
        vec3(0.0, 0.0, -3.0),
    ];

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

        let model = Matrix4::<f32>::identity();
        let view = Matrix4::<f32>::look_at_dir(camera_pos, camera_dir, camera_up);
        let projection = perspective(Deg(fov), 800.0 as f32 / 600.0 as f32, 0.1, 100.0);

        unsafe {
            // first pass
            gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer);

            gl::ClearColor(0.8, 0.8, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);

            shader_program.use_program();
            shader_program.set_vec3(c_str!("cameraPos"), camera_pos.x, camera_pos.y, camera_pos.z);

            shader_program.set_float(c_str!("material.shininess"), 32.0);

            //let light_color = vec3((current_time * 2.0).sin(), (current_time * 0.7).sin(), (current_time * 1.3).sin());
            let light_color = vec3(1.0, 1.0, 1.0);
            let diffuse_color = light_color * 0.5;
            let ambient_color = light_color * 0.2;
            shader_program.set_vec3(c_str!("spotLight.position"), camera_pos.x, camera_pos.y, camera_pos.z);
            shader_program.set_vec3(c_str!("spotLight.direction"), camera_dir.x, camera_dir.y, camera_dir.z);
            shader_program.set_float(c_str!("spotLight.cutOff"), 12.5_f32.to_radians().cos());
            shader_program.set_float(c_str!("spotLight.outerCutOff"), 17.5_f32.to_radians().cos());
            shader_program.set_vec3(c_str!("spotLight.ambient"), ambient_color.x, ambient_color.y, ambient_color.z);
            shader_program.set_vec3(c_str!("spotLight.diffuse"), diffuse_color.x, diffuse_color.y, diffuse_color.z);
            shader_program.set_vec3(c_str!("spotLight.specular"), 1.0, 1.0, 1.0);

            shader_program.set_vec3(c_str!("pointLights[0].position"), point_light_positions[0].x,  point_light_positions[0].y, point_light_positions[0].z);
            shader_program.set_vec3(c_str!("pointLights[0].ambient"), 0.1, 0.1, 0.1);
            shader_program.set_vec3(c_str!("pointLights[0].diffuse"), 0.8, 0.8, 0.8);
            shader_program.set_vec3(c_str!("pointLights[0].specular"), 1.0, 1.0, 1.0);
            shader_program.set_float(c_str!("pointLights[0].constant"), 1.0);
            shader_program.set_float(c_str!("pointLights[0].linear"), 0.09);
            shader_program.set_float(c_str!("pointLights[0].quadratic"), 0.032);

            shader_program.set_vec3(c_str!("pointLights[1].position"), point_light_positions[1].x,  point_light_positions[1].y, point_light_positions[1].z);
            shader_program.set_vec3(c_str!("pointLights[1].ambient"), 0.1, 0.1, 0.1);
            shader_program.set_vec3(c_str!("pointLights[1].diffuse"), 0.8, 0.8, 0.8);
            shader_program.set_vec3(c_str!("pointLights[1].specular"), 1.0, 1.0, 1.0);
            shader_program.set_float(c_str!("pointLights[1].constant"), 1.0);
            shader_program.set_float(c_str!("pointLights[1].linear"), 0.09);
            shader_program.set_float(c_str!("pointLights[1].quadratic"), 0.032);

            shader_program.set_vec3(c_str!("pointLights[2].position"), point_light_positions[2].x,  point_light_positions[2].y, point_light_positions[2].z);
            shader_program.set_vec3(c_str!("pointLights[2].ambient"), 0.1, 0.1, 0.1);
            shader_program.set_vec3(c_str!("pointLights[2].diffuse"), 0.8, 0.8, 0.8);
            shader_program.set_vec3(c_str!("pointLights[2].specular"), 1.0, 1.0, 1.0);
            shader_program.set_float(c_str!("pointLights[2].constant"), 1.0);
            shader_program.set_float(c_str!("pointLights[2].linear"), 0.09);
            shader_program.set_float(c_str!("pointLights[2].quadratic"), 0.032);

            shader_program.set_vec3(c_str!("pointLights[3].position"), point_light_positions[3].x,  point_light_positions[3].y, point_light_positions[3].z);
            shader_program.set_vec3(c_str!("pointLights[3].ambient"), 0.05, 0.05, 0.05);
            shader_program.set_vec3(c_str!("pointLights[3].diffuse"), 0.8, 0.8, 0.8);
            shader_program.set_vec3(c_str!("pointLights[3].specular"), 1.0, 1.0, 1.0);
            shader_program.set_float(c_str!("pointLights[3].constant"), 1.0);
            shader_program.set_float(c_str!("pointLights[3].linear"), 0.09);
            shader_program.set_float(c_str!("pointLights[3].quadratic"), 0.032);

            shader_program.set_matrix4(c_str!("model"), &model);
            shader_program.set_matrix4(c_str!("view"), &view);
            shader_program.set_matrix4(c_str!("projection"), &projection);

            model_obj.draw(shader_program);

            // second pass
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            screen_shader.use_program();
            gl::BindVertexArray(quad_vao);
            gl::Disable(gl::DEPTH_TEST);
            gl::BindTexture(gl::TEXTURE_2D, tex_color_buffer);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
