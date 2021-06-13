use std::{cell, collections::VecDeque};

use super::*;

const EPS: f32 = 1e-5;
const GRAVITY: f32 = 50.0;
const BALL_SWING_DISTANCE: f32 = 0.8;

struct Collision {
    normal: Vec2<f32>,
    penetration: f32,
}

#[derive(Clone)]
struct Ball {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    size: f32,
    stand: bool,
}

impl Ball {
    fn new(pos: Vec2<f32>, size: f32) -> Self {
        Self {
            pos,
            size,
            vel: vec2(0.0, 0.0),
            stand: false,
        }
    }
    fn collide(&self, &[p1, p2]: &Segment) -> Option<Collision> {
        let v = p2 - p1;
        if Vec2::dot(v, self.pos - p1) < 0.0 {
            let n = self.pos - p1;
            let penetration = self.size - n.len();
            if penetration > 0.0 {
                return Some(Collision {
                    normal: n.normalize(),
                    penetration,
                });
            } else {
                return None;
            }
        }
        if Vec2::dot(-v, self.pos - p2) < 0.0 {
            let n = self.pos - p2;
            let penetration = self.size - n.len();
            if penetration > 0.0 {
                return Some(Collision {
                    normal: n.normalize(),
                    penetration,
                });
            } else {
                return None;
            }
        }
        let n = Vec2::rotate_90(v.normalize());
        let distance = Vec2::dot(n, self.pos - p1);
        if distance > 0.0 && distance < self.size {
            return Some(Collision {
                normal: n,
                penetration: self.size - distance,
            });
        }
        if distance < 0.0 && distance > -self.size {
            let n = -n;
            let distance = -distance;
            return Some(Collision {
                normal: n,
                penetration: self.size - distance,
            });
        }
        None
    }
    fn update(&mut self, level: &[Segment], delta_time: f32) {
        if !self.stand {
            self.vel.y -= GRAVITY * delta_time;
            self.pos += self.vel * delta_time;
        } else {
            self.vel = vec2(0.0, 0.0);
        }
        for segment in level {
            if let Some(collision) = self.collide(segment) {
                self.pos += collision.normal * collision.penetration;
                let relative_vel = Vec2::dot(collision.normal, self.vel);
                if relative_vel < 0.0 {
                    if collision.normal.y > collision.normal.x.abs() * 2.0 {
                        self.stand = true;
                    }
                    self.vel -= relative_vel * collision.normal;
                }
            }
        }
    }
    fn matrix(&self) -> Mat4<f32> {
        Mat4::translate(self.pos.extend(0.0)) * Mat4::scale_uniform(self.size)
    }
}

#[derive(Clone)]
struct Player {
    character: Ball,
    ball: Ball,
    ball_in_hands: bool,
    chain_len: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            character: Ball::new(vec2(0.0, 0.0), 1.0),
            ball: Ball::new(vec2(0.0, 0.0), 0.5),
            ball_in_hands: true,
            chain_len: 1.0,
        }
    }
    fn update(&mut self, level: &[Segment], delta_time: f32) {
        if self.ball_in_hands {
            self.ball.pos = self.character.pos + self.ball.vel.normalize() * BALL_SWING_DISTANCE;
        } else {
            self.ball.update(level, delta_time);
            if self.ball.stand {
                self.chain_len -= 5.0 * delta_time;
                if self.chain_len < 0.1 {
                    self.chain_len = 0.1;
                    self.ball_in_hands = true;
                }
            }
            let delta_pos = self.ball.pos - self.character.pos;
            if delta_pos.len() > self.chain_len {
                self.character.pos += delta_pos.normalize() * (delta_pos.len() - self.chain_len);
            }
        }
        self.character.update(level, delta_time);
    }
}

type Segment = [Vec2<f32>; 2];

pub struct Game {
    time: f32,
    geng: Rc<Geng>,
    assets: Rc<Assets>,
    renderer: Renderer,
    line_renderer: LineRenderer,
    camera: Camera,
    player: Player,
    save: Option<Player>,
    level: Vec<Segment>,
    tiles: Vec<Vec2<f32>>,
    framebuffer_size: Vec2<usize>,
    spin: bool,
}

impl Game {
    pub fn new(geng: &Rc<Geng>, assets: &Rc<Assets>) -> Self {
        // let framebuffer = ugli::FramebufferRead::new_color(
        //     geng.ugli(),
        //     ugli::ColorAttachmentRead::Texture(&assets.level),
        // );
        // let data = framebuffer.read_color();
        // let cell_size = 20;
        // assert!(assets.level.size().x % cell_size == 0);
        // assert!(assets.level.size().y % cell_size == 0);
        // let mut values = Vec::new();
        // for x in (0..assets.level.size().x).step_by(cell_size) {
        //     let mut row = Vec::new();
        //     for y in (0..assets.level.size().y).step_by(cell_size) {
        //         let mut sum = 0.0;
        //         for dx in 0..cell_size {
        //             for dy in 0..cell_size {
        //                 let color = data.get(x + dx, y + dy);
        //                 let color: Color<f32> = color.convert();
        //                 sum += color.a;
        //             }
        //         }
        //         sum /= (cell_size * cell_size) as f32;
        //         row.push(sum);
        //     }
        //     row.reverse();
        //     values.push(row);
        // }
        // let mut level = Vec::new();
        // let mut help = |p: [(Vec2<f32>, f32); 3]| {
        //     let mut zeros = Vec::new();
        //     for i in 0..3 {
        //         let (p1, v1) = p[i];
        //         let (p2, v2) = p[(i + 1) % 3];
        //         if v1 == 0.5 && v2 == 0.5 {
        //             zeros.push(p1);
        //             zeros.push(p2);
        //         }
        //         if (v1 < 0.5 && v2 > 0.5) || (v1 > 0.5 && v2 < 0.5) {
        //             // println!("{:?}", p);
        //             // v1 + (v2 - v1) * t = 0.5
        //             let t = (0.5 - v1) / (v2 - v1);
        //             let p = p1 + (p2 - p1) * t;
        //             // println!("{:?}", p);
        //             zeros.push(p);
        //         }
        //     }
        //     for p in &mut zeros {
        //         *p /= 5.0;
        //     }
        //     if zeros.len() == 2 {
        //         level.push([zeros[0], zeros[1]]);
        //     }
        // };
        // let get = |x: usize, y: usize| (vec2(x as f32, y as f32), values[x][y]);
        // for x in 0..values.len() {
        //     for y in 0..values[x].len() {
        //         if values[x][y] > 0.5 {
        //             let tile_pos = vec2(x as f32, y as f32);
        //             level.push([tile_pos, tile_pos + vec2(1.0, 0.0)]);
        //             level.push([tile_pos, tile_pos + vec2(0.0, 1.0)]);
        //             level.push([tile_pos + vec2(1.0, 1.0), tile_pos + vec2(1.0, 0.0)]);
        //             level.push([tile_pos + vec2(1.0, 1.0), tile_pos + vec2(0.0, 1.0)]);
        //         }
        //     }
        // }
        let (level, tiles) = serde_json::from_str(&assets.level).unwrap();
        Self {
            time: 0.0,
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera::new(30.0),
            player: Player::new(),
            // tiles: Vec::new(),
            renderer: Renderer::new(geng),
            line_renderer: LineRenderer::new(geng),
            // level: Vec::new(),
            level,
            tiles,
            spin: false,
            // level_size: (assets.level.size() / cell_size).map(|x| x as f32),
            save: None,
            framebuffer_size: vec2(1, 1),
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.time += delta_time;
        if self.geng.window().is_key_pressed(geng::Key::S) {
            self.player.chain_len = (self.player.chain_len - 2.0 * delta_time).max(0.05);
        }
        const STEPS: usize = 100;
        for _ in 0..STEPS {
            self.player.update(&self.level, delta_time / STEPS as f32);
        }
        if self.player.ball_in_hands {
            self.player.ball.vel = Vec2::rotated(vec2(25.0, 0.0), self.time * 15.0);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::rgb(0.8, 0.8, 1.0)), None);
        // self.renderer.draw(
        //     framebuffer,
        //     &self.camera,
        //     Mat4::scale(self.level_size.extend(1.0)),
        //     &self.assets.level,
        //     Color::WHITE,
        // );
        for &tile in &self.tiles {
            self.renderer.draw(
                framebuffer,
                &self.camera,
                Mat4::translate(tile.extend(0.0)),
                &self.assets.block,
                Color::WHITE,
            );
        }
        if !self.player.ball_in_hands {
            self.line_renderer.draw_strip(
                framebuffer,
                &self.camera,
                Color::BLACK,
                vec![self.player.character.pos, self.player.ball.pos],
            );
            let e1 = self.player.ball.pos - self.player.character.pos;
            let e2 = Vec2::rotate_90(e1).normalize();
            self.renderer.draw(
                framebuffer,
                &self.camera,
                Mat4::translate(self.player.character.pos.extend(0.0))
                    * Mat4::from_orts(e2.extend(0.0), e1.extend(0.0), vec3(0.0, 0.0, 1.0))
                    * Mat4::translate(vec3(-1.0, 0.0, 0.0))
                    * Mat4::scale(vec3(2.0, 1.0, 1.0)),
                &self.assets.chain,
                Color::WHITE,
            );
        }
        self.renderer.draw(
            framebuffer,
            &self.camera,
            self.player.character.matrix()
                * Mat4::translate(vec3(-1.0, -1.0, 0.0))
                * Mat4::scale_uniform(2.0),
            &self.assets.player,
            Color::WHITE,
        );
        if !self.spin && self.player.ball_in_hands {
            self.player.ball.pos = self.player.character.pos + vec2(0.0, 1.0);
        }
        self.renderer.draw(
            framebuffer,
            &self.camera,
            self.player.ball.matrix()
                * Mat4::translate(vec3(-1.0, -1.0, 0.0))
                * Mat4::scale_uniform(2.0),
            &self.assets.ball,
            Color::WHITE,
        );
        // self.line_renderer.draw(
        //     framebuffer,
        //     &self.camera,
        //     Color::WHITE,
        //     self.level
        //         .iter()
        //         .flat_map(|&[p1, p2]| std::iter::once(p1).chain(std::iter::once(p2))),
        // );
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            // geng::Event::MouseDown {
            //     position,
            //     button: geng::MouseButton::Right,
            // } => {
            //     let world_pos = self.camera.screen_to_world(
            //         self.framebuffer_size.map(|x| x as f32),
            //         position.map(|x| x as f32),
            //     );
            //     let tile_pos = world_pos.map(|x| x.floor());
            //     self.tiles.push(tile_pos);
            //     self.level.push([tile_pos, tile_pos + vec2(1.0, 0.0)]);
            //     self.level.push([tile_pos, tile_pos + vec2(0.0, 1.0)]);
            //     self.level
            //         .push([tile_pos + vec2(1.0, 1.0), tile_pos + vec2(1.0, 0.0)]);
            //     self.level
            //         .push([tile_pos + vec2(1.0, 1.0), tile_pos + vec2(0.0, 1.0)]);
            // }
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                ..
            } => {
                self.spin = true;
            }
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                self.spin = false;
                if self.player.ball_in_hands {
                    self.player.ball_in_hands = false;
                    // self.player.ball.pos = self.player.character.pos;
                    self.player.ball.vel = Vec2::rotate_90(self.player.ball.vel);
                    self.player.ball.stand = false;
                    self.player.chain_len = 1.0;
                }
                self.player.chain_len = 2.0;
            }
            geng::Event::KeyDown { key } => match key {
                geng::Key::W => {}
                // geng::Key::Z => {
                //     for _ in 0..4 {
                //         self.level.pop();
                //     }
                //     self.tiles.pop();
                // }
                geng::Key::P => {
                    self.save = Some(self.player.clone());
                }
                // geng::Key::S if self.geng.window().is_key_pressed(geng::Key::LCtrl) => {
                //     serde_json::to_writer(
                //         std::fs::File::create("level.json").unwrap(),
                //         &(&self.level, &self.tiles),
                //     )
                //     .unwrap();
                // }
                geng::Key::L => {
                    if let Some(save) = &self.save {
                        self.player = save.clone();
                    }
                }
                geng::Key::R => self.player = Player::new(),
                _ => {}
            },
            _ => {}
        }
    }
}
