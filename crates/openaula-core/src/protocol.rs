//! Transport-neutral encoding for the AULA vendor HID protocol.

use core::fmt;

/// Host-to-device bulk-frame marker.
pub const HOST_MAGIC: u8 = 0xaa;

/// Device-to-host bulk-frame marker observed on the Mini60 HE Pro dongle.
pub const DEVICE_MAGIC: u8 = 0x55;

/// Bytes before a bulk frame's payload.
pub const HEADER_LEN: usize = 8;

/// The report size exposed by the wired configuration interface.
pub const WIRED_REPORT_LEN: usize = 64;

/// The report size exposed by the wireless dongle configuration interface.
pub const DONGLE_REPORT_LEN: usize = 32;

/// Commands recovered from the official web configurator.
///
/// Some command numbers have more than one meaning for device families with a
/// screen or music mode. Those aliases are intentionally represented by one
/// descriptive variant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Command {
    Finalize = 0x00,
    Reset = 0x0f,
    ReadDeviceInfo = 0x10,
    ReadKeyboardSettings = 0x11,
    ReadKeymap = 0x12,
    ReadLighting = 0x13,
    ReadPerKeyLighting = 0x14,
    ReadMacros = 0x15,
    ReadFnKeymap = 0x16,
    ReadRapidTrigger = 0x17,
    ReadDks = 0x18,
    ReadLightBox = 0x1b,
    ReadDefaultFn = 0x1c,
    WriteKeyboardSettings = 0x21,
    WriteKeymap = 0x22,
    WriteLighting = 0x23,
    WritePerKeyLighting = 0x24,
    WriteMacros = 0x25,
    WriteFnKeymap = 0x26,
    WriteRapidTrigger = 0x27,
    WriteDks = 0x28,
    WriteLightBox = 0x2b,
    ReadMusic = 0x32,
    WriteClockOrTelemetry = 0x34,
    StreamMusic32 = 0x35,
    FlashFrame = 0x41,
    WriteMusicOrGifFlash = 0x42,
    WriteTft = 0x50,
    Status = 0x62,
    StartCalibration = 0x64,
    StopCalibration = 0x65,
    StartKeyTest = 0x66,
    StopKeyTest = 0x67,
    ScreenInfo = 0xfa,
    CalibrationData = 0xfb,
    DeviceControl = 0xfc,
}

impl Command {
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Command {
    type Error = UnknownCommand;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let command = match value {
            0x00 => Self::Finalize,
            0x0f => Self::Reset,
            0x10 => Self::ReadDeviceInfo,
            0x11 => Self::ReadKeyboardSettings,
            0x12 => Self::ReadKeymap,
            0x13 => Self::ReadLighting,
            0x14 => Self::ReadPerKeyLighting,
            0x15 => Self::ReadMacros,
            0x16 => Self::ReadFnKeymap,
            0x17 => Self::ReadRapidTrigger,
            0x18 => Self::ReadDks,
            0x1b => Self::ReadLightBox,
            0x1c => Self::ReadDefaultFn,
            0x21 => Self::WriteKeyboardSettings,
            0x22 => Self::WriteKeymap,
            0x23 => Self::WriteLighting,
            0x24 => Self::WritePerKeyLighting,
            0x25 => Self::WriteMacros,
            0x26 => Self::WriteFnKeymap,
            0x27 => Self::WriteRapidTrigger,
            0x28 => Self::WriteDks,
            0x2b => Self::WriteLightBox,
            0x32 => Self::ReadMusic,
            0x34 => Self::WriteClockOrTelemetry,
            0x35 => Self::StreamMusic32,
            0x41 => Self::FlashFrame,
            0x42 => Self::WriteMusicOrGifFlash,
            0x50 => Self::WriteTft,
            0x62 => Self::Status,
            0x64 => Self::StartCalibration,
            0x65 => Self::StopCalibration,
            0x66 => Self::StartKeyTest,
            0x67 => Self::StopKeyTest,
            0xfa => Self::ScreenInfo,
            0xfb => Self::CalibrationData,
            0xfc => Self::DeviceControl,
            value => return Err(UnknownCommand(value)),
        };
        Ok(command)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnknownCommand(pub u8);

impl fmt::Display for UnknownCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown AULA command 0x{:02x}", self.0)
    }
}

impl std::error::Error for UnknownCommand {}

/// A decoded bulk-transfer frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    pub magic: u8,
    pub command: u8,
    pub offset: u16,
    pub metadata: u8,
    pub final_chunk: bool,
    pub payload: Vec<u8>,
}

impl Frame {
    /// Decode a 32- or 64-byte HID report.
    pub fn decode(report: &[u8]) -> Result<Self, ProtocolError> {
        if report.len() < HEADER_LEN {
            return Err(ProtocolError::ReportTooShort(report.len()));
        }
        if !matches!(report[0], HOST_MAGIC | DEVICE_MAGIC) {
            return Err(ProtocolError::InvalidMagic(report[0]));
        }

        let payload_len = usize::from(report[2]);
        let capacity = report.len() - HEADER_LEN;
        if payload_len > capacity {
            return Err(ProtocolError::PayloadTooLarge {
                payload_len,
                capacity,
            });
        }

        Ok(Self {
            magic: report[0],
            command: report[1],
            offset: u16::from_le_bytes([report[3], report[4]]),
            metadata: report[5],
            final_chunk: report[6] != 0,
            payload: report[HEADER_LEN..HEADER_LEN + payload_len].to_vec(),
        })
    }

    /// Encode a bulk frame at the requested HID report length.
    pub fn encode(&self, report_len: usize) -> Result<Vec<u8>, ProtocolError> {
        validate_report_len(report_len)?;
        let capacity = report_len - HEADER_LEN;
        if self.payload.len() > capacity || self.payload.len() > usize::from(u8::MAX) {
            return Err(ProtocolError::PayloadTooLarge {
                payload_len: self.payload.len(),
                capacity,
            });
        }

        let mut report = vec![0; report_len];
        report[0] = self.magic;
        report[1] = self.command;
        report[2] = self.payload.len() as u8;
        report[3..5].copy_from_slice(&self.offset.to_le_bytes());
        report[5] = self.metadata;
        report[6] = u8::from(self.final_chunk);
        report[HEADER_LEN..HEADER_LEN + self.payload.len()].copy_from_slice(&self.payload);
        Ok(report)
    }
}

/// Split a state buffer into the acknowledged bulk frames used by the driver.
pub fn chunk_payload(
    command: Command,
    payload: &[u8],
    report_len: usize,
) -> Result<Vec<Vec<u8>>, ProtocolError> {
    validate_report_len(report_len)?;
    if payload.is_empty() {
        return Err(ProtocolError::EmptyPayload);
    }

    let capacity = report_len - HEADER_LEN;
    let mut frames = Vec::with_capacity(payload.len().div_ceil(capacity));
    for (index, chunk) in payload.chunks(capacity).enumerate() {
        let offset = index
            .checked_mul(capacity)
            .and_then(|value| u16::try_from(value).ok())
            .ok_or(ProtocolError::OffsetOverflow)?;
        let frame = Frame {
            magic: HOST_MAGIC,
            command: command.as_u8(),
            offset,
            metadata: 0,
            final_chunk: usize::from(offset) + chunk.len() == payload.len(),
            payload: chunk.to_vec(),
        };
        frames.push(frame.encode(report_len)?);
    }
    Ok(frames)
}

/// Build one of the short command packets whose arguments begin at byte 2.
pub fn simple_command(
    command: Command,
    arguments: &[u8],
    report_len: usize,
) -> Result<Vec<u8>, ProtocolError> {
    validate_report_len(report_len)?;
    if arguments.len() > report_len - 2 {
        return Err(ProtocolError::PayloadTooLarge {
            payload_len: arguments.len(),
            capacity: report_len - 2,
        });
    }

    let mut report = vec![0; report_len];
    report[0] = HOST_MAGIC;
    report[1] = command.as_u8();
    report[2..2 + arguments.len()].copy_from_slice(arguments);
    Ok(report)
}

/// Reassemble response frames into a fixed-size state buffer.
pub fn assemble_response(
    command: Command,
    expected_len: usize,
    reports: &[Vec<u8>],
) -> Result<Vec<u8>, ProtocolError> {
    let mut result = vec![0; expected_len];
    let mut seen = vec![false; expected_len];
    let mut saw_final = false;

    for report in reports {
        let frame = Frame::decode(report)?;
        if frame.magic != DEVICE_MAGIC {
            return Err(ProtocolError::UnexpectedDirection(frame.magic));
        }
        if frame.command != command.as_u8() {
            return Err(ProtocolError::UnexpectedCommand {
                expected: command.as_u8(),
                actual: frame.command,
            });
        }

        let start = usize::from(frame.offset);
        let end = start
            .checked_add(frame.payload.len())
            .ok_or(ProtocolError::OffsetOverflow)?;
        if end > expected_len {
            return Err(ProtocolError::ChunkOutOfBounds {
                offset: start,
                payload_len: frame.payload.len(),
                expected_len,
            });
        }
        result[start..end].copy_from_slice(&frame.payload);
        seen[start..end].fill(true);
        saw_final |= frame.final_chunk;
    }

    if !saw_final || seen.iter().any(|value| !value) {
        return Err(ProtocolError::IncompleteResponse);
    }
    Ok(result)
}

fn validate_report_len(report_len: usize) -> Result<(), ProtocolError> {
    if matches!(report_len, DONGLE_REPORT_LEN | WIRED_REPORT_LEN) {
        Ok(())
    } else {
        Err(ProtocolError::UnsupportedReportLength(report_len))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    ReportTooShort(usize),
    UnsupportedReportLength(usize),
    InvalidMagic(u8),
    UnexpectedDirection(u8),
    UnexpectedCommand {
        expected: u8,
        actual: u8,
    },
    PayloadTooLarge {
        payload_len: usize,
        capacity: usize,
    },
    ChunkOutOfBounds {
        offset: usize,
        payload_len: usize,
        expected_len: usize,
    },
    EmptyPayload,
    OffsetOverflow,
    IncompleteResponse,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReportTooShort(length) => write!(formatter, "report is only {length} bytes"),
            Self::UnsupportedReportLength(length) => {
                write!(formatter, "unsupported HID report length {length}")
            }
            Self::InvalidMagic(value) => write!(formatter, "invalid frame marker 0x{value:02x}"),
            Self::UnexpectedDirection(value) => {
                write!(
                    formatter,
                    "expected a device response, got marker 0x{value:02x}"
                )
            }
            Self::UnexpectedCommand { expected, actual } => write!(
                formatter,
                "expected command 0x{expected:02x}, got 0x{actual:02x}"
            ),
            Self::PayloadTooLarge {
                payload_len,
                capacity,
            } => write!(
                formatter,
                "payload is {payload_len} bytes but capacity is {capacity}"
            ),
            Self::ChunkOutOfBounds {
                offset,
                payload_len,
                expected_len,
            } => write!(
                formatter,
                "chunk at {offset} with {payload_len} bytes exceeds {expected_len}-byte response"
            ),
            Self::EmptyPayload => formatter.write_str("bulk payload must not be empty"),
            Self::OffsetOverflow => formatter.write_str("bulk frame offset exceeds u16"),
            Self::IncompleteResponse => {
                formatter.write_str("response is missing data or final marker")
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_captured_device_info_frame() {
        let report = [
            0x55, 0x10, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x92, 0x45, 0x0c,
            0xa2, 0x80, 0x52, 0x01, 0x00, 0x00, 0x66, 0x01, 0x0c, 0x11, 0x02, 0x3c, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00,
        ];
        let frame = Frame::decode(&report).expect("captured report should decode");
        assert_eq!(frame.magic, DEVICE_MAGIC);
        assert_eq!(frame.command, Command::ReadDeviceInfo.as_u8());
        assert_eq!(frame.offset, 0);
        assert_eq!(frame.payload.len(), 24);
        assert_eq!(frame.payload[8], 0x52);
        assert_eq!(frame.payload[9], 0x01);
    }

    #[test]
    fn chunks_dongle_state_at_24_bytes() {
        let payload: Vec<u8> = (0..32).collect();
        let reports = chunk_payload(Command::ReadDeviceInfo, &payload, DONGLE_REPORT_LEN)
            .expect("valid payload");
        assert_eq!(reports.len(), 2);

        let first = Frame::decode(&reports[0]).expect("first frame");
        assert_eq!(first.offset, 0);
        assert_eq!(first.payload.len(), 24);
        assert!(!first.final_chunk);

        let last = Frame::decode(&reports[1]).expect("last frame");
        assert_eq!(last.offset, 24);
        assert_eq!(last.payload.len(), 8);
        assert!(last.final_chunk);
    }

    #[test]
    fn chunks_wired_state_at_56_bytes() {
        let payload = vec![7; 64];
        let reports =
            chunk_payload(Command::WriteKeymap, &payload, WIRED_REPORT_LEN).expect("valid payload");
        assert_eq!(reports.len(), 2);
        assert_eq!(Frame::decode(&reports[0]).unwrap().payload.len(), 56);
        assert_eq!(Frame::decode(&reports[1]).unwrap().payload.len(), 8);
    }

    #[test]
    fn builds_captured_calibration_command_shape() {
        let report = simple_command(Command::StartCalibration, &[0, 0, 0, 0], DONGLE_REPORT_LEN)
            .expect("valid command");
        assert_eq!(&report[..6], &[0xaa, 0x64, 0, 0, 0, 0]);
    }

    #[test]
    fn rejects_incomplete_response() {
        let frame = Frame {
            magic: DEVICE_MAGIC,
            command: Command::ReadDeviceInfo.as_u8(),
            offset: 0,
            metadata: 0,
            final_chunk: false,
            payload: vec![0; 24],
        }
        .encode(DONGLE_REPORT_LEN)
        .unwrap();
        assert_eq!(
            assemble_response(Command::ReadDeviceInfo, 32, &[frame]),
            Err(ProtocolError::IncompleteResponse)
        );
    }
}
