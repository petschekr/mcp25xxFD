use modular_bitfield::prelude::*;
use defmt::Format;

pub trait RegisterAddress {
    const ADDRESS: u16;
}

pub trait Register: RegisterAddress + Sized {
    type Bitfield: Specifier<Bytes=u32, InOut=Self::Bitfield>;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self;
    fn into_bitfield(self) -> Self::Bitfield;
    fn parse(data: &[u8]) -> Self {
        let data = u32::from_le_bytes(data.try_into().unwrap());
        Self::from_bitfield(Self::Bitfield::from_bytes(data).unwrap())
    }
    fn serialize(self) -> [u8; 4] {
        Self::Bitfield::into_bytes(self.into_bitfield())
            .unwrap()
            .to_le_bytes()
    }
}
impl<T> Register for T
where
    T: Specifier<Bytes=u32, InOut=Self> + RegisterAddress
{
    type Bitfield = Self;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self { bitfield }
    fn into_bitfield(self) -> Self::Bitfield { self }
}

/// Data Length Code
#[derive(BitfieldSpecifier, PartialEq, Eq, Copy, Clone, Debug, Format)]
#[bits = 4]
#[allow(non_camel_case_types)]
pub enum DataLengthCode {
    DLC_0 = 0,
    DLC_1 = 1,
    DLC_2 = 2,
    DLC_3 = 3,
    DLC_4 = 4,
    DLC_5 = 5,
    DLC_6 = 6,
    DLC_7 = 7,
    DLC_8 = 8,
    DLC_12 = 9,
    DLC_16 = 10,
    DLC_20 = 11,
    DLC_24 = 12,
    DLC_32 = 13,
    DLC_48 = 14,
    DLC_64 = 15,
}

#[bitfield(bits = 64)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitMessageObjectHeader {
    /// Data Length Code
    pub dlc: DataLengthCode,
    /// Identifier Extension Flag
    /// Distinguishes between base and extended format
    pub ide: bool,
    /// Remote Transmission Request (not used for CAN-FD)
    pub rtr: bool,
    /// Bit Rate Switch
    pub brs: bool,
    /// FD Frame
    pub fdf: bool,
    /// Error Status Indicator
    pub esi: bool,
    /// Sequence to keep track of transmitted messages in Transmit Event FIFO
    pub seq: B23,
    /// Standard Identifier
    pub sid: B11,
    /// Extended Identifier
    pub eid: B18,
    /// In FD mode the standard ID can be extended to 12 bit using r1
    pub sid11: bool,
    #[skip] __: B2,
}

#[bitfield(bits = 64)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct ReceiveMessageObjectHeader {
    /// Data Length Code
    pub dlc: DataLengthCode,
    /// Identifier Extension Flag
    /// Distinguishes between base and extended format
    pub ide: bool,
    /// Remote Transmission Request (not used for CAN-FD)
    pub rtr: bool,
    /// Bit Rate Switch
    pub brs: bool,
    /// FD Frame
    pub fdf: bool,
    /// Error Status Indicator
    pub esi: bool,
    #[skip] __: B2,
    /// Filter Hit, number of filter that matched
    pub filthit: B5,
    #[skip] __: B16,
    /// Standard Identifier
    pub sid: B11,
    /// Extended Identifier
    pub eid: B18,
    /// In FD mode the standard ID can be extended to 12 bit using r1
    pub sid11: bool,
    #[skip] __: B2,
}

/// Clock Output Divisor
#[derive(BitfieldSpecifier, PartialEq, Eq, Copy, Clone, Debug, Format)]
#[bits = 2]
pub enum ClockOutputDivisor {
    DivideBy1 = 0b00,
    DivideBy2 = 0b01,
    DivideBy4 = 0b10,
    DivideBy10 = 0b11,
}

/// System Clock Divisor
#[derive(BitfieldSpecifier, PartialEq, Eq, Copy, Clone, Debug, Format)]
#[bits = 1]
pub enum ClockDivisor {
    DivideBy1 = 0b00,
    DivideBy2 = 0b01,
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct OscillatorControl {
    /// PLL enable
    pub pllen: bool,
    #[skip] __: B1,
    /// Clock (Oscillator) disable (i.e. sleep mode)
    pub oscdis: bool,
    /// Low Power Mode
    pub lpmen: bool,
    /// System Clock Divisor
    pub sclkdiv: ClockDivisor,
    /// Clock Output Divisor
    pub clkodiv: ClockOutputDivisor,
    #[skip] __: B1,
    #[skip(setters)]
    /// PLL ready
    pub pllrdy: bool,
    #[skip] __: B1,
    /// Clock ready
    #[skip(setters)]
    pub oscrdy: bool,
    #[skip] __: B1,
    /// Synchronized SCLKDIV
    #[skip(setters)]
    pub sclkrdy: bool,
    #[skip] __: B19,
}
impl RegisterAddress for OscillatorControl {
    const ADDRESS: u16 = 0xE00;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct IOControl {
    /// GPIO0 Data Direction
    pub tris0: bool,
    /// GPIO1 Data Direction
    pub tris1: bool,
    #[skip] __: B4,
    /// Enable Transceiver Standby Pin Control
    pub xstbyen: bool,
    #[skip] __: B1,
    /// GPIO0 latch
    pub lat0: bool,
    /// GPIO1 latch
    pub lat1: bool,
    #[skip] __: B6,
    /// GPIO0 status
    pub gpio0: bool,
    /// GPIO1 status
    pub gpio1: bool,
    #[skip] __: B6,
    /// GPIO0 pin mode
    pub pm0: bool,
    /// GPIO1 pin mode
    pub pm1: bool,
    #[skip] __: B2,
    /// TXCAN Open Drain Mode
    pub txcanod: bool,
    /// Start-of-Frame signal
    pub sof: bool,
    /// Interrupt pins Open Drain Mode
    pub intod: bool,
    #[skip] __: B1,
}
impl RegisterAddress for IOControl {
    const ADDRESS: u16 = 0xE04;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct CRCStatus {
    /// CRC value from last CRC mismatch
    pub crc: u16,
    /// CRC Error Interrupt Flag
    pub crcerrif: bool,
    /// CRC Command Format Error Interrupt Flag
    pub ferrif: bool,
    #[skip] __: B6,
    /// CRC Error Interrupt Enable
    pub crcerrie: bool,
    /// CRC Command Format Error Interrupt Enable
    pub ferrie: bool,
    #[skip] __: B6,
}
impl RegisterAddress for CRCStatus {
    const ADDRESS: u16 = 0xE08;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct ECCControl {
    /// ECC enable
    pub eccen: bool,
    /// Single Error Detection Interrupt Enable
    pub secie: bool,
    /// Double Error Detection Interrupt Enable
    pub dedie: bool,
    #[skip] __: B5,
    /// Parity bits used during write to RAM when ECC is disabled
    pub parity: B7,
    #[skip] __: B17,
}
impl RegisterAddress for ECCControl {
    const ADDRESS: u16 = 0xE0C;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct ECCStatus {
    #[skip] __: B1,
    /// Single Error Detection Interrupt Flag
    pub secie: bool,
    /// Double Error Detection Interrupt Flag
    pub dedie: bool,
    #[skip] __: B5,
    #[skip] __: B8,
    /// Address where last ECC error occurred
    #[skip(setters)]
    pub erraddr: B12,
    #[skip] __: B4,
}
impl RegisterAddress for ECCStatus {
    const ADDRESS: u16 = 0xE10;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct DeviceID {
    /// Silicon Revision
    #[skip(setters)]
    pub rev: B4,
    /// Device ID
    #[skip(setters)]
    pub id: B4,
    #[skip] __: B24,
}
impl RegisterAddress for DeviceID {
    const ADDRESS: u16 = 0xE14;
}

/// Request Operation mode
#[derive(BitfieldSpecifier, PartialEq, Eq, Copy, Clone, Debug, Format)]
#[bits = 3]
pub enum OperationMode {
    Normal = 0b000,
    Sleep = 0b001,
    InternalLoopback = 0b010,
    ListenOnly = 0b011,
    Configuration = 0b100,
    ExternalLoopback = 0b101,
    Classic = 0b110,
    Restricted = 0b111,
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct CANControl {
    /// Device Net Filter Bit Number
    pub dncnt: B5,
    /// Enable ISO CRC in CAN FD Frames
    pub isocrcen: bool,
    /// Protocol Exception Event Detection Disabled
    pub pxedis: bool,
    #[skip] __: B1,
    /// Enable CAN Bus Line Wake-up Filter
    pub wakfil: bool,
    /// Selectable Wake-up Filter Time
    pub wft: B2,
    /// CAN Module is Busy
    #[skip(setters)]
    pub busy: bool,
    /// Bit Rate Switching Disable
    pub brsdis: bool,
    #[skip] __: B3,
    /// Restrict Retransmission Attempts
    pub rtxat: bool,
    /// Transmit ESI in Gateway Mode
    pub esigm: bool,
    /// Transition to Listen Only Mode on System Error
    pub serr2lom: bool,
    /// Store in Transmit Event FIFO
    pub stef: bool,
    /// Enable Transmission Queue
    pub txqen: bool,
    /// Operation Mode Status
    #[skip(setters)]
    pub opmode: OperationMode,
    /// Request Operation Mode
    pub reqop: OperationMode,
    /// Abort All Pending Transmissions
    pub abat: bool,
    /// Transmit Bandwidth Sharing bits
    pub txbws: B4,
}
impl RegisterAddress for CANControl {
    const ADDRESS: u16 = 0x000;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct NominalBitTimeConfig {
    /// Synchronization Jump Width
    pub sjw: B7,
    #[skip] __: B1,
    /// Time Segment 2 bits (Phase Segment 2)
    pub tseg2: B7,
    #[skip] __: B1,
    /// Time Segment 1 bits (Propagation Segment + Phase Segment 1)
    pub tseg1: B8,
    /// Baud Rate Prescaler
    pub brp: B8,
}
impl RegisterAddress for NominalBitTimeConfig {
    const ADDRESS: u16 = 0x004;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct DataBitTimeConfig {
    /// Synchronization Jump Width
    pub sjw: B4,
    #[skip] __: B4,
    /// Time Segment 2 bits (Phase Segment 2)
    pub tseg2: B4,
    #[skip] __: B4,
    /// Time Segment 1 bits (Propagation Segment + Phase Segment 1)
    pub tseg1: B5,
    #[skip] __: B3,
    /// Baud Rate Prescaler
    pub brp: B8,
}
impl RegisterAddress for DataBitTimeConfig {
    const ADDRESS: u16 = 0x008;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitterDelayCompensation {
    /// Transmitter Delay Compensation Value bits; Secondary Sample Point (SSP)
    pub tdcv: B6,
    #[skip] __: B2,
    /// Transmitter Delay Compensation Offset bits; Secondary Sample Point (SSP)
    pub tdco: B7,
    #[skip] __: B1,
    /// Transmitter Delay Compensation Mode bits; Secondary Sample Point (SSP)
    pub tdcmod: B2,
    #[skip] __: B6,
    /// Enable 12-Bit SID in CAN FD Base Format Messages
    pub sid11en: bool,
    /// Enable Edge Filtering during Bus Integration state
    pub edgflten: bool,
    #[skip] __: B6,
}
impl RegisterAddress for TransmitterDelayCompensation {
    const ADDRESS: u16 = 0x00C;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TimeBaseCounter {
    /// Time Base Counter
    /// This is a free running timer that increments every TBCPRE clocks when TBCEN is set
    pub tbc: u32,
}
impl RegisterAddress for TimeBaseCounter {
    const ADDRESS: u16 = 0x010;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TimeStampControl {
    /// Time Base Counter Prescaler
    pub tbcpre: B10,
    #[skip] __: B6,
    /// Time Base Counter Enable
    pub tbcen: bool,
    /// Time Stamp EOF
    pub tseof: bool,
    /// Time Stamp res bit (FD frames only)
    pub tsres: bool,
    #[skip] __: B13,
}
impl RegisterAddress for TimeStampControl {
    const ADDRESS: u16 = 0x014;
}

/// Interrupt Flag
/// If multiple interrupts are pending, the interrupt with the highest number will be indicated
#[derive(BitfieldSpecifier, PartialEq, Eq, Copy, Clone, Debug, Format)]
#[bits = 7]
pub enum InterruptFlag {
    TXQ = 0b0000000,
    FIFO1 = 1,
    FIFO2 = 2,
    FIFO3 = 3,
    FIFO4 = 4,
    FIFO5 = 5,
    FIFO6 = 6,
    FIFO7 = 7,
    FIFO8 = 8,
    FIFO9 = 9,
    FIFO10 = 10,
    FIFO11 = 11,
    FIFO12 = 12,
    FIFO13 = 13,
    FIFO14 = 14,
    FIFO15 = 15,
    FIFO16 = 16,
    FIFO17 = 17,
    FIFO18 = 18,
    FIFO19 = 19,
    FIFO20 = 20,
    FIFO21 = 21,
    FIFO22 = 22,
    FIFO23 = 23,
    FIFO24 = 24,
    FIFO25 = 25,
    FIFO26 = 26,
    FIFO27 = 27,
    FIFO28 = 28,
    FIFO29 = 29,
    FIFO30 = 30,
    FIFO31 = 31,
    NoInterrupt = 0b1000000,
    Error = 0b1000001,
    WakeUp = 0b1000010,
    ReceiveFIFOOverflow = 0b1000011,
    AddressError = 0b1000100,
    RXTXMABUnderflowOverflow = 0b1000101,
    TBCOverflow = 0b1000110,
    OperationModeChange = 0b1000111,
    InvalidMessage = 0b1001000,
    TransmitEventFIFO = 0b1001001,
    TransmitAttempt = 0b1001010,
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct InterruptCode {
    /// Interrupt Flag Code
    #[skip(setters)]
    pub icode: InterruptFlag,
    #[skip] __: B1,
    /// Filter Hit Number
    #[skip(setters)]
    pub filhit: B5,
    #[skip] __: B3,
    /// Transmit Interrupt Flag Code
    #[skip(setters)]
    pub txcode: InterruptFlag,
    #[skip] __: B1,
    /// Receive Interrupt Flag Code
    #[skip(setters)]
    pub rxcode: InterruptFlag,
    #[skip] __: B1,
}
impl RegisterAddress for InterruptCode {
    const ADDRESS: u16 = 0x018;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct Interrupts {
    /// Transmit FIFO Interrupt Flag
    #[skip(setters)]
    pub txif: bool,
    /// Receive FIFO Interrupt Flag
    #[skip(setters)]
    pub rxif: bool,
    /// Time Base Counter Overflow Interrupt Flag
    pub tbcif: bool,
    /// Operation Mode Change Interrupt Flag
    pub modif: bool,
    /// Transmit Event FIFO Interrupt Flag
    #[skip(setters)]
    pub tefif: bool,
    #[skip] __: B3,
    /// ECC Error Interrupt Flag
    #[skip(setters)]
    pub eccif: bool,
    /// SPI CRC Error Interrupt Flag
    #[skip(setters)]
    pub spicrcif: bool,
    /// Transmit Attempt Interrupt Flag
    #[skip(setters)]
    pub txatif: bool,
    /// Receive Object Overflow Interrupt Flag
    #[skip(setters)]
    pub rxovif: bool,
    /// System Error Interrupt Flag
    pub serrif: bool,
    /// CAN Bus Error Interrupt Flag
    pub cerrif: bool,
    /// Bus Wake Up Interrupt Flag
    pub wakif: bool,
    /// Invalid Message Interrupt Flag
    pub ivmif: bool,
    /// Transmit FIFO Interrupt Enable
    pub txie: bool,
    /// Receive FIFO Interrupt Enable
    pub rxie: bool,
    /// Time Base Counter Interrupt Enable
    pub tbcie: bool,
    /// Mode Change Interrupt Enable
    pub modie: bool,
    /// Transmit Event FIFO Interrupt Enable
    pub tefie: bool,
    #[skip] __: B3,
    /// ECC Error Interrupt Enable
    pub eccie: bool,
    /// SPI CRC Error Interrupt Enable
    pub spicrcie: bool,
    /// Transmit Attempt Interrupt Enable
    pub txatie: bool,
    /// Receive FIFO Overflow Interrupt Enable
    pub rxovie: bool,
    /// System Error Interrupt Enable
    pub serrie: bool,
    /// CAN Bus Error Interrupt Enable
    pub cerrie: bool,
    /// Bus Wake Up Interrupt Enable
    pub wakeie: bool,
    /// Invalid Message Interrupt Enable
    pub ivmie: bool,
}
impl RegisterAddress for Interrupts {
    const ADDRESS: u16 = 0x01C;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct ReceiveInterruptStatus {
    #[skip] __: B1,
    /// Receive FIFO Interrupt Pending
    /// 'or’ of enabled RXFIFO flags; flags will be cleared when the condition of the FIFO terminates
    #[skip(setters)]
    pub rfif: B31,
}
impl RegisterAddress for ReceiveInterruptStatus {
    const ADDRESS: u16 = 0x020;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct ReceiveOverflowInterruptStatus {
    #[skip] __: B1,
    /// Receive FIFO Overflow Interrupt Pending
    #[skip(setters)]
    pub rfovif: B31,
}
impl RegisterAddress for ReceiveOverflowInterruptStatus {
    const ADDRESS: u16 = 0x028;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitInterruptStatus {
    /// TXQ Interrupt Pending
    #[skip(setters)]
    pub txqif: B1,
    /// Transmit FIFO Interrupt Pending
    /// 'or’ of enabled TXFIFO flags; flags will be cleared when the condition of the FIFO terminates
    #[skip(setters)]
    pub tfif: B31,
}
impl RegisterAddress for TransmitInterruptStatus {
    const ADDRESS: u16 = 0x024;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitAttemptInterruptStatus {
    /// TXQ Attempt Interrupt Pending
    #[skip(setters)]
    pub txqatif: B1,
    /// Transmit FIFO Attempt Interrupt Pending
    /// 'or’ of enabled TXFIFO flags; flags will be cleared when the condition of the FIFO terminates
    #[skip(setters)]
    pub tfatif: B31,
}
impl RegisterAddress for TransmitAttemptInterruptStatus {
    const ADDRESS: u16 = 0x02C;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitRequest {
    /// Transmit Queue Message Send Request
    /// Will automatically clear when the message(s) queued is/are successfully sent
    #[skip(setters)]
    pub txqreq: bool,
    /// Transmit FIFO Message Send Request
    /// Will automatically clear when the message(s) queued is/are successfully sent
    #[skip(setters)]
    pub txreq: B31,
}
impl RegisterAddress for TransmitRequest {
    const ADDRESS: u16 = 0x030;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitReceiveErrorCount {
    /// Receive Error Counter
    #[skip(setters)]
    pub rec: u8,
    /// Transmit Error Counter
    #[skip(setters)]
    pub tec: u8,
    /// Transmitter or Receiver is in Error Warning State
    #[skip(setters)]
    pub ewarn: bool,
    /// Receiver in Error Warning State (128 > REC > 95)
    #[skip(setters)]
    pub rxwarn: bool,
    /// Transmitter in Error Warning State (128 > TEC > 95
    #[skip(setters)]
    pub txwarn: bool,
    /// Receiver in Error Passive State (REC > 127
    #[skip(setters)]
    pub rxbp: bool,
    /// Transmitter in Error Passive State (TEC > 127
    #[skip(setters)]
    pub txbp: bool,
    /// Transmitter in Bus Off State bit (TEC > 255)
    #[skip(setters)]
    pub txbo: bool,
    #[skip] __: B10,
}
impl RegisterAddress for TransmitReceiveErrorCount {
    const ADDRESS: u16 = 0x034;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct BusDiagnostic0 {
    /// Nominal Bit Rate Receive Error Counter
    pub nrerrcnt: u8,
    /// Nominal Bit Rate Transmit Error Counter
    pub nterrcnt: u8,
    /// Data Bit Rate Receive Error Counter
    pub drerrcnt: u8,
    /// Data Bit Rate Transmit Error Counter
    pub dterrcnt: u8,
}
impl RegisterAddress for BusDiagnostic0 {
    const ADDRESS: u16 = 0x038;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct BusDiagnostic1 {
    /// Error-free Message Counter
    pub efmsgcnt: u16,
    /// During the transmission of a message (or acknowledge bit, or active error flag, or overload
    /// flag), the device wanted to send a dominant level (data or identifier bit logical value ‘0’), but the
    /// monitored bus value was recessive.
    pub nbit0err: bool,
    /// During the transmission of a message (with the exception of the arbitration field), the
    /// device wanted to send a recessive level (bit of logical value ‘1’), but the monitored bus value was
    /// dominant.
    pub nbit1err: bool,
    /// Transmitted message was not acknowledged
    pub nackerr: bool,
    /// A fixed format part of a received frame has the wrong format
    pub nformerr: bool,
    /// More than 5 equal bits in a sequence have occurred in a part of a received message
    /// where this is not allowed.
    pub nstuferr: bool,
    /// The CRC check sum of a received message was incorrect. The CRC of an incoming
    /// message does not match with the CRC calculated from the received data
    pub ncrcerr: bool,
    #[skip] __: B1,
    /// Device went to bus-off (and auto-recovered)
    pub txboerr: bool,
    pub dbit0err: bool,
    pub dbit1err: bool,
    #[skip] __: B1,
    pub dformerr: bool,
    pub dstuferr: bool,
    pub dcrcerr: bool,
    /// ESI flag of a received CAN FD message was set
    pub esi: bool,
    /// DLC Mismatch
    /// During a transmission or reception, the specified DLC is larger than the PLSIZE of the FIFO element
    pub dlcmm: bool,
}
impl RegisterAddress for BusDiagnostic1 {
    const ADDRESS: u16 = 0x03C;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitEventFIFOControl {
    /// Transmit Event FIFO Not Empty Interrupt Enable
    pub tefneie: bool,
    /// Transmit Event FIFO Half Full Interrupt Enable
    pub tefhie: bool,
    /// Transmit Event FIFO Full Interrupt Enable
    pub teffie: bool,
    /// Transmit Event FIFO Overflow Interrupt Enable
    pub tefovie: bool,
    #[skip] __: B1,
    /// Transmit Event FIFO Time Stamp Enable
    pub teftsen: bool,
    #[skip] __: B2,
    /// Increment Tail
    /// When this bit is set, the FIFO tail will increment by a single message
    pub uinc: bool,
    #[skip] __: B1,
    /// FIFO will reset -- wait to clear before taking action
    pub freset: bool,
    #[skip] __: B13,
    /// FIFO Size
    /// Begins at 0b0000 = 1
    pub fsize: B5,
    #[skip] __: B3,
}
impl RegisterAddress for TransmitEventFIFOControl {
    const ADDRESS: u16 = 0x040;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitEventFIFOStatus {
    /// Transmit Event FIFO Not Empty Interrupt Flag
    #[skip(setters)]
    pub tefneif: bool,
    /// Transmit Event FIFO Half Full Interrupt Flag
    #[skip(setters)]
    pub tefhif: bool,
    /// Transmit Event FIFO Full Interrupt Flag
    #[skip(setters)]
    pub teffif: bool,
    /// Transmit Event FIFO Overflow Interrupt Flag
    #[skip(setters)]
    pub tefovif: bool,
    #[skip] __: B28,
}
impl RegisterAddress for TransmitEventFIFOStatus {
    const ADDRESS: u16 = 0x044;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitEventFIFOUserAddress {
    /// Transmit Event FIFO User Address
    /// The address where the next object is to be read (FIFO tail)
    #[skip(setters)]
    pub tefua: u32,
}
impl RegisterAddress for TransmitEventFIFOUserAddress {
    const ADDRESS: u16 = 0x048;
}

#[derive(BitfieldSpecifier, PartialEq, Eq, Copy, Clone, Debug, Format)]
#[bits = 2]
pub enum RetransmissionAttempts {
    Disable = 0b00,
    Three = 0b01,
    Unlimited1 = 0b10,
    Unlimited2 = 0b11,
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitQueueControl {
    /// Transmit Queue Not Full Interrupt Enable
    pub txqnie: bool,
    #[skip] __: B1,
    /// Transmit Queue Empty Interrupt Enable
    pub txqeie: bool,
    #[skip] __: B1,
    /// Transmit Attempts Exhausted Interrupt Enable
    pub txatie: bool,
    #[skip] __: B2,
    /// TX Enable (always true)
    pub txen: bool,
    /// Increment Head
    pub uinc: bool,
    /// Message Send Request
    pub txreq: bool,
    /// FIFO Reset
    pub freset: bool,
    #[skip] __: B5,
    /// Message Transmit Priority
    /// 0 = lowest, 31 = highest
    pub txpri: B5,
    /// Retransmission Attempts
    pub txat: RetransmissionAttempts,
    #[skip] __: B1,
    /// FIFO Size
    /// Begins at 0b0000 = 1
    pub fsize: B5,
    /// Payload Size
    /// 0b000 = 8, 0b111 = 64
    pub plsize: B3,
}
impl RegisterAddress for TransmitQueueControl {
    const ADDRESS: u16 = 0x050;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitQueueStatus {
    /// Transmit Queue Not Full Interrupt Flag
    #[skip(setters)]
    pub txqnif: bool,
    #[skip] __: B1,
    /// Transmit Queue Empty Interrupt Flag
    #[skip(setters)]
    pub txqeif: bool,
    #[skip] __: B1,
    /// Transmit Attempts Exhausted Interrupt Flag
    pub txatif: bool,
    /// Error Detected During Transmission
    pub txerr: bool,
    /// Message Lost Arbitration Status
    pub txlarb: bool,
    /// Message Aborted Status
    pub txabt: bool,
    /// Transmit Queue Message Index
    /// Index to the message that the FIFO will next attempt to transmit
    #[skip(setters)]
    pub txqci: B5,
    #[skip] __: B19,
}
impl RegisterAddress for TransmitQueueStatus {
    const ADDRESS: u16 = 0x054;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct TransmitQueueUserAddress {
    /// TXQ User Address
    /// The address where the next message is to be written (TXQ head)
    #[skip(setters)]
    pub txqua: u32,
}
impl RegisterAddress for TransmitQueueUserAddress {
    const ADDRESS: u16 = 0x058;
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct FIFOControlM {
    /// Transmit/Receive FIFO Not Full/Not Empty Interrupt Enable
    pub tfnrfnie: bool,
    /// Transmit/Receive FIFO Half Empty/Half Full Interrupt Enable
    pub tfhrfhie: bool,
    /// Transmit/Receive FIFO Empty/Full Interrupt Enable
    pub tferffie: bool,
    /// Overflow Interrupt Enable
    pub rxovie: bool,
    /// Transmit Attempts Exhausted Interrupt Enable
    pub txatie: bool,
    /// Received Message Time Stamp Enable
    pub rxtsen: bool,
    /// Auto RTR Enable
    pub rtren: bool,
    /// TX/RX FIFO Selection
    /// true = Transmit, false = Receive
    pub txen: bool,
    /// Increment Head/Tail
    pub uinc: bool,
    /// Message Send Request
    pub rxreq: bool,
    /// FIFO Reset
    pub freset: bool,
    #[skip] __: B5,
    /// Message Transmit Priority
    pub txpri: B5,
    /// Retransmission Attempts
    pub txat: RetransmissionAttempts,
    #[skip] __: B1,
    /// FIFO Size
    pub fsize: B5,
    /// Payload Size
    pub plsize: B3,
}
pub struct FIFOControl<const M: u8> {
    pub contents: FIFOControlM,
}

impl<const M: u8> RegisterAddress for FIFOControl<M> {
    const ADDRESS: u16 = 0x05C + 12 * (M as u16 - 1); // M is 1-indexed
}
impl<const M: u8> Register for FIFOControl<M> {
    type Bitfield = FIFOControlM;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self {
        Self { contents: bitfield }
    }
    fn into_bitfield(self) -> Self::Bitfield {
        self.contents
    }
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct FIFOStatusM {
    /// Transmit/Receive FIFO Not Full/Not Empty Interrupt Flag
    #[skip(setters)]
    pub tfnrfnif: bool,
    /// Transmit/Receive FIFO Half Empty/Half Full Interrupt Flag
    #[skip(setters)]
    pub tfhrfhif: bool,
    /// Transmit/Receive FIFO Empty/Full Interrupt Flag
    #[skip(setters)]
    pub tferffif: bool,
    /// Receive FIFO Overflow Interrupt Flag
    pub rxovif: bool,
    /// Transmit Attempts Exhausted Interrupt Pending
    pub txatif: bool,
    /// Error Detected During Transmission
    pub txerr: bool,
    /// Message Lost Arbitration Status
    pub txlarb: bool,
    /// Message Aborted Status
    pub txabt: bool,
    /// FIFO Message Index
    #[skip(setters)]
    pub fifoci: B5,
    #[skip] __: B19,
}
pub struct FIFOStatus<const M: u8> {
    contents: FIFOStatusM,
}
impl<const M: u8> RegisterAddress for FIFOStatus<M> {
    const ADDRESS: u16 = 0x060 + 12 * (M as u16 - 1); // M is 1-indexed
}
impl<const M: u8> Register for FIFOStatus<M> {
    type Bitfield = FIFOStatusM;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self {
        Self { contents: bitfield }
    }
    fn into_bitfield(self) -> Self::Bitfield {
        self.contents
    }
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct FIFOUserAddressM {
    /// FIFO User Address
    /// The address where the next message is to be written (FIFO head)
    /// The address where the next message is to be read (FIFO tail)
    #[skip(setters)]
    pub fifoua: u32,
}
pub struct FIFOUserAddress<const M: u8> {
    contents: FIFOUserAddressM,
}
impl<const M: u8> RegisterAddress for FIFOUserAddress<M> {
    const ADDRESS: u16 = 0x064 + 12 * (M as u16 - 1); // M is 1-indexed
}
impl<const M: u8> Register for FIFOUserAddress<M> {
    type Bitfield = FIFOUserAddressM;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self {
        Self { contents: bitfield }
    }
    fn into_bitfield(self) -> Self::Bitfield {
        self.contents
    }
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct FilterControlM {
    /// Pointer to FIFO when Filter 0 hits
    pub f0bp: B5,
    #[skip] __: B2,
    /// Enable Filter 0 to Accept Messages
    pub flten0: bool,
    /// Pointer to FIFO when Filter 1 hits
    pub f1bp: B5,
    #[skip] __: B2,
    /// Enable Filter 1 to Accept Messages
    pub flten1: bool,
    /// Pointer to FIFO when Filter 2 hits
    pub f2bp: B5,
    #[skip] __: B2,
    /// Enable Filter 2 to Accept Messages
    pub flten2: bool,
    /// Pointer to FIFO when Filter 3 hits
    pub f3bp: B5,
    #[skip] __: B2,
    /// Enable Filter 3 to Accept Messages
    pub flten3: bool,
}
pub struct FilterControl<const M: u8> {
    contents: FilterControlM,
}
impl<const M: u8> RegisterAddress for FilterControl<M> {
    const ADDRESS: u16 = 0x1D0 + 4 * (M as u16); // M is 0-indexed
}
impl<const M: u8> Register for FilterControl<M> {
    type Bitfield = FilterControlM;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self {
        Self { contents: bitfield }
    }
    fn into_bitfield(self) -> Self::Bitfield {
        self.contents
    }
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct FilterObjectM {
    /// Standard Identifier filter
    pub sid: B11,
    /// Extended Identifier bits
    pub eid: B18,
    /// Standard Identifier filter
    pub sid11: bool,
    /// Extended Identifier enable
    pub exide: bool,
    #[skip] __: B1,
}
pub struct FilterObject<const M: u8> {
    contents: FilterObjectM,
}
impl<const M: u8> RegisterAddress for FilterObject<M> {
    const ADDRESS: u16 = 0x1F0 + 8 * (M as u16); // M is 0-indexed
}
impl<const M: u8> Register for FilterObject<M> {
    type Bitfield = FilterObjectM;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self {
        Self { contents: bitfield }
    }
    fn into_bitfield(self) -> Self::Bitfield {
        self.contents
    }
}

#[bitfield(bits = 32)]
#[derive(BitfieldSpecifier, Copy, Clone, Debug, Format, Default)]
pub struct MaskM {
    /// Standard Identifier mask
    pub msid: B11,
    /// Extended Identifier mask
    pub meid: B18,
    /// Standard Identifier mask
    pub msid11: bool,
    /// Identifier Receive mode bit
    pub mide: bool,
    #[skip] __: B1,
}
pub struct Mask<const M: u8> {
    contents: MaskM,
}
impl<const M: u8> RegisterAddress for Mask<M> {
    const ADDRESS: u16 = 0x1F4 + 8 * (M as u16); // M is 0-indexed
}
impl<const M: u8> Register for Mask<M> {
    type Bitfield = MaskM;
    fn from_bitfield(bitfield: Self::Bitfield) -> Self {
        Self { contents: bitfield }
    }
    fn into_bitfield(self) -> Self::Bitfield {
        self.contents
    }
}