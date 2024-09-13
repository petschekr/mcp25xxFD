#![no_std]
use core::fmt::Debug;
use embedded_hal_async::spi::{ SpiDevice, Operation };
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
    pub async fn read_register<R: Register>(&mut self) -> Result<R, ()> {
        let tx = Instruction::Read.header(R::ADDRESS);
        let mut rx = [0u8; 6];
        self.spi.transfer(&mut rx, &tx).await.unwrap(); // TODO: SPI error handling

        Ok(R::parse(&rx[2..]))
    }

    /// Write a single register
    pub async fn write_register<R: Register>(&mut self, register: R) -> Result<(), ()> {
        self.spi.transaction(&mut [
            Operation::Write(&Instruction::Write.header(R::ADDRESS)),
            Operation::Write(&R::serialize(register)),
        ]).await.unwrap(); // TODO: SPI error handling

        Ok(())
    }

    pub async fn read_bytes<const B: usize>(&mut self, address: u16) -> Result<[u8; B], ()> {
        let tx = Instruction::Read.header(address);
        let mut rx = [0u8; B];
        self.spi.transaction(&mut [
            Operation::Write(&tx),
            Operation::Read(&mut rx),
        ]).await.unwrap(); // TODO: SPI error handling

        Ok(rx)
    }

    pub async fn write_bytes(&mut self, address: u16, data: &[u8]) -> Result<(), ()> {
        const RAM_START: u16 = 0x400;
        self.spi.transaction(&mut [
            Operation::Write(&Instruction::Write.header(RAM_START + address)),
            Operation::Write(data),
        ]).await.unwrap(); // TODO: SPI error handling

        Ok(())
    }

    pub async fn initialize_ram(&mut self, data: u8) -> Result<(), ()> {
        const RAM_SIZE: u16 = 2048;
        const INIT_INCREMENT: usize = 64; // Write 64 bytes at a time
        let bytes = [data; INIT_INCREMENT];

        for addr in (0..RAM_SIZE).step_by(INIT_INCREMENT) {
            self.write_bytes(addr, &bytes).await?;
        }
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