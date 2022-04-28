#![allow(clippy::integer_arithmetic, clippy::cast_possible_truncation)]

// ref: https://github.com/phip1611/generic-tm1637-gpio-driver-rust/blob/main/src/lib.rs
// ref: https://github.com/igelbox/tm1637-rs/blob/master/examples/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/master/examples/stm32f103/src/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/b5e0759c41442d4e28c0ae26ad2bc393c43f814c/src/lib.rs

pub mod errors;
pub mod mappings;

extern crate embedded_hal as hal;

use crate::libs::tm1637::errors::TmError;
use crate::libs::tm1637::mappings::{
    Brightness, DisplayState, GpioPinValue, LowerCharBits, NumberCharBits, SpecialCharBits,
    UpperCharBits, ISA,
};
use embedded_hal::prelude::{
    _embedded_hal_blocking_delay_DelayMs, _embedded_hal_blocking_delay_DelayUs,
};
use esp_idf_hal::delay;
use hal::digital::v2::{InputPin, OutputPin};

pub const DISPLAY_REGISTERS_COUNT: usize = 6;

pub struct Tm1637BannerAutoScrollConfig {
    pub(crate) scroll_min_char_count: u8,
    pub(crate) delay_ms: u16,
    pub(crate) min_char_count_to_be_displayed: u8,
}

pub struct Tm1637<'a, CLK, DIO> {
    clk: &'a mut CLK,
    dio: &'a mut DIO,
    pub(crate) display_size: u8,
    delay_fn: delay::FreeRtos,

    /// Representation of the display state in bits for the TM1637.
    /// Bits 7-4 are zero. Later the "display control"-command prefix will be there.
    /// Bits 3-0 are for display on/off and brightness.
    brightness: u8,

    delay_us: u16,
}

impl<'a, CLK, DIO, E> Tm1637<'a, CLK, DIO>
where
    CLK: OutputPin<Error = E>,
    DIO: InputPin<Error = E> + OutputPin<Error = E>,
{
    pub fn new(clk: &'a mut CLK, dio: &'a mut DIO) -> Self {
        Self {
            clk,
            dio,
            display_size: 4,
            delay_fn: delay::FreeRtos {},
            brightness: DisplayState::On as u8 | Brightness::L7 as u8,

            delay_us: 100_u16,
        }
    }

    pub fn char_to_bytes(&self, chr: char) -> u8 {
        match chr {
            // upper case matching
            'A' => UpperCharBits::CharA as u8,
            'C' => UpperCharBits::CharC as u8,
            'E' => UpperCharBits::CharE as u8,
            'F' | 'f' => UpperCharBits::CharF as u8,
            'G' | 'g' => UpperCharBits::CharG as u8,
            'H' => UpperCharBits::CharH as u8,
            'I' => UpperCharBits::CharI as u8,
            'J' | 'j' => UpperCharBits::CharJ as u8,
            'L' => UpperCharBits::CharL as u8,
            'N' => UpperCharBits::CharN as u8,
            'O' => UpperCharBits::CharO as u8,
            'P' | 'p' => UpperCharBits::CharP as u8,
            'R' => UpperCharBits::CharR as u8,
            'S' | 's' => UpperCharBits::CharS as u8,
            'U' => UpperCharBits::CharU as u8,

            // lower case matching
            'a' => LowerCharBits::CharA as u8,
            'b' | 'B' => LowerCharBits::CharB as u8,
            'c' => LowerCharBits::CharC as u8,
            'd' | 'D' => LowerCharBits::CharD as u8,
            'e' => LowerCharBits::CharE as u8,
            'h' => LowerCharBits::CharH as u8,
            'i' => LowerCharBits::CharI as u8,
            'l' => LowerCharBits::CharL as u8,
            'n' => LowerCharBits::CharN as u8,
            'o' => LowerCharBits::CharO as u8,
            'q' | 'Q' => LowerCharBits::CharQ as u8,
            'r' => LowerCharBits::CharR as u8,
            't' | 'T' => LowerCharBits::CharT as u8,
            'u' => LowerCharBits::CharU as u8,
            'y' | 'Y' => LowerCharBits::CharY as u8,

            // number matching
            '0' => NumberCharBits::Zero as u8,
            '1' => NumberCharBits::One as u8,
            '2' => NumberCharBits::Two as u8,
            '3' => NumberCharBits::Three as u8,
            '4' => NumberCharBits::Four as u8,
            '5' => NumberCharBits::Five as u8,
            '6' => NumberCharBits::Six as u8,
            '7' => NumberCharBits::Seven as u8,
            '8' => NumberCharBits::Eight as u8,
            '9' => NumberCharBits::Nine as u8,

            // special character matching
            ' ' => SpecialCharBits::Space as u8,
            '-' => SpecialCharBits::Minus as u8,
            '_' => SpecialCharBits::Underscore as u8,
            '=' => SpecialCharBits::Equals as u8,
            '[' => SpecialCharBits::BracketLeft as u8,
            ']' => SpecialCharBits::BracketRight as u8,

            // '?' and default everything else to a question mark (unknown char)
            _ => SpecialCharBits::QuestionMark as u8,
        }
    }

    pub fn print_char(
        &mut self,
        address: u8,
        chr: char,
        show_colon: bool,
    ) -> anyhow::Result<(), TmError<E>> {
        let c = self.char_to_bytes(chr);

        self.print_char_byte(address, c, show_colon)
    }

    fn print_char_byte(
        &mut self,
        address: u8,
        chr: u8,
        show_colon: bool,
    ) -> anyhow::Result<(), TmError<E>> {
        let chars: Vec<u8> = [chr]
            .iter()
            .copied()
            .map(|mut c| {
                if address == 1 && show_colon {
                    c |= SpecialCharBits::ColonOrDot as u8;
                }

                c
            })
            .collect();

        self.write_segments_raw(&chars, address)
    }

    pub fn print_string(
        &mut self,
        string: &str,
        show_colon: bool,
        auto_scroll: Option<&Tm1637BannerAutoScrollConfig>,
        delay_ms: u16,
    ) -> anyhow::Result<(), TmError<E>> {
        let mut string_bucket: Option<Vec<String>> = None;

        // this will be only available when auto scroll has been approved for the input string
        let mut approved_auto_scroll_config: Option<&Tm1637BannerAutoScrollConfig> = None;

        if let Some(c) = auto_scroll {
            if string.len() >= c.scroll_min_char_count as usize {
                string_bucket = Some(self.get_auto_scrolling_banners(string, c)?);
                approved_auto_scroll_config = Some(c);
            }
        }

        if string_bucket.is_none() {
            // todo currently display size of upto 4 is taken care of.
            //  add support for smaller or bigger display size
            let sanitized_string = format!("{:>4}", string);

            string_bucket = Some(vec![sanitized_string]);
        }

        if let Some(v) = string_bucket {
            for next_string in &v {
                // clear screen if the string is empty
                if next_string.is_empty() {
                    return self.clear();
                }

                let char_bucket: Vec<char> = next_string.chars().into_iter().collect();
                for (pos, item) in char_bucket.iter().enumerate() {
                    if pos >= self.display_size as usize {
                        break;
                    }

                    self.print_char(pos as u8, *item, show_colon)?;
                }

                if let Some(ac) = approved_auto_scroll_config {
                    self.set_delay_ms(ac.delay_ms);
                }

                self.set_delay_ms(delay_ms);
            }
        }

        Ok(())
    }

    fn get_auto_scrolling_banners(
        &self,
        string: &str,
        config: &Tm1637BannerAutoScrollConfig,
    ) -> anyhow::Result<Vec<String>, TmError<E>> {
        if config.min_char_count_to_be_displayed > self.display_size {
            return Err(TmError::AutoScroll(
                "E0026".to_owned(),
                "'min_char_count_to_be_displayed' should not be greater than the 'display_size'"
                    .to_owned(),
            ));
        }

        // if the length of the string is equal to or lesser than the [min_char_count_to_be_displayed] then pad the string and return it back
        if string.len() <= config.min_char_count_to_be_displayed as usize {
            let sanitized_string = format!("{:>4}", string);

            return Ok(vec![sanitized_string]);
        }

        let mut output: Vec<String> = vec![];
        let mut char_bucket: Vec<char> = string.chars().into_iter().collect();

        'outerloop: for outer in 0..char_bucket.len() {
            let mut current_string: String = String::new();
            for (_, item) in char_bucket
                .iter_mut()
                .enumerate()
                .skip(outer)
                .take(self.display_size as usize)
            {
                current_string.push_str(&*item.to_string());
            }

            if current_string.len() < config.min_char_count_to_be_displayed as usize {
                break 'outerloop;
            }

            // todo currently display size of upto 4 is taken care of.
            //  add support for smaller or bigger display size
            let item = format!("{:<4}", current_string);

            output.push(item);
        }

        Ok(output)
    }

    /// Sets the display state. The display state is the 3rd bit of the
    /// "display control"-command.
    /// This setting is not committed until a write operation has been made.
    pub fn set_display_state(&mut self, ds: DisplayState) {
        // keep old state for brightness
        let old_brightness = self.brightness & 0b0000_0111;
        // take 3rd bit (the one that says display on/off) into the new value
        self.brightness = ds as u8 | old_brightness;
    }

    /// Sets the brightness of the screen. The brightness are the lower
    /// 3 bits of the "display control"-command.
    /// This setting is not committed until a write operation has been made.
    pub fn set_brightness(&mut self, brightness: Brightness) {
        // look if display is configured as on
        let display_on = self.brightness as u8 & 0b0000_1000;
        self.brightness = display_on | brightness as u8;
    }

    /// This uses fixed address mode (see data sheet) internally to write data to
    /// a specific position of the display.
    /// Position is 0, 1, 2, or 3.
    pub fn _write_segment_raw(
        &mut self,
        segments: u8,
        position: u8,
    ) -> anyhow::Result<(), TmError<E>> {
        self.write_segments_raw(&[segments], position)
    }

    /// Writes all raw segments data beginning at the position into the display registers.
    /// It uses auto increment internally to write into all further registers.
    /// This functions does an internal check so that not more than 6 registers can be
    /// addressed/written.
    /// * `segments` Raw data describing the bits of the 7 segment display.
    /// * `n` Length of segments array.
    /// * `pos` The start position of the display register. While bytes are
    ///         written, address is adjusted internally via auto increment.
    ///         Usually this is 0, if you want to write data to all 7 segment
    ///         displays.
    pub fn write_segments_raw(
        &mut self,
        segments: &[u8],
        pos: u8,
    ) -> anyhow::Result<(), TmError<E>> {
        let mut n = segments.len() as u8;
        // beeing a little bit more failure tolerant
        if n == 0 {
            return Ok(());
        } // nothing to do
        let pos = pos % DISPLAY_REGISTERS_COUNT as u8; // only valid positions/registers

        // valid values are
        //   n = 1, pos = {0, 1, 2, 3, 4, 5}
        //   n = 2, pos = {0, 1, 2, 3, 4}
        //   n = 3, pos = {0, 1, 2, 3}
        //   n = 4, pos = {0, 1, 2}
        //   n = 5, pos = {0, 1}
        //   n = 6, pos = {0}
        // => n + pos must be <= DISPLAY_REGISTERS_COUNT

        if n + pos > DISPLAY_REGISTERS_COUNT as u8 {
            // only write as much data as registers are available
            n = DISPLAY_REGISTERS_COUNT as u8 - pos;
        }

        // Command 1
        // for more information about this flow: see data sheet / specification of TM1637
        // or AZDelivery's 7 segment display
        self.start()?;
        self.write_byte_and_wait_ack(ISA::DataCommandWriteToDisplay as u8)?;
        self.stop()?;

        // Write COMM2
        self.start()?;
        self.write_byte_and_wait_ack(ISA::AddressCommandD0 as u8 | pos)?;

        // Write the remaining data bytes
        // TM1637 does auto increment internally
        for i in 0..n {
            #[allow(clippy::indexing_slicing)]
            self.write_byte_and_wait_ack(segments[i as usize])?;
        }
        self.stop()?;

        // we do this everytime because it will be a common flow that people write something
        // and expect the display to be on
        self.write_display_state()
    }

    /// Send command that sets the display state on the micro controller.
    pub fn write_display_state(&mut self) -> anyhow::Result<(), TmError<E>> {
        self.start()?;
        // bits 0-2 brightness; bit 3 is on/off
        self.write_byte_and_wait_ack(ISA::DisplayControlOff as u8 | self.brightness)?;
        self.stop()
    }

    /// Clears the display.
    pub fn clear(&mut self) -> anyhow::Result<(), TmError<E>> {
        // begin at position 0 and write 0 into display registers 0 to 5
        self.write_segments_raw(&[0, 0, 0, 0, 0, 0], 0)
    }

    /// Writes a byte bit by bit and waits for the acknowledge.
    fn write_byte_and_wait_ack(&mut self, byte: u8) -> anyhow::Result<(), TmError<E>> {
        let mut data = byte;

        // 8 bits
        for _ in 0_u8..8_u8 {
            // CLK low
            self.clk.set_low()?;

            // Set data bit (we send one bit of our byte per iteration)
            // LSF (least significant bit) first
            // => target has (probably?) shift register => this way the byte has the
            // correct order on the target
            let next_gpio_state = GpioPinValue::from(data & 0x01);

            if next_gpio_state == GpioPinValue::High {
                self.dio.set_high()?;
            } else {
                self.dio.set_low()?;
            }

            self.set_delay_us();

            // CLK high
            self.clk.set_high()?;
            self.set_delay_us();

            // shift to next bit
            data >>= 1_i32;
        }

        self.recv_ack()
    }

    /// This tells the TM1637 that data input starts.
    /// This information stands in the official data sheet.
    #[inline]
    fn start(&mut self) -> anyhow::Result<(), TmError<E>> {
        self.dio.set_high()?;
        self.clk.set_high()?;
        self.set_delay_us();
        self.dio.set_low()?;
        self.set_delay_us();

        // transition from high to low on DIO while CLK is high
        // means: data starts at next clock

        Ok(())
    }

    /// This tells the TM1637 that data input stops.
    /// This information stands in the official data sheet.
    #[inline]
    fn stop(&mut self) -> anyhow::Result<(), TmError<E>> {
        self.dio.set_low()?;
        self.clk.set_high()?;
        self.set_delay_us();
        self.dio.set_high()?;
        self.set_delay_us();

        Ok(())
    }

    /// Receives one acknowledgment after a byte was sent.
    fn recv_ack(&mut self) -> anyhow::Result<(), TmError<E>> {
        self.clk.set_low()?;
        self.dio.set_low()?;
        self.set_delay_us();
        self.clk.set_high()?;

        let is_dio_low: bool = self.dio.is_low()?;

        // wait a few cycles for ACK to be more fail safe
        for _ in 0_i32..10_i32 {
            if is_dio_low {
                break;
            }
        }

        self.clk.set_low()?;
        self.dio.set_low()?;
        self.set_delay_us();

        Ok(())
    }

    #[inline]
    fn set_delay_ms(&mut self, ms_delay: u16) {
        self.delay_fn.delay_ms(ms_delay);
    }

    #[inline]
    fn set_delay_us(&mut self) {
        self.delay_fn.delay_us(self.delay_us);
    }
}
