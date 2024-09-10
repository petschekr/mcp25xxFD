#![allow(clippy::identity_op)] // FIXME https://github.com/Robbepop/modular-bitfield/issues/62

use core::mem;
use modular_bitfield::prelude::*;

type RegisterSize = u32;

/// 32 bit device register
trait Register<const INDEX: u16 = 0, const INTERVAL: u16 = 1> {
    /// Base address of the register
    /// If an array, address of the first register in the array
    const ADDRESS: u16;
    fn get_address() -> u16 {
        Self::ADDRESS + INDEX * size_of::<RegisterSize>() as u16 * INTERVAL
    }
}

/// Request Operation mode
#[derive(BitfieldSpecifier, Copy, Clone, Debug)]
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

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CANControl {
    /// Device Net Filter Bit Number
    dncnt: B5,
    /// Enable ISO CRC in CAN FD Frames
    isocrcen: bool,
    /// Protocol Exception Event Detection Disabled
    pxedis: bool,
    #[skip] __: B1,
    /// Enable CAN Bus Line Wake-up Filter
    wakfil: bool,
    /// Selectable Wake-up Filter Time
    wft: B2,
    /// CAN Module is Busy
    #[skip(setters)] busy: bool,
    /// Bit Rate Switching Disable
    brsdis: bool,
    #[skip] __: B3,
    /// Restrict Retransmission Attempts
    rtxat: bool,
    /// Transmit ESI in Gateway Mode
    esigm: bool,
    /// Transition to Listen Only Mode on System Error
    serr2lom: bool,
    /// Store in Transmit Event FIFO
    stef: bool,
    /// Enable Transmission Queue
    txqen: bool,
    /// Operation Mode Status
    #[skip(setters)] opmode: OperationMode,
    /// Request Operation Mode
    reqop: OperationMode,
    /// Abort All Pending Transmissions
    abat: bool,
    /// Transmit Bandwidth Sharing bits
    txbws: B4,
}
impl Register for CANControl {
    const ADDRESS: u16 = 0x000;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct NominalBitTimeConfig {
    /// Synchronization Jump Width
    sjw: B7,
    #[skip] __: B1,
    /// Time Segment 2 bits (Phase Segment 2)
    tseg2: B7,
    #[skip] __: B1,
    /// Time Segment 1 bits (Propagation Segment + Phase Segment 1)
    tseg1: B8,
    /// Baud Rate Prescaler
    brp: B8,
}
impl Register for NominalBitTimeConfig {
    const ADDRESS: u16 = 0x004;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DataBitTimeConfig {
    /// Synchronization Jump Width
    sjw: B4,
    #[skip] __: B4,
    /// Time Segment 2 bits (Phase Segment 2)
    tseg2: B4,
    #[skip] __: B4,
    /// Time Segment 1 bits (Propagation Segment + Phase Segment 1)
    tseg1: B5,
    #[skip] __: B3,
    /// Baud Rate Prescaler
    brp: B8,
}
impl Register for DataBitTimeConfig {
    const ADDRESS: u16 = 0x008;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitterDelayCompensation {
    /// Transmitter Delay Compensation Value bits; Secondary Sample Point (SSP)
    tdcv: B6,
    #[skip] __: B2,
    /// Transmitter Delay Compensation Offset bits; Secondary Sample Point (SSP)
    tdco: B7,
    #[skip] __: B1,
    /// Transmitter Delay Compensation Mode bits; Secondary Sample Point (SSP)
    tdcmod: B2,
    #[skip] __: B6,
    /// Enable 12-Bit SID in CAN FD Base Format Messages
    sid11en: bool,
    /// Enable Edge Filtering during Bus Integration state
    edgflten: bool,
    #[skip] __: B6,
}
impl Register for TransmitterDelayCompensation {
    const ADDRESS: u16 = 0x00C;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TimeBaseCounter {
    /// Time Base Counter
    /// This is a free running timer that increments every TBCPRE clocks when TBCEN is set
    tbc: u32,
}
impl Register for TimeBaseCounter {
    const ADDRESS: u16 = 0x010;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TimeStampControl {
    /// Time Base Counter Prescaler
    tbcpre: B10,
    #[skip] __: B6,
    /// Time Base Counter Enable
    tbcen: bool,
    /// Time Stamp EOF
    tseof: bool,
    /// Time Stamp res bit (FD frames only)
    tsres: bool,
    #[skip] __: B13,
}
impl Register for TimeStampControl {
    const ADDRESS: u16 = 0x014;
}

/// Interrupt Flag
/// If multiple interrupts are pending, the interrupt with the highest number will be indicated
#[derive(BitfieldSpecifier, Copy, Clone, Debug)]
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

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct InterruptCode {
    /// Interrupt Flag Code
    #[skip(setters)]
    icode: InterruptFlag,
    #[skip] __: B1,
    /// Filter Hit Number
    #[skip(setters)]
    filhit: B5,
    #[skip] __: B3,
    /// Transmit Interrupt Flag Code
    #[skip(setters)]
    txcode: InterruptFlag,
    #[skip] __: B1,
    /// Receive Interrupt Flag Code
    #[skip(setters)]
    rxcode: InterruptFlag,
    #[skip] __: B1,
}
impl Register for InterruptCode {
    const ADDRESS: u16 = 0x018;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Interrupts {
    /// Transmit FIFO Interrupt Flag
    #[skip(setters)]
    txif: bool,
    /// Receive FIFO Interrupt Flag
    #[skip(setters)]
    rxif: bool,
    /// Time Base Counter Overflow Interrupt Flag
    tbcif: bool,
    /// Operation Mode Change Interrupt Flag
    modif: bool,
    /// Transmit Event FIFO Interrupt Flag
    #[skip(setters)]
    tefif: bool,
    #[skip] __: B3,
    /// ECC Error Interrupt Flag
    #[skip(setters)]
    eccif: bool,
    /// SPI CRC Error Interrupt Flag
    #[skip(setters)]
    spicrcif: bool,
    /// Transmit Attempt Interrupt Flag
    #[skip(setters)]
    txatif: bool,
    /// Receive Object Overflow Interrupt Flag
    #[skip(setters)]
    rxovif: bool,
    /// System Error Interrupt Flag
    serrif: bool,
    /// CAN Bus Error Interrupt Flag
    cerrif: bool,
    /// Bus Wake Up Interrupt Flag
    wakif: bool,
    /// Invalid Message Interrupt Flag
    ivmif: bool,
    /// Transmit FIFO Interrupt Enable
    txie: bool,
    /// Receive FIFO Interrupt Enable
    rxie: bool,
    /// Time Base Counter Interrupt Enable
    tbcie: bool,
    /// Mode Change Interrupt Enable
    modie: bool,
    /// Transmit Event FIFO Interrupt Enable
    tefie: bool,
    #[skip] __: B3,
    /// ECC Error Interrupt Enable
    eccie: bool,
    /// SPI CRC Error Interrupt Enable
    spicrcie: bool,
    /// Transmit Attempt Interrupt Enable
    txatie: bool,
    /// Receive FIFO Overflow Interrupt Enable
    rxovie: bool,
    /// System Error Interrupt Enable
    serrie: bool,
    /// CAN Bus Error Interrupt Enable
    cerrie: bool,
    /// Bus Wake Up Interrupt Enable
    wakeie: bool,
    /// Invalid Message Interrupt Enable
    ivmie: bool,
}
impl Register for Interrupts {
    const ADDRESS: u16 = 0x01C;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ReceiveInterruptStatus {
    #[skip] __: B1,
    /// Receive FIFO Interrupt Pending
    /// 'or’ of enabled RXFIFO flags; flags will be cleared when the condition of the FIFO terminates
    #[skip(setters)]
    rfif: B31,
}
impl Register for ReceiveInterruptStatus {
    const ADDRESS: u16 = 0x020;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ReceiveOverflowInterruptStatus {
    #[skip] __: B1,
    /// Receive FIFO Overflow Interrupt Pending
    #[skip(setters)]
    rfovif: B31,
}
impl Register for ReceiveOverflowInterruptStatus {
    const ADDRESS: u16 = 0x028;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitInterruptStatus {
    /// TXQ Interrupt Pending
    #[skip(setters)]
    txqif: B1,
    /// Transmit FIFO Interrupt Pending
    /// 'or’ of enabled TXFIFO flags; flags will be cleared when the condition of the FIFO terminates
    #[skip(setters)]
    tfif: B31,
}
impl Register for TransmitInterruptStatus {
    const ADDRESS: u16 = 0x024;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitAttemptInterruptStatus {
    /// TXQ Attempt Interrupt Pending
    #[skip(setters)]
    txqatif: B1,
    /// Transmit FIFO Attempt Interrupt Pending
    /// 'or’ of enabled TXFIFO flags; flags will be cleared when the condition of the FIFO terminates
    #[skip(setters)]
    tfatif: B31,
}
impl Register for TransmitAttemptInterruptStatus {
    const ADDRESS: u16 = 0x02C;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitRequest {
    /// Transmit Queue Message Send Request
    /// Will automatically clear when the message(s) queued is/are successfully sent
    #[skip(setters)]
    txqreq: bool,
    /// Transmit FIFO Message Send Request
    /// Will automatically clear when the message(s) queued is/are successfully sent
    #[skip(setters)]
    txreq: B31,
}
impl Register for TransmitRequest {
    const ADDRESS: u16 = 0x030;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitReceiveErrorCount {
    /// Receive Error Counter
    #[skip(setters)]
    rec: u8,
    /// Transmit Error Counter
    #[skip(setters)]
    tec: u8,
    /// Transmitter or Receiver is in Error Warning State
    #[skip(setters)]
    ewarn: bool,
    /// Receiver in Error Warning State (128 > REC > 95)
    #[skip(setters)]
    rxwarn: bool,
    /// Transmitter in Error Warning State (128 > TEC > 95
    #[skip(setters)]
    txwarn: bool,
    /// Receiver in Error Passive State (REC > 127
    #[skip(setters)]
    rxbp: bool,
    /// Transmitter in Error Passive State (TEC > 127
    #[skip(setters)]
    txbp: bool,
    /// Transmitter in Bus Off State bit (TEC > 255)
    #[skip(setters)]
    txbo: bool,
    #[skip] __: B10,
}
impl Register for TransmitReceiveErrorCount {
    const ADDRESS: u16 = 0x034;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct BusDiagnostic0 {
    /// Nominal Bit Rate Receive Error Counter
    nrerrcnt: u8,
    /// Nominal Bit Rate Transmit Error Counter
    nterrcnt: u8,
    /// Data Bit Rate Receive Error Counter
    drerrcnt: u8,
    /// Data Bit Rate Transmit Error Counter
    dterrcnt: u8,
}
impl Register for BusDiagnostic0 {
    const ADDRESS: u16 = 0x038;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct BusDiagnostic1 {
    /// Error-free Message Counter
    efmsgcnt: u16,
    /// During the transmission of a message (or acknowledge bit, or active error flag, or overload
    /// flag), the device wanted to send a dominant level (data or identifier bit logical value ‘0’), but the
    /// monitored bus value was recessive.
    nbit0err: bool,
    /// During the transmission of a message (with the exception of the arbitration field), the
    /// device wanted to send a recessive level (bit of logical value ‘1’), but the monitored bus value was
    /// dominant.
    nbit1err: bool,
    /// Transmitted message was not acknowledged
    nackerr: bool,
    /// A fixed format part of a received frame has the wrong format
    nformerr: bool,
    /// More than 5 equal bits in a sequence have occurred in a part of a received message
    /// where this is not allowed.
    nstuferr: bool,
    /// The CRC check sum of a received message was incorrect. The CRC of an incoming
    /// message does not match with the CRC calculated from the received data
    ncrcerr: bool,
    #[skip] __: B1,
    /// Device went to bus-off (and auto-recovered)
    txboerr: bool,
    dbit0err: bool,
    dbit1err: bool,
    #[skip] __: B1,
    dformerr: bool,
    dstuferr: bool,
    dcrcerr: bool,
    /// ESI flag of a received CAN FD message was set
    esi: bool,
    /// DLC Mismatch
    /// During a transmission or reception, the specified DLC is larger than the PLSIZE of the FIFO element
    dlcmm: bool,
}
impl Register for BusDiagnostic1 {
    const ADDRESS: u16 = 0x03C;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitEventFIFOControl {
    /// Transmit Event FIFO Not Empty Interrupt Enable
    tefneie: bool,
    /// Transmit Event FIFO Half Full Interrupt Enable
    tefhie: bool,
    /// Transmit Event FIFO Full Interrupt Enable
    teffie: bool,
    /// Transmit Event FIFO Overflow Interrupt Enable
    tefovie: bool,
    #[skip] __: B1,
    /// Transmit Event FIFO Time Stamp Enable
    teftsen: bool,
    #[skip] __: B2,
    /// Increment Tail
    /// When this bit is set, the FIFO tail will increment by a single message
    uinc: bool,
    #[skip] __: B1,
    /// FIFO will reset -- wait to clear before taking action
    freset: bool,
    #[skip] __: B13,
    /// FIFO Size
    /// Begins at 0b0000 = 1
    fsize: B5,
    #[skip] __: B3,
}
impl Register for TransmitEventFIFOControl {
    const ADDRESS: u16 = 0x040;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitEventFIFOStatus {
    /// Transmit Event FIFO Not Empty Interrupt Flag
    #[skip(setters)]
    tefneif: bool,
    /// Transmit Event FIFO Half Full Interrupt Flag
    #[skip(setters)]
    tefhif: bool,
    /// Transmit Event FIFO Full Interrupt Flag
    #[skip(setters)]
    teffif: bool,
    /// Transmit Event FIFO Overflow Interrupt Flag
    #[skip(setters)]
    tefovif: bool,
    #[skip] __: B28,
}
impl Register for TransmitEventFIFOStatus {
    const ADDRESS: u16 = 0x044;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitEventFIFOUserAddress {
    /// Transmit Event FIFO User Address
    /// The address where the next object is to be read (FIFO tail)
    #[skip(setters)]
    tefua: u32,
}
impl Register for TransmitEventFIFOUserAddress {
    const ADDRESS: u16 = 0x048;
}

#[derive(BitfieldSpecifier, Copy, Clone, Debug)]
#[bits = 2]
pub enum RetransmissionAttempts {
    Disable = 0b00,
    Three = 0b01,
    Unlimited1 = 0b10,
    Unlimited2 = 0b11,
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitQueueControl {
    /// Transmit Queue Not Full Interrupt Enable
    txqnie: bool,
    #[skip] __: B1,
    /// Transmit Queue Empty Interrupt Enable
    txqeie: bool,
    #[skip] __: B1,
    /// Transmit Attempts Exhausted Interrupt Enable
    txatie: bool,
    #[skip] __: B2,
    /// TX Enable (always true)
    txen: bool,
    /// Increment Head
    uinc: bool,
    /// Message Send Request
    txreq: bool,
    /// FIFO Reset
    freset: bool,
    #[skip] __: B5,
    /// Message Transmit Priority
    /// 0 = lowest, 31 = highest
    txpri: B5,
    /// Retransmission Attempts
    txat: RetransmissionAttempts,
    #[skip] __: B1,
    /// FIFO Size
    /// Begins at 0b0000 = 1
    fsize: B5,
    /// Payload Size
    /// 0b000 = 8, 0b111 = 64
    plsize: B3,
}
impl Register for TransmitQueueControl {
    const ADDRESS: u16 = 0x050;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitQueueStatus {
    /// Transmit Queue Not Full Interrupt Flag
    #[skip(setters)]
    txqnif: bool,
    #[skip] __: B1,
    /// Transmit Queue Empty Interrupt Flag
    #[skip(setters)]
    txqeif: bool,
    #[skip] __: B1,
    /// Transmit Attempts Exhausted Interrupt Flag
    txatif: bool,
    /// Error Detected During Transmission
    txerr: bool,
    /// Message Lost Arbitration Status
    txlarb: bool,
    /// Message Aborted Status
    txabt: bool,
    /// Transmit Queue Message Index
    /// Index to the message that the FIFO will next attempt to transmit
    #[skip(setters)]
    txqci: B5,
    #[skip] __: B19,
}
impl Register for TransmitQueueStatus {
    const ADDRESS: u16 = 0x054;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TransmitQueueUserAddress {
    /// TXQ User Address
    /// The address where the next message is to be written (TXQ head)
    #[skip(setters)]
    txqua: u32,
}
impl Register for TransmitQueueUserAddress {
    const ADDRESS: u16 = 0x058;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct FIFOControl {
    /// Transmit/Receive FIFO Not Full/Not Empty Interrupt Enable
    tfnrfnie: bool,
    /// Transmit/Receive FIFO Half Empty/Half Full Interrupt Enable
    tfhrfhie: bool,
    /// Transmit/Receive FIFO Empty/Full Interrupt Enable
    tferffie: bool,
    /// Overflow Interrupt Enable
    rxovie: bool,
    /// Transmit Attempts Exhausted Interrupt Enable
    txatie: bool,
    /// Received Message Time Stamp Enable
    rxtsen: bool,
    /// Auto RTR Enable
    rtren: bool,
    /// TX/RX FIFO Selection
    /// true = Transmit, false = Receive
    txen: bool,
    /// Increment Head/Tail
    uinc: bool,
    /// Message Send Request
    rxreq: bool,
    /// FIFO Reset
    freset: bool,
    #[skip] __: B5,
    /// Message Transmit Priority
    txpri: B5,
    /// Retransmission Attempts
    txat: RetransmissionAttempts,
    #[skip] __: B1,
    /// FIFO Size
    fsize: B5,
    /// Payload Size
    plsize: B3,
}
impl Register for FIFOControl {
    const ADDRESS: u16 = 0x05C;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct FIFOStatus {
    /// Transmit/Receive FIFO Not Full/Not Empty Interrupt Flag
    #[skip(setters)]
    tfnrfnif: bool,
    /// Transmit/Receive FIFO Half Empty/Half Full Interrupt Flag
    #[skip(setters)]
    tfhrfhif: bool,
    /// Transmit/Receive FIFO Empty/Full Interrupt Flag
    #[skip(setters)]
    tferffif: bool,
    /// Receive FIFO Overflow Interrupt Flag
    rxovif: bool,
    /// Transmit Attempts Exhausted Interrupt Pending
    txatif: bool,
    /// Error Detected During Transmission
    txerr: bool,
    /// Message Lost Arbitration Status
    txlarb: bool,
    /// Message Aborted Status
    txabt: bool,
    /// FIFO Message Index
    #[skip(setters)]
    fifoci: B5,
    #[skip] __: B19,
}
impl Register for FIFOStatus {
    const ADDRESS: u16 = 0x060;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct FIFOUserAddress {
    /// FIFO User Address
    /// The address where the next message is to be written (FIFO head)
    /// The address where the next message is to be read (FIFO tail)
    #[skip(setters)]
    fifoua: u32,
}
impl Register for FIFOUserAddress {
    const ADDRESS: u16 = 0x064;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct FilterControl {
    /// Pointer to FIFO when Filter 0 hits
    f0bp: B5,
    #[skip] __: B2,
    /// Enable Filter 0 to Accept Messages
    flten0: bool,
    /// Pointer to FIFO when Filter 1 hits
    f1bp: B5,
    #[skip] __: B2,
    /// Enable Filter 1 to Accept Messages
    flten1: bool,
    /// Pointer to FIFO when Filter 2 hits
    f2bp: B5,
    #[skip] __: B2,
    /// Enable Filter 2 to Accept Messages
    flten2: bool,
    /// Pointer to FIFO when Filter 3 hits
    f3bp: B5,
    #[skip] __: B2,
    /// Enable Filter 3 to Accept Messages
    flten3: bool,
}
impl Register for FilterControl {
    const ADDRESS: u16 = 0x1D0;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct FilterObject {
    /// Standard Identifier filter
    sid: B11,
    /// Extended Identifier bits
    eid: B18,
    /// Standard Identifier filter
    sid11: bool,
    /// Extended Identifier enable
    exide: bool,
    #[skip] __: B1,
}
impl Register for FilterObject {
    const ADDRESS: u16 = 0x1F0;
}

#[bitfield]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Mask {
    /// Standard Identifier mask
    msid: B11,
    /// Extended Identifier mask
    meid: B18,
    /// Standard Identifier mask
    msid11: bool,
    /// Identifier Receive mode bit
    mide: bool,
}
impl Register for Mask {
    const ADDRESS: u16 = 0x1F4;
}