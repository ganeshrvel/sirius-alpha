// ref: https://github.com/phip1611/generic-tm1637-gpio-driver-rust/blob/main/src/lib.rs
// ref: https://github.com/igelbox/tm1637-rs/blob/master/examples/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/master/examples/stm32f103/src/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/b5e0759c41442d4e28c0ae26ad2bc393c43f814c/src/lib.rs

extern crate embedded_hal as hal;

use embedded_hal::prelude::{
    _embedded_hal_blocking_delay_DelayMs, _embedded_hal_blocking_delay_DelayUs,
};
use esp_idf_hal::delay;
use hal::digital::v2::{InputPin, OutputPin};

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

    /// Representation of the display state in bits for the TM1637.
    /// Bits 7-4 are zero. Later the "display control"-command prefix will be there.
    /// Bits 3-0 are for display on/off and brightness.
    brightness: u8,

    delay_us: u16,
}

pub struct Tm1637BannerAutoScrollConfig {
    pub(crate) scroll_min_char_count: u8,
    pub(crate) delay_ms: u16,
    pub(crate) min_char_count_to_be_displayed: u8,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpioPinValue {
    /// Low.
    Low,
    /// High.
    High,
}

impl From<u8> for GpioPinValue {
    fn from(x: u8) -> Self {
        if x == 0 {
            Self::Low
        } else {
            Self::High
        }
    }
}

/// The level of brightness.
/// The TM1637 "DisplayControl"-command transports the brightness information
/// in bits 0 to 2.
#[repr(u8)]
pub enum Brightness {
    // useless assignment because it is default but it shows clearly
    // that 3 bits are used
    /// Lowest brightness.
    L0 = 0b000,
    L1 = 0b001,
    L2 = 0b010,
    L3 = 0b011,
    L4 = 0b100,
    L5 = 0b101,
    L6 = 0b110,
    /// Highest brightness.
    L7 = 0b111,
}

/// Whether the display is on or off.
/// The TM1637 "DisplayControl"-command transports the display on/off information
/// in the third bit (2^3) of the command.
#[repr(u8)]
pub enum DisplayState {
    /// Display off.
    Off = 0b0000,
    /// Display On.
    On = 0b1000,
}

pub const DISPLAY_REGISTERS_COUNT: usize = 6;

/// The "ISA"/Commands of the TM1637. See data sheet
/// for more information. This is only a subset of the possible values.
#[repr(u8)]
pub enum ISA {
    /// Start instruction
    DataCommandWriteToDisplay = 0b0100_0000, // "write data to display register"-mode

    // send this + <recv ack> + send byte 0 + <recv ack> + ... send byte 3
    /// Starts at display address zero. Each further byte that is send will go
    /// into the next display address. The micro controller does an internal auto increment
    /// of the address. See the data sheet for more information.
    AddressCommandD0 = 0b1100_0000,
    AddressCommandD1 = 0b1100_0001,
    AddressCommandD2 = 0b1100_0010,
    AddressCommandD3 = 0b1100_0011,

    // bits 0 - 2 tell the brightness.
    // bit 3 is display on/off
    /// Command that sets the display off.
    DisplayControlOff = 0b1000_0000,
    /// Command that sets the display on with lowest brightness.
    DisplayControlOnL0 = 0b1000_1000,
    DisplayControlOnL1 = 0b1000_1001,
    DisplayControlOnL2 = 0b1000_1010,
    DisplayControlOnL3 = 0b1000_1011,
    DisplayControlOnL4 = 0b1000_1100,
    DisplayControlOnL5 = 0b1000_1101,
    DisplayControlOnL6 = 0b1000_1110,
    /// Command that sets the display on with highest brightness.
    DisplayControlOnL7 = 0b1000_1111,
    /*
    these are the 3 base commands: to see the meaning
    of bits 0 to 5 see data sheet;
    6 & 7 are reserved to mark the kind of command
    // data command
    COMM1_BASE = 0b0100_000,
    // addressing mode
    COMM2_BASE = 0b1100_000,
    // display control
    COMM3_BASE = 0b1000_000,*/
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
            brightness: DisplayState::On as u8 | Brightness::L7 as u8,
            delay_us: 100_u16,
        }
    }

    pub fn char_to_bytes(&self, chr: &char) -> u8 {
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
        let chars: Vec<u8> = [chr]
            .iter()
            .copied()
            .map(|c| {
                let mut c = c;

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
    ) -> Res<E> {
        let mut string_bucket: Option<Vec<String>> = None;

        // this will be only available when auto scroll has been approved for the input string
        let mut approved_auto_scroll_config: Option<&Tm1637BannerAutoScrollConfig> = None;

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
                    // let c = self.char_to_bytes(item);

                    //  self.write_segment_raw(c, pos as u8)?;
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
        config: &Tm1637BannerAutoScrollConfig,
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
    pub fn write_segment_raw(&mut self, segments: u8, position: u8) -> Res<E> {
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
    pub fn write_segments_raw(&mut self, segments: &[u8], pos: u8) -> Res<E> {
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
            self.write_byte_and_wait_ack(segments[i as usize])?;
        }
        self.stop()?;

        // we do this everytime because it will be a common flow that people write something
        // and expect the display to be on
        self.write_display_state()
    }

    /// Send command that sets the display state on the micro controller.
    pub fn write_display_state(&mut self) -> Res<E> {
        self.start();
        // bits 0-2 brightness; bit 3 is on/off
        self.write_byte_and_wait_ack(ISA::DisplayControlOff as u8 | self.brightness);
        self.stop()
    }

    /// Clears the display.
    pub fn clear(&mut self) -> Res<E> {
        // begin at position 0 and write 0 into display registers 0 to 5
        self.write_segments_raw(&[0, 0, 0, 0, 0, 0], 0)
    }

    /// Writes a byte bit by bit and waits for the acknowledge.
    fn write_byte_and_wait_ack(&mut self, byte: u8) -> Res<E> {
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
            data >>= 1;
        }

        self.recv_ack()
    }

    /// This tells the TM1637 that data input starts.
    /// This information stands in the official data sheet.
    #[inline]
    fn start(&mut self) -> Res<E> {
        self.dio.set_high()?;
        self.clk.set_high()?;

        self.set_delay_us();

        self.dio.set_low()?;

        self.set_delay_us();

        Ok(())

        // transition from high to low on DIO while CLK is high
        // means: data starts at next clock
    }

    /// This tells the TM1637 that data input stops.
    /// This information stands in the official data sheet.
    #[inline]
    fn stop(&mut self) -> Res<E> {
        self.dio.set_low()?;
        self.clk.set_high()?;

        self.set_delay_us();

        self.dio.set_high()?;

        self.set_delay_us();

        Ok(())
    }

    /// Receives one acknowledgment after a byte was sent.
    fn recv_ack(&mut self) -> Res<E> {
        self.clk.set_low()?;
        self.dio.set_low()?;

        self.set_delay_us();

        self.clk.set_high()?;

        let is_dio_low: bool = self.dio.is_low()?;

        // wait a few cycles for ACK to be more fail safe
        for _ in 0..10 {
            if is_dio_low {
                break;
            } else {
                // ACK should be one clock with zero on data lane

                // not possible with no_std; TODO provide debug function
                // eprintln!("ack is not 0! Probably not a problem, tho.")
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
    One = 0b0000_0110,
    Two = 0b0101_1011,
    Three = 0b0100_1111,
    Four = 0b0110_0110,
    Five = 0b0110_1101,
    Six = 0b0111_1101,
    Seven = 0b0000_0111,
    Eight = 0b0111_1111,
    Nine = 0b0110_1111,
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
