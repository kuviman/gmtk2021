use super::*;

#[derive(ugli::Vertex, Clone)]
pub struct Vertex {
    pub a_pos: Vec2<f32>,
}

pub struct LineRenderer {
    geng: Rc<Geng>,
    program: ugli::Program,
}

impl LineRenderer {
    pub fn new(geng: &Rc<Geng>) -> Self {
        Self {
            geng: geng.clone(),
            program: geng
                .shader_lib()
                .compile(include_str!("program.glsl"))
                .unwrap(),
        }
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        color: Color<f32>,
        points: impl IntoIterator<Item = Vec2<f32>>,
    ) {
        let camera_uniforms = camera.uniforms(framebuffer.size().map(|x| x as f32));
        let uniforms = (
            camera_uniforms,
            ugli::uniforms! {
                u_color: color,
            },
        );
        let vertices = ugli::VertexBuffer::new_dynamic(
            self.geng.ugli(),
            points
                .into_iter()
                .map(|point| Vertex { a_pos: point })
                .collect(),
        );
        ugli::draw(
            framebuffer,
            &self.program,
            ugli::DrawMode::Lines { line_width: 1.0 },
            &vertices,
            uniforms,
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
    pub fn draw_strip(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        color: Color<f32>,
        points: impl IntoIterator<Item = Vec2<f32>>,
    ) {
        let camera_uniforms = camera.uniforms(framebuffer.size().map(|x| x as f32));
        let uniforms = (
            camera_uniforms,
            ugli::uniforms! {
                u_color: color,
            },
        );
        let vertices = ugli::VertexBuffer::new_dynamic(
            self.geng.ugli(),
            points
                .into_iter()
                .map(|point| Vertex { a_pos: point })
                .collect(),
        );
        ugli::draw(
            framebuffer,
            &self.program,
            ugli::DrawMode::LineStrip { line_width: 1.0 },
            &vertices,
            uniforms,
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
}
