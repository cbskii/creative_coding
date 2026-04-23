use nannou::color::{hsv, Hsv};
use nannou::math::deg_to_rad;
use nannou::noise::{NoiseFn, Perlin, Seedable};
use nannou::prelude::{pt2, vec2, App, Frame, Key, LoopMode, Rect, Vec2};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Start seed chosen based on preference
const START_SEED: u32 = 4;

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
    let mut start_point = vec2(0.0, 0.0);
    let mut points: Vec<ColorPoint> = vec![];
    let perlin = Perlin::new().set_seed(seed);
    let deg_jump = 360.0 / shape_sides as f32;

    // Tuning smoothness and noisyness
    let noise_scale = 0.06;
    let percent_radius_for_noise = 0.07;
    let num_points_between_corners = 360 * 4 / shape_sides;

    // Tuning spiral growth
    let growth_per_rotation: f32 = 1.1;
    let growth_factor = growth_per_rotation.powf(deg_jump / 360.0);

    loop {
        // The shape is drawn by connecting all corners. The angle of
        // each corner is determined by the number of sides of the shape.
        // The radius of each corner is incremented to create a growing spiral effect.
        // Calculate start and end points that connect two of the corners.
        let radian = deg_to_rad(deg + rotate_deg);
        let end_point = pt2(radius * radian.cos(), radius * radian.sin());
        if !window.contains(start_point) || !window.contains(end_point) {
            break;
        }

        // Create a line of points between start and end using linear interpolation.
        // For each point add perlin noise to create a "wiggle" effect.
        let noisy_points: Vec<ColorPoint> = (0..num_points_between_corners)
            .map(|i| {
                let percent = i as f32 / num_points_between_corners as f32;
                let point = start_point.lerp(end_point, percent);
                let cur_deg = deg + (deg_jump * percent);
                let noise: f32 = perlin.get([cur_deg as f64 * noise_scale, seed as f64]) as f32;
                let noisy_point = point + (point * percent_radius_for_noise * noise);
                (noisy_point, fg_color)
            })
            .collect();

        // Push all noisy points and proceed to the next corner
        points.extend(noisy_points);
        start_point = end_point;
        deg += deg_jump;
        radius *= growth_factor;
    }

    points
}

fn update_model(app: &App, model: &mut Model) {
    let num_sides: u32;
    let circle_num_sides = 360;
    let window = app.window_rect().pad(20.0);
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(model.seed.into());
    let mut bg_color = hsv(0.0, 0.0, 1.0);
    let mut fg_color = hsv(0.0, 0.0, 0.0);

    if COLOR_MODE {
        bg_color = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());
        fg_color = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());
    }

    // Cycle between various number of shapes
    // (order chosen based on preference with seeds)
    let num_shapes = 3;
    match model.seed % num_shapes {
        0 => num_sides = 4,
        1 => num_sides = circle_num_sides,
        2 => num_sides = 3,
        _ => unreachable!(),
    }

    // Generate a random "unraveling" spiraled shape and rotate so
    // it is roughly parallel to x-axis
    let rotation = (180.0 / num_sides as f32) - 90.0;
    model.bg_color = bg_color;
    model.fg_points = get_spiral_points(num_sides, rotation, window, fg_color, model.seed);
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
