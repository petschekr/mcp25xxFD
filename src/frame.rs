use embedded_can::{ExtendedId, Id, StandardId};
use crate::registers::{DataLengthCode, ReceiveMessageObjectHeader, TransmitMessageObjectHeader};

#[derive(Clone, Debug)]
pub struct Frame {
    id: Id,
    dlc: DataLengthCode,
    data: [u8; 64],
    sequence_number: u32,
}

impl Frame {
    pub fn new(id: impl Into<Id>, data_slice: &[u8]) -> Option<Self> {
        let mut data = [0; 64];
        data[0..data_slice.len()].copy_from_slice(data_slice);
        Some(Self {
            id: id.into(),
            dlc: DataLengthCode::best_fit(data_slice.len())?,
            data,
            sequence_number: 0,
        })
    }
    #[inline]
    pub fn with_sequence_number(mut self, sequence_number: u32) -> Self {
        self.sequence_number = sequence_number;
        self
    }
    #[inline]
    pub fn with_dlc(mut self, dlc: DataLengthCode) -> Self {
        self.dlc = dlc;
        self
    }

    #[inline]
    pub fn id(&self) -> Id { self.id }
    pub fn raw_id(&self) -> u32 {
        match self.id {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        }
    }
    #[inline]
    pub fn dlc(&self) -> DataLengthCode { self.dlc }
    #[inline]
    pub fn data(&self) -> &[u8] { &self.data[..self.dlc.bytes()] }
    #[inline]
    pub fn sequence_number(&self) -> u32 { self.sequence_number }

    pub(crate) fn as_components(&self) -> (TransmitMessageObjectHeader, &[u8]) {
        let (sid, eid) = match self.id {
            Id::Standard(id) => (id.as_raw(), 0),
            Id::Extended(id) => ((id.as_raw() & 0x7FF) as u16, id.as_raw() >> 11),
        };
        let is_fd_frame = self.dlc.bytes() > 8;
        let header = TransmitMessageObjectHeader::new()
            .with_sid(sid)
            .with_eid(eid)
            .with_seq(self.sequence_number)
            .with_ide(eid > 0)
            .with_fdf(is_fd_frame)
            .with_brs(is_fd_frame) // Always send FD frames at the data bitrate
            .with_dlc(self.dlc);
        (header, self.data())
    }
    pub(crate) fn from_rx_message(header: ReceiveMessageObjectHeader, data: [u8; 64]) -> Self {
        Self {
            id: if header.eid() == 0 {
                StandardId::new(header.sid()).unwrap().into()
            } else {
                ExtendedId::new(header.eid() << 11 + header.sid()).unwrap().into()
            },
            dlc: header.dlc(),
            data,
            sequence_number: 0,
        }
    }
}