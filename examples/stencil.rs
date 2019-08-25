use byte_strings::c_str;
use cgmath::{Matrix4, vec3, Deg, perspective, InnerSpace, Point3, SquareMatrix};
use glfw::{Action, Context, Key};

use std::str;

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

const SINGLE_COLOR_FRAGMENT_SHADER: &str = r#"
#version 330 core

out vec4 FragColor;

void main()
{
    FragColor = vec4(0.7, 0.3, 0.2, 1.0);
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

    let border_shader = unsafe {
        Shader::from_str(VERTEX_SHADER_SOURCE, SINGLE_COLOR_FRAGMENT_SHADER)
    };

    let model_obj = unsafe { 
        Model::load_obj("./examples/nanosuit/nanosuit.obj").unwrap()
    };

    let point_light_positions = [
        vec3(0.7, 0.2, 2.0),
        vec3(2.3, -3.3, -4.0),
        vec3(-4.0, 2.0, -12.0),
        vec3(0.0, 0.0, -3.0),
    ];

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::STENCIL_TEST);
        gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);
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
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

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

            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
            gl::StencilMask(0xFF);
            model_obj.draw(shader_program);

            gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);
            gl::StencilMask(0x00);
            gl::Disable(gl::DEPTH_TEST);
            border_shader.use_program();
            let scaled_model = model * Matrix4::from_scale(1.01);

            border_shader.set_matrix4(c_str!("model"), &scaled_model);
            border_shader.set_matrix4(c_str!("view"), &view);
            border_shader.set_matrix4(c_str!("projection"), &projection);

            model_obj.draw(border_shader);
            gl::StencilMask(0xFF);
            gl::Enable(gl::DEPTH_TEST);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}
