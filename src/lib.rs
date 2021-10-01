#![no_std]

use core::convert::Infallible;
use core::fmt::Debug;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use crate::registers::*;
use embedded_can::Frame;

pub mod bitrates;
pub mod frame;
pub mod registers;

#[repr(u8)]
pub enum Instruction {
    /// Resets internal registers to the default state, sets Configuration mode.
    Reset = 0b1100_0000,
    /// Reads data from the register beginning at the selected address.
    Read = 0b0000_0011,
    /// Writes data to the register beginning at the selected address.
    Write = 0b0000_0010,
    /// Instructs the controller to begin the message transmission sequence for
    /// any of the transmit buffers specified in `0b1000_0nnn`.
    Rts = 0b1000_0000,
    /// Quick polling command that reads several Status bits for transmit and receive functions.
    ReadStatus = 0b1010_0000,
    /// Allows the user to set or clear individual bits in a particular register.
    ///
    /// Note: Not all registers can be bit modified with this command.
    /// Executing this command on registers that are not bit modifiable will force the mask to FFh.
    ///
    /// Registers that can be modified with this command implement [`Modify`].
    BitModify = 0b0000_0101,

    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    /// Quick polling command that indicates a filter match and message type
    /// (standard, extended and/or remote) of the received message.
    RxStatus = 0b1011_0000,
    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    /// When reading a receive buffer, reduces the overhead of a normal `Read`
    /// command by placing the Address Pointer at one of four locations, as
    /// indicated by ‘nm’ in `0b1001_0nm0`.
    ///
    /// Note: The associated RX flag bit (`rxNif` bits in the [`CANINTF`] register) will be cleared after bringing CS high.
    ReadRxBuffer = 0b1001_0000,
    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    /// When loading a transmit buffer, reduces the overhead of a normal `Write`
    /// command by placing the Address Pointer at one of six locations, as
    /// indicated by ‘abc’ in `0b0100_0abc`.
    LoadTxBuffer = 0b0100_0000,
}

#[derive(Copy, Clone)]
pub enum TxBufferIndex {
    Idx0 = 0,
    Idx1 = 1,
    Idx2 = 2,
}

#[derive(Copy, Clone)]
pub enum RxBufferIndex {
    Idx0 = 0,
    Idx1 = 1,
}

pub struct MCP25xx<SPI, CS>
where
    SPI: Transfer<u8>,
    SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    <SPI as Transfer<u8>>::Error: Debug,
    CS: OutputPin<Error = Infallible>,
{
    pub spi: SPI,
    pub cs: CS,
}

impl<SPI, CS> MCP25xx<SPI, CS>
where
    SPI: Transfer<u8>,
    SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    <SPI as Transfer<u8>>::Error: Debug,
    CS: OutputPin<Error = Infallible>,
{
    pub fn set_mode(&mut self, mode: REQOP) -> Result<(), <SPI as Transfer<u8>>::Error> {
        let reg = CANCTRL::new().with_reqop(mode);
        self.modify_register(reg, 0b11100000)
    }
    pub fn set_bitrate(&mut self, cnf: CNF) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.write_registers(CNF::ADDRESS, &cnf.into_bytes())
    }
}

impl<SPI, CS> embedded_can::Can for MCP25xx<SPI, CS>
where
    SPI: Transfer<u8>,
    SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    <SPI as Transfer<u8>>::Error: Debug,
    CS: OutputPin<Error = Infallible>,
{
    type Frame = crate::frame::Frame;
    type Error = <SPI as Transfer<u8>>::Error;

    fn try_transmit(
        &mut self,
        frame: &Self::Frame,
    ) -> nb::Result<Option<Self::Frame>, Self::Error> {
        let status = self.read_status()?;
        let mut buf_idx = TxBufferIndex::Idx0;
        if status.txreq0() {
            buf_idx = TxBufferIndex::Idx1;
            if status.txreq1() {
                buf_idx = TxBufferIndex::Idx2;
                if status.txreq2() {
                    // TODO replace a pending lower priority frame
                    return Err(nb::Error::WouldBlock);
                }
            }
        }

        let registers = &frame.as_bytes()[0..5 + frame.dlc()];
        self.load_tx_buffer(buf_idx, registers)?;
        self.request_to_send(buf_idx)?;
        Ok(None)
    }

    fn try_receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        // TODO look at https://www.microchip.com/forums/tm.aspx?m=620741
        let status = self.read_status()?;
        if status.rx0if() {
            Ok(self.read_rx_buffer(RxBufferIndex::Idx0)?)
        } else if status.rx1if() {
            Ok(self.read_rx_buffer(RxBufferIndex::Idx1)?)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<SPI, CS> MCP25xx<SPI, CS>
where
    SPI: Transfer<u8>,
    SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    <SPI as Transfer<u8>>::Error: Debug,
    CS: OutputPin<Error = Infallible>,
{
    pub fn read_register<R: Register + From<u8>>(
        &mut self,
    ) -> Result<R, <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[Instruction::Read as u8, R::ADDRESS])?;
        let mut reg = [0];
        self.spi.transfer(&mut reg)?;
        self.cs.set_high().ok();
        Ok(reg[0].into())
    }

    pub fn write_register<R: Register + Into<u8>>(
        &mut self,
        reg: R,
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi
            .write(&[Instruction::Write as u8, R::ADDRESS, reg.into()])?;
        self.cs.set_high().ok();
        Ok(())
    }

    pub fn modify_register<R: Register + Modify + Into<u8>>(
        &mut self,
        reg: R,
        mask: u8,
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi
            .write(&[Instruction::BitModify as u8, R::ADDRESS, mask, reg.into()])?;
        self.cs.set_high().ok();
        Ok(())
    }

    pub fn read_registers(
        &mut self,
        start_address: u8,
        buf: &mut [u8],
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[Instruction::Read as u8, start_address])?;
        self.spi.transfer(buf)?;
        self.cs.set_high().ok();
        Ok(())
    }

    pub fn write_registers(
        &mut self,
        start_address: u8,
        data: &[u8],
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[Instruction::Write as u8, start_address])?;
        self.spi.write(data)?;
        self.cs.set_high().ok();
        Ok(())
    }

    pub fn request_to_send(
        &mut self,
        buf_idx: TxBufferIndex,
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi
            .write(&[Instruction::Rts as u8 | (1 << buf_idx as u8)])?;
        self.cs.set_high().ok();
        Ok(())
    }

    pub fn read_status(&mut self) -> Result<ReadStatusResponse, <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[Instruction::ReadStatus as u8])?;
        let mut buf = [0];
        self.spi.transfer(&mut buf)?;
        self.cs.set_high().ok();
        Ok(ReadStatusResponse::from_bytes(buf))
    }

    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    pub fn load_tx_buffer(
        &mut self,
        buf_idx: TxBufferIndex,
        data: &[u8],
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.cs.set_low().ok();
        self.spi
            .write(&[Instruction::LoadTxBuffer as u8 | (buf_idx as u8 * 2)])?;
        self.spi.write(data)?;
        self.cs.set_high().ok();
        Ok(())
    }

    #[cfg(not(any(feature = "mcp2515", feature = "mcp25625")))]
    #[inline]
    pub fn load_tx_buffer(
        &mut self,
        buf_idx: TxBufferIndex,
        data: &[u8],
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.write_registers(0x31 + 0x10 * buf_idx as u8, data)
    }

    pub fn read_rx_buffer(
        &mut self,
        buf_idx: RxBufferIndex,
    ) -> Result<crate::frame::Frame, <SPI as Transfer<u8>>::Error> {
        // gets a view into the first 5 bytes of Frame
        fn id_bytes(frame: &mut crate::frame::Frame) -> &mut [u8; 5] {
            // SAFETY:
            // Frame is [repr(C)] without any padding bytes
            // All bit patterns are valid
            unsafe { &mut *(frame as *mut crate::frame::Frame as *mut [u8; 5]) }
        }

        let mut frame = crate::frame::Frame::default();

        self.cs.set_low().ok();

        self.send_read_rx_instruction(buf_idx)?;
        self.spi.transfer(id_bytes(&mut frame))?;
        let mut dlc = frame.dlc();
        if dlc > 8 {
            dlc = 8;
            frame.dlc.set_dlc(8);
        }
        self.spi.transfer(&mut frame.data[0..dlc])?;

        self.cs.set_high().ok();

        #[cfg(not(any(feature = "mcp2515", feature = "mcp25625")))]
        // need to manually reset the interrupt flag bit if Instruction::ReadRxBuffer is not available
        self.modify_register(CANINTF::new(), buf_idx as u8)?;
        Ok(frame)
    }

    #[cfg(any(feature = "mcp2515", feature = "mcp25625"))]
    fn send_read_rx_instruction(
        &mut self,
        buf_idx: RxBufferIndex,
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.spi
            .write(&[Instruction::ReadRxBuffer as u8 | (buf_idx as u8 * 2)])
    }

    #[cfg(not(any(feature = "mcp2515", feature = "mcp25625")))]
    fn send_read_rx_instruction(
        &mut self,
        buf_idx: RxBufferIndex,
    ) -> Result<(), <SPI as Transfer<u8>>::Error> {
        self.spi
            .write(&[Instruction::Read as u8, 0x61 + 0x10 * buf_idx as u8])
    }
}
