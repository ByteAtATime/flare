use iced::mouse;
use iced::widget::canvas::{self, Geometry, path};
use iced::{Color, Point, Rectangle, Renderer, Theme};
use rand::Rng;
use std::collections::HashMap;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub enum Shape {
    Square,
    Circle,
    Star,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub particle_count: usize,
    pub angle: f32,
    pub spread: f32,
    pub start_velocity: f32,
    pub decay: f32,
    pub gravity: f32,
    pub drift: f32,
    pub ticks: f32,
    pub origin: Point,
    pub colors: Vec<Color>,
    pub shapes: Vec<Shape>,
    pub scalar: f32,
    pub flat: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            particle_count: 50,
            angle: 90.0,
            spread: 45.0,
            start_velocity: 45.0,
            decay: 0.9,
            gravity: 1.0,
            drift: 0.0,
            ticks: 200.0,
            origin: Point::new(0.5, 0.5),
            colors: vec![
                Color::from_rgb8(0x26, 0xcc, 0xff),
                Color::from_rgb8(0xa2, 0x5a, 0xfd),
                Color::from_rgb8(0xff, 0x5e, 0x7e),
                Color::from_rgb8(0x88, 0xff, 0x5a),
                Color::from_rgb8(0xfc, 0xff, 0x42),
                Color::from_rgb8(0xff, 0xa6, 0x2d),
                Color::from_rgb8(0xff, 0x36, 0xff),
            ],
            shapes: vec![Shape::Square, Shape::Circle],
            scalar: 1.0,
            flat: false,
        }
    }
}

struct Particle {
    x: f32,
    y: f32,
    wobble: f32,
    wobble_speed: f32,
    velocity: f32,
    angle_2d: f32,
    tilt_angle: f32,
    color: Color,
    shape: Shape,
    tick: f32,
    total_ticks: f32,
    decay: f32,
    drift: f32,
    random_factor: f32,
    gravity: f32,
    scalar: f32,
    flat: bool,
    tilt_sin: f32,
    tilt_cos: f32,
    wobble_x: f32,
    wobble_y: f32,
}

pub struct Manager {
    particles: Vec<Particle>,
    cache: canvas::Cache,
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            particles: Vec::new(),
            cache: canvas::Cache::default(),
        }
    }
}

impl Manager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fire_with_bounds(&mut self, options: Options, bounds: Rectangle) {
        let mut rng = rand::rng();
        let rad_angle = options.angle * (PI / 180.0);
        let rad_spread = options.spread * (PI / 180.0);

        let start_x = options.origin.x * bounds.width;
        let start_y = options.origin.y * bounds.height;

        for i in 0..options.particle_count {
            let velocity =
                (options.start_velocity * 0.5) + (rng.random::<f32>() * options.start_velocity);
            let angle_2d = -rad_angle + ((0.5 * rad_spread) - (rng.random::<f32>() * rad_spread));

            let shape = if options.shapes.is_empty() {
                Shape::Square
            } else {
                options.shapes[rng.random_range(0..options.shapes.len())].clone()
            };

            let color = if options.colors.is_empty() {
                Color::BLACK
            } else {
                options.colors[i % options.colors.len()]
            };

            let wobble = rng.random::<f32>() * 10.0;
            let scalar = options.scalar;

            let (wobble_x, wobble_y) = if options.flat {
                (start_x + (10.0 * scalar), start_y + (10.0 * scalar))
            } else {
                (
                    start_x + (10.0 * scalar) * wobble.cos(),
                    start_y + (10.0 * scalar) * wobble.sin(),
                )
            };

            self.particles.push(Particle {
                x: start_x,
                y: start_y,
                wobble,
                wobble_speed: f32::min(0.11, rng.random::<f32>() * 0.1 + 0.05),
                velocity,
                angle_2d,
                tilt_angle: (rng.random::<f32>() * 0.5 + 0.25) * PI,
                color,
                shape,
                tick: 0.0,
                total_ticks: options.ticks,
                decay: options.decay,
                drift: options.drift,
                random_factor: rng.random::<f32>() + 2.0,
                gravity: options.gravity * 3.0,
                scalar,
                flat: options.flat,
                tilt_sin: 0.0,
                tilt_cos: 0.0,
                wobble_x,
                wobble_y,
            });
        }

        self.cache.clear();
    }

    pub fn update(&mut self) {
        if self.particles.is_empty() {
            return;
        }

        self.particles.retain_mut(|p| {
            p.x += p.angle_2d.cos() * p.velocity + p.drift;
            p.y += p.angle_2d.sin() * p.velocity + p.gravity;
            p.velocity *= p.decay;

            if p.flat {
                p.wobble = 0.0;
                p.wobble_x = p.x + (10.0 * p.scalar);
                p.wobble_y = p.y + (10.0 * p.scalar);
                p.tilt_sin = 0.0;
                p.tilt_cos = 0.0;
                p.random_factor = 1.0;
            } else {
                p.wobble += p.wobble_speed;
                p.wobble_x = p.x + ((10.0 * p.scalar) * p.wobble.cos());
                p.wobble_y = p.y + ((10.0 * p.scalar) * p.wobble.sin());
                p.tilt_angle += 0.1;
                p.tilt_sin = p.tilt_angle.sin();
                p.tilt_cos = p.tilt_angle.cos();
            }

            p.tick += 1.0;
            p.tick < p.total_ticks
        });

        self.cache.clear();
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct ColorKey([u8; 4]);

impl<Message> canvas::Program<Message> for Manager {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        if self.particles.is_empty() {
            return vec![];
        }

        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            let mut batches: HashMap<ColorKey, path::Builder> = HashMap::new();

            let bounds_width = bounds.width;
            let bounds_height = bounds.height;

            for p in &self.particles {
                if p.x < -50.0
                    || p.x > bounds_width + 50.0
                    || p.y < -50.0
                    || p.y > bounds_height + 50.0
                {
                    continue;
                }

                let progress = p.tick / p.total_ticks;
                let alpha = 1.0 - progress;

                let alpha_quantized = (alpha * 20.0).ceil() / 20.0;
                if alpha_quantized <= 0.0 {
                    continue;
                }

                let r = (p.color.r * 255.0) as u8;
                let g = (p.color.g * 255.0) as u8;
                let b = (p.color.b * 255.0) as u8;
                let a = (alpha_quantized * 255.0) as u8;

                let key = ColorKey([r, g, b, a]);

                let builder = batches.entry(key).or_insert_with(path::Builder::new);

                let x1 = p.x + (p.random_factor * p.tilt_cos);
                let y1 = p.y + (p.random_factor * p.tilt_sin);
                let x2 = p.wobble_x + (p.random_factor * p.tilt_cos);
                let y2 = p.wobble_y + (p.random_factor * p.tilt_sin);

                match p.shape {
                    Shape::Square => {
                        builder.move_to(Point::new(p.x, p.y));
                        builder.line_to(Point::new(p.wobble_x, y1));
                        builder.line_to(Point::new(x2, y2));
                        builder.line_to(Point::new(x1, p.wobble_y));
                        builder.close();
                    }
                    Shape::Circle => {
                        let width = (x2 - x1).abs() * 0.6;
                        let height = (y2 - y1).abs() * 0.6;

                        let rotation = PI / 10.0 * p.wobble;
                        let cos_r = rotation.cos();
                        let sin_r = rotation.sin();

                        let transform = |px: f32, py: f32| -> Point {
                            let sx = px * width;
                            let sy = py * height;
                            let rx = sx * cos_r - sy * sin_r;
                            let ry = sx * sin_r + sy * cos_r;
                            Point::new(p.x + rx, p.y + ry)
                        };

                        // approximate circle as octagon
                        builder.move_to(transform(1.0, 0.0));
                        builder.line_to(transform(0.707, 0.707));
                        builder.line_to(transform(0.0, 1.0));
                        builder.line_to(transform(-0.707, 0.707));
                        builder.line_to(transform(-1.0, 0.0));
                        builder.line_to(transform(-0.707, -0.707));
                        builder.line_to(transform(0.0, -1.0));
                        builder.line_to(transform(0.707, -0.707));
                        builder.close();
                    }
                    Shape::Star => {
                        let width = (x2 - x1).abs();
                        let height = (y2 - y1).abs();
                        let rotation = PI / 10.0 * p.wobble;

                        let scale_x = width * 0.1;
                        let scale_y = height * 0.1;

                        let cos_r = rotation.cos();
                        let sin_r = rotation.sin();

                        let transform = |px: f32, py: f32| -> Point {
                            let sx = px * scale_x;
                            let sy = py * scale_y;
                            let rx = sx * cos_r - sy * sin_r;
                            let ry = sx * sin_r + sy * cos_r;
                            Point::new(p.x + rx, p.y + ry)
                        };

                        let inner_radius = 4.0;
                        let outer_radius = 8.0;
                        let start_rot = PI / 2.0 * 3.0;
                        let step = PI / 5.0;
                        let mut curr_rot = start_rot;

                        builder.move_to(transform(
                            curr_rot.cos() * outer_radius,
                            curr_rot.sin() * outer_radius,
                        ));

                        for _ in 0..5 {
                            curr_rot += step;
                            builder.line_to(transform(
                                curr_rot.cos() * inner_radius,
                                curr_rot.sin() * inner_radius,
                            ));
                            curr_rot += step;
                            builder.line_to(transform(
                                curr_rot.cos() * outer_radius,
                                curr_rot.sin() * outer_radius,
                            ));
                        }
                        builder.close();
                    }
                }
            }

            for (key, builder) in batches {
                let color = Color {
                    r: key.0[0] as f32 / 255.0,
                    g: key.0[1] as f32 / 255.0,
                    b: key.0[2] as f32 / 255.0,
                    a: key.0[3] as f32 / 255.0,
                };

                frame.fill(&builder.build(), color);
            }
        });

        vec![geom]
    }
}
