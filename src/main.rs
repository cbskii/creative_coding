use nannou::color::{hsv, Hsv};
use nannou::math::deg_to_rad;
use nannou::noise::{NoiseFn, Perlin, Seedable};
use nannou::prelude::{pt2, vec2, App, Frame, Key, LoopMode, Rect, Vec2};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Start seed chosen based on preference
const START_SEED: u32 = 0;

// Output size
const WIDTH: u32 = 720;
const HEIGHT: u32 = 1080;

// Whether to use black and white or random colors
const COLOR_MODE: bool = true;

type ColorPoint = (Vec2, Hsv);

struct Model {
    seed: u32,
    bg_color: Hsv,
    fg_points: Vec<ColorPoint>,
}

fn main() {
    nannou::app(model).view(view).run()
}

fn get_spiral_points(
    shape_sides: u32,
    rotate_deg: f32,
    window: Rect,
    fg_color: Hsv,
    seed: u32,
) -> Vec<ColorPoint> {
    let mut deg = 0.0;
    let mut radius = 0.1;
    let mut noisy_radius;
    let noise_scale = 0.07;
    let base_growth = 30.0 / shape_sides as f32;
    let mut point = vec2(0.0, 0.0);
    let mut points: Vec<ColorPoint> = vec![];
    let perlin = Perlin::new().set_seed(seed);
    let deg_jump = 360.0 / shape_sides as f32;

    // To create a spiral for any shape we add a point for each corner
    // and draw a line between corners. The corner connecting is handled by
    // using the corner points to draw a polyline. Each corner is determined by
    // calculating the angle difference between each corner. As the angle
    // increases the radius increases as well to produce a spiral. Perlin
    // noise is added to the radius to create "wiggly" lines and the noise
    // grows as the radius grows.
    while window.contains(point) {
        points.push((point, fg_color));
        deg += deg_jump;
        radius += base_growth;
        let radian = deg_to_rad(deg + rotate_deg);
        let noise: f32 = perlin.get([deg as f64 * noise_scale, seed as f64]) as f32;
        noisy_radius = radius + (noise * radius * 0.02);
        point = pt2(noisy_radius * radian.cos(), noisy_radius * radian.sin());
    }

    points
}

fn update_model(app: &App, model: &mut Model) {
    let mut num_sides: u32;
    let min_num_sides = 2;
    let max_num_sides = 5;
    let circle_num_sides = 360;
    let window = app.window_rect().pad(20.0);
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(model.seed.into());
    let mut bg_color = hsv(0.0, 0.0, 1.0);
    let mut fg_color = hsv(0.0, 0.0, 0.0);

    if COLOR_MODE {
        bg_color = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());
        fg_color = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());
    }

    num_sides = min_num_sides + model.seed % max_num_sides;
    if num_sides == min_num_sides {
        num_sides = circle_num_sides;
    }

    // Generate a random "unraveling" spiraled shape at some degree of rotation
    model.bg_color = bg_color;
    model.fg_points = get_spiral_points(
        num_sides,
        seeded_rng.gen_range(0..=360) as f32,
        window,
        fg_color,
        model.seed,
    );
}

fn model(app: &App) -> Model {
    let _draw = app.draw();

    app.new_window()
        .size(WIDTH, HEIGHT)
        .key_released(key_released)
        .resizable(false)
        .build()
        .unwrap();

    app.set_loop_mode(LoopMode::Wait);

    let mut model = Model {
        seed: START_SEED,
        bg_color: hsv(0.0, 0.0, 0.0),
        fg_points: vec![],
    };

    update_model(app, &mut model);
    model
}

fn key_released(app: &App, model: &mut Model, key: Key) {
    match key {
        nannou::event::Key::Left => {
            model.seed = model.seed.wrapping_sub(1);
            update_model(app, model);
        }
        nannou::event::Key::Right => {
            model.seed = model.seed.wrapping_add(1);
            update_model(app, model);
        }
        nannou::event::Key::Space => {
            let dir = "favorite_images";
            let filename = format!("{}/{}.png", dir, model.seed);
            if let Err(e) = std::fs::create_dir_all(dir) {
                eprintln!("Error creating directory: {}", e);
            } else {
                app.main_window().capture_frame(&filename);
                println!("Saved to {}", filename);
            }
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // Background
    let color = model.bg_color;
    draw.background().color(color);

    // Foreground
    draw.polyline()
        .weight(6.0)
        .points_colored(model.fg_points.clone());
    draw.to_frame(app, &frame).unwrap();
}
