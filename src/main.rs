use std::time::SystemTime;

use generator::Generator;
use macroquad::prelude::*;
use rand::srand;

mod generator;

const SCALE: f32 = 30.0;
const SEPARATION: f32 = 0.0;

fn window_conf() -> Conf {
    Conf {
        window_width: 800,
        window_height: 600,
        window_resizable: true,
        window_title: String::from("Dungeon Generator"),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let start_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    srand(start_time);

    let mut generator = Generator::new();
    let mut map = generator.generate();

    let mut current_width = screen_width();
    let mut current_height = screen_height();

    loop {
        clear_background(BLACK);

        if screen_width() != current_width || current_height != screen_height() {
            current_width = screen_width();
            current_height = screen_height();

            let mut new_width = (current_width / SCALE) as usize;
            let mut new_height = (current_height / SCALE) as usize;

            if new_width % 2 != 0 {
                new_width -= 1;
            }
            if new_height % 2 != 0 {
                new_height -= 1;
            }

            if new_width != generator.dungeon_width || new_height != generator.dungeon_height {
                generator.dungeon_width = new_width;
                generator.dungeon_height = new_height;
                map = generator.generate();
            }
        }

        if is_key_pressed(KeyCode::Space) {
            map = generator.generate();
        }

        for y in 0..generator.dungeon_height {
            for x in 0..generator.dungeon_width {
                if map[y][x] != 0 {
                    let col_gen = rand::RandGenerator::new();
                    col_gen.srand(map[y][x] as u64 + start_time);

                    let col = Color::new(
                        col_gen.gen_range(0.2, 1.0),
                        col_gen.gen_range(0.2, 1.0),
                        col_gen.gen_range(0.2, 1.0),
                        1.0,
                    );
                    draw_rectangle(
                        x as f32 * SCALE,
                        y as f32 * SCALE,
                        SCALE - SEPARATION,
                        SCALE - SEPARATION,
                        col,
                    );
                }
            }
        }

        next_frame().await
    }
}
