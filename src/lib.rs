#![no_std]

use core::fmt::{Debug, Display, Formatter};
use embedded_can::Id;
use embedded_hal_async::spi::{SpiDevice, Operation };
use embedded_hal_async::digital::Wait;
use crate::config::{Config, FIFOConfig, FilterConfig, MaskConfig};
use crate::frame::Frame;
use crate::registers::*;

/// Register bitfields
pub mod registers;
pub mod config;
pub mod frame;

const RAM_START: u16 = 0x400;
const RAM_SIZE: u16 = 2048;

/// Either a MCP2517, MCP2518 or MCP251863 CAN-FD controller
pub struct MCP25xxFD<SPI, Input> {
    spi: SPI,
    interrupt: Input,
}

impl<SPI: SpiDevice, Input: Wait> MCP25xxFD<SPI, Input> {
    pub fn new(spi: SPI, interrupt: Input) -> Self {
        Self {
            spi,
            interrupt,
        }
    }

    pub async fn reset_and_apply_config(&mut self, config: &Config) -> Result<(), Error<SPI, Input>> {
        self.reset().await?;

        let mut ecc_register: ECCControl = self.read_register().await?;
        ecc_register.set_eccen(config.ecc_enabled);
        self.write_register(ecc_register).await?;

        self.initialize_ram(0xFF).await?;

        let mut can_config: CANControl = self.read_register().await?;
        can_config.set_isocrcen(config.iso_crc_enabled);
        can_config.set_stef(config.tx_event_fifo_enabled);
        can_config.set_txqen(config.txq_enabled);
        can_config.set_rtxat(config.restrict_retx_attempts);
        self.write_register(can_config).await?;

        let bitrate_config = config.bit_rate.get_config(&config.clock).expect("Invalid bit rate pair for system clock");

        let mut nominal_bit_time_config = NominalBitTimeConfig::new();
        nominal_bit_time_config.set_brp(bitrate_config.arbitration_brp);
        nominal_bit_time_config.set_tseg1(bitrate_config.arbitration_tseg1);
        nominal_bit_time_config.set_tseg2(bitrate_config.arbitration_tseg2);
        nominal_bit_time_config.set_sjw(bitrate_config.arbitration_sjw);
        self.write_register(nominal_bit_time_config).await?;

        let mut data_bit_time_config = DataBitTimeConfig::new();
        data_bit_time_config.set_brp(bitrate_config.data_brp);
        data_bit_time_config.set_tseg1(bitrate_config.data_tseg1);
        data_bit_time_config.set_tseg2(bitrate_config.data_tseg2);
        data_bit_time_config.set_sjw(bitrate_config.data_sjw);
        self.write_register(data_bit_time_config).await?;

        let mut tx_delay_compensation: TransmitterDelayCompensation = self.read_register().await?;
        tx_delay_compensation.set_tdcmod(bitrate_config.tdc_mode);
        tx_delay_compensation.set_tdco(bitrate_config.tdc_offset);
        tx_delay_compensation.set_tdcv(bitrate_config.tdc_value);
        self.write_register(tx_delay_compensation).await?;

        // Setup interrupts
        let mut interrupt_config: Interrupts = self.read_register().await?;
        interrupt_config.set_txie(false);
        interrupt_config.set_rxie(true);
        interrupt_config.set_cerrie(true);
        self.write_register(interrupt_config).await?;

        Ok(())
    }

    pub async fn configure_fifo<const M: u8>(&mut self, fifo: FIFOConfig<M>) -> Result<(), Error<SPI, Input>> {
        let mut fifo_control = FIFOControl::<M>::from_bitfield(FIFOControlM::new());
        fifo_control.contents.set_fsize(fifo.size - 1); // FSIZE of 0 is 1 message deep
        fifo_control.contents.set_plsize(fifo.payload_size);
        fifo_control.contents.set_txen(fifo.transmit);
        fifo_control.contents.set_txat(fifo.tx_attempts);
        fifo_control.contents.set_txpri(fifo.priority);
        fifo_control.contents.set_freset(true);
        if !fifo.transmit {
            fifo_control.contents.set_tfnrfnie(true); // Interrupt for RX FIFO not empty
        }
        self.write_register(fifo_control).await?;
        Ok(())
    }

    pub async fn configure_filter<const M: u8, const RXFIFO: u8>(&mut self, filter: FilterConfig<M, RXFIFO>, mask: MaskConfig<M>) -> Result<(), Error<SPI, Input>> {
        // Set up the filter configuration
        let mut filter_object = FilterObject::<M>::from_bitfield(FilterObjectM::new());
        filter_object.contents.set_exide(filter.match_only_extended);
        match filter.id {
            Id::Standard(id) => {
                filter_object.contents.set_sid(id.as_raw());
            },
            Id::Extended(id) => {
                let sid_component = id.as_raw() & 0x7FF;
                let eid_component = id.as_raw() >> 11;
                filter_object.contents.set_sid(sid_component as u16);
                filter_object.contents.set_eid(eid_component);
            },
        };
        self.write_register(filter_object).await?;

        // Set the mask
        let mut mask_config = Mask::<M>::from_bitfield(MaskM::new());
        mask_config.contents.set_mide(mask.match_id_type);
        match mask.id {
            Id::Standard(id) => {
                mask_config.contents.set_msid(id.as_raw());
            },
            Id::Extended(id) => {
                let sid_component = id.as_raw() & 0x7FF;
                let eid_component = id.as_raw() >> 11;
                mask_config.contents.set_msid(sid_component as u16);
                mask_config.contents.set_meid(eid_component);
            },
        };
        self.write_register(mask_config).await?;

        // Enable the filter
        let filter_control_address = FilterControl::<0>::ADDRESS + (M as u16);
        self.write_register_byte(filter_control_address,(1 << 7) | RXFIFO).await?;

        Ok(())
    }

    /// Request the controller transition to the specified mode
    pub async fn set_mode(&mut self, mode: OperationMode) -> Result<(), Error<SPI, Input>> {
        let mut can_config: CANControl = self.read_register().await?;
        can_config.set_reqop(mode);
        self.write_register(can_config).await
    }

    /// Resets the controller and places it back into Configuration Mode
    pub async fn reset(&mut self) -> Result<(), Error<SPI, Input>> {
        let tx = Instruction::Reset.header(0x00);
        self.spi.write(&tx).await.map_err(Error::SPIError)?;
        Ok(())
    }

    /// Read a single register
    pub async fn read_register<R: Register>(&mut self) -> Result<R, Error<SPI, Input>> {
        let tx = Instruction::Read.header(R::ADDRESS);
        let mut rx = [0u8; 6];
        self.spi.transfer(&mut rx, &tx).await.map_err(Error::SPIError)?;

        Ok(R::parse(&rx[2..]))
    }

    /// Write a single register
    pub async fn write_register<R: Register>(&mut self, register: R) -> Result<(), Error<SPI, Input>> {
        self.spi.transaction(&mut [
            Operation::Write(&Instruction::Write.header(R::ADDRESS)),
            Operation::Write(&R::serialize(register)),
        ]).await.map_err(Error::SPIError)?;

        Ok(())
    }

    pub async fn read_bytes<const B: usize>(&mut self, address: u16) -> Result<[u8; B], Error<SPI, Input>> {
        assert_eq!(B % 4, 0, "Must read in multiples of 4 data bytes");
        let tx = Instruction::Read.header(RAM_START + address);
        let mut rx = [0u8; B];

        self.spi.transaction(&mut [
            Operation::Write(&tx),
            Operation::Read(&mut rx),
        ]).await.map_err(Error::SPIError)?;

        Ok(rx)
    }

    pub async fn write_register_byte(&mut self, address: u16, data: u8) -> Result<(), Error<SPI, Input>> {
        let tx = Instruction::Write.header(address);

        self.spi.transaction(&mut [
            Operation::Write(&tx),
            Operation::Write(&[data]),
        ]).await.map_err(Error::SPIError)?;

        Ok(())
    }

    pub async fn write_bytes(&mut self, address: u16, data: &[u8]) -> Result<(), Error<SPI, Input>> {
        assert_eq!(data.len() % 4, 0, "Must write in multiples of 4 data bytes");
        let tx = Instruction::Write.header(RAM_START + address);

        self.spi.transaction(&mut [
            Operation::Write(&tx),
            Operation::Write(data),
        ]).await.map_err(Error::SPIError)?;

        Ok(())
    }

    pub async fn initialize_ram(&mut self, data: u8) -> Result<(), Error<SPI, Input>> {
        const INIT_INCREMENT: usize = 64; // Write 64 bytes at a time
        let bytes = [data; INIT_INCREMENT];

        for addr in (0..RAM_SIZE).step_by(INIT_INCREMENT) {
            self.write_bytes(addr, &bytes).await?;
        }
        Ok(())
    }

    pub async fn transmit<const M: u8>(&mut self, frame: &Frame) -> Result<(), Error<SPI, Input>> {
        // Check FIFO availability
        let tx_status: FIFOStatus<M> = self.read_register().await?;
        if !tx_status.contents.tfnrfnif() {
            return Err(Error::ControllerError("No room in TX FIFO!"));
        }

        let (header, data) = frame.as_components();

        let tx_addr = self.read_register::<FIFOUserAddress<M>>().await?
            .contents
            .fifoua() as u16;

        self.write_bytes(tx_addr, &header.into_bytes()).await?;
        self.write_bytes(tx_addr + size_of::<TransmitMessageObjectHeader>() as u16, data).await?;

        let mut tx_control: FIFOControl<M> = self.read_register().await?;
        tx_control.contents.set_uinc(true); // Increment FIFO pointer
        tx_control.contents.set_txreq(true); // Request send
        self.write_register(tx_control).await?;

        Ok(())
    }

    async fn get_rx_frame<const M: u8>(&mut self) -> Result<Frame, Error<SPI, Input>> {
        // Get the RAM address of the message
        let rx_addr = self.read_register::<FIFOUserAddress<M>>().await?.contents.fifoua() as u16;

        let rx_header = ReceiveMessageObjectHeader::from_bytes(self.read_bytes(rx_addr).await?);
        let rx_raw_data: [u8; 64] = self.read_bytes(rx_addr + size_of::<ReceiveMessageObjectHeader>() as u16).await?;
        let frame = Frame::from_rx_message(rx_header, rx_raw_data);

        // Advance the FIFO
        let mut rx_control: FIFOControl<M> = self.read_register().await?;
        rx_control.contents.set_uinc(true);
        self.write_register(rx_control).await?;

        Ok(frame)
    }

    pub async fn receive(&mut self) -> Result<Frame, Error<SPI, Input>> {
        loop {
            self.interrupt.wait_for_low().await.map_err(Error::GPIOError)?;

            let mut interrupts: Interrupts = self.read_register().await?;
            if interrupts.cerrif() {
                // CAN Bus error
                interrupts.set_cerrif(false);
                self.write_register(interrupts).await?;
                return Err(Error::ControllerError("CAN Bus error!"));
            }
            if interrupts.rxif() {
                let rx_interrupt_status: ReceiveInterruptStatus = self.read_register().await?;

                // Map dynamic FIFO number to const generic
                let frame = match rx_interrupt_status {
                    i if i.fifo1() => self.get_rx_frame::<1>().await?,
                    i if i.fifo2() => self.get_rx_frame::<2>().await?,
                    i if i.fifo3() => self.get_rx_frame::<3>().await?,
                    i if i.fifo4() => self.get_rx_frame::<4>().await?,
                    i if i.fifo5() => self.get_rx_frame::<5>().await?,
                    i if i.fifo6() => self.get_rx_frame::<6>().await?,
                    i if i.fifo7() => self.get_rx_frame::<7>().await?,
                    i if i.fifo8() => self.get_rx_frame::<8>().await?,
                    i if i.fifo9() => self.get_rx_frame::<9>().await?,
                    i if i.fifo10() => self.get_rx_frame::<10>().await?,
                    i if i.fifo11() => self.get_rx_frame::<11>().await?,
                    i if i.fifo12() => self.get_rx_frame::<12>().await?,
                    i if i.fifo13() => self.get_rx_frame::<13>().await?,
                    i if i.fifo14() => self.get_rx_frame::<14>().await?,
                    i if i.fifo15() => self.get_rx_frame::<15>().await?,
                    i if i.fifo16() => self.get_rx_frame::<16>().await?,
                    i if i.fifo17() => self.get_rx_frame::<17>().await?,
                    i if i.fifo18() => self.get_rx_frame::<18>().await?,
                    i if i.fifo19() => self.get_rx_frame::<19>().await?,
                    i if i.fifo20() => self.get_rx_frame::<20>().await?,
                    i if i.fifo21() => self.get_rx_frame::<21>().await?,
                    i if i.fifo22() => self.get_rx_frame::<22>().await?,
                    i if i.fifo23() => self.get_rx_frame::<23>().await?,
                    i if i.fifo24() => self.get_rx_frame::<24>().await?,
                    i if i.fifo25() => self.get_rx_frame::<25>().await?,
                    i if i.fifo26() => self.get_rx_frame::<26>().await?,
                    i if i.fifo27() => self.get_rx_frame::<27>().await?,
                    i if i.fifo28() => self.get_rx_frame::<28>().await?,
                    i if i.fifo29() => self.get_rx_frame::<29>().await?,
                    i if i.fifo30() => self.get_rx_frame::<30>().await?,
                    i if i.fifo31() => self.get_rx_frame::<31>().await?,
                    _ => continue,
                };
                return Ok(frame);
            }
        }
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

pub enum Error<SPI: SpiDevice, Input: Wait> {
    SPIError(SPI::Error),
    GPIOError(Input::Error),
    ControllerError(&'static str),
}
impl<SPI: SpiDevice, Input: Wait> Display for Error<SPI, Input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::SPIError(err) => err.fmt(f),
            Error::GPIOError(err) => err.fmt(f),
            Error::ControllerError(msg) => f.write_str(msg),
        }
    }
}

impl<SPI: SpiDevice, Input: Wait> Debug for Error<SPI, Input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl<SPI: SpiDevice, Input: Wait> core::error::Error for Error<SPI, Input> {}