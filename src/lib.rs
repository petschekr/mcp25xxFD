#![no_std]
use core::fmt::Debug;
use embedded_hal_async::spi::SpiDevice;

use crate::registers::*;

/// Register bitfields
pub mod registers;

/// Either a MCP2517, MCP2518 or MCP251863 CAN-FD controller
pub struct MCP25xxFD<SPI> {
    spi: SPI,
}

impl<SPI: SpiDevice> MCP25xxFD<SPI> {
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
        }
    }

    /// Read a single register
    pub async fn read_register<R: Register>(&mut self) -> Result<R, ()>
        where R: modular_bitfield::Specifier<Bytes = u32, InOut = R>,
    {
        let tx = Instruction::Read.header(R::get_address());
        let mut rx = [0u8; 6];
        self.spi.transfer(&mut rx, &tx).await.unwrap();

        let data = u32::from_le_bytes(rx[2..].try_into().unwrap());
        Ok(R::from_bytes(data).unwrap())
    }

    /// Write a single register
    pub async fn write_register<R: Register + Into<u32>>(&mut self) -> () {

    }
}

/// SPI instructions supported by the CAN controller
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Instruction {
    /// Resets internal registers to the default state, sets Configuration mode.
    Reset = 0b0000,
    /// Reads SFR/RAM from address A
    Read = 0b0011,
    /// Write SFR/RAM to address A
    Write = 0b0010,
    /// Read SFR/RAM from address A. N data bytes. Two bytes CRC.
    /// CRC is calculated on C, A, N and D.
    ReadCRC = 0b1011,
    /// Write SFR/RAM to address A. N data bytes. Two bytes CRC.
    /// CRC is calculated on C, A, N and D.
    WriteCRC = 0b1010,
    /// Write SFR/RAM to address A. Check CRC before write. CRC is calculated on C, A, and D.
    WriteSafe = 0b1100,
}
impl Instruction {
    fn header(&self, address: u16) -> [u8; 2] {
        let instruction: u8 = (*self) as u8;
        let mut header = [0; 2];
        header[0] = (instruction << 4) + ((address >> 8) & 0xF) as u8;
        header[1] = (address & 0xFF) as u8;
        header
    }
}