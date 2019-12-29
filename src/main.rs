use ggez;

use ggez::event::{KeyCode, KeyMods};
use ggez::{event, graphics, Context, GameResult};

use std::f64::consts::PI;
use std::f64;


const FPS: u32 = 60;
const SCREEN_SIZE: (f32, f32) = (800.0, 600.0);

const CAR_ACCELERATION: f64 = 200.0;
const CAR_MAX_SPEED: f64 = 300.0;
const CAR_STEER_LIMIT: f64 = 1.0 / 65.0;
const CAR_STEER_SPEED: f64 = CAR_STEER_LIMIT * 3.0;
const CAR_BRAKES_ACCELERATION: f64 = 500.0;


// ======================================================================

#[derive(PartialEq)]
enum Steering { Right, Left, Forward }

struct Car {
    // state
    pos: (f64, f64), // Coordinates of the car
    rot: f64,        // Rotation (in radians)
    speed: f64,      // Speed (pixels/second)
    wheels: f64,     // How much car turns when moves one pixel
    // controls
    acc: f64,        // Acceleration (pixels/second^2)
    brakes: bool,    // Are brakes active
    steer: Steering, // Steering wheel state (enum)
    fast_steer: bool // If ture, wheels steer immidiately
}


impl Car {
    fn new(coords: (f64, f64)) -> Self {
        Car {
            pos:   coords,
            rot:   0.0,
            speed: 0.0, acc: 0.0,
            wheels: 0.0, brakes: false,
            steer: Steering::Forward,
            fast_steer: false
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
        if !self.fast_steer {
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
        let frame_speed = self.speed / FPS as f64;
        self.rot +=   frame_speed * self.wheels; 
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
                .dest([self.pos.0 as f32, self.pos.1 as f32])
                .rotation(self.rot as f32);

        graphics::draw(ctx, &rectangle, draw_params)?;
        Ok(())
    }
}

/// Step-by-step lowers value with given speed per second to zero.
fn approach_zero(value: &mut f64, speed: &f64) {
    if *value == 0.0 { return; }
    let frame_speed = speed * value.signum() / FPS as f64;
    if (*value - frame_speed) * value.signum() > 0.0 {
        *value -= frame_speed;
    } else {
        *value = 0.0;
    }
}

/// Step by step inc's the value until it reaches max.
fn approach_max(value: &mut f64, speed: &f64, max: &f64) {
    if value.abs() == *max { return; }
    *value += *speed / FPS as f64;
    if value.abs() > *max {
        *value = max * value.signum();
    }

}

struct State {
    player: Car
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ggez::timer::check_update_time(ctx, FPS) {
            self.player.update()?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.6, 0.6, 0.6, 0.6].into());

        self.player.draw(ctx)?;

        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }
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

}

fn main() -> GameResult{
    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("car-city-2", "Alexey Metlitski")
        .window_setup(ggez::conf::WindowSetup::default().title("Drive The Car !"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    // state initialisation
    let mut state = State { player: Car::new((100.0, 100.0)) };
    event::run(ctx, events_loop, &mut state)
}



