extern crate gl;
extern crate rusttype;
extern crate unicode_normalization;

use std;

use rusttype::{point, vector, Font, FontCollection, PositionedGlyph, Rect, Scale};

use gl_basic;
use gl_basic::types::*;

const MIN_CACHE_SIZE: u32 = 256;
const MAX_CACHE_SIZE: u32 = 2048;

pub struct GlGlyphCache<'font> {
    cache: rusttype::gpu_cache::Cache<'font>,
    texture: gl_basic::Texture,
}

impl<'font> GlGlyphCache<'font> {
    pub fn new() -> Result<GlGlyphCache<'font>, String> {
        let mut texture = match gl_basic::Texture::new() {
            Ok(p) => p,
            Err(e) => return Err(format!("GlGlyphCache texture: {}", e)),
        };
        texture.bind_then(|| {
            unsafe {
                // set the texture wrapping parameters
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                // set texture filtering parameters
                //gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                //gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            }
        });
        let mut cache = rusttype::gpu_cache::CacheBuilder {
            width: 0,
            height: 0,
            ..rusttype::gpu_cache::CacheBuilder::default()
        }.build();
        Ok(GlGlyphCache {
            cache: cache,
            texture: texture,
        })
    }

    fn resize_cache(&mut self, width: u32, height: u32) {
        unsafe {
            self.texture.bind_then(|| {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::ALPHA as i32,
                    width as i32,
                    height as i32,
                    0,
                    gl::ALPHA,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null() as *const _,
                );
            });
        }
        self.cache = rusttype::gpu_cache::CacheBuilder {
            width: width,
            height: height,
            scale_tolerance: 0.25,
            position_tolerance: 0.25,
            pad_glyphs: true,
        }.build();
        println!("cached resized to {}, {}", width, height);
    }

    pub fn increase_cache_size(&mut self) -> Result<(), ()> {
        let (old_x, old_y) = self.cache.dimensions();
        if old_x >= MAX_CACHE_SIZE && old_y >= MAX_CACHE_SIZE {
            return Err(());
        }
        let new_x = match old_x * 2 {
            x if x < MIN_CACHE_SIZE => MIN_CACHE_SIZE,
            x if x > MAX_CACHE_SIZE => MAX_CACHE_SIZE,
            x => x,
        };
        let new_y = match old_y * 2 {
            y if y < MIN_CACHE_SIZE => MIN_CACHE_SIZE,
            y if y > MAX_CACHE_SIZE => MAX_CACHE_SIZE,
            y => y,
        };
        self.resize_cache(new_x, new_y);
        Ok(())
    }

    pub fn cache_glyphs(&mut self, glyphs: &Vec<(usize, &Vec<PositionedGlyph<'font>>)>) {
        loop {
            for ref glyphs in glyphs {
                for glyph in glyphs.1 {
                    self.cache.queue_glyph(glyphs.0, glyph.clone());
                }
            }
            let cache_queued_result;
            {
                let texture = &self.texture;
                cache_queued_result = self.cache.cache_queued(|rect, data| {
                    texture.bind_then(|| unsafe {
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
                    })
                })
            }

            match cache_queued_result {
                Err(_) => match self.increase_cache_size() {
                    Ok(_) => (),
                    Err(_) => {
                        eprintln!("Failed to increase GPU text cache size");
                        break;
                    }
                },
                Ok(_) => break,
            }
        }
    }
}

pub struct GlGlyphRenderer<'font> {
    cache: GlGlyphCache<'font>,
    font: rusttype::Font<'font>,
    object: gl_basic::Object,
    size: (f32, f32),
}

impl<'font> GlGlyphRenderer<'font> {
    pub fn new(screen_width: f32, screen_height: f32) -> Result<GlGlyphRenderer<'font>, String> {
        let font_data = include_bytes!("../fonts/wqy-microhei/WenQuanYiMicroHei.ttf");
        let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
            panic!("error constructing a FontCollection from bytes: {}", e);
        });
        // only succeeds if collection consists of one font
        let font = match collection.into_font() {
            Ok(f) => f,
            Err(e) => {
                return Err(format!(
                    "collection.into_font failed, perhaps it has multiple fonts? error: {}",
                    e
                ))
            }
        };

        let program = match gl_basic::Program::compile(VERT_SHADER_SRC, FRAG_SHADER_SOURCE) {
            Ok(p) => std::rc::Rc::new(p),
            Err(e) => return Err(format!("text shader: {}", e)),
        };

        let mut object = match Vertex::new_object(program) {
            Ok(d) => d,
            Err(e) => return Err(format!("text object: {}", e)),
        };

        let mut cache = match GlGlyphCache::new() {
            Ok(d) => d,
            Err(e) => return Err(format!("text object: {}", e)),
        };

        Ok(GlGlyphRenderer {
            cache: cache,
            font: font,
            object: object,
            size: (screen_width, screen_height),
        })
    }

    fn layout_paragraph(&self, text: &str, size: f32) -> Vec<PositionedGlyph<'font>> {
        use self::unicode_normalization::UnicodeNormalization;
        let scale = Scale::uniform(size);
        let mut result = Vec::new();
        let v_metrics = self.font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;
        for c in text.nfc() {
            if c.is_control() {
                match c {
                    '\n' => {
                        caret = point(0.0, caret.y + advance_height);
                    }
                    _ => {}
                }
                continue;
            }
            let base_glyph = self.font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += self.font.pair_kerning(scale, id, base_glyph.id());
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

    pub fn set_text(&mut self, text: &str) {
        let glyphs: Vec<PositionedGlyph> = self.layout_paragraph(&text, 64.0);
        self.cache.cache_glyphs(&vec![(0, &glyphs)]);
        let origin = point(0.0, 0.0);
        let mut glyph_count = 0;
        let vertices: Vec<Vertex> = glyphs
            .iter()
            .flat_map(|g| {
                if let Ok(Some((uv_rect, screen_rect))) = self.cache.cache.rect_for(0, g) {
                    glyph_count += 1;
                    let gl_rect = Rect {
                        min: origin
                            + (vector(
                                screen_rect.min.x as f32 / self.size.0 - 0.5,
                                1.0 - screen_rect.min.y as f32 / self.size.1 - 0.5,
                            )) * 2.0,
                        max: origin
                            + (vector(
                                screen_rect.max.x as f32 / self.size.0 - 0.5,
                                1.0 - screen_rect.max.y as f32 / self.size.1 - 0.5,
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

        let indices: Vec<[gl::types::GLuint; 3]> = (0..glyph_count)
            .flat_map(|i| {
                let i = i * 4;
                vec![[i + 0, i + 1, i + 2], [i + 0, i + 2, i + 3]]
            })
            .collect();

        Vertex::set_vertices(&mut self.object, vertices);
        self.object.set_indices(indices);
    }

    pub fn draw(&self) {
        self.cache.texture.bind_then(|| {
            self.object.draw();
        });
    }

    /*
    pub fn draw_cache(&mut self) {
        if let None = self.cache_object {
            let cache_object = match Vertex::new_object(program) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("cache object: {}", e);
                    return;
                }
            };
                let vertices = vec![Vertex {
            position: Vec2 { x: 0f32, y: 1f32 },
            tex_coords: Vec2 { x: 0f32, y: 0f32 },
        }, Vertex {
            position: Vec2 { x: 0f32, y: 0f32 },
            tex_coords: Vec2 { x: 0f32, y: 1f32 },
        },Vertex {
            position: Vec2 { x: 1f32, y: 0f32 },
            tex_coords: Vec2 { x: 1f32, y: 1f32 },
        },Vertex {
            position: Vec2 { x: 1f32, y: 1f32 },
            tex_coords: Vec2 { x: 1f32, y: 0f32 },
        }];
        let indices = vec![[0, 1, 2],[0, 2, 3]];

        Vertex::set_vertices(&mut cache_object, vertices);
        cache_object.set_indices(indices);
        self.cache_object = cache_object;
        }
        self.cache.texture.bind_then(|| {
            self.cache_object.draw();
        });
    }*/
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
