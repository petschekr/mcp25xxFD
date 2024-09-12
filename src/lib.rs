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

    pub async fn reset(&mut self) -> Result<(), ()> {
        let tx = Instruction::Reset.header(0x00);
        self.spi.write(&tx).await.unwrap();

        Ok(())
    }

    /// Read a single register
    pub async fn read_register<R: Register>(&mut self) -> Result<R::Bitfield, ()> {
        let tx = Instruction::Read.header(R::ADDRESS);
        let mut rx = [0u8; 6];
        self.spi.transfer(&mut rx, &tx).await.unwrap(); // TODO: SPI error handling

        Ok(R::parse(&rx[2..]))
    }

    /// Write a single register
    pub async fn write_register<R: Register<Bitfield = R>>(&mut self, register: R) -> Result<(), ()> {
        let mut tx = [0u8; 6];
        {
            let (header, data) = tx.split_at_mut(2);
            header.copy_from_slice(&Instruction::Write.header(R::ADDRESS));
            data.copy_from_slice(&R::serialize(register));
        }
        self.spi.write(&tx).await.unwrap(); // TODO: SPI error handling

        Ok(())
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