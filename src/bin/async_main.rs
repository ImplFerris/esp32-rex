#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp32_rex::game::{Game, GameState};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Pull},
    i2c::{self, master::I2c},
    prelude::*,
};
use log::info;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_println::logger::init_logger_from_env();

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Setting up I2C send text to OLED display
    // let i2c = i2c::I2c::new_async(peripherals.I2C1, scl, sda, Irqs, i2c::Config::default());
    let i2c = I2c::new(peripherals.I2C0, i2c::master::Config::default())
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22);

    // Setup the OLED Display
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display.flush().unwrap();

    let button = Input::new(peripherals.GPIO4, Pull::Up);

    let random_gen = RandomGen::new(esp_hal::rng::Rng::new(peripherals.RNG));
    let mut game = Game::new(random_gen, display);
    game.draw_trex().unwrap();
    let mut clicked_count = 0; // To restart the game when it two times button clicked

    info!("Starting Game!");

    loop {
        if game.state == GameState::GameOver {
            if button.is_low() {
                clicked_count += 1;
            }
            if clicked_count > 2 {
                clicked_count = 0;
                game = Game::new(game.obstacles.rng, game.display);
                Timer::after_millis(500).await;
            }
            Timer::after_millis(50).await;
            continue;
        }

        game.clear_screen().unwrap();
        game.draw_score().unwrap();

        if button.is_low() {
            game.trex_jump();
        }

        game.move_world().unwrap();
        game.draw_ground().unwrap();
        game.draw_trex().unwrap();

        if game.check_collison() {
            game.game_over().unwrap();
            game.display.flush().unwrap();
            Timer::after_millis(500).await;
            continue;
        }

        game.display.flush().unwrap();
        Timer::after_millis(5).await;
    }
}

// Helper struct for Random number generation
struct RandomGen {
    rng: esp_hal::rng::Rng,
}

impl RandomGen {
    fn new(rng: esp_hal::rng::Rng) -> Self {
        Self { rng }
    }
}

impl esp32_rex::rng::Rng for RandomGen {
    fn random_u32(&mut self) -> u32 {
        self.rng.random()
    }
}
