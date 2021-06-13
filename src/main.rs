#![windows_subsystem = "windows"]

use geng::prelude::*;

pub mod camera;
pub mod game;
pub mod line_renderer;
pub mod renderer;

pub use camera::*;
pub use game::*;
pub use line_renderer::*;
pub use renderer::*;

pub fn hsv(h: f32, s: f32, v: f32) -> Color<f32> {
    hsva(h, s, v, 1.0)
}
pub fn hsva(mut h: f32, s: f32, v: f32, a: f32) -> Color<f32> {
    h -= h.floor();
    let r;
    let g;
    let b;
    let f = h * 6.0 - (h * 6.0).floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    if h * 6.0 < 1.0 {
        r = v;
        g = t;
        b = p;
    } else if h * 6.0 < 2.0 {
        r = q;
        g = v;
        b = p;
    } else if h * 6.0 < 3.0 {
        r = p;
        g = v;
        b = t;
    } else if h * 6.0 < 4.0 {
        r = p;
        g = q;
        b = v;
    } else if h * 6.0 < 5.0 {
        r = t;
        g = p;
        b = v;
    } else {
        r = v;
        g = p;
        b = q;
    }
    Color::rgba(r, g, b, a)
}

#[derive(Deref)]
pub struct Font {
    #[deref]
    inner: Rc<geng::Font>,
}

impl geng::LoadAsset for Font {
    fn load(geng: &Rc<Geng>, path: &str) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        <Vec<u8> as geng::LoadAsset>::load(&geng, path)
            .map(move |data| {
                Ok(Font {
                    inner: Rc::new(geng::Font::new(&geng, data?)?),
                })
            })
            .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("ttf");
}

#[derive(derive_more::Deref)]
pub struct Animation {
    #[deref]
    frames: Vec<ugli::Texture>,
}

impl geng::LoadAsset for Animation {
    fn load(geng: &Rc<Geng>, path: &str) -> geng::AssetFuture<Self> {
        let data = <Vec<u8> as geng::LoadAsset>::load(geng, path);
        let geng = geng.clone();
        async move {
            let data = data.await?;
            use image::AnimationDecoder;
            Ok(Self {
                frames: image::codecs::png::PngDecoder::new(data.as_slice())
                    .unwrap()
                    .apng()
                    .into_frames()
                    .map(|frame| {
                        let frame = frame.unwrap();
                        ugli::Texture::from_image_image(geng.ugli(), frame.into_buffer())
                    })
                    .collect(),
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("png");
}

#[derive(geng::Assets)]
pub struct Assets {
    player: ugli::Texture,
    #[asset(path = "level.json")]
    level: String,
    ball: ugli::Texture,
    chain: ugli::Texture,
    block: ugli::Texture,
}

impl Assets {}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    if let Some(dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        std::env::set_current_dir(std::path::Path::new(&dir).join("static")).unwrap();
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = std::env::current_exe().unwrap().parent() {
                std::env::set_current_dir(path).unwrap();
            }
        }
    }

    let geng = Rc::new(Geng::new(geng::ContextOptions {
        title: "GMTK 2021 - Ball & Chain".to_owned(),
        ..default()
    }));
    let assets = <Assets as geng::LoadAsset>::load(&geng, ".");
    geng::run(
        geng.clone(),
        geng::LoadingScreen::new(&geng, geng::EmptyLoadingScreen, assets, {
            let geng = geng.clone();
            move |assets| {
                let mut assets = assets.unwrap();
                Game::new(&geng, &Rc::new(assets))
            }
        }),
    );
}
