use ggez;

#[allow(unused_imports)]
use ggez::event::{KeyCode, KeyMods};
use ggez::{event, graphics, Context, GameResult};

use std::f32::consts::PI;
use std::f32;


const FPS: u32 = 60;
const SCREEN_SIZE: (f32, f32) = (800.0, 600.0);

const CAR_ACCELERATION: f32 = 200.0;
const CAR_MAX_SPEED: f32 = 100.0; //good is 300.0
const CAR_STEER_LIMIT: f32 = 1.0 / 65.0;
const CAR_STEER_SPEED: f32 = CAR_STEER_LIMIT * 3.0;
const CAR_BRAKES_ACCELERATION: f32 = 500.0;


// ======================================================================

#[derive(PartialEq)]
enum Steering { Right, Left, Forward }

struct Car {
    // state
    pos: (f32, f32), // Coordinates of the car
    rot: f32,        // Rotation (in radians)
    speed: f32,      // Speed (pixels/second)
    wheels: f32,     // How much car turns when moves one pixel
    // controls
    acc: f32,        // Acceleration (pixels/second^2)
    brakes: bool,    // Are brakes active
    steer: Steering, // Steering wheel state (enum)
    soft_steer: bool // If ture, wheels steer immidiately
}


impl Car {
    fn new(coords: (f32, f32)) -> Self {
        Car {
            pos:   coords,
            rot:   0.0,
            speed: 0.0, acc: 0.0,
            wheels: 0.0, brakes: false,
            steer: Steering::Forward,
            soft_steer: true
        }
    }

    fn update(&mut self) -> GameResult<()> {
        // update speed
        if !self.brakes {
            approach_max(&mut self.speed, &self.acc, &CAR_MAX_SPEED);
        } else {
            approach_zero(&mut self.speed, &CAR_BRAKES_ACCELERATION);
        }
        // update wheels
        if self.soft_steer {
            match self.steer {
                Steering::Forward => 
                    approach_zero(&mut self.wheels, &CAR_STEER_SPEED),
                Steering::Right   => 
                    approach_max( &mut self.wheels, &CAR_STEER_SPEED, &CAR_STEER_LIMIT),
                Steering::Left    =>
                    approach_max( &mut self.wheels,&-CAR_STEER_SPEED, &CAR_STEER_LIMIT)
            }
        } else {
            match self.steer {
                Steering::Forward => self.wheels =  0.0,
                Steering::Right   => self.wheels =  CAR_STEER_LIMIT,
                Steering::Left    => self.wheels = -CAR_STEER_LIMIT
            }
        }

        // update positions
        let frame_speed = self.speed / FPS as f32;
        self.rot +=   frame_speed * self.wheels;
        if self.rot.abs() > PI { self.rot -= self.rot.signum()*PI*2.0 }
        self.pos.0 += frame_speed * self.rot.cos();
        self.pos.1 += frame_speed * self.rot.sin();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new_i32(-15, -8, 30, 16),
                [0.0, 0.0, 0.0, 1.0].into(),
            )?;
        let draw_params = graphics::DrawParam::new()
                .dest([self.pos.0, self.pos.1])
                .rotation(self.rot);

        graphics::draw(ctx, &rectangle, draw_params)?;
        /*graphics::draw(ctx, 
            &graphics::Text::new(format!("{}", (self.rot%PI)*180.0/PI)),
            graphics::DrawParam::new().dest([self.pos.0+20.0, self.pos.1-20.0]))?; */

        Ok(())
    }
}

/// Step-by-step lowers value with given speed per second to zero.
fn approach_zero(value: &mut f32, speed: &f32) {
    if *value == 0.0 { return; }
    let frame_speed = speed * value.signum() / FPS as f32;
    if (*value - frame_speed) * value.signum() > 0.0 {
        *value -= frame_speed;
    } else {
        *value = 0.0;
    }
}

/// Step by step inc's the value until it reaches max.
fn approach_max(value: &mut f32, speed: &f32, max: &f32) {
    if value.abs() == *max { return; }
    *value += *speed / FPS as f32;
    if value.abs() > *max {
        *value = max * value.signum();
    }

}

// ======================================================================

struct Autopilot {
    car: Car,
    waypoint: (f32, f32)
}

impl Autopilot {
    fn update(&mut self) -> GameResult {
        self.car.acc = CAR_ACCELERATION;
        let dx = self.waypoint.0 - self.car.pos.0;
        let dy = self.waypoint.1 - self.car.pos.1;
        let mut differance = dy.atan2(dx) - self.car.rot;
        // FUN: If following line is removed, behaves strange on overlap
        if differance.abs() > PI { differance -= PI * 2.0 * differance.signum() }
        if differance.abs() <= PI / 60.0 {
            self.car.steer = Steering::Forward;
        } else if differance > 0.0 {
            self.car.steer = Steering::Right;
        } else {
            self.car.steer = Steering::Left;
        }
        Ok(())
    }

}

// ======================================================================

struct State {
    /* player: Car, */
    autopilot: Autopilot
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ggez::timer::check_update_time(ctx, FPS) {
            self.autopilot.update()?;
            self.autopilot.car.update()?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.6, 0.6, 0.6, 0.6].into());

        self.autopilot.car.draw(ctx)?;
        //draw waypoint
        let circle = graphics::Mesh::new_circle(
            ctx, graphics::DrawMode::stroke(1.0), 
            [self.autopilot.waypoint.0 as f32, self.autopilot.waypoint.1 as f32],
            10.0, 100.0, (0, 0, 0).into()
        )?;
        graphics::draw(ctx, &circle, graphics::DrawParam::new())?;

        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }
    /*
    fn key_down_event( &mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::W => { self.player.acc =  CAR_ACCELERATION; }
            KeyCode::S => { self.player.acc = -CAR_ACCELERATION; }
            KeyCode::A => { self.player.steer = Steering::Left;  }
            KeyCode::D => { self.player.steer = Steering::Right; }
            KeyCode::Space => { self.player.brakes = true; }
            KeyCode::Escape => event::quit(ctx),
            _ => (), // Do nothing
        }
    }

    fn key_up_event( &mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        match keycode {
            KeyCode::W => { if self.player.acc > 0.0 { self.player.acc = 0.0 } }
            KeyCode::S => { if self.player.acc < 0.0 { self.player.acc = 0.0 } }
            KeyCode::A => { if self.player.steer == Steering::Left  { self.player.steer = Steering::Forward } }
            KeyCode::D => { if self.player.steer == Steering::Right { self.player.steer = Steering::Forward } }
            KeyCode::Space => { self.player.brakes = false; }
            KeyCode::Escape => event::quit(ctx),
            _ => (), // aDo nothing
        }
    }
    */

}

fn main() -> GameResult{
    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("car-city-2", "Alexey Metlitski")
        .window_setup(ggez::conf::WindowSetup::default().title("Drive The Car !"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    // state initialisation
    let autopilot = Autopilot { waypoint: (100.0, 200.0), car: Car::new((400.0, 500.0)) };
    let mut state = State { autopilot: autopilot };
    event::run(ctx, events_loop, &mut state)
}




