//! ST7567 LCD Display driver for use with [`embedded-hal`]
//!
//! This LCD display is found on the Pimoroni GFX HAT for the
//! Raspberry PI
//!
use embedded_hal::blocking::spi::Write;
mod consts;

use crate::consts::*;

pub use crate::consts::{HEIGHT, SPI_SPEED_HZ, WIDTH};
use crate::PinState::{High, Low};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::time::Duration;

pub enum PinState {
    High,
    Low,
}

/// A control pin, typically used to model DC & RST pin of the ST7567 display
pub trait Pin {
    type Error;

    fn set_value(&mut self, pin_state: PinState) -> Result<(), Self::Error>;
}

pub enum Error<P, S>
where
    P: Pin,
    S: Write<u8>,
{
    SpiError(S::Error),
    PinError(P::Error),
}

impl<P, S> Debug for Error<P, S>
where
    P: Pin,
    S: Write<u8>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Error::SpiError(_) => write!(f, "SpiError"),
            Error::PinError(_) => write!(f, "PinError"),
        }
    }
}

impl<P, S> std::error::Error for Error<P, S>
where
    P: Pin,
    S: Write<u8>,
{
}

impl<P, S> fmt::Display for Error<P, S>
where
    P: Pin,
    S: Write<u8>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Error::SpiError(_) => write!(f, "SpiError"),
            Error::PinError(_) => write!(f, "PinError"),
        }
    }
}
/// Utility function to deal with Error mess
fn set_pin<P: Pin, S: Write<u8>>(pin: &mut P, pin_state: PinState) -> Result<(), Error<P, S>> {
    match pin.set_value(pin_state) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::PinError(e)),
    }
}

/// Utility function to deal with Error mess
fn spi_write<P: Pin, S: Write<u8>>(spi: &mut S, data: &[u8]) -> Result<(), Error<P, S>> {
    match spi.write(data) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::SpiError(e)),
    }
}

const BUFFER_SIZE: usize = 1024;

/// Controls the ST7567 LCD Display.
///
pub struct ST7567<P: Pin, S: Write<u8>> {
    dc_pin: P,
    rst_pin: P,
    spi: S,
    buf: [u8; BUFFER_SIZE],
}

impl<P: Pin, S: Write<u8>> ST7567<P, S> {
    pub fn new(spi: S, dc_pin: P, rst_pin: P) -> Self {
        Self {
            spi,
            dc_pin,
            rst_pin,
            buf: [0; BUFFER_SIZE],
        }
    }

    fn command(&mut self, data: &[u8]) -> Result<(), Error<P, S>> {
        set_pin(&mut self.dc_pin, Low)?;
        spi_write(&mut self.spi, data)
    }

    fn data(&mut self, data: &[u8]) -> Result<(), Error<P, S>> {
        set_pin(&mut self.dc_pin, High)?;
        spi_write(&mut self.spi, data)
    }

    pub fn reset(&mut self) -> Result<(), Error<P, S>> {
        set_pin(&mut self.rst_pin, Low)?;
        std::thread::sleep(Duration::from_millis(10));
        set_pin(&mut self.rst_pin, High)?;
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    pub fn set_contrast(&mut self, value: u8) -> Result<(), Error<P, S>> {
        self.command(&[ST7567_SETCONTRAST, value])
    }

    pub fn init(&mut self) -> Result<(), Error<P, S>> {
        self.command(&[
            ST7567_BIAS_1_7, // Bais 1/7 (0xA2 = Bias 1/9)
            ST7567_SEG_DIR_NORMAL,
            ST7567_SETCOMREVERSE,    // Reverse COM - vertical flip
            ST7567_DISPNORMAL,       // Inverse display (0xA6 normal)
            ST7567_SETSTARTLINE | 0, // Start at line 0
            ST7567_POWERCTRL,
            ST7567_REG_RATIO | 3,
            ST7567_DISPON,
            ST7567_SETCONTRAST, // Set contrast
            40,                 // Contrast value])
        ])
    }

    /// Clear the display buffer
    pub fn clear(&mut self) {
        self.buf = [0; BUFFER_SIZE];
    }
    /// Set a single pixel in the  display buffer.
    ///
    /// Ignore out of bound values for x & y
    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        if x >= WIDTH as usize || y >= HEIGHT as usize {
            return;
        }
        let offset = ((y / 8) * WIDTH as usize) + x;
        let bit = y as u8 % 8;
        if value {
            // ON
            self.buf[offset] = self.buf[offset] | 1 << bit;
        } else {
            // OFF
            self.buf[offset] = self.buf[offset] & !(1 << bit);
        }
    }

    /// Update the ST7567 display with the buffer contents.
    pub fn show(&mut self) -> Result<(), Error<P, S>> {
        self.command(&[ST7567_ENTER_RMWMODE])?;
        for page in 0..8 {
            let offset: usize = page * ST7567_PAGESIZE as usize;
            self.command(&[
                ST7567_SETPAGESTART | page as u8,
                ST7567_SETCOLL,
                ST7567_SETCOLH,
            ])?;
            let start_offset = offset as usize;
            let end_offset = start_offset + ST7567_PAGESIZE as usize;
            let mut data = [0u8; ST7567_PAGESIZE as usize];
            data.clone_from_slice(&self.buf[start_offset..end_offset]);
            self.data(&data)?;
        }
        self.command(&[ST7567_EXIT_RMWMODE])
    }
}
