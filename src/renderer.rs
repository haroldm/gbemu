use luminance_glfw::GlfwSurface;
use luminance_gl::gl33::GL33;
use luminance::framebuffer::Framebuffer;
use luminance_windowing::{WindowDim, WindowOpt};
// use luminance_derive::Semantics;
use luminance_derive::UniformInterface;
use luminance::texture::{Dim2, GenMipmaps, Sampler, Texture, MagFilter};
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::backend::texture::Texture as TextureBackend;
use luminance::shader::{Uniform, Program};
use luminance::context::GraphicsContext;
use luminance::tess::{Mode, Tess, Interleaved};
use luminance::render_state::RenderState;
use glfw::{Action, Context as _, Key, WindowEvent};

const VS: &'static str = include_str!("texture-vs.glsl");
const FS: &'static str = include_str!("texture-fs.glsl");

// we also need a special uniform interface here to pass the texture to the shader
#[derive(UniformInterface)]
struct ShaderInterface {
    tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub struct Renderer {
    surface: GlfwSurface,
    // buffer: Framebuffer<GL33, Dim2, (), ()>,
    bitmap: Texture<GL33, Dim2, NormRGB8UI>,
    program: Program<GL33, (), (), ShaderInterface>,
    // tess: Tess<GL33, (), (), Interleaved>,
    // render_st: RenderState,
}

impl Renderer {
    pub fn new() -> Renderer {

        let dim = WindowDim::Windowed { width: 160*4, height: 144*4 };
        let mut surface = GlfwSurface::new_gl33("GBEMU", WindowOpt::default().set_dim(dim))
        .expect("GLFW surface creation");

        let mut sampler = Sampler::default();
        sampler.mag_filter = MagFilter::Nearest;
        let tex: Texture<GL33, Dim2, NormRGB8UI> =
            Texture::new(&mut surface, [160, 144], 0, sampler).unwrap();

        // set the uniform interface to our type so that we can read textures from the shader
        let program = surface
        .new_shader_program::<(), (), ShaderInterface>()
        .from_strings(VS, None, None, FS)
        .expect("program creation")
        .ignore_warnings();
        
        Renderer {
            surface: surface,
            // buffer: back_buffer,
            bitmap: tex,
            program: program,
            // tess: tess,
            // render_st: render_st,
        }
    }


    pub fn render_line(&mut self, line: u32, texels: Vec<(u8, u8, u8)>) {
        

        self.bitmap.upload_part(
            GenMipmaps::Yes, [0, 144 - line as u32], [160, 1], &texels[..]
        ).unwrap();
    }

    pub fn render_frame(&mut self) {

        let Self {
            surface,
            // buffer,
            bitmap,
            program,
            // tess,
            // render_st,
            ..
        } = self;
       
        let tess = surface
        .new_tess()
        .set_vertex_nb(4)
        .set_mode(Mode::TriangleFan)
        .build()
        .unwrap();
        
        let mut back_buffer = surface.back_buffer().unwrap();
        let render_st = RenderState::default();
        
        surface.window.glfw.poll_events();
        for (_, event) in surface.events_rx.try_iter() {
            match event {
                WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => 
                    panic!("EXIT asked"),
                
                
                _ => (),
            }
        }


        // let mut sampler = Sampler::default();
        // sampler.mag_filter = MagFilter::Nearest;
        // let mut tex: Texture<GL33, Dim2, NormRGB8UI> =
        //     Texture::new(&mut surface, [160, 144], 0, sampler).unwrap();
        
        // tex.upload_raw(GenMipmaps::No, bitmap.get_raw_texels().unwrap().as_slice()).unwrap();
        
        let mut program = surface
        .new_shader_program::<(), (), ShaderInterface>()
        .from_strings(VS, None, None, FS)
        .expect("program creation")
        .ignore_warnings();

        // here, we need to bind the pipeline variable; it will enable us to bind the texture to the GPU
        // and use it in the shader
        let render = surface
        .new_pipeline_gate()
        .pipeline(
            &back_buffer,
            &PipelineState::default(),
            |pipeline, mut shd_gate| {
                // bind our fancy texture to the GPU: it gives us a bound texture we can use with the shader
                let bound_tex = pipeline.bind_texture(bitmap)?;
                
                shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
                    // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
                    // to use the texture passed as argument (no allocation or copy is performed)
                    iface.set(&uni.tex, bound_tex.binding());
                    
                    rdr_gate.render(&render_st, |mut tess_gate| {
                        // render the tessellation to the surface the regular way and let the vertex shader’s
                        // magic do the rest!
                        tess_gate.render(&tess)
                    })
                })
            },
        )
        .assume();
        
        if render.is_ok() {
            self.surface.window.swap_buffers();
        } else {
            panic!("Render not ok !!!");
        }
    }

}