use nannou::color::{hsv, Hsv};
use nannou::math::deg_to_rad;
use nannou::noise::{NoiseFn, Perlin, Seedable};
use nannou::prelude::{pt2, vec2, App, Frame, Key, LoopMode, Vec2};
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const START_SEED: u32 = 4;
const WIDTH: u32 = 850;
const HEIGHT: u32 = 1080;

fn main() {
    nannou::app(model).view(view).run()
}

struct Model {
    seed: u32,
    bg_color: Hsv,
    spiral_points: Vec<(Vec2, Hsv)>,
}

fn new_point(deg: f32, radius: f32) -> Vec2 {
    let radian = deg_to_rad(deg);
    let y = radius * radian.sin();
    let x = radius * radian.cos();
    pt2(x, y)
}

// TODO support other shapes (after saving best spiral pic):
// - square
// - triangle
// Pass as an input argument
fn update_model(app: &App, model: &mut Model) {
    let _draw = app.draw();
    let win = app.window_rect().pad(20.0);
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(model.seed.into());
    let _rng = thread_rng();
    let mut deg = 0.0;
    let mut radius = 0.0;
    let mut point = vec2(0.0, 0.0);

    model.bg_color = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());

    // Foreground spiral
    model.spiral_points.clear();
    let fg_color: Hsv = hsv(seeded_rng.gen(), seeded_rng.gen(), seeded_rng.gen());
    let perlin = Perlin::new().set_seed(model.seed);

    while win.contains(point) {
        deg += 1.0;

        // Radius must increase with exponential noise growth
        // -- tune by changing hardcoded values slightly
        let noise_radius: f32 = (deg * 0.00001_f32).powf(2.0);
        radius += noise_radius;

        // Also have to add "wiggles" to the spiral that
        // increase with exponential noise growth
        // -- tune by changing hardcoded values slightly
        let noise_wiggle = perlin.get([deg as f64 * 0.07, 0.0]) as f32 * 0.02 * radius;
        radius += noise_wiggle;

        // Push the newly calculated point
        point = new_point(deg, radius);
        model.spiral_points.push((point, fg_color));
    }
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
        spiral_points: vec![],
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
            // TODO name based on the current shape (circle, square, triangle)
            let filename = format!("{}/circle-{}.png", dir, model.seed);
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
        .points_colored(model.spiral_points.clone());
    draw.to_frame(app, &frame).unwrap();
}
