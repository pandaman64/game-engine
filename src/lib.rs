use std::ffi::{CStr, CString};
use std::ptr;
use std::path::Path;
use std::str;
use std::error::Error;
use std::mem;

use cgmath::{Matrix, Matrix4, Vector2, Vector3, vec2, vec3};
use gl::types::*;
use image::{open, DynamicImage::*, GenericImageView};

#[macro_export]
macro_rules! conv {
    ($e:expr) => {
        std::convert::TryInto::try_into($e).expect("failed to convert")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Shader {
    id: GLuint,
}

#[derive(Debug)]
pub struct CreateShaderError {
    message: String,
}

impl std::fmt::Display for CreateShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to create a shader: {}", self.message)
    }
}

struct DeleteShaderOnDrop(GLuint);

impl Drop for DeleteShaderOnDrop {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

impl Shader {
    pub unsafe fn from_str(vertex: &str, fragment: &str) -> Self {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let vertex = CString::new(vertex.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &vertex.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut info_log = vec![0; 512];
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            let pos = info_log.iter().position(|&x| x == 0).unwrap();
            panic!(
                "failed to compile vertex shader: {}",
                CStr::from_bytes_with_nul(&info_log[0..(pos + 1)])
                    .unwrap()
                    .to_string_lossy()
            );
        }
        let _drop_vertex_shader = DeleteShaderOnDrop(vertex_shader);

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fragment = CString::new(fragment.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &fragment.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut info_log = vec![0; 512];
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            let pos = info_log.iter().position(|&x| x == 0).unwrap();
            panic!(
                "failed to compile fragment shader: {}",
                CStr::from_bytes_with_nul(&info_log[0..(pos + 1)])
                    .unwrap()
                    .to_string_lossy()
            );
        }
        let _drop_fragment_shader = DeleteShaderOnDrop(fragment_shader);

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut info_log = vec![0; 512];
            gl::GetProgramInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            let pos = info_log.iter().position(|&x| x == 0).unwrap();
            panic!(
                "failed to link program: {}",
                CStr::from_bytes_with_nul(&info_log[0..(pos + 1)])
                    .unwrap()
                    .to_string_lossy()
            );
        }

        Self { id: shader_program }
    }

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    unsafe fn get_uniform_location(&self, name: &CStr) -> GLint {
        let result = gl::GetUniformLocation(self.id, name.as_ptr());
        if result == -1 {
            log::warn!("failed to retrieve uniform location: {}", name.to_string_lossy());
        }
        result
    }

    pub unsafe fn set_float(&self, name: &CStr, value: f32) {
        gl::Uniform1f(self.get_uniform_location(name), value);
    }

    pub unsafe fn set_integer(&self, name: &CStr, value: i32) {
        gl::Uniform1i(self.get_uniform_location(name), value);
    }

    pub unsafe fn set_matrix4(&self, name: &CStr, mat: &Matrix4<f32>) {
        gl::UniformMatrix4fv(self.get_uniform_location(name), 1, gl::FALSE, mat.as_ptr());
    }

    pub unsafe fn set_vec3(&self, name: &CStr, x: f32, y: f32, z: f32) {
        gl::Uniform3f(self.get_uniform_location(name), x, y, z);
    }
}

pub unsafe fn load_texture<P: AsRef<Path>>(path: P) -> GLuint {
    let img = open(path).expect("failed to open image file");

    let mut texture = 0;
    gl::GenTextures(1, &mut texture);

    let pixels = img.raw_pixels();

    let format = match img {
        ImageRgb8(_) => gl::RGB,
        ImageRgba8(_) => gl::RGBA,
        img => unimplemented!("image type not supported: {:?}", img.color()),
    };

    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        format as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        format,
        gl::UNSIGNED_BYTE,
        pixels.as_ptr() as *const _,
    );

    gl::GenerateMipmap(gl::TEXTURE_2D);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, if format == gl::RGBA { gl::CLAMP_TO_EDGE } else { gl::REPEAT } as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, if format == gl::RGBA { gl::CLAMP_TO_EDGE } else { gl::REPEAT } as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    texture
}

pub unsafe fn load_cubemap<P: AsRef<Path>>(paths: &[P]) -> GLuint {
    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);

    for (i, path) in paths.iter().enumerate() {
        let img = open(path).expect("failed to open image file");
        
        let pixels = img.raw_pixels();

        let format = match img {
            ImageRgb8(_) => gl::RGB,
            img => unimplemented!("image type not supported: {:?}", img.color()),
        };

        let i: u32 = conv!(i);
        gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
            0,
            conv!(format),
            conv!(img.width()),
            conv!(img.height()),
            0,
            format,
            gl::UNSIGNED_BYTE,
            pixels.as_ptr() as *const _,
        );
    }

    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, conv!(gl::LINEAR));
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, conv!(gl::LINEAR));
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, conv!(gl::CLAMP_TO_EDGE));
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, conv!(gl::CLAMP_TO_EDGE));
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, conv!(gl::CLAMP_TO_EDGE));

    texture
}

// not sure about alignment
#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coords: Vector2<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureType {
    Diffuse,
    Specular,
}

#[derive(Debug, Clone, Copy)]
pub struct Texture {
    id: GLuint,
    type_: TextureType,
}

impl Texture {
    pub unsafe fn new<P: AsRef<Path>>(path: P, type_: TextureType) -> Self {
        Self {
            id: load_texture(path),
            type_,
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub verticies: Vec<Vertex>,
    pub indices: Vec<GLuint>,
    pub textures: Vec<Texture>,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

impl Mesh {
    pub unsafe fn new(verticies: Vec<Vertex>, indices: Vec<GLuint>, textures: Vec<Texture>) -> Self {
        // require a vertex is tightly packed
        let vertex_size = mem::size_of::<Vertex>();
        assert!(vertex_size == mem::size_of::<f32>() * 8, "size of vertex is: {}", vertex_size);

        let mut mesh = Mesh {
            verticies,
            indices,
            textures,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        gl::GenVertexArrays(1, &mut mesh.vao);
        gl::GenBuffers(1, &mut mesh.vbo);
        gl::GenBuffers(1, &mut mesh.ebo);

        gl::BindVertexArray(mesh.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            conv!(mesh.verticies.len() * vertex_size),
            mesh.verticies.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            conv!(mesh.indices.len() * mem::size_of::<GLuint>()),
            mesh.indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // position
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0, 
            3, 
            gl::FLOAT, 
            gl::FALSE, 
            conv!(vertex_size),
            ptr::null(),
        );

        // normal
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            conv!(vertex_size),
            (3 * mem::size_of::<f32>()) as *const _,
        );

        // texture coordinate
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            conv!(vertex_size),
            (6 * mem::size_of::<f32>()) as *const _,
        );

        // reset global vao
        gl::BindVertexArray(0);

        mesh
    }

    pub unsafe fn draw(&self, shader: Shader) {
        let mut diffuse_num = 0;
        let mut specular_num = 0;

        for (i, texture) in self.textures.iter().enumerate() {
            let i: GLuint = conv!(i);
            gl::ActiveTexture(gl::TEXTURE0 + i);

            match texture.type_ {
                TextureType::Diffuse => {
                    diffuse_num += 1;
                    let name = CString::new(format!("material.texture_diffuse{}", diffuse_num)).unwrap();
                    shader.set_integer(name.as_ref(), conv!(i));
                }
                TextureType::Specular => {
                    specular_num += 1;
                    let name = CString::new(format!("material.texture_specular{}", specular_num)).unwrap();
                    shader.set_integer(name.as_ref(), conv!(i));
                }
            }

            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }

        // reset active texture: needed?
        gl::ActiveTexture(gl::TEXTURE0);

        // draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElements(gl::TRIANGLES, conv!(self.indices.len()), gl::UNSIGNED_INT, ptr::null());
        gl::BindVertexArray(0);
    }
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    //pub textures: Vec<Texture>,
}

impl Model {
    pub unsafe fn load_obj<P: AsRef<Path>>(name: P) -> Result<Self, Box<dyn Error + 'static>> {
        use std::collections::HashMap;
        use std::collections::hash_map::Entry::*;

        let name = name.as_ref();
        let mut meshes = vec![];

        let (models, materials) = tobj::load_obj(name)?;
        
        for model in models.into_iter() {
            let mesh = model.mesh;

            let len = mesh.positions.len() / 3;
            let mut verticies = Vec::with_capacity(len);

            for i in 0..len {
                verticies.push(Vertex {
                    position: vec3(mesh.positions[3 * i], mesh.positions[3 * i + 1], mesh.positions[3 * i + 2]),
                    normal: vec3(mesh.normals[3 * i], mesh.normals[3 * i + 1], mesh.normals[3 * i + 2]),
                    tex_coords: vec2(mesh.texcoords[2 * i], mesh.texcoords[2 * i + 1]),
                });
            }

            let mut loaded_textures = HashMap::new();

            let mut textures = vec![];
            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];

                // TODO: prevent loading texture multiple times
                if !material.diffuse_texture.is_empty() {
                    let tex_name = name.with_file_name(&material.diffuse_texture);

                    match loaded_textures.entry(tex_name) {
                        Occupied(o) => textures.push(*o.get()),
                        Vacant(v) => {
                            let texture = Texture::new(v.key(), TextureType::Diffuse);
                            v.insert(texture);
                            textures.push(texture);
                        }
                    }
                }

                if !material.specular_texture.is_empty() {
                    let tex_name = name.with_file_name(&material.specular_texture);

                    match loaded_textures.entry(tex_name) {
                        Occupied(o) => textures.push(*o.get()),
                        Vacant(v) => {
                            let texture = Texture::new(v.key(), TextureType::Specular);
                            v.insert(texture);
                            textures.push(texture);
                        }
                    }
                }
            }

            meshes.push(Mesh::new(verticies, mesh.indices, textures));
        }

        Ok(Self {
            meshes,
        })
    }

    pub unsafe fn draw(&self, shader: Shader) {
        for mesh in self.meshes.iter() {
            mesh.draw(shader);
        }
    }
}
