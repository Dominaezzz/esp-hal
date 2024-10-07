//! Drives the 16-bit parallel RGB display on Makerfabs ESP32-S3-Parallel-TFT-with-Touch-4.3inch
//!
//! This example draws some colour bars at the top of the screen.
//!
//! The following wiring is assumed:
//! - LCD_VYSNC  => GPIO3
//! - LCD_HSYNC  => GPIO46
//! - LCD_DE     => GPIO17
//! - LCD_PCLK   => GPIO9
//! - LCD_DATA0  => GPIO10
//! - LCD_DATA1  => GPIO11
//! - LCD_DATA2  => GPIO12
//! - LCD_DATA3  => GPIO13
//! - LCD_DATA4  => GPIO14
//! - LCD_DATA5  => GPIO21
//! - LCD_DATA6  => GPIO8
//! - LCD_DATA7  => GPIO18
//! - LCD_DATA8  => GPIO45
//! - LCD_DATA9  => GPIO38
//! - LCD_DATA10 => GPIO39
//! - LCD_DATA11 => GPIO40
//! - LCD_DATA12 => GPIO41
//! - LCD_DATA13 => GPIO42
//! - LCD_DATA14 => GPIO2
//! - LCD_DATA15 => GPIO1

//% CHIPS: esp32s3

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority},
    dma_tx_buffer,
    gpio::{Io, Level},
    lcd_cam::{
        lcd::{
            dpi::{Config, Dpi, Format, FrameTiming},
            ClockMode,
            Phase,
            Polarity,
        },
        LcdCam,
    },
    prelude::*,
};
use esp_println::println;

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let dma = Dma::new(peripherals.DMA);
    let channel = dma.channel2.configure(false, DmaPriority::Priority0);
    let lcd_cam = LcdCam::new(peripherals.LCD_CAM);

    let mut dma_tx_buf = dma_tx_buffer!(800 * 2 * 100).unwrap();

    let config = Config {
        clock_mode: ClockMode {
            polarity: Polarity::IdleLow,
            phase: Phase::ShiftHigh,
        },
        format: Format {
            enable_2byte_mode: true,
            ..Default::default()
        },
        // https://www.makerfabs.com/desfile/files/QT4300H40R10-V03-Spec.pdf
        timing: FrameTiming {
            horizontal_active_width: 800,
            horizontal_total_width: 928,      // 889..1143
            horizontal_blank_front_porch: 40, // 1..255

            vertical_active_height: 480,
            vertical_total_height: 525,     // 513..767
            vertical_blank_front_porch: 13, // 1..255

            hsync_width: 48, // 1..255
            vsync_width: 3,  // 3..255

            hsync_position: 0,
        },
        vsync_idle_level: Level::High,
        hsync_idle_level: Level::High,
        de_idle_level: Level::Low,
        disable_black_region: false,
        ..Default::default()
    };

    // https://raw.githubusercontent.com/Makerfabs/ESP32-S3-Parallel-TFT-with-Touch-4.3inch/main/hardware/ESP32-S3%20Parallel%20TFT%20with%20Touch%204.3%E2%80%9C%20V3.1.PDF
    let mut dpi = Dpi::new(lcd_cam.lcd, channel.tx, 30.MHz(), config)
        .with_ctrl_pins(
            io.pins.gpio41,
            io.pins.gpio39,
            io.pins.gpio40,
            io.pins.gpio42,
        )
        .with_data_pins(
            // Blue
            io.pins.gpio8,
            io.pins.gpio3,
            io.pins.gpio46,
            io.pins.gpio9,
            io.pins.gpio1,
            // Green
            io.pins.gpio5,
            io.pins.gpio6,
            io.pins.gpio7,
            io.pins.gpio15,
            io.pins.gpio16,
            io.pins.gpio4,
            // Red
            io.pins.gpio45,
            io.pins.gpio48,
            io.pins.gpio47,
            io.pins.gpio21,
            io.pins.gpio14,
        );

    const RED: u16 = 0b11111_000000_00000;
    const GREEN: u16 = 0b00000_111111_00000;
    const BLUE: u16 = 0b00000_000000_11111;

    dma_tx_buf.as_mut_slice().fill(0);
    let mut chunks = dma_tx_buf.as_mut_slice().chunks_mut(2);

    for chunk in chunks.by_ref().take(800 * 30) {
        chunk.copy_from_slice(&RED.to_le_bytes());
    }
    for chunk in chunks.by_ref().take(800 * 30) {
        chunk.copy_from_slice(&GREEN.to_le_bytes());
    }
    for chunk in chunks.by_ref().take(800 * 30) {
        chunk.copy_from_slice(&BLUE.to_le_bytes());
    }
    for chunk in chunks {
        chunk.copy_from_slice(&(BLUE | RED).to_le_bytes());
    }

    println!("Rendering");
    loop {
        let transfer = dpi.send(false, dma_tx_buf).map_err(|e| e.0).unwrap();
        (_, dpi, dma_tx_buf) = transfer.wait();
    }
}
