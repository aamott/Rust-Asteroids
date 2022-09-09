/// This code was based on the Macroquad asteroids example:
/// https://github.com/not-fl3/macroquad/blob/master/examples/asteroids.rs
/// Modifications were made where it felt better to focus on Rust learning rather
/// than on the Macroquad interface (mostly no Vec2 usage).
use macroquad::prelude::*;

const SHIP_HEIGHT: f32 = 25.;
const SHIP_BASE: f32 = 22.;
const TIME_BETWEEN_SHOTS: f64 = 0.2;

#[derive(Debug, Default, Copy, Clone)]
struct Point {
    x: f32,
    y: f32,
}
impl Point {
    /// finds the distance between this point and another point
    fn distance(&self, point: &Point) -> f32 {
        // self :x2/y2
        // point: x1/y1
        ((self.x - point.x).powi(2) + (self.y - point.y).powi(2)).sqrt()
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Velocity {
    fn add_at_angle(&mut self, velocity: f32, angle: f32) {
        let radians = angle.to_radians();

        self.x += radians.sin() / 3. * velocity;
        self.y += -radians.cos() / 3. * velocity;
    }

    fn add_velocity(&mut self, velocity: Velocity) {
        self.x += velocity.x;
        self.y += velocity.y;
    }
}

struct Ship {
    pos: Point,
    vel: Velocity,
    rotation: f32,
}
impl Default for Ship {
    fn default() -> Ship {
        Ship {
            pos: Point::default(),
            vel: Velocity::default(),
            rotation: 0.,
        }
    }
}
impl Ship {
    fn advance(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }
}

struct Bullet {
    pos: Point,
    vel: Velocity,
    initial_frame: f64,
    collided: bool,
}
impl Bullet {
    fn advance(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }
}

struct Asteroid {
    pos: Point,
    vel: Velocity,
    rotation: f32,
    rot_speed: f32,
    size: f32,
    sides: u8,
    collided: bool,
}
impl Asteroid {
    fn advance(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;

        self.rotation += self.rot_speed;
    }
}

/// creates a set number of starting asteroids
fn generate_asteroid(avoid_point: Point, avoid_distance: f32) -> Asteroid {
    // generate a random point that is at least 1/6th screen width from the ship
    let mut asteroid_pos = Point::default();
    let asteroid_size = screen_width().min(screen_height()) / 10.;

    let mut point_ready = false;
    while point_ready == false {
        asteroid_pos = Point {
            x: rand::gen_range(-0., 1.) * screen_width(),
            y: rand::gen_range(0., 1.) * screen_height(),
        };

        point_ready = asteroid_pos.distance(&avoid_point) > asteroid_size + avoid_distance;
    }

    Asteroid {
        pos: asteroid_pos,
        vel: Velocity { x: rand::gen_range(-1., 1.), y: rand::gen_range(-1., 1.) },
        rotation: rand::gen_range(-1., 1.),
        rot_speed: rand::gen_range(-1., 1.),
        size: asteroid_size,
        sides: 6,
        collided: false,
    }
}

/// Wraps objects when they hit the edge of the screen
fn wrap_around(point: &mut Point) {
    let width = screen_width();
    if point.x > width {
        point.x = 0.;
    }
    if point.x < 0. {
        point.x = width
    }

    let height = screen_height();
    if point.y > height {
        point.y = 0.;
    }
    if point.y < 0. {
        point.y = height
    }
}

#[macroquad::main("Asteroids")]
async fn main() {
    let mut ship;
    let mut asteroids = Vec::new();
    let mut bullets = Vec::new();
    let mut last_shot = get_time();
    let mut gameover = false;

    // setup game
    ship = Ship {
        pos: Point {
            x: screen_width() / 2.,
            y: screen_height() / 2.,
        },
        vel: Velocity::default(),
        rotation: 0.,
    };

    // prepare the asteroids
    for _ in 0..10 {
        asteroids.push(generate_asteroid(
            ship.pos,
            SHIP_HEIGHT * 3.,
        ));
    }

    loop {
        if gameover {
            let mut text = "You win! Press enter to play again.";
            let font_size = 23.;

            // Reset the Game on Enter
            if is_key_down(KeyCode::Enter) {
                ship = Ship {
                    pos: Point {
                        x: screen_width() / 2.,
                        y: screen_height() / 2.,
                    },
                    vel: Velocity::default(),
                    rotation: 0.,
                };

                asteroids = Vec::new();
                bullets = Vec::new();

                // prepare the asteroids
                for _ in 0..10 {
                    asteroids.push(generate_asteroid(
                        ship.pos,
                        SHIP_HEIGHT * 3.,
                    ));
                }

                gameover = false;
                continue;
            }

            if asteroids.len() > 0 {
                text = "Game Over. Press enter to play again.";
            }

            let text_size = measure_text(text, None, font_size as _, 1.0);
            draw_text(
                text,
                screen_width() / 2. - text_size.width / 2.,
                screen_height() / 2. - text_size.height / 2.,
                font_size,
                DARKGRAY,
            );
            next_frame().await;
            continue;
        }

        let frame_time = get_time();

        if is_key_down(KeyCode::Up) {
            ship.vel.add_at_angle(0.5, ship.rotation)
        } else {
            // decelerate over time
            if ship.vel.x > 0.1 {
                ship.vel.x -= 0.01 * ship.vel.x.abs();
            } else if ship.vel.x < -0.1 {
                ship.vel.x += 0.01 * ship.vel.x.abs();
            }
            if ship.vel.y > 0.1 {
                ship.vel.y -= 0.01 * ship.vel.y.abs();
            } else if ship.vel.y < -0.1 {
                ship.vel.y += 0.01 * ship.vel.y.abs();
            }
        }

        if is_key_down(KeyCode::Space) && frame_time - last_shot > TIME_BETWEEN_SHOTS{
            let mut velocity = Velocity::default();
            velocity.add_at_angle(7., ship.rotation);

            let mut bullet = Bullet {
                pos: ship.pos.clone(),
                vel: velocity,
                initial_frame: frame_time,
                collided: false,
            };

            // advance the bullet to get it past the ship.
            bullet.advance();
            bullet.advance();
            bullet.vel.add_velocity(ship.vel);
            bullets.push(bullet);

            last_shot = frame_time;
        }

        if is_key_down(KeyCode::Right) {
            ship.rotation += 3.;
        } else if is_key_down(KeyCode::Left) {
            ship.rotation -= 3.;
        }

        // move ship forward
        ship.advance();
        wrap_around(&mut ship.pos);

        for bullet in bullets.iter_mut() {
            bullet.advance();
            wrap_around(&mut bullet.pos);
        }
        for asteroid in asteroids.iter_mut() {
            asteroid.advance();
            wrap_around(&mut asteroid.pos);
        }

        // Check for collisions
        let mut new_asteroids = Vec::new();
        for asteroid in asteroids.iter_mut() {
            // check for asteroid strikes
            if asteroid.pos.distance(&ship.pos) < asteroid.size + SHIP_HEIGHT / 3. {
                gameover = true;
                break;
            }

            // check for asteroid
            for bullet in bullets.iter_mut() {
                if asteroid.pos.distance(&bullet.pos) < asteroid.size {
                    asteroid.collided = true;
                    bullet.collided = true;

                    if asteroid.sides > 4 {
                        let explosiveness = rand::gen_range(0., 1.);
                        new_asteroids.push(Asteroid {
                            pos: asteroid.pos,
                            vel: Velocity {
                                x: bullet.vel.x / 5. + (asteroid.vel.x + explosiveness) * rand::gen_range(0., 2.),
                                y: bullet.vel.y  / 5. + (asteroid.vel.y + explosiveness) * rand::gen_range(0., 2.),
                            },
                            rotation: rand::gen_range(0., 360.),
                            rot_speed: rand::gen_range(-2., 2.),
                            size: asteroid.size * 0.6,
                            sides: asteroid.sides - 1,
                            collided: false,
                        });
                        new_asteroids.push(Asteroid {
                            pos: asteroid.pos,
                            vel: Velocity {
                                x: bullet.vel.x / 5. + (asteroid.vel.x + explosiveness) * rand::gen_range(0., 2.),
                                y: bullet.vel.y  / 5. + (asteroid.vel.y + explosiveness) * rand::gen_range(0., 2.),
                            },
                            rotation: rand::gen_range(0., 360.),
                            rot_speed: rand::gen_range(-2., 2.),
                            size: asteroid.size * 0.6,
                            sides: asteroid.sides - 1,
                            collided: false,
                        });
                    }
                    break;
                }
            }
        }

        // retains bullets that meet the criteria of the closure
        bullets.retain(|bullet| bullet.initial_frame + 1.5 > frame_time && !bullet.collided);
        asteroids.retain(|asteroid| !asteroid.collided);
        asteroids.append(&mut new_asteroids);

        if asteroids.len() == 0 {
            gameover = true;
            continue;
        }

        // DRAWING
        clear_background(LIGHTGRAY);
        for bullet in bullets.iter() {
            draw_circle(bullet.pos.x, bullet.pos.y, 2., BLACK);
        }

        for asteroid in asteroids.iter() {
            draw_poly_lines(
                asteroid.pos.x,
                asteroid.pos.y,
                asteroid.sides,
                asteroid.size,
                asteroid.rotation,
                2.,
                BLACK,
            );
        }

        let rotation = ship.rotation.to_radians();

        let v1 = Vec2::new(
            ship.pos.x + rotation.sin() * SHIP_HEIGHT / 2.,
            ship.pos.y - rotation.cos() * SHIP_HEIGHT / 2.,
        );
        let v2 = Vec2::new(
            ship.pos.x - rotation.cos() * SHIP_BASE / 2. - rotation.sin() * SHIP_HEIGHT / 2.,
            ship.pos.y - rotation.sin() * SHIP_BASE / 2. + rotation.cos() * SHIP_HEIGHT / 2.,
        );
        let v3 = Vec2::new(
            ship.pos.x + rotation.cos() * SHIP_BASE / 2. - rotation.sin() * SHIP_HEIGHT / 2.,
            ship.pos.y + rotation.sin() * SHIP_BASE / 2. + rotation.cos() * SHIP_HEIGHT / 2.,
        );
        draw_triangle_lines(v1, v2, v3, 2., BLACK);

        next_frame().await
    }
}
