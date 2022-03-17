// ref: https://github.com/igelbox/tm1637-rs/blob/master/examples/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/master/examples/stm32f103/src/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/b5e0759c41442d4e28c0ae26ad2bc393c43f814c/src/lib.rs

extern crate embedded_hal as hal;

use embedded_hal::prelude::{
    _embedded_hal_blocking_delay_DelayMs, _embedded_hal_blocking_delay_DelayUs,
};
use esp_idf_hal::delay;
use hal::digital::v2::{InputPin, OutputPin};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum Error<E> {
    Ack,
    IO(E),
}

impl<E> From<E> for Error<E> {
    fn from(err: E) -> Error<E> {
        Error::IO(err)
    }
}

type Res<E> = Result<(), Error<E>>;

pub struct TM1637<'a, CLK, DIO> {
    clk: &'a mut CLK,
    dio: &'a mut DIO,
    pub(crate) display_size: u8,
    delay_fn: delay::FreeRtos,
}

pub struct TM1637BannerAutoScrollConfig {
    pub(crate) scroll_min_char_count: u8,
    pub(crate) delay_ms: u16,
    pub(crate) min_char_count_to_be_displayed: u8,
}

enum Bit {
    Zero,
    One,
}

impl<'a, CLK, DIO, E> TM1637<'a, CLK, DIO>
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
        }
    }

    pub fn init(&mut self) -> Res<E> {
        self.start()?;
        self.send(ADDRESS_AUTO_INCREMENT_1_MODE)?;
        self.stop()?;

        Ok(())
    }

    pub fn clear(&mut self) -> Res<E> {
        self.print_raw_iter(0, core::iter::repeat(0).take(4))
    }

    pub fn print_raw(&mut self, address: u8, bytes: &[u8]) -> Res<E> {
        self.print_raw_iter(address, bytes.iter().copied())
    }

    fn char_to_bytes(&self, chr: &char) -> u8 {
        match chr {
            // upper case matching
            'A' => self::UpperCharBits::CharA as u8,
            'C' => self::UpperCharBits::CharC as u8,
            'E' => self::UpperCharBits::CharE as u8,
            'F' => self::UpperCharBits::CharF as u8,
            'G' => self::UpperCharBits::CharG as u8,
            'H' => self::UpperCharBits::CharH as u8,
            'I' => self::UpperCharBits::CharI as u8,
            'J' => self::UpperCharBits::CharJ as u8,
            'L' => self::UpperCharBits::CharL as u8,
            'N' => self::UpperCharBits::CharN as u8,
            'O' => self::UpperCharBits::CharO as u8,
            'P' => self::UpperCharBits::CharP as u8,
            'R' => self::UpperCharBits::CharR as u8,
            'S' => self::UpperCharBits::CharS as u8,
            'U' => self::UpperCharBits::CharU as u8,

            // lower case matching
            'a' => self::LowerCharBits::CharA as u8,
            'b' => self::LowerCharBits::CharB as u8,
            'c' => self::LowerCharBits::CharC as u8,
            'd' => self::LowerCharBits::CharD as u8,
            'e' => self::LowerCharBits::CharE as u8,
            'h' => self::LowerCharBits::CharH as u8,
            'i' => self::LowerCharBits::CharI as u8,
            'l' => self::LowerCharBits::CharL as u8,
            'n' => self::LowerCharBits::CharN as u8,
            'o' => self::LowerCharBits::CharO as u8,
            'q' => self::LowerCharBits::CharQ as u8,
            'r' => self::LowerCharBits::CharR as u8,
            't' => self::LowerCharBits::CharT as u8,
            'u' => self::LowerCharBits::CharU as u8,
            'y' => self::LowerCharBits::CharY as u8,

            // match the char with the available upper case
            'f' => self::UpperCharBits::CharF as u8,
            'g' => self::UpperCharBits::CharG as u8,
            'j' => self::UpperCharBits::CharJ as u8,
            'p' => self::UpperCharBits::CharP as u8,
            's' => self::UpperCharBits::CharS as u8,

            // match the char with the available lower case
            'Y' => self::LowerCharBits::CharY as u8,
            'T' => self::LowerCharBits::CharT as u8,
            'Q' => self::LowerCharBits::CharQ as u8,
            'D' => self::LowerCharBits::CharD as u8,
            'B' => self::LowerCharBits::CharB as u8,

            // number matching
            '0' => self::NumberCharBits::Zero as u8,
            '1' => self::NumberCharBits::One as u8,
            '2' => self::NumberCharBits::Two as u8,
            '3' => self::NumberCharBits::Three as u8,
            '4' => self::NumberCharBits::Four as u8,
            '5' => self::NumberCharBits::Five as u8,
            '6' => self::NumberCharBits::Six as u8,
            '7' => self::NumberCharBits::Seven as u8,
            '8' => self::NumberCharBits::Eight as u8,
            '9' => self::NumberCharBits::Nine as u8,

            // special character matching
            ' ' => self::SpecialCharBits::Space as u8,
            '-' => self::SpecialCharBits::Minus as u8,
            '_' => self::SpecialCharBits::Underscore as u8,
            '=' => self::SpecialCharBits::Equals as u8,
            '?' => self::SpecialCharBits::QuestionMark as u8,
            '[' => self::SpecialCharBits::BracketLeft as u8,
            ']' => self::SpecialCharBits::BracketRight as u8,

            /// default everything to a question mark (unknown char)
            _ => self::SpecialCharBits::QuestionMark as u8,
        }
    }

    pub fn print_char(&mut self, address: u8, chr: &char, show_colon: bool) -> Res<E> {
        let c = self.char_to_bytes(chr);

        self.print_char_byte(address, c, show_colon)
    }

    fn print_char_byte(&mut self, address: u8, chr: u8, show_colon: bool) -> Res<E> {
        self.print_raw_iter(
            address,
            [chr].iter().copied().map(|c| {
                let mut c = c;

                if address == 1 && show_colon {
                    c |= SpecialCharBits::ColonOrDot as u8;
                }

                c
            }),
        )
    }

    pub fn print_string(
        &mut self,
        string: &str,
        show_colon: bool,
        auto_scroll: Option<&TM1637BannerAutoScrollConfig>,
    ) -> Res<E> {
        let mut string_bucket: Option<Vec<String>> = None;

        // this will be only available when auto scroll has been approved for the input string
        let mut approved_auto_scroll_config: Option<&TM1637BannerAutoScrollConfig> = None;

        if let Some(c) = auto_scroll {
            if string.len() >= c.scroll_min_char_count as usize {
                string_bucket = Some(self.get_auto_scrolling_banners(string, c).unwrap());
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
                //todo remove
                println!("next string: `{}`", next_string);

                // clear screen if the string is empty
                if next_string.is_empty() {
                    return self.clear();
                }

                let char_bucket: Vec<char> = next_string.chars().into_iter().collect();
                for (pos, item) in char_bucket.iter().enumerate() {
                    if pos >= self.display_size as usize {
                        break;
                    }

                    self.print_char(pos as u8, item, show_colon)?;
                }

                if let Some(ac) = approved_auto_scroll_config {
                    self.set_delay_ms(ac.delay_ms);
                }
            }
        }

        Ok(())
    }

    fn get_auto_scrolling_banners(
        &self,
        string: &str,
        config: &TM1637BannerAutoScrollConfig,
    ) -> anyhow::Result<Vec<String>> {
        if config.min_char_count_to_be_displayed > self.display_size {
            return Err(anyhow::Error::msg("[auto scroll] 'min_char_count_to_be_displayed' should not be greater than the 'display_size'"));
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

    fn print_raw_iter<Iter: Iterator<Item = u8>>(&mut self, address: u8, bytes: Iter) -> Res<E> {
        self.start()?;
        self.send(ADDRESS_COMMAND_BITS | (address & ADDRESS_COMMAND_MASK))?;
        for byte in bytes {
            self.send(byte)?;
        }
        self.stop()?;
        Ok(())
    }

    pub fn set_brightness(&mut self, level: u8) -> Res<E> {
        self.start()?;
        self.send(DISPLAY_CONTROL_BRIGHTNESS_BITS | (level & DISPLAY_CONTROL_BRIGHTNESS_MASK))?;
        self.stop()?;

        Ok(())
    }

    fn send(&mut self, byte: u8) -> Res<E> {
        let mut rest = byte;
        for _ in 0..8 {
            let bit = if &rest & 1 != 0 { Bit::One } else { Bit::Zero };
            self.send_bit_and_delay(bit)?;
            rest >>= 1;
        }

        // Wait for the ACK
        self.send_bit_and_delay(Bit::One)?;
        for _ in 0..255_i32 {
            if self.dio.is_low()? {
                return Ok(());
            }
            self.set_delay_us();
        }

        Err(Error::Ack)
    }

    fn start(&mut self) -> Res<E> {
        self.send_bit_and_delay(Bit::One)?;
        self.dio.set_low()?;

        Ok(())
    }

    fn stop(&mut self) -> Res<E> {
        self.send_bit_and_delay(Bit::Zero)?;
        self.dio.set_high()?;
        self.set_delay_us();

        Ok(())
    }

    fn send_bit_and_delay(&mut self, value: Bit) -> Res<E> {
        self.clk.set_low()?;
        if let Bit::One = value {
            self.dio.set_high()?;
        } else {
            self.dio.set_low()?;
        }
        self.clk.set_high()?;
        self.set_delay_us();

        Ok(())
    }

    #[inline]
    fn set_delay_us(&mut self) {
        self.delay_fn.delay_us(DELAY_USECS);
        //thread::sleep(Duration::from_micros(DELAY_USECS as u64));
    }

    #[inline]
    fn set_delay_ms(&mut self, ms_delay: u16) {
        self.delay_fn.delay_ms(ms_delay);

        //thread::sleep(Duration::from_millis(ms_delay as u64));
    }
}

const MAX_FREQ_KHZ: u16 = 500; // todo get max freq
const USECS_IN_MSEC: u16 = 1_000; // todo fetch this value
const DELAY_USECS: u16 = USECS_IN_MSEC / MAX_FREQ_KHZ;

const ADDRESS_AUTO_INCREMENT_1_MODE: u8 = 0x40;

const ADDRESS_COMMAND_BITS: u8 = 0xc0;
const ADDRESS_COMMAND_MASK: u8 = 0x0f;

const DISPLAY_CONTROL_BRIGHTNESS_BITS: u8 = 0x88;
const DISPLAY_CONTROL_BRIGHTNESS_MASK: u8 = 0x07;

// const DIGITS: [u8; 16] = [
//     0x3f, 0x06, 0x5b, 0x4f, //
//     0x66, 0x6d, 0x7d, 0x07, //
//     0x7f, 0x6f, 0x77, 0x7c, //
//     0x39, 0x5e, 0x79, 0x71, //
// ];

/////

/// Shows which segment has which bit.
#[repr(u8)]
pub enum SegmentBits {
    SegA = 0b00000001,
    SegB = 0b00000010,
    SegC = 0b00000100,
    SegD = 0b00001000,
    SegE = 0b00010000,
    SegF = 0b00100000,
    SegG = 0b01000000,

    // double point on AzDelivery 4-digit 7 segment display.
    SegColonOrDot = 0b10000000,
}

#[repr(u8)]
pub enum NumberCharBits {
    Zero = 0b00111111,
    One = 0b00000110,
    Two = 0b01011011,
    Three = 0b01001111,
    Four = 0b01100110,
    Five = 0b01101101,
    Six = 0b01111101,
    Seven = 0b00000111,
    Eight = 0b01111111,
    Nine = 0b01101111,
}

/// Maps a character to its closest possible representation on a 7-segment display.
/// The 8th segment is the dot.
#[repr(u8)]
pub enum UpperCharBits {
    CharA = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharC = SegmentBits::SegA as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8,
    CharE = SegmentBits::SegA as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharF = SegmentBits::SegA as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharG = SegmentBits::SegA as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8,
    CharH = SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharI = SegmentBits::SegB as u8 | SegmentBits::SegC as u8,
    CharJ = SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8,
    CharL = SegmentBits::SegD as u8 | SegmentBits::SegE as u8 | SegmentBits::SegF as u8,
    CharN = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8,
    CharO = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8,
    CharP = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharR = SegmentBits::SegA as u8 | SegmentBits::SegE as u8 | SegmentBits::SegF as u8,
    CharS = SegmentBits::SegA as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharU = SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8,
}

/// Maps a character to its closest possible representation on a 7-segment display.
/// The 8th segment is the dot.
#[repr(u8)]
pub enum LowerCharBits {
    CharA = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegG as u8,
    CharB = SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharC = SegmentBits::SegD as u8 | SegmentBits::SegE as u8 | SegmentBits::SegG as u8,
    CharD = SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegG as u8,
    CharE = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharH = SegmentBits::SegC as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharI = SegmentBits::SegC as u8,
    CharL = SegmentBits::SegD as u8 | SegmentBits::SegE as u8,
    CharN = SegmentBits::SegC as u8 | SegmentBits::SegE as u8 | SegmentBits::SegG as u8,
    CharO = SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegG as u8,
    CharQ = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharR = SegmentBits::SegE as u8 | SegmentBits::SegG as u8,
    CharT = SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
    CharU = SegmentBits::SegC as u8 | SegmentBits::SegD as u8 | SegmentBits::SegE as u8,
    CharY = SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegF as u8
        | SegmentBits::SegG as u8,
}

/// Maps a character to its closest possible representation on a 7-segment display.
/// The 8th segment is the dot.
#[repr(u8)]
pub enum SpecialCharBits {
    Space = 0,
    Minus = SegmentBits::SegG as u8,
    Underscore = SegmentBits::SegD as u8,
    Equals = SegmentBits::SegG as u8 | SegmentBits::SegD as u8,
    QuestionMark = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegG as u8
        | SegmentBits::SegE as u8,
    ColonOrDot = SegmentBits::SegColonOrDot as u8,
    BracketLeft = SegmentBits::SegA as u8
        | SegmentBits::SegD as u8
        | SegmentBits::SegE as u8
        | SegmentBits::SegF as u8,
    BracketRight = SegmentBits::SegA as u8
        | SegmentBits::SegB as u8
        | SegmentBits::SegC as u8
        | SegmentBits::SegD as u8,
}
