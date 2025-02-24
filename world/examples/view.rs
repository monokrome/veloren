use std::ops::{Add, Mul, Sub};
use vek::*;
use veloren_world::World;

const W: usize = 640;
const H: usize = 480;

fn main() {
    let world = World::generate(0);

    let mut win =
        minifb::Window::new("World Viewer", W, H, minifb::WindowOptions::default()).unwrap();

    let mut focus = Vec2::zero();
    let mut gain = 1.0;

    while win.is_open() {
        let mut buf = [0; W * H];

        for i in 0..W {
            for j in 0..H {
                let pos = focus + Vec2::new(i as i32, j as i32) * 4;

                let alt = world
                    .sim()
                    .sampler()
                    .sample_2d(pos)
                    .map(|sample| sample.alt.sub(64.0).add(gain).mul(0.7).max(0.0).min(255.0) as u8)
                    .unwrap_or(0);

                buf[j * W + i] = u32::from_le_bytes([alt; 4]);
            }
        }

        let spd = 32;
        if win.is_key_down(minifb::Key::W) {
            focus.y -= spd;
        }
        if win.is_key_down(minifb::Key::A) {
            focus.x -= spd;
        }
        if win.is_key_down(minifb::Key::S) {
            focus.y += spd;
        }
        if win.is_key_down(minifb::Key::D) {
            focus.x += spd;
        }
        if win.is_key_down(minifb::Key::Q) {
            gain += 10.0;
        }
        if win.is_key_down(minifb::Key::E) {
            gain -= 10.0;
        }

        win.update_with_buffer(&buf).unwrap();
    }
}
