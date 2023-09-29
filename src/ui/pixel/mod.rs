use glium::implement_vertex;
use glium::program;
use glium::uniform;
use glium::{IndexBuffer, Program, Surface, VertexBuffer};
use glium::index::PrimitiveType;
use glium::texture::Texture2d;
use image::{RgbaImage, Rgba};

#[derive(Copy, Clone)]
struct Vertex
{
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

/// Displays pixels in a OpenGL window via glium.
/// Adapted from the example from
/// https://github.com/glium/glium/blob/master/examples/image.rs
/// as accessed on 2023-09-29
pub struct PixelDisplay
{
    opengl_texture: Texture2d,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    program: Program,
}

impl PixelDisplay
{
    pub fn new(system: &crate::ui::System) -> Self
    {
        let display = system.display();

        let mut image = RgbaImage::new(128, 128);
        for x in 0..128
        {
            for y in 0..128
            {
                image.put_pixel(x as u32, y as u32, Rgba([x, y, 0, 255]));
            }
        }

        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let opengl_texture = glium::texture::Texture2d::new(display, image).unwrap();

        let vertex_buffer = {
            glium::VertexBuffer::new(
                display,
                &[
                    Vertex {
                        position: [-1.0, -1.0],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [-1.0, 1.0],
                        tex_coords: [0.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, 1.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, -1.0],
                        tex_coords: [1.0, 0.0],
                    },
                ],
            )
            .unwrap()
        };

        let index_buffer =
            glium::IndexBuffer::new(display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
                .unwrap();

                let program = program!(display,
                    140 => {
                        vertex: "
                            #version 140
                            in vec2 position;
                            in vec2 tex_coords;
                            out vec2 v_tex_coords;
                            void main() {
                                gl_Position = vec4(position, 0.0, 1.0);
                                v_tex_coords = tex_coords;
                            }
                        ",
        
                        fragment: "
                            #version 140
                            uniform sampler2D tex;
                            in vec2 v_tex_coords;
                            out vec4 f_color;
                            void main() {
                                f_color = texture(tex, v_tex_coords);
                            }
                        "
                    },
                )
                .unwrap();

        PixelDisplay
        {
            opengl_texture,
            vertex_buffer,
            index_buffer,
            program,
        }
    }

    pub fn render(&mut self, frame: &mut glium::Frame)
    {
        let uniforms = uniform! {
            tex: glium::uniforms::Sampler::new(&self.opengl_texture)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
        };

        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
}
