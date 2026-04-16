use nannou::color::{hsv, Hsv};
use nannou::math::deg_to_rad;
use nannou::noise::{NoiseFn, Perlin, Seedable};
use nannou::prelude::{pt2, vec2, App, Frame, Key, LoopMode, Rect, Vec2};
use rand::{thread_rng, Rng, SeedableRng};
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
    circle_points: Vec<ColorPoint>,
}

fn main() {
    nannou::app(model).view(view).run()
}

fn new_circle_point(deg: f32, radius: f32) -> Vec2 {
    let radian = deg_to_rad(deg);
    let y = radius * radian.sin();
    let x = radius * radian.cos();
    pt2(x, y)
}

fn get_circle_points(window: Rect<f32>, fg_color: Hsv, seed: u32) -> Vec<ColorPoint> {
    let mut deg = 0.0;
    let mut radius = 0.0;
    let mut point = vec2(0.0, 0.0);
    let mut points: Vec<ColorPoint> = vec![];
    let perlin = Perlin::new().set_seed(seed);

    while window.contains(point) {
        points.push((point, fg_color));
        deg += 1.0;

        // Radius must increase with exponential noise growth
        // -- tune by changing hardcoded values slightly
        let noise_radius: f32 = (deg * 0.00001_f32).powf(2.0);
        radius += noise_radius;

        // Also have to add "wiggles" to the unraveling line that
        // increase with exponential noise growth
        // -- tune by changing hardcoded values slightly
        let noise_wiggle = perlin.get([deg as f64 * 0.07, 0.0]) as f32 * 0.02 * radius;
        radius += noise_wiggle;

        point = new_circle_point(deg, radius);
    }

    points
}

fn update_model(app: &App, model: &mut Model) {
    let window = app.window_rect().pad(20.0);
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(model.seed.into());
    let mut bg_color = hsv(0.0, 0.0, 1.0);
    let mut fg_color = hsv(0.0, 0.0, 0.0);

    if COLOR_MODE {
        bg_color = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());
        fg_color = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());
    }

    model.bg_color = bg_color;
    model.circle_points = get_circle_points(window, fg_color, model.seed);
    // TODO alternate displaying rect, circle, triangle based on seed % 3
    // model.rect_points = get_rect_points(window, model.seed.into());
    // model.triangle_points = get_triangle_points(window, model.seed.into());
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
        circle_points: vec![],
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
        .points_colored(model.circle_points.clone());
    draw.to_frame(app, &frame).unwrap();
}
