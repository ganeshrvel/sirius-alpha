// ref: https://github.com/igelbox/tm1637-rs/blob/master/examples/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/master/examples/stm32f103/src/main.rs
// ref: https://github.com/rustrum/tmledkey-hal-drv/blob/b5e0759c41442d4e28c0ae26ad2bc393c43f814c/src/lib.rs

extern crate embedded_hal as hal;

use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs;
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
}

enum Bit {
    ZERO,
    ONE,
}

impl<'a, CLK, DIO, E> TM1637<'a, CLK, DIO>
where
    CLK: OutputPin<Error = E>,
    DIO: InputPin<Error = E> + OutputPin<Error = E>,
{
    pub fn new(clk: &'a mut CLK, dio: &'a mut DIO) -> Self {
        Self { clk, dio }
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
        self.print_raw_iter(address, bytes.iter().map(|b| *b))
    }

    pub fn print_hex(&mut self, address: u8, digits: &[u8], show_colon: bool) -> Res<E> {
        self.print_raw_iter(
            address,
            digits.iter().map(|digit| {
                let mut d = DIGITS[(digit & 0xf) as usize];

                if address == 1 && show_colon {
                    d |= SEG_8 as u8;
                }

                d
            }),
        )
    }

    pub fn print_char(&mut self, address: u8, digits: &[u8]) -> Res<E> {
        self.print_raw_iter(
            address,
            digits
                .iter()
                .map(|digit| CHAR_y /*DIGITS[(digit & 0xf) as usize]*/),
        )
    }

    pub fn set_colon(&mut self, address: u8) -> Res<E> {
        let d = vec![SEG_8];

        // self.print_raw_iter(
        //     address,
        //     [SEG_8]
        //         .iter()
        //         .map(|digit| SEG_8 /*DIGITS[(digit & 0xf) as usize]*/),
        //    // d.into_iter()
        // )

        let bytes = [SEG_8].iter().map(|digit| SEG_8);

        self.start()?;
        self.send(ADDRESS_COMMAND_BITS | (address & ADDRESS_COMMAND_MASK))?;

        for byte in bytes {
            self.send(byte)?;
        }

        self.stop()?;
        Ok(())
    }

    pub fn print_raw_iter<Iter: Iterator<Item = u8>>(
        &mut self,
        address: u8,
        bytes: Iter,
    ) -> Res<E> {
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
            let bit = if &rest & 1 != 0 { Bit::ONE } else { Bit::ZERO };
            self.send_bit_and_delay(bit)?;
            rest >>= 1;
        }

        // Wait for the ACK
        self.send_bit_and_delay(Bit::ONE)?;
        for _ in 0..255_i32 {
            if self.dio.is_low()? {
                return Ok(());
            }
            self.set_delay();
        }

        Err(Error::Ack)
    }

    fn start(&mut self) -> Res<E> {
        self.send_bit_and_delay(Bit::ONE)?;
        self.dio.set_low()?;

        Ok(())
    }

    fn stop(&mut self) -> Res<E> {
        self.send_bit_and_delay(Bit::ZERO)?;
        self.dio.set_high()?;
        self.set_delay();

        Ok(())
    }

    fn send_bit_and_delay(&mut self, value: Bit) -> Res<E> {
        self.clk.set_low()?;
        if let Bit::ONE = value {
            self.dio.set_high()?;
        } else {
            self.dio.set_low()?;
        }
        self.clk.set_high()?;
        self.set_delay();

        Ok(())
    }

    fn set_delay(&mut self) {
        let mut r = delay::FreeRtos {};
        r.delay_us(DELAY_USECS);
    }
}

const MAX_FREQ_KHZ: u16 = 500;
const USECS_IN_MSEC: u16 = 1_000;
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

/// Data control instruction set
pub const COM_DATA: u8 = 0b01000000;

/// Display control instruction set
pub const COM_DISPLAY: u8 = 0b10000000;

/// Address instruction set
pub const COM_ADDRESS: u8 = 0b11000000;

/// Address adding mode (write to display)
pub const COM_DATA_ADDRESS_ADD: u8 = COM_DATA | 0b000000;
/// Data fix address mode (write to display)
pub const COM_DATA_ADDRESS_FIXED: u8 = COM_DATA | 0b000100;
/// Read key scan data
pub const COM_DATA_READ: u8 = COM_DATA | 0b000010;

/// Display ON max brightness.
/// Can be combined with masked bytes to adjust brightness level
pub const COM_DISPLAY_ON: u8 = 0b10001000;
/// Display brightness mask
pub const DISPLAY_BRIGHTNESS_MASK: u8 = 0b00000111;
// Display OFF
pub const COM_DISPLAY_OFF: u8 = 0b10000000;

/// Segment A - top
pub const SEG_1: u8 = 0b1;
/// Segment B - top right
pub const SEG_2: u8 = 0b10;
/// Segment C - bottom right
pub const SEG_3: u8 = 0b100;
/// Segment D - bottom
pub const SEG_4: u8 = 0b1000;
/// Segment E - bottom left
pub const SEG_5: u8 = 0b10000;
/// Segment F - top left
pub const SEG_6: u8 = 0b100000;
/// Segment G - middle
pub const SEG_7: u8 = 0b1000000;
/// Segment DP (eight) - dot or colon
pub const SEG_8: u8 = 0b10000000;

/// Used with 3 wire interface for second byte
pub const SEG_9: u8 = SEG_1;
/// Used with 3 wire interface for second byte
pub const SEG_10: u8 = SEG_2;
/// Used with 3 wire interface for second byte
pub const SEG_11: u8 = SEG_3;
/// Used with 3 wire interface for second byte
pub const SEG_12: u8 = SEG_4;

pub const CHAR_0: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_1: u8 = SEG_2 | SEG_3;
pub const CHAR_2: u8 = SEG_1 | SEG_2 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_3: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_7;
pub const CHAR_4: u8 = SEG_2 | SEG_3 | SEG_6 | SEG_7;
pub const CHAR_5: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_6: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_7: u8 = SEG_1 | SEG_2 | SEG_3;
pub const CHAR_8: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_9: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_A: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_a: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_b: u8 = SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_C: u8 = SEG_1 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_c: u8 = SEG_4 | SEG_5 | SEG_7;
pub const CHAR_d: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_E: u8 = SEG_1 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_e: u8 = SEG_1 | SEG_2 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_F: u8 = SEG_1 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_G: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_H: u8 = SEG_2 | SEG_3 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_h: u8 = SEG_3 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_I: u8 = SEG_2 | SEG_3;
pub const CHAR_i: u8 = SEG_3;
pub const CHAR_J: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_5;
pub const CHAR_L: u8 = SEG_4 | SEG_5 | SEG_6;
pub const CHAR_l: u8 = SEG_4 | SEG_5;
pub const CHAR_N: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_5 | SEG_6;
pub const CHAR_n: u8 = SEG_3 | SEG_5 | SEG_7;
pub const CHAR_O: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_o: u8 = SEG_3 | SEG_4 | SEG_5 | SEG_7;
pub const CHAR_P: u8 = SEG_1 | SEG_2 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_q: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_6 | SEG_7;
pub const CHAR_R: u8 = SEG_1 | SEG_5 | SEG_6;
pub const CHAR_r: u8 = SEG_5 | SEG_7;
pub const CHAR_S: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_t: u8 = SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_U: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_u: u8 = SEG_3 | SEG_4 | SEG_5;
pub const CHAR_y: u8 = SEG_2 | SEG_3 | SEG_4 | SEG_6 | SEG_7;
pub const CHAR_CYR_E: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4 | SEG_7;
pub const CHAR_CYR_B: u8 = SEG_1 | SEG_3 | SEG_4 | SEG_5 | SEG_6 | SEG_7;
pub const CHAR_DEGREE: u8 = SEG_1 | SEG_2 | SEG_6 | SEG_7;
pub const CHAR_MINUS: u8 = SEG_7;
pub const CHAR_UNDERSCORE: u8 = SEG_4;
pub const CHAR_BRACKET_LEFT: u8 = SEG_1 | SEG_4 | SEG_5 | SEG_6;
pub const CHAR_BRACKET_RIGHT: u8 = SEG_1 | SEG_2 | SEG_3 | SEG_4;

/// List of digit characters where values correlates with array index 0-9.
pub const DIGITS: [u8; 10] = [
    CHAR_0, CHAR_1, CHAR_2, CHAR_3, CHAR_4, CHAR_5, CHAR_6, CHAR_7, CHAR_8, CHAR_9,
];
