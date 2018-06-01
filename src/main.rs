extern crate gl;
extern crate glutin;
extern crate rusttype;

#[macro_use]
mod gl_basic;

use glutin::{GlContext, GlRequest, Api};

use rusttype::{point, vector, Font, PositionedGlyph, Rect, Scale, FontCollection};
use rusttype::gpu_cache::CacheBuilder;

use std::mem;
use std::ptr;
use std::str;

use gl_basic::types::*;

fn main() {
    println!("main started!!!");
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!");
        //.with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGlEs, (2, 0)))
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
    }

    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    /*let font_data = include_bytes!("../fonts/wqy-microhei/WenQuanYiMicroHei.ttf");
    let font = Font::from_bytes(font_data as &[u8]).unwrap();

    let dpi_factor = 1;

    let (cache_width, cache_height) = (512 * dpi_factor as u32, 512 * dpi_factor as u32);
    let mut cache = CacheBuilder {
        width: cache_width,
        height: cache_height,
        ..CacheBuilder::default()
    }.build();*/

    let font_data = include_bytes!("../fonts/wqy-microhei/WenQuanYiMicroHei.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
        panic!("error constructing a FontCollection from bytes: {}", e);
    });
    let font = collection.into_font() // only succeeds if collection consists of one font
        .unwrap_or_else(|e| {
            panic!("error turning FontCollection into a Font: {}", e);
        });

    // Desired font pixel height
    let height: f32 = 12.4; // to get 80 chars across (fits most terminals); adjust as desired
    let pixel_height = height.ceil() as usize;

    // 2x scale in x direction to counter the aspect ratio of monospace characters.
    let scale = Scale {
        x: height * 2.0,
        y: height,
    };

    // The origin of a line of text is at the baseline (roughly where
    // non-descending letters sit). We don't want to clip the text, so we shift
    // it down with an offset when laying it out. v_metrics.ascent is the
    // distance between the baseline and the highest edge of any glyph in
    // the font. That's enough to guarantee that there's no clipping.
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    // Glyphs to draw for "RustType". Feel free to try other strings.
    let glyphs: Vec<PositionedGlyph> = font.layout("RustType", scale, offset).collect();

    // Find the most visually pleasing width to display
    let width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;

    println!("width: {}, height: {}", width, pixel_height);

    // Rasterise directly into ASCII art.
    let mut pixel_data: Vec<u8> = vec![128; width * pixel_height * 4];
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                // v should be in the range 0.0 to 1.0
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let x = x as usize;
                    let y = y as usize;
                    pixel_data[(x + y * width) * 4 + 2] = (v * 255.0) as u8;
                    pixel_data[(x + y * width) * 4 + 1] = 0;
                    pixel_data[(x + y * width) * 4 + 0] = 255;
                    print!(".");
                }
            })
        }
    }

    unsafe {
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       gl::RGBA as i32,
                       width as i32,
                       pixel_height as i32,
                       0,
                       gl::RGBA,
                       gl::UNSIGNED_BYTE,
                       &pixel_data[0] as *const u8 as *const std::os::raw::c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);

        //let tex_loc = gl::GetUniformLocation(program, b"tex\0".as_ptr() as *const _);
        gl::BindTexture(gl::TEXTURE_2D, texture);
    }

    let program = match gl_basic::Program::compile(VS_SRC, FS_SRC) {
                Ok(p) => p,
                Err(e) => panic!("shader program: {}", e),
            };

    attribs!(
        pub struct Vertex {
            pub position: Vec2,
            pub tex_coords: Vec2,
            pub color: Vec4,
        }
    );

    let mut drawable = match Vertex::new_object(program) {
                Ok(d) => d,
                Err(e) => panic!("Drawable: {}", e),
            };

    Vertex::set_object_vertices(&mut drawable, vec![
            Vertex{
                position: gl_basic::Vec2{x: -0.5, y: -0.5},
                tex_coords: gl_basic::Vec2{x: -1.0, y: -1.0},
                color: gl_basic::Vec4{x: 0.2, y: 1.0, z: 0.0, w: 1.0}},
            Vertex{
                position: gl_basic::Vec2{x: -0.5, y: 1.0},
                tex_coords: gl_basic::Vec2{x: -1.0, y: 2.0},
                color: gl_basic::Vec4{x: 0.0, y: 0.5, z: 0.1, w: 1.0}},
            Vertex{
                position: gl_basic::Vec2{x: 1.0, y: -0.5},
                tex_coords: gl_basic::Vec2{x: 2.0, y: -1.0},
                color: gl_basic::Vec4{x: 0.0, y: 0.3, z: 0.6, w: 1.0}},
        ]);

    drawable.set_indices(vec![[0, 1, 2]]);

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            println!("main loop!!!");
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                    _ => ()
                },
                _ => ()
            }
        });

        unsafe {
            gl::ClearColor(0.0, 1.0, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        drawable.draw();

        gl_window.swap_buffers().unwrap();
    }
}

const VS_SRC: &str = "
#version 100
precision mediump float;

attribute vec2 position;
attribute vec2 tex_coords;
attribute vec3 color;

varying vec3 v_color;
varying vec2 v_tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);

    v_color = color;
    v_tex_coords = tex_coords;
}";

const FS_SRC: &str = "
#version 100
precision mediump float;

uniform sampler2D tex;

varying vec3 v_color;
varying vec2 v_tex_coords;

void main() {
    gl_FragColor = texture2D(tex, v_tex_coords);
}";

/*
const VS_TXT_SRC: &'static [u8] = b"
    #version 140
    in vec2 position;
    in vec2 tex_coords;
    in vec4 colour;
    out vec2 v_tex_coords;
    out vec4 v_colour;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        v_tex_coords = tex_coords;
        v_colour = colour;
    }
\0";

const FS_TXT_SRC: &'static [u8] = b"
#version 140
uniform sampler2D tex;
in vec2 v_tex_coords;
in vec4 v_colour;
out vec4 f_colour;
void main() {
    f_colour = v_colour;// * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
}
\0";
*/