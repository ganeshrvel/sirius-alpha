/// Shows which segment has which bit.
#[repr(u8)]
pub enum SegmentBits {
    SegA = 0b0000_0001,
    SegB = 0b0000_0010,
    SegC = 0b0000_0100,
    SegD = 0b0000_1000,
    SegE = 0b0001_0000,
    SegF = 0b0010_0000,
    SegG = 0b0100_0000,

    // double point on AzDelivery 4-digit 7 segment display.
    SegColonOrDot = 0b1000_0000,
}

#[repr(u8)]
pub enum NumberCharBits {
    Zero = 0b0011_1111,
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
    /***
    these are the 3 base commands: to see the meaning
    of bits 0 to 5 see data sheet;
    6 & 7 are reserved to mark the kind of command
    // data command
    COMM1_BASE = 0b0100_000,
    // addressing mode
    COMM2_BASE = 0b1100_000,
    // display control
    COMM3_BASE = 0b1000_000,
     ***/
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
