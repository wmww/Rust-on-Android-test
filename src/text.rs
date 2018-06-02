extern crate gl;
extern crate rusttype;
extern crate unicode_normalization;

use std;

use rusttype::gpu_cache::CacheBuilder;
use rusttype::{point, vector, Font, FontCollection, PositionedGlyph, Rect, Scale};

use gl_basic;
use gl_basic::types::*;

pub struct GlGlyphCache {
    texture: gl_basic::Texture,
    object: gl_basic::Object,
}

impl GlGlyphCache {
    pub fn new(screen_width: f32, screen_height: f32) -> Result<GlGlyphCache, String> {
        let font_data = include_bytes!("../fonts/wqy-microhei/WenQuanYiMicroHei.ttf");
        let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
            panic!("error constructing a FontCollection from bytes: {}", e);
        });
        let font = collection.into_font() // only succeeds if collection consists of one font
            .unwrap_or_else(|e| {
                panic!("error turning FontCollection into a Font: {}", e);
            });
        let dpi_factor = 1;
        let (cache_width, cache_height) = (300 * dpi_factor as u32, 300 * dpi_factor as u32);
        let mut cache = CacheBuilder {
            width: cache_width,
            height: cache_height,
            ..CacheBuilder::default()
        }.build();
        /*let mut text: String = "A japanese poem:\r
\r
色は匂へど散りぬるを我が世誰ぞ常ならむ有為の奥山今日越えて浅き夢見じ酔ひもせず\r
\r
Feel free to type out some text, and delete it with Backspace. \
You can also try resizing this window."
            .into();*/
        let mut text: String = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".into();
        let glyphs = layout_paragraph(&font, &text);
        for glyph in &glyphs {
            cache.queue_glyph(0, glyph.clone());
        }
        let mut texture = match gl_basic::Texture::new() {
            Ok(p) => p,
            Err(e) => panic!("Texture: {}", e),
        };

        texture.bind_then(|| {
            unsafe {
                // set the texture wrapping parameters
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                // set texture filtering parameters
                //gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                //gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::ALPHA as i32,
                    cache_width as i32,
                    cache_height as i32,
                    0,
                    gl::ALPHA,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null() as *const _,
                );
            }
        });

        cache
            .cache_queued(|rect, data| texture.bind_then(||unsafe {
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    rect.min.x as i32,
                    rect.min.y as i32,
                    rect.width() as i32,
                    rect.height() as i32,
                    gl::ALPHA,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as *const _,
                );
                for y in (0..rect.height()) {
                    for x in (0..rect.width()) {
                        if (data[(y * rect.width() + x) as usize] > 128) {
                            print!(":");
                        } else {
                            print!("#");
                        }
                    }
                    println!();
                }
            }))
            .unwrap();

        let origin = point(0.0, 0.0);
        let mut glyph_count = 0;
        let mut vertices: Vec<Vertex> = glyphs
            .iter()
            .flat_map(|g| {
                if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, g) {
                    glyph_count += 1;
                    let gl_rect = Rect {
                        min: origin
                            + (vector(
                                screen_rect.min.x as f32 / screen_width - 0.5,
                                1.0 - screen_rect.min.y as f32 / screen_height - 0.5,
                            )) * 2.0,
                        max: origin
                            + (vector(
                                screen_rect.max.x as f32 / screen_width - 0.5,
                                1.0 - screen_rect.max.y as f32 / screen_height - 0.5,
                            )) * 2.0,
                    };
                    println!(
                        "UV: min: ({}, {}), max: ({}, {})",
                        uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y
                    );
                    vec![
                        Vertex {
                            position: Vec2 {
                                x: gl_rect.min.x,
                                y: gl_rect.max.y,
                            },
                            tex_coords: Vec2 {
                                x: uv_rect.min.x,
                                y: uv_rect.max.y,
                            },
                        },
                        Vertex {
                            position: Vec2 {
                                x: gl_rect.min.x,
                                y: gl_rect.min.y,
                            },
                            tex_coords: Vec2 {
                                x: uv_rect.min.x,
                                y: uv_rect.min.y,
                            },
                        },
                        Vertex {
                            position: Vec2 {
                                x: gl_rect.max.x,
                                y: gl_rect.min.y,
                            },
                            tex_coords: Vec2 {
                                x: uv_rect.max.x,
                                y: uv_rect.min.y,
                            },
                        },
                        Vertex {
                            position: Vec2 {
                                x: gl_rect.max.x,
                                y: gl_rect.max.y,
                            },
                            tex_coords: Vec2 {
                                x: uv_rect.max.x,
                                y: uv_rect.max.y,
                            },
                        },
                    ]
                } else {
                    vec![]
                }
            })
            .collect();

        let mut indices: Vec<[gl::types::GLuint; 3]> = (0..glyph_count)
            .flat_map(|i| {
                let i = i * 4;
                vec![[i + 0, i + 1, i + 2], [i + 0, i + 2, i + 3]]
            })
            .collect();

        vertices.push(Vertex {
                            position: Vec2 {
                                x: 0f32,
                                y: 1f32,
                            },
                            tex_coords: Vec2 {
                                x: 0f32,
                                y: 1f32,
                            },
                        });
        vertices.push(Vertex {
                            position: Vec2 {
                                x: 0f32,
                                y: 0f32,
                            },
                            tex_coords: Vec2 {
                                x: 0f32,
                                y: 0f32,
                            },
                        });
        vertices.push(Vertex {
                            position: Vec2 {
                                x: 1f32,
                                y: 0f32,
                            },
                            tex_coords: Vec2 {
                                x: 1f32,
                                y: 0f32,
                            },
                        });
        vertices.push(Vertex {
                            position: Vec2 {
                                x: 1f32,
                                y: 1f32,
                            },
                            tex_coords: Vec2 {
                                x: 1f32,
                                y: 1f32,
                            },
                        });
        let len: u32 = vertices.len() as u32;
        indices.push([len - 4, len - 3, len - 2]);
        indices.push([len - 4, len - 2, len - 1]);

        let program = match gl_basic::Program::compile(VERT_SHADER_SRC, FRAG_SHADER_SOURCE) {
            Ok(p) => p,
            Err(e) => panic!("shader program: {}", e),
        };

        let mut object = match Vertex::new_object(program) {
            Ok(d) => d,
            Err(e) => panic!("Drawable: {}", e),
        };

        Vertex::set_vertices(&mut object, vertices);
        object.set_indices(indices);

        Ok(GlGlyphCache {
            texture: texture,
            object: object,
        })
    }

    pub fn draw(&self) {
        self.texture.bind_then(|| {
            self.object.draw();
        });
    }
}

fn layout_paragraph<'a>(font: &'a Font, text: &str) -> Vec<PositionedGlyph<'a>> {
    use self::unicode_normalization::UnicodeNormalization;
    let dpi_factor = 1.0;
    let scale = Scale::uniform(64.0 * dpi_factor);
    let mut result = Vec::new();
    let v_metrics = font.v_metrics(scale);
    let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let mut caret = point(0.0, v_metrics.ascent);
    let mut last_glyph_id = None;
    for c in text.nfc() {
        if c.is_control() {
            match c {
                '\r' => {
                    caret = point(0.0, caret.y + advance_height);
                }
                '\n' => {}
                _ => {}
            }
            continue;
        }
        let base_glyph = font.glyph(c);
        if let Some(id) = last_glyph_id.take() {
            caret.x += font.pair_kerning(scale, id, base_glyph.id());
        }
        last_glyph_id = Some(base_glyph.id());
        let mut glyph = base_glyph.scaled(scale).positioned(caret);
        /*if let Some(bb) = glyph.pixel_bounding_box() {
            if bb.max.x > width as i32 {
                caret = point(0.0, caret.y + advance_height);
                glyph = glyph.into_unpositioned().positioned(caret);
                last_glyph_id = None;
            }
        }*/
        caret.x += glyph.unpositioned().h_metrics().advance_width;

        result.push(glyph);
    }
    result
}

attribs!(pub struct Vertex {
    pub position: Vec2,
    pub tex_coords: Vec2,
});

const VERT_SHADER_SRC: &str = "
#version 300 es
precision mediump float;

in vec2 position;
in vec2 tex_coords;

out vec2 frag_tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    frag_tex_coords = tex_coords;
}";

const FRAG_SHADER_SOURCE: &str = "
#version 300 es
precision mediump float;

uniform sampler2D tex;

in vec2 frag_tex_coords;

out vec4 fragColor;

void main() {
    fragColor = vec4(1.0, 1.0, 1.0, texture(tex, frag_tex_coords).a);
}";

pub fn render_text() {
    let program = match gl_basic::Program::compile(VERT_SHADER_SRC, FRAG_SHADER_SOURCE) {
        Ok(p) => p,
        Err(e) => panic!("shader program: {}", e),
    };

    let mut drawable = match Vertex::new_object(program) {
        Ok(d) => d,
        Err(e) => panic!("Drawable: {}", e),
    };
}
