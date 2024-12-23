use crate::sprites::{self, Ground, Obstacles, Trex, TrexState};
use core::fmt::Write;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{self, BinaryColor},
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use heapless::String;

const SCORE_BOARD_X: i32 = 60;
const SCORE_BOARD_Y: i32 = 5;

#[derive(PartialEq)]
pub enum GameState {
    // MainMenu,
    Playing,
    GameOver,
}

pub struct Game<D, R>
where
    D: DrawTarget<Color = pixelcolor::BinaryColor>,
    R: super::rng::Rng,
{
    pub state: GameState,
    pub display: D,
    pub obstacles: Obstacles<R>,
    score: u32,
    trex: Trex,
    ground: Ground,
    text_style: MonoTextStyle<'static, BinaryColor>,
}

impl<D, R> Game<D, R>
where
    D: DrawTarget<Color = pixelcolor::BinaryColor>,
    R: super::rng::Rng,
{
    pub fn new(rng: R, display: D) -> Self {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        Self {
            score: 0,
            display,
            text_style,
            trex: Trex::new(sprites::TREX_X, sprites::TREX_GROUND_Y),
            ground: Ground::default(),
            obstacles: Obstacles::new(rng),
            // TODO: Start with GameState::MainMenu
            state: GameState::Playing,
        }
    }

    pub fn move_world(&mut self) -> Result<(), D::Error> {
        if self.obstacles.update_state() {
            self.score += 1;
        }
        self.ground.move_by_velocity(sprites::OBSTACLE_VELOCITY);
        self.draw_obstacles()?;
        Ok(())
    }

    pub fn draw_obstacles(&mut self) -> Result<(), D::Error> {
        for obs in self.obstacles.get_current().iter() {
            obs.img.draw(&mut self.display)?;
        }
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<(), D::Error> {
        let text_area = Rectangle::new(Point::new(0, self.trex.position.y), Size::new(128, 64));
        text_area
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut self.display)?;
        Ok(())
    }

    pub fn draw_score(&mut self) -> Result<(), D::Error> {
        let text_area =
            Rectangle::new(Point::new(SCORE_BOARD_X, SCORE_BOARD_Y), Size::new(128, 64));
        text_area
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut self.display)?;

        let mut buff: String<32> = String::new();
        write!(buff, "Score: {}", self.score).unwrap();
        Text::with_baseline(
            &buff,
            Point::new(SCORE_BOARD_X, SCORE_BOARD_Y),
            self.text_style,
            Baseline::Top,
        )
        .draw(&mut self.display)?;
        Ok(())
    }

    pub fn draw_game_over(&mut self) -> Result<(), D::Error> {
        self.clear_screen()?;
        Image::new(&sprites::RAW_GAME_OVER, Point::new(16, 32)).draw(&mut self.display)?;
        Ok(())
    }

    pub fn draw_trex(&mut self) -> Result<(), D::Error> {
        if self.trex.state != TrexState::Running {
            self.trex.update_state();
        }

        self.trex.img.draw(&mut self.display)?;

        Ok(())
    }

    pub fn draw_ground(&mut self) -> Result<(), D::Error> {
        self.ground.img.draw(&mut self.display)?;
        Ok(())
    }

    pub fn trex_jump(&mut self) -> bool {
        if self.trex.state == TrexState::Running {
            self.trex.state = TrexState::Jumping;
            self.trex.update_state();
            return true;
        }
        false
    }

    pub fn check_collison(&mut self) -> bool {
        let trex_bbox = self.trex.img.bounding_box();

        for obs in self.obstacles.get_current().iter() {
            let obs_bbox = obs.img.bounding_box();
            if bounding_boxes_overlap(trex_bbox, obs_bbox) {
                return true;
            }
        }
        false
    }

    pub fn game_over(&mut self) -> Result<(), D::Error> {
        self.state = GameState::GameOver;
        self.draw_game_over()?;
        Ok(())
    }
}

/// Checks if two rectangular bounding boxes overlap.
///
/// This function determines whether two rectangles intersect by comparing their
/// coordinates. It's typically used to detect collisions between game objects,
/// such as the T-Rex and obstacles.
///
/// # Arguments
///
/// * `bbox1` - A `Rectangle` representing the first bounding box (typically the T-Rex's).
/// * `bbox2` - A `Rectangle` representing the second bounding box (typically an obstacle's).
///
/// # Returns
///
/// Returns `true` if the bounding boxes overlap, `false` otherwise.
fn bounding_boxes_overlap(bbox1: Rectangle, bbox2: Rectangle) -> bool {
    if let (Some(bbox1_bottom_right), Some(bbox2_bottom_right)) =
        (bbox1.bottom_right(), bbox2.bottom_right())
    {
        let x_overlap =
            bbox1.top_left.x < bbox2_bottom_right.x && bbox1_bottom_right.x > bbox2.top_left.x;
        let y_overlap =
            bbox1.top_left.y < bbox2_bottom_right.y && bbox1_bottom_right.y > bbox2.top_left.y;
        x_overlap && y_overlap
    } else {
        false
    }
}
