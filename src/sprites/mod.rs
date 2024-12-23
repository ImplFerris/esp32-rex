mod resources;

use resources::*;

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::{Point, Transform},
};
use heapless::spsc::Queue;
const BUFF_SIZE: usize = 4;
const OLED_WIDTH: i32 = 128;
// const OLED_HEIGHT: i32 = 64;

type ImgRawType = ImageRaw<'static, BinaryColor>;

// Raw image of sprites
const RAW_GROUND: ImgRawType = ImageRaw::new(&SPRITE_GROUND, GROUND_X_LENGTH as u32);
const RAW_TREX: ImgRawType = ImageRaw::new(&SPRITE_TREX, 25);
const RAW_CACTUS1: ImgRawType = ImageRaw::new(&SPRITE_CACTUS1, 11);
const RAW_CACTUS2: ImgRawType = ImageRaw::new(&SPRITE_CACTUS2, 22);
const RAW_CACTUS3: ImgRawType = ImageRaw::new(&SPRITE_CACTUS3, 21);

pub const RAW_GAME_OVER: ImgRawType = ImageRaw::new(&SPRITE_GAME_OVER, 100);

const OBSTACLES: [ImgRawType; 3] = [RAW_CACTUS1, RAW_CACTUS2, RAW_CACTUS3];

// Ground line Info
pub const GROUND_X_LENGTH: i32 = 1200;
pub const GROUND_Y: i32 = 54;
pub const GROUND_X_START: i32 = 0;
// pub const GROUND_X_END: i32 = 127;

// Trex Init Position
pub const TREX_X: i32 = 10;
// pub const TREX_INIT_Y: i32 = 29;
pub const TREX_GROUND_Y: i32 = 29;
pub const TREX_MIN_Y: i32 = 3;

// Cactus info
pub const CACTUS_Y: i32 = 35;

// Movements - Adjust these values to redue/increase speed
pub const TREX_VELOCITY: i32 = -10;
pub const GRAVITY: i32 = 6;
pub const OBSTACLE_VELOCITY: i32 = -14; // obstacles moving left side, so it is X-velocity
pub const OBSTACLE_GAP: i32 = 100;

#[derive(Debug, PartialEq)]
pub enum TrexState {
    Running,
    Jumping,
    Falling,
}

#[derive(Debug)]
pub struct Trex {
    pub img: Image<'static, ImgRawType>,
    pub position: Point,
    pub state: TrexState,
}

impl Trex {
    pub fn new(x: i32, y: i32) -> Self {
        let position = Point::new(x, y);
        let image = Image::new(&RAW_TREX, position);
        Self {
            img: image,
            state: TrexState::Running,
            position,
        }
    }

    pub fn update_posistion(&mut self, x: i32, y: i32) {
        //TODO:: updating existing image
        self.img = Image::new(&RAW_TREX, Point::new(x, y));
        // self.img = self.img.translate(Point::new(self.position.x, velocity));
    }

    pub fn update_state(&mut self) {
        match self.state {
            TrexState::Jumping => {
                // Velocity is negative, the Y value decreases, causing the T-Rex to move upwards
                self.position.y += TREX_VELOCITY;
                if self.position.y <= TREX_MIN_Y {
                    self.position.y = TREX_MIN_Y;
                    self.state = TrexState::Falling;
                }
                self.update_posistion(self.position.x, self.position.y);
            }
            TrexState::Falling => {
                //Gravity is positive, the Y value increase, causing the T-Rex to move downwards
                self.position.y += GRAVITY;
                if self.position.y >= TREX_GROUND_Y {
                    self.position.y = TREX_GROUND_Y;
                    self.state = TrexState::Running;
                }
                self.update_posistion(self.position.x, self.position.y);
            }
            _ => (),
        };
    }
}

#[derive(Debug)]
pub struct Obstacle {
    pub img: Image<'static, ImgRawType>,
    pub x: i32,
    // pub y: i32,
}

impl Obstacle {
    pub fn new(raw_img: &'static ImgRawType, x: i32, y: i32) -> Self {
        let img = Image::new(raw_img, Point::new(x, y));
        Self { img, x }
    }

    /// If the velocity is negative (in our case), the obstacle moves to the left.
    pub fn move_by_velocity(&mut self, velocity: i32) {
        // self.x += velocity;
        // self.img = Image::new(&RAW_CACTUS1, Point::new(self.x, self.y));
        self.x += velocity;
        self.img = self.img.translate(Point::new(velocity, 0));
    }
}

pub struct Obstacles<R>
where
    R: super::rng::Rng,
{
    pub buffer: Queue<Obstacle, BUFF_SIZE>,
    pub rng: R,
}

impl<R> Obstacles<R>
where
    R: super::rng::Rng,
{
    pub fn new(rng: R) -> Self {
        let mut buffer = Queue::new();
        buffer
            .enqueue(Obstacle::new(&RAW_CACTUS1, OLED_WIDTH, CACTUS_Y))
            .unwrap();
        buffer
            .enqueue(Obstacle::new(
                &RAW_CACTUS2,
                OLED_WIDTH + (OBSTACLE_GAP),
                CACTUS_Y,
            ))
            .unwrap();

        Obstacles { rng, buffer }
    }

    pub fn get_current(&self) -> &Queue<Obstacle, BUFF_SIZE> {
        &self.buffer
    }

    pub fn update_state(&mut self) -> bool {
        for obstacle in self.buffer.iter_mut() {
            obstacle.move_by_velocity(OBSTACLE_VELOCITY);
        }

        let mut new_cactus = false;
        // If the first cactus has moved off the screen, replace it with a new cactus
        if let Some(first) = self.buffer.peek() {
            if first.x < 0 {
                new_cactus = true;
                // Remove the first obstacle and add a new one at the end
                self.buffer.dequeue();
                let obs_idx = self.get_random_num(3) as usize;
                self.buffer
                    .enqueue(Obstacle::new(
                        &OBSTACLES[obs_idx],
                        OLED_WIDTH + OBSTACLE_GAP,
                        35,
                    ))
                    .ok();
            }
        }
        new_cactus
    }

    /// Generates a random number within a specified limit.
    ///
    /// This function uses the `RoscRng` (Ring Oscillator Random Number Generator)
    /// to generate a random 32-bit number, which is then constrained to the given limit.
    ///
    /// # Arguments
    ///
    /// * `limit` - An unsigned 32-bit integer specifying the upper bound (exclusive) for the random number.
    ///
    /// # Returns
    ///
    /// Returns a random `u32` value in the range [0, limit).
    fn get_random_num(&mut self, limit: u32) -> u32 {
        let random = self.rng.random_u32();

        random % limit
    }
}

#[derive(Debug)]
pub struct Ground {
    pub img: Image<'static, ImgRawType>,
    position: Point,
}

impl Default for Ground {
    fn default() -> Self {
        let position = Point::new(GROUND_X_START, GROUND_Y);
        let image = Image::new(&RAW_GROUND, position);
        Self {
            img: image,
            position,
        }
    }
}

impl Ground {
    /// If the velocity is negative (in our case), the ground moves to the left.
    pub fn move_by_velocity(&mut self, velocity: i32) {
        self.position.x += velocity;
        if self.position.x < (OLED_WIDTH - GROUND_X_LENGTH) {
            self.position.x = GROUND_X_START;
        }
        self.img = Image::new(&RAW_GROUND, self.position);
    }
}
