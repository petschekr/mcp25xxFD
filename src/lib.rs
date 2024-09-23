#![no_std]
use core::fmt::Debug;
use embedded_can::Id;
use embedded_hal_async::spi::{SpiDevice, Operation };
use embedded_hal::digital::InputPin;
use crate::config::{Config, FIFOConfig, FilterConfig, MaskConfig};
use crate::registers::*;

/// Register bitfields
pub mod registers;
pub mod config;

const RAM_START: u16 = 0x400;
const RAM_SIZE: u16 = 2048;

/// Either a MCP2517, MCP2518 or MCP251863 CAN-FD controller
pub struct MCP25xxFD<SPI, Input: InputPin> {
    spi: SPI,
    #[allow(unused)]
    interrupt: Option<Input>,
}

impl<SPI: SpiDevice, Input: InputPin> MCP25xxFD<SPI, Input> {
    pub fn new(spi: SPI, interrupt: Option<Input>) -> Self {
        Self {
            spi,
            interrupt,
        }
    }

    pub async fn reset_and_apply_config(&mut self, config: &Config) -> Result<(), SPI::Error> {
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

    pub async fn configure_fifo<const M: u8>(&mut self, fifo: FIFOConfig<M>) -> Result<(), SPI::Error> {
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

    pub async fn configure_filter<const M: u8, const RXFIFO: u8>(&mut self, filter: FilterConfig<M, RXFIFO>, mask: MaskConfig<M>) -> Result<(), SPI::Error> {
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
        let filter_control_address = FilterControl::<0>::ADDRESS + (RXFIFO as u16 - 1) * 8;
        self.write_bytes(filter_control_address,&[(1 << 7) | RXFIFO]).await?;

        Ok(())
    }

    /// Request the controller transition to the specified mode
    pub async fn set_mode(&mut self, mode: OperationMode) -> Result<(), SPI::Error> {
        let mut can_config: CANControl = self.read_register().await?;
        can_config.set_reqop(mode);
        self.write_register(can_config).await
    }

    /// Resets the controller and places it back into Configuration Mode
    pub async fn reset(&mut self) -> Result<(), SPI::Error> {
        let tx = Instruction::Reset.header(0x00);
        self.spi.write(&tx).await
    }

    /// Read a single register
    pub async fn read_register<R: Register>(&mut self) -> Result<R, SPI::Error> {
        let tx = Instruction::Read.header(R::ADDRESS);
        let mut rx = [0u8; 6];
        self.spi.transfer(&mut rx, &tx).await?;

        Ok(R::parse(&rx[2..]))
    }

    /// Write a single register
    pub async fn write_register<R: Register>(&mut self, register: R) -> Result<(), SPI::Error> {
        self.spi.transaction(&mut [
            Operation::Write(&Instruction::Write.header(R::ADDRESS)),
            Operation::Write(&R::serialize(register)),
        ]).await?;

        Ok(())
    }

    pub async fn read_bytes<const B: usize>(&mut self, address: u16) -> Result<[u8; B], SPI::Error> {
        assert_eq!(B % 4, 0, "Must read in multiples of 4 data bytes");
        let tx = Instruction::Read.header(RAM_START + address);
        let mut rx = [0u8; B];

        self.spi.transaction(&mut [
            Operation::Write(&tx),
            Operation::Read(&mut rx),
        ]).await?;

        Ok(rx)
    }

    pub async fn write_bytes(&mut self, address: u16, data: &[u8]) -> Result<(), SPI::Error> {
        assert_eq!(data.len() % 4, 0, "Must write in multiples of 4 data bytes");
        let tx = Instruction::Write.header(RAM_START + address);

        self.spi.transaction(&mut [
            Operation::Write(&tx),
            Operation::Write(data),
        ]).await?;

        Ok(())
    }

    pub async fn initialize_ram(&mut self, data: u8) -> Result<(), SPI::Error> {
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