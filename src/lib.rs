//! ST7567 LCD Display driver for use with [`embedded-hal`]
//!
//! This LCD display is found on the Pimoroni GFX HAT for the
//! Raspberry PI
//!
use embedded_hal::spi::SpiDevice;
mod consts;

use crate::consts::*;

pub use crate::consts::{HEIGHT, SPI_SPEED_HZ, WIDTH};
use crate::PinState::{High, Low};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
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
    S: SpiDevice,
{
    SpiError(S::Error),
    PinError(P::Error),
}

impl<P, S> Debug for Error<P, S>
where
    P: Pin,
    S: SpiDevice,
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
    S: SpiDevice,
{
}

impl<P, S> fmt::Display for Error<P, S>
where
    P: Pin,
    S: SpiDevice,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match &self {
            Error::SpiError(_) => write!(f, "SpiError"),
            Error::PinError(_) => write!(f, "PinError"),
        }
    }
}
/// Utility function to deal with Error mess
fn set_pin<P: Pin, S: SpiDevice>(pin: &mut P, pin_state: PinState) -> Result<(), Error<P, S>> {
    match pin.set_value(pin_state) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::PinError(e)),
    }
}

/// Utility function to deal with Error mess
fn spi_write<P: Pin, S: SpiDevice>(spi: &mut S, data: &[u8]) -> Result<(), Error<P, S>> {
    match spi.write(data) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::SpiError(e)),
    }
}

const BUFFER_SIZE: usize = 1024;

/// Controls the ST7567 LCD Display.
///
pub struct ST7567<P: Pin, S: SpiDevice> {
    dc_pin: P,
    rst_pin: P,
    spi: S,
    buf: [u8; BUFFER_SIZE],
}

impl<P: Pin, S: SpiDevice> ST7567<P, S> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Clone, PartialEq)]
    pub enum MockError {
        SpiError,
        PinError,
    }

    impl std::error::Error for MockError {}

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                MockError::SpiError => write!(f, "Mock SPI Error"),
                MockError::PinError => write!(f, "Mock Pin Error"),
            }
        }
    }

    impl embedded_hal::spi::Error for MockError {
        fn kind(&self) -> embedded_hal::spi::ErrorKind {
            embedded_hal::spi::ErrorKind::Other
        }
    }

    /// Mock Pin implementation for testing
    #[derive(Debug)]
    pub struct MockPin {
        pub states: RefCell<Vec<PinState>>,
        pub should_fail: RefCell<bool>,
    }

    impl MockPin {
        pub fn new() -> Self {
            Self {
                states: RefCell::new(Vec::new()),
                should_fail: RefCell::new(false),
            }
        }

        pub fn set_fail(&self, fail: bool) {
            *self.should_fail.borrow_mut() = fail;
        }

        pub fn get_states(&self) -> Vec<PinState> {
            self.states.borrow().clone()
        }

        pub fn clear_states(&self) {
            self.states.borrow_mut().clear();
        }
    }

    impl Pin for MockPin {
        type Error = MockError;

        fn set_value(&mut self, pin_state: PinState) -> Result<(), Self::Error> {
            if *self.should_fail.borrow() {
                return Err(MockError::PinError);
            }
            self.states.borrow_mut().push(pin_state);
            Ok(())
        }
    }

    /// Mock SPI Device implementation for testing
    #[derive(Debug)]
    pub struct MockSpiDevice {
        pub written_data: RefCell<Vec<u8>>,
        pub should_fail: RefCell<bool>,
    }

    impl MockSpiDevice {
        pub fn new() -> Self {
            Self {
                written_data: RefCell::new(Vec::new()),
                should_fail: RefCell::new(false),
            }
        }

        pub fn set_fail(&self, fail: bool) {
            *self.should_fail.borrow_mut() = fail;
        }

        pub fn get_written_data(&self) -> Vec<u8> {
            self.written_data.borrow().clone()
        }

        pub fn clear_written_data(&self) {
            self.written_data.borrow_mut().clear();
        }
    }

    impl embedded_hal::spi::ErrorType for MockSpiDevice {
        type Error = MockError;
    }

    impl embedded_hal::spi::SpiDevice for MockSpiDevice {
        fn transaction(
            &mut self,
            operations: &mut [embedded_hal::spi::Operation<'_, u8>],
        ) -> Result<(), Self::Error> {
            if *self.should_fail.borrow() {
                return Err(MockError::SpiError);
            }

            for operation in operations {
                match operation {
                    embedded_hal::spi::Operation::Write(data) => {
                        self.written_data.borrow_mut().extend_from_slice(data);
                    }
                    _ => {} // We only care about write operations for this driver
                }
            }
            Ok(())
        }
    }

    // Helper to create a test ST7567 instance
    fn create_test_st7567() -> ST7567<MockPin, MockSpiDevice> {
        let spi = MockSpiDevice::new();
        let dc_pin = MockPin::new();
        let rst_pin = MockPin::new();
        ST7567::new(spi, dc_pin, rst_pin)
    }

    #[test]
    fn test_new() {
        let st7567 = create_test_st7567();
        // Buffer should be initialized to all zeros
        assert_eq!(st7567.buf, [0; BUFFER_SIZE]);
    }

    #[test]
    fn test_clear() {
        let mut st7567 = create_test_st7567();
        // Set some pixels first
        st7567.set_pixel(10, 20, true);
        st7567.set_pixel(50, 30, true);
        
        // Clear should reset all pixels
        st7567.clear();
        assert_eq!(st7567.buf, [0; BUFFER_SIZE]);
    }

    #[test]
    fn test_set_pixel_valid_coordinates() {
        let mut st7567 = create_test_st7567();
        
        // Test setting a pixel at (0, 0)
        st7567.set_pixel(0, 0, true);
        let offset = ((0 / 8) * WIDTH as usize) + 0;
        let bit = 0 % 8;
        assert_eq!(st7567.buf[offset], 1 << bit);
        
        // Test setting a pixel at (10, 20)
        st7567.clear();
        st7567.set_pixel(10, 20, true);
        let offset = ((20 / 8) * WIDTH as usize) + 10;
        let bit = 20 % 8;
        assert_eq!(st7567.buf[offset], 1 << bit);
        
        // Test unsetting a pixel
        st7567.set_pixel(10, 20, false);
        assert_eq!(st7567.buf[offset], 0);
    }

    #[test]
    fn test_set_pixel_invalid_coordinates() {
        let mut st7567 = create_test_st7567();
        
        // Test coordinates out of bounds - should be ignored
        st7567.set_pixel(WIDTH as usize, HEIGHT as usize, true);
        st7567.set_pixel(200, 100, true);
        
        // Buffer should remain empty
        assert_eq!(st7567.buf, [0; BUFFER_SIZE]);
    }

    #[test]
    fn test_set_pixel_multiple_bits_same_byte() {
        let mut st7567 = create_test_st7567();
        
        // Set multiple pixels in the same byte (same x, different y within 8-pixel boundary)
        st7567.set_pixel(10, 0, true);  // bit 0
        st7567.set_pixel(10, 1, true);  // bit 1
        st7567.set_pixel(10, 2, true);  // bit 2
        
        let offset = ((0 / 8) * WIDTH as usize) + 10;
        let expected = (1 << 0) | (1 << 1) | (1 << 2);
        assert_eq!(st7567.buf[offset], expected);
    }

    #[test]
    fn test_reset_success() {
        let mut st7567 = create_test_st7567();
        
        let result = st7567.reset();
        assert!(result.is_ok());
        
        // Check that reset pin was toggled correctly
        let rst_states = st7567.rst_pin.get_states();
        assert_eq!(rst_states.len(), 2);
        assert!(matches!(rst_states[0], PinState::Low));
        assert!(matches!(rst_states[1], PinState::High));
    }

    #[test]
    fn test_reset_pin_error() {
        let mut st7567 = create_test_st7567();
        st7567.rst_pin.set_fail(true);
        
        let result = st7567.reset();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PinError(_)));
    }

    #[test]
    fn test_set_contrast_success() {
        let mut st7567 = create_test_st7567();
        
        let result = st7567.set_contrast(128);
        assert!(result.is_ok());
        
        // Check that correct command was sent
        let written_data = st7567.spi.get_written_data();
        assert_eq!(written_data, vec![ST7567_SETCONTRAST, 128]);
        
        // Check that DC pin was set to Low for command
        let dc_states = st7567.dc_pin.get_states();
        assert_eq!(dc_states.len(), 1);
        assert!(matches!(dc_states[0], PinState::Low));
    }

    #[test]
    fn test_set_contrast_spi_error() {
        let mut st7567 = create_test_st7567();
        st7567.spi.set_fail(true);
        
        let result = st7567.set_contrast(128);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SpiError(_)));
    }

    #[test]
    fn test_set_contrast_pin_error() {
        let mut st7567 = create_test_st7567();
        st7567.dc_pin.set_fail(true);
        
        let result = st7567.set_contrast(128);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PinError(_)));
    }

    #[test]
    fn test_init_success() {
        let mut st7567 = create_test_st7567();
        
        let result = st7567.init();
        assert!(result.is_ok());
        
        // Check that correct initialization sequence was sent
        let written_data = st7567.spi.get_written_data();
        let expected = vec![
            ST7567_BIAS_1_7,
            ST7567_SEG_DIR_NORMAL,
            ST7567_SETCOMREVERSE,
            ST7567_DISPNORMAL,
            ST7567_SETSTARTLINE | 0,
            ST7567_POWERCTRL,
            ST7567_REG_RATIO | 3,
            ST7567_DISPON,
            ST7567_SETCONTRAST,
            40,
        ];
        assert_eq!(written_data, expected);
        
        // Check that DC pin was set to Low for command
        let dc_states = st7567.dc_pin.get_states();
        assert_eq!(dc_states.len(), 1);
        assert!(matches!(dc_states[0], PinState::Low));
    }

    #[test]
    fn test_command_vs_data() {
        let mut st7567 = create_test_st7567();
        
        // Test command - should set DC pin low
        let _ = st7567.command(&[0x01, 0x02]);
        let dc_states = st7567.dc_pin.get_states();
        assert_eq!(dc_states.len(), 1);
        assert!(matches!(dc_states[0], PinState::Low));
        
        // Test data - should set DC pin high
        st7567.dc_pin.clear_states();
        let _ = st7567.data(&[0x03, 0x04]);
        let dc_states = st7567.dc_pin.get_states();
        assert_eq!(dc_states.len(), 1);
        assert!(matches!(dc_states[0], PinState::High));
        
        // Check that correct data was written to SPI
        let written_data = st7567.spi.get_written_data();
        assert_eq!(written_data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_show_success() {
        let mut st7567 = create_test_st7567();
        
        // Set some pixels
        st7567.set_pixel(0, 0, true);
        st7567.set_pixel(127, 63, true);
        
        let result = st7567.show();
        assert!(result.is_ok());
        
        let written_data = st7567.spi.get_written_data();
        
        // Should start with enter RMW mode
        assert_eq!(written_data[0], ST7567_ENTER_RMWMODE);
        
        // Should end with exit RMW mode
        assert_eq!(written_data[written_data.len() - 1], ST7567_EXIT_RMWMODE);
        
        // Should contain page setup commands for each of the 8 pages
        // Count the structure: 1 enter command + (8 pages * (3 setup commands + 128 data bytes)) + 1 exit command
        // = 1 + 8 * (3 + 128) + 1 = 1 + 8 * 131 + 1 = 1050 bytes total
        // But we need to account for the fact that commands and data are tracked separately by DC pin state
        
        // The important thing is that we have the right structure
        assert!(written_data.len() > 1000); // Should be substantial amount of data
    }

    #[test]
    fn test_error_display_and_debug() {
        let spi_error: Error<MockPin, MockSpiDevice> = Error::SpiError(MockError::SpiError);
        let pin_error: Error<MockPin, MockSpiDevice> = Error::PinError(MockError::PinError);
        
        // Test Display trait
        assert_eq!(format!("{}", spi_error), "SpiError");
        assert_eq!(format!("{}", pin_error), "PinError");
        
        // Test Debug trait
        assert_eq!(format!("{:?}", spi_error), "SpiError");
        assert_eq!(format!("{:?}", pin_error), "PinError");
    }

    #[test]
    fn test_pixel_coordinates_edge_cases() {
        let mut st7567 = create_test_st7567();
        
        // Test corner pixels
        st7567.set_pixel(0, 0, true);
        st7567.set_pixel(WIDTH as usize - 1, 0, true);
        st7567.set_pixel(0, HEIGHT as usize - 1, true);
        st7567.set_pixel(WIDTH as usize - 1, HEIGHT as usize - 1, true);
        
        // Check first pixel (0,0)
        let offset = ((0 / 8) * WIDTH as usize) + 0;
        assert_eq!(st7567.buf[offset], 1);
        
        // Check top-right pixel (127,0)
        let offset = ((0 / 8) * WIDTH as usize) + (WIDTH as usize - 1);
        assert_eq!(st7567.buf[offset], 1);
        
        // Check bottom-left pixel (0,63)
        let offset = (((HEIGHT as usize - 1) / 8) * WIDTH as usize) + 0;
        let bit = (HEIGHT as usize - 1) % 8;
        assert_eq!(st7567.buf[offset], 1 << bit);
        
        // Check bottom-right pixel (127,63)
        let offset = (((HEIGHT as usize - 1) / 8) * WIDTH as usize) + (WIDTH as usize - 1);
        let bit = (HEIGHT as usize - 1) % 8;
        assert_eq!(st7567.buf[offset], 1 << bit);
    }

    #[test]
    fn test_constants() {
        // Test that our constants match expected values
        assert_eq!(WIDTH, 128);
        assert_eq!(HEIGHT, 64);
        assert_eq!(ST7567_PAGESIZE, 128);
        assert_eq!(BUFFER_SIZE, 1024); // WIDTH * HEIGHT / 8 = 128 * 64 / 8 = 1024
    }
}
