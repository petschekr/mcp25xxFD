use embedded_can::Id;
use crate::registers::{PayloadSize, RetransmissionAttempts};

#[derive(Clone, Debug)]
pub struct Config {
    pub ecc_enabled: bool,
    pub txq_enabled: bool,
    pub tx_event_fifo_enabled: bool,
    pub iso_crc_enabled: bool,
    pub restrict_retx_attempts: bool,
    pub bit_rate: BitRate,
    pub clock: Clock,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ecc_enabled: true,
            txq_enabled: false,
            tx_event_fifo_enabled: false,
            iso_crc_enabled: true,
            restrict_retx_attempts: false,
            bit_rate: BitRate::default(),
            clock: Clock::Clock40MHz,
        }
    }
}

pub struct FIFOConfig<const M: u8> {
    pub size: u8,
    pub payload_size: PayloadSize,
    pub transmit: bool,
    pub tx_attempts: RetransmissionAttempts,
    pub priority: u8,
}

pub struct FilterConfig<const M: u8, const RXFIFO: u8> {
    pub match_only_extended: bool,
    pub id: Id,
}
impl<const M: u8, const RXFIFO: u8> FilterConfig<M, RXFIFO> {
    pub fn from_id(id: impl Into<Id>) -> Self {
        let id: Id = id.into();
        Self {
            match_only_extended: match id {
                Id::Extended(_) => true,
                Id::Standard(_) => false,
            },
            id,
        }
    }
}
pub struct MaskConfig<const M: u8> {
    pub match_id_type: bool,
    pub id: Id,
}
impl<const M: u8> MaskConfig<M> {
    pub fn from_id(id: impl Into<Id>) -> Self {
        Self {
            match_id_type: false,
            id: id.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Clock {
    Clock20MHz,
    Clock40MHz,
}
#[derive(Clone, Debug)]
pub enum ArbitrationBitRate {
    Rate125K,
    Rate250K,
    Rate500K,
    Rate1000K,
}
#[derive(Clone, Debug)]
pub enum DataBitRate {
    Rate500K,
    Rate833K,
    Rate1M,
    Rate1M5,
    Rate2M,
    Rate3M,
    Rate4M,
    Rate5M,
    Rate6M7,
    Rate8M,
    Rate10M,
}

#[derive(Clone, Debug)]
pub struct BitRate {
    pub arbitration: ArbitrationBitRate,
    pub data: DataBitRate,
}
impl Default for BitRate {
    fn default() -> Self {
        Self { arbitration: ArbitrationBitRate::Rate500K, data: DataBitRate::Rate2M }
    }
}
#[derive(Default)]
pub(crate) struct BitRateConfig {
    pub(crate) arbitration_brp: u8,
    pub(crate) arbitration_tseg1: u8,
    pub(crate) arbitration_tseg2: u8,
    pub(crate) arbitration_sjw: u8,
    pub(crate) data_brp: u8,
    pub(crate) data_tseg1: u8,
    pub(crate) data_tseg2: u8,
    pub(crate) data_sjw: u8,
    pub(crate) tdc_offset: u8,
    pub(crate) tdc_value: u8,
    pub(crate) tdc_mode: u8,
}

impl BitRate {
    pub(crate) fn get_config(&self, clock: &Clock) -> Option<BitRateConfig> {
        let mut config = BitRateConfig::default();
        config.tdc_mode = 0b10; // Auto mode
        match clock {
            Clock::Clock40MHz => {
                match &self.arbitration {
                    ArbitrationBitRate::Rate125K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 254;
                        config.arbitration_tseg2 = 63;
                        config.arbitration_sjw = 63;
                    },
                    ArbitrationBitRate::Rate250K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 126;
                        config.arbitration_tseg2 = 31;
                        config.arbitration_sjw = 31;
                    },
                    ArbitrationBitRate::Rate500K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 62;
                        config.arbitration_tseg2 = 15;
                        config.arbitration_sjw = 15;
                    },
                    ArbitrationBitRate::Rate1000K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 30;
                        config.arbitration_tseg2 = 7;
                        config.arbitration_sjw = 7;
                    },
                };
                match (&self.arbitration, &self.data) {
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate1M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 30;
                        config.data_tseg2 = 7;
                        config.data_sjw = 7;
                        config.tdc_offset = 31;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate2M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 14;
                        config.data_tseg2 = 3;
                        config.data_sjw = 3;
                        config.tdc_offset = 15;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate3M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 8;
                        config.data_tseg2 = 2;
                        config.data_sjw = 2;
                        config.tdc_offset = 9;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate4M) |
                    (ArbitrationBitRate::Rate1000K, DataBitRate::Rate4M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 6;
                        config.data_tseg2 = 1;
                        config.data_sjw = 1;
                        config.tdc_offset = 7;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate5M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 4;
                        config.data_tseg2 = 1;
                        config.data_sjw = 1;
                        config.tdc_offset = 5;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate6M7) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 3;
                        config.data_tseg2 = 0;
                        config.data_sjw = 0;
                        config.tdc_offset = 4;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate8M) |
                    (ArbitrationBitRate::Rate1000K, DataBitRate::Rate8M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 2;
                        config.data_tseg2 = 0;
                        config.data_sjw = 0;
                        config.tdc_offset = 3;
                        config.tdc_value = 1;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate10M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 1;
                        config.data_tseg2 = 0;
                        config.data_sjw = 0;
                        config.tdc_offset = 2;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate500K) |
                    (ArbitrationBitRate::Rate125K, DataBitRate::Rate500K) => {
                        config.data_brp = 1;
                        config.data_tseg1 = 30;
                        config.data_tseg2 = 7;
                        config.data_sjw = 7;
                        config.tdc_offset = 31;
                        config.tdc_value = 0;
                        config.tdc_mode = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate833K) => {
                        config.data_brp = 1;
                        config.data_tseg1 = 17;
                        config.data_tseg2 = 4;
                        config.data_sjw = 4;
                        config.tdc_offset = 18;
                        config.tdc_value = 0;
                        config.tdc_mode = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate1M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 30;
                        config.data_tseg2 = 7;
                        config.data_sjw = 7;
                        config.tdc_offset = 31;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate1M5) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 18;
                        config.data_tseg2 = 5;
                        config.data_sjw = 5;
                        config.tdc_offset = 19;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate2M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 14;
                        config.data_tseg2 = 3;
                        config.data_sjw = 3;
                        config.tdc_offset = 15;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate3M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 8;
                        config.data_tseg2 = 2;
                        config.data_sjw = 2;
                        config.tdc_offset = 9;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate4M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 6;
                        config.data_tseg2 = 1;
                        config.data_sjw = 1;
                        config.tdc_offset = 7;
                        config.tdc_value = 0;
                    },
                    _ => { return None },
                };
            },
            Clock::Clock20MHz => {
                match &self.arbitration {
                    ArbitrationBitRate::Rate125K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 126;
                        config.arbitration_tseg2 = 31;
                        config.arbitration_sjw = 31;
                    },
                    ArbitrationBitRate::Rate250K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 62;
                        config.arbitration_tseg2 = 15;
                        config.arbitration_sjw = 15;
                    },
                    ArbitrationBitRate::Rate500K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 30;
                        config.arbitration_tseg2 = 7;
                        config.arbitration_sjw = 7;
                    },
                    ArbitrationBitRate::Rate1000K => {
                        config.arbitration_brp = 0;
                        config.arbitration_tseg1 = 14;
                        config.arbitration_tseg2 = 3;
                        config.arbitration_sjw = 3;
                    },
                };
                match (&self.arbitration, &self.data) {
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate1M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 14;
                        config.data_tseg2 = 3;
                        config.data_sjw = 3;
                        config.tdc_offset = 15;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate2M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 6;
                        config.data_tseg2 = 1;
                        config.data_sjw = 1;
                        config.tdc_offset = 7;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate4M) |
                    (ArbitrationBitRate::Rate1000K, DataBitRate::Rate4M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 2;
                        config.data_tseg2 = 0;
                        config.data_sjw = 0;
                        config.tdc_offset = 3;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate500K, DataBitRate::Rate5M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 1;
                        config.data_tseg2 = 0;
                        config.data_sjw = 0;
                        config.tdc_offset = 2;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate500K) |
                    (ArbitrationBitRate::Rate125K, DataBitRate::Rate500K) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 30;
                        config.data_tseg2 = 7;
                        config.data_sjw = 7;
                        config.tdc_offset = 31;
                        config.tdc_value = 0;
                        config.tdc_mode = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate833K) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 17;
                        config.data_tseg2 = 4;
                        config.data_sjw = 4;
                        config.tdc_offset = 18;
                        config.tdc_value = 0;
                        config.tdc_mode = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate1M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 14;
                        config.data_tseg2 = 3;
                        config.data_sjw = 3;
                        config.tdc_offset = 15;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate1M5) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 8;
                        config.data_tseg2 = 2;
                        config.data_sjw = 2;
                        config.tdc_offset = 9;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate2M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 6;
                        config.data_tseg2 = 1;
                        config.data_sjw = 1;
                        config.tdc_offset = 7;
                        config.tdc_value = 0;
                    },
                    (ArbitrationBitRate::Rate250K, DataBitRate::Rate4M) => {
                        config.data_brp = 0;
                        config.data_tseg1 = 2;
                        config.data_tseg2 = 0;
                        config.data_sjw = 0;
                        config.tdc_offset = 3;
                        config.tdc_value = 0;
                    },
                    _ => { return None },
                };
            },
        }

        Some(config)
    }
}