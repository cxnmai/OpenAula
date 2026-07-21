//! HID discovery and acknowledged configuration sessions.

use core::fmt;
use std::ffi::CStr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hidapi::{DeviceInfo as HidDeviceInfo, HidApi, HidDevice};
use serde::{Deserialize, Serialize};

use crate::dump::{ConfigurationDump, DUMP_SCHEMA_VERSION, DumpError, HexBytes};
use crate::model::{
    DEVICE_INFO_LEN, DKS_BUFFER_LEN, FN_KEYMAP_LEN, KEYBOARD_SETTINGS_LEN, KEYMAP_LEN,
    LIGHTING_MAX_LEN, LightingConfig, MACRO_BUFFER_LEN, ModelError, PER_KEY_LIGHTING_LEN,
    RAPID_TRIGGER_LEN,
};
use crate::protocol::{
    Command, DEVICE_MAGIC, DONGLE_REPORT_LEN, Frame, ProtocolError, WIRED_REPORT_LEN,
    chunk_payload, simple_command,
};
use crate::{AULA_VENDOR_ID, DeviceId, MINI60_HE_PRO_DONGLE_PRODUCT_ID, MINI60_HE_PRO_PRODUCT_ID};

pub const WIRED_USAGE_PAGE: u16 = 0xff68;
pub const DONGLE_USAGE_PAGE: u16 = 0xff60;
pub const CONFIG_USAGE: u16 = 0x61;
pub const DEFAULT_TIMEOUT_MS: u64 = 5_000;

/// A configuration endpoint, not one of the keyboard's ordinary HID
/// interfaces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_name: String,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub release_number: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub interface_number: i32,
    pub report_len: usize,
    pub path: String,
    pub bus: String,
}

impl DeviceDescriptor {
    #[must_use]
    pub const fn id(&self) -> DeviceId {
        DeviceId {
            vendor_id: self.vendor_id,
            product_id: self.product_id,
        }
    }
}

/// Enumerate only known AULA configuration interfaces.
pub fn discover_devices() -> Result<Vec<DeviceDescriptor>, TransportError> {
    let api = HidApi::new()?;
    let mut devices: Vec<_> = api.device_list().filter_map(descriptor_from_hid).collect();
    devices.sort_by(|left, right| {
        left.product_name
            .cmp(&right.product_name)
            .then(left.path.cmp(&right.path))
    });
    Ok(devices)
}

/// Open one supported keyboard. The selector can be a displayed index, a
/// `VID:PID` pair, or an exact HID path. With no selector, exactly one device
/// must be present.
pub fn open_device(selector: Option<&str>, timeout_ms: u64) -> Result<Keyboard, TransportError> {
    let api = HidApi::new()?;
    let candidates: Vec<_> = api
        .device_list()
        .filter(|info| descriptor_from_hid(info).is_some())
        .cloned()
        .collect();
    let descriptors: Vec<_> = candidates.iter().filter_map(descriptor_from_hid).collect();
    let index = select_device(&descriptors, selector)?;
    let device = candidates[index].open_device(&api)?;
    Ok(Keyboard {
        device,
        descriptor: descriptors[index].clone(),
        timeout: Duration::from_millis(timeout_ms),
    })
}

fn descriptor_from_hid(info: &HidDeviceInfo) -> Option<DeviceDescriptor> {
    let id = DeviceId {
        vendor_id: info.vendor_id(),
        product_id: info.product_id(),
    };
    if !id.is_supported() || info.vendor_id() != AULA_VENDOR_ID || info.usage() != CONFIG_USAGE {
        return None;
    }

    let report_len = match (info.product_id(), info.usage_page()) {
        (MINI60_HE_PRO_PRODUCT_ID, WIRED_USAGE_PAGE) => WIRED_REPORT_LEN,
        (MINI60_HE_PRO_DONGLE_PRODUCT_ID, DONGLE_USAGE_PAGE) => DONGLE_REPORT_LEN,
        _ => return None,
    };

    Some(DeviceDescriptor {
        vendor_id: info.vendor_id(),
        product_id: info.product_id(),
        product_name: info
            .product_string()
            .unwrap_or("Unknown AULA device")
            .to_owned(),
        manufacturer: info.manufacturer_string().map(str::to_owned),
        serial_number: info.serial_number().map(str::to_owned),
        release_number: info.release_number(),
        usage_page: info.usage_page(),
        usage: info.usage(),
        interface_number: info.interface_number(),
        report_len,
        path: cstr_lossy(info.path()),
        bus: format!("{:?}", info.bus_type()),
    })
}

fn cstr_lossy(path: &CStr) -> String {
    path.to_string_lossy().into_owned()
}

fn select_device(
    devices: &[DeviceDescriptor],
    selector: Option<&str>,
) -> Result<usize, TransportError> {
    if devices.is_empty() {
        return Err(TransportError::NoDevice);
    }
    let Some(selector) = selector else {
        return match devices.len() {
            1 => Ok(0),
            count => Err(TransportError::MultipleDevices(count)),
        };
    };

    if let Ok(index) = selector.parse::<usize>() {
        return (index < devices.len())
            .then_some(index)
            .ok_or_else(|| TransportError::SelectorNotFound(selector.to_owned()));
    }

    if let Some((vendor, product)) = selector.split_once(':') {
        let vendor = parse_hex_u16(vendor)
            .ok_or_else(|| TransportError::InvalidSelector(selector.to_owned()))?;
        let product = parse_hex_u16(product)
            .ok_or_else(|| TransportError::InvalidSelector(selector.to_owned()))?;
        let matches: Vec<_> = devices
            .iter()
            .enumerate()
            .filter(|(_, device)| device.vendor_id == vendor && device.product_id == product)
            .map(|(index, _)| index)
            .collect();
        return match matches.as_slice() {
            [index] => Ok(*index),
            [] => Err(TransportError::SelectorNotFound(selector.to_owned())),
            _ => Err(TransportError::AmbiguousSelector(selector.to_owned())),
        };
    }

    devices
        .iter()
        .position(|device| device.path == selector)
        .ok_or_else(|| TransportError::SelectorNotFound(selector.to_owned()))
}

fn parse_hex_u16(value: &str) -> Option<u16> {
    u16::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

/// An open acknowledged protocol session.
pub struct Keyboard {
    device: HidDevice,
    descriptor: DeviceDescriptor,
    timeout: Duration,
}

impl Keyboard {
    #[must_use]
    pub const fn descriptor(&self) -> &DeviceDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub const fn report_len(&self) -> usize {
        self.descriptor.report_len
    }

    /// Read all configuration sections in the same order as the official app.
    pub fn dump(&self) -> Result<ConfigurationDump, TransportError> {
        let device_info = self.read_bulk(Command::ReadDeviceInfo, DEVICE_INFO_LEN)?;
        let keyboard_settings =
            self.read_bulk(Command::ReadKeyboardSettings, KEYBOARD_SETTINGS_LEN)?;
        let keymap = self.read_bulk(Command::ReadKeymap, KEYMAP_LEN)?;
        let lighting = self.read_bulk(Command::ReadLighting, 16)?;
        let per_key_lighting = self.read_bulk(Command::ReadPerKeyLighting, PER_KEY_LIGHTING_LEN)?;
        let macros = self.read_bulk(Command::ReadMacros, MACRO_BUFFER_LEN)?;
        let fn_keymap = self.read_bulk(Command::ReadFnKeymap, FN_KEYMAP_LEN)?;
        let rapid_trigger = self.read_bulk(Command::ReadRapidTrigger, RAPID_TRIGGER_LEN)?;
        let dks = self.read_bulk(Command::ReadDks, DKS_BUFFER_LEN)?;
        self.finalize()?;

        let captured_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);
        Ok(ConfigurationDump {
            schema_version: DUMP_SCHEMA_VERSION,
            captured_unix_ms,
            device: self.descriptor.clone(),
            device_info: HexBytes(device_info),
            keyboard_settings: HexBytes(keyboard_settings),
            keymap: HexBytes(keymap),
            lighting: HexBytes(lighting),
            per_key_lighting: HexBytes(per_key_lighting),
            macros: HexBytes(macros),
            fn_keymap: HexBytes(fn_keymap),
            rapid_trigger: HexBytes(rapid_trigger),
            dks: HexBytes(dks),
        })
    }

    /// Restore every writable section and commit once at the end.
    pub fn restore(&self, dump: &ConfigurationDump) -> Result<(), TransportError> {
        dump.validate()?;
        if !dump.device.id().is_compatible_with(self.descriptor.id()) {
            return Err(TransportError::WrongDevice {
                expected: dump.device.id(),
                actual: self.descriptor.id(),
            });
        }

        self.write_bulk_uncommitted(Command::WriteKeymap, dump.keymap.as_slice())?;
        let lighting = LightingConfig::decode(dump.lighting.as_slice())?
            .encode_preserving(dump.lighting.as_slice())?;
        self.write_bulk_uncommitted(Command::WriteLighting, &lighting)?;
        self.write_bulk_uncommitted(
            Command::WritePerKeyLighting,
            dump.per_key_lighting.as_slice(),
        )?;
        self.write_bulk_uncommitted(Command::WriteMacros, dump.macros.as_slice())?;
        self.write_bulk_uncommitted(Command::WriteFnKeymap, dump.fn_keymap.as_slice())?;
        self.write_bulk_uncommitted(Command::WriteRapidTrigger, dump.rapid_trigger.as_slice())?;
        self.write_bulk_uncommitted(Command::WriteDks, dump.dks.as_slice())?;
        self.write_bulk_uncommitted(
            Command::WriteKeyboardSettings,
            &dump.keyboard_settings.as_slice()[..16],
        )?;
        self.finalize()
    }

    /// Read one known bulk section.
    pub fn read_bulk(
        &self,
        command: Command,
        expected_len: usize,
    ) -> Result<Vec<u8>, TransportError> {
        let requests = chunk_payload(command, &vec![0; expected_len], self.report_len())?;
        let mut result = vec![0; expected_len];
        let mut received = vec![false; expected_len];
        let maximum_len = if command == Command::ReadLighting {
            LIGHTING_MAX_LEN
        } else {
            expected_len
        };

        for request in requests {
            let requested = Frame::decode(&request)?;
            let response = self.exchange(&request, command)?;
            if response.offset != requested.offset {
                return Err(TransportError::UnexpectedOffset {
                    expected: requested.offset,
                    actual: response.offset,
                });
            }

            let start = usize::from(response.offset);
            let end = start
                .checked_add(response.payload.len())
                .ok_or(TransportError::OffsetOverflow)?;
            if end > maximum_len {
                return Err(TransportError::ResponseTooLarge {
                    command: command.as_u8(),
                    expected_len: maximum_len,
                    actual_end: end,
                });
            }
            if end > result.len() {
                result.resize(end, 0);
                received.resize(end, false);
            }
            result[start..end].copy_from_slice(&response.payload);
            received[start..end].fill(true);

            // The deployed driver treats the first empty macro chunk after the
            // 400-byte pointer table as an early terminator.
            if command == Command::ReadMacros
                && start > 400
                && response.payload.iter().all(|byte| *byte == 0)
            {
                received[end..].fill(true);
                break;
            }
        }

        if received.iter().any(|value| !value) {
            return Err(TransportError::IncompleteResponse(command.as_u8()));
        }
        Ok(result)
    }

    /// Write one bulk section and issue the final commit barrier.
    pub fn write_bulk(&self, command: Command, payload: &[u8]) -> Result<(), TransportError> {
        self.write_bulk_uncommitted(command, payload)?;
        self.finalize()
    }

    /// Issue the same commit/finalize barrier used by the official app.
    pub fn commit(&self) -> Result<(), TransportError> {
        self.finalize()
    }

    /// Send a reset/calibration/test-style command whose arguments start at
    /// byte 2. A response is awaited unless `wait_for_ack` is false.
    pub fn send_short(
        &self,
        command: Command,
        arguments: &[u8],
        wait_for_ack: bool,
    ) -> Result<(), TransportError> {
        let report = simple_command(command, arguments, self.report_len())?;
        self.write_report(&report)?;
        if wait_for_ack {
            self.read_matching(command)?;
        }
        Ok(())
    }

    fn write_bulk_uncommitted(
        &self,
        command: Command,
        payload: &[u8],
    ) -> Result<(), TransportError> {
        for report in chunk_payload(command, payload, self.report_len())? {
            self.exchange(&report, command)?;
        }
        Ok(())
    }

    fn finalize(&self) -> Result<(), TransportError> {
        self.write_bulk_uncommitted(Command::Finalize, &[0, 0, 0, 0])
    }

    fn exchange(&self, report: &[u8], command: Command) -> Result<Frame, TransportError> {
        self.write_report(report)?;
        self.read_matching(command)
    }

    fn write_report(&self, report: &[u8]) -> Result<(), TransportError> {
        let mut with_report_id = Vec::with_capacity(report.len() + 1);
        with_report_id.push(0);
        with_report_id.extend_from_slice(report);
        let written = self.device.write(&with_report_id)?;
        if written != with_report_id.len() {
            return Err(TransportError::ShortWrite {
                expected: with_report_id.len(),
                actual: written,
            });
        }
        Ok(())
    }

    fn read_matching(&self, command: Command) -> Result<Frame, TransportError> {
        let deadline = Instant::now() + self.timeout;
        let mut buffer = vec![0; self.report_len() + 1];
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(TransportError::Timeout(command.as_u8()));
            }
            let remaining = deadline.saturating_duration_since(now).as_millis();
            let timeout_ms = i32::try_from(remaining.max(1)).unwrap_or(i32::MAX);
            let count = self.device.read_timeout(&mut buffer, timeout_ms)?;
            if count == 0 {
                return Err(TransportError::Timeout(command.as_u8()));
            }

            let bytes = if count == self.report_len() + 1 && buffer[0] == 0 {
                &buffer[1..count]
            } else {
                &buffer[..count]
            };
            if bytes.len() < 8 {
                continue;
            }
            let frame = match Frame::decode(bytes) {
                Ok(frame) => frame,
                Err(ProtocolError::InvalidMagic(_)) => continue,
                Err(error) => return Err(error.into()),
            };
            if frame.command == Command::DeviceControl.as_u8() {
                return Err(TransportError::DeviceControl(
                    frame.payload.first().copied().unwrap_or_default(),
                ));
            }
            if frame.command == command.as_u8() {
                if frame.magic != DEVICE_MAGIC {
                    return Err(TransportError::UnexpectedMagic(frame.magic));
                }
                return Ok(frame);
            }
            if matches!(frame.command, 0x37 | 0xfb) {
                continue;
            }
        }
    }
}

#[derive(Debug)]
pub enum TransportError {
    Hid(hidapi::HidError),
    Protocol(ProtocolError),
    Dump(DumpError),
    Model(ModelError),
    NoDevice,
    MultipleDevices(usize),
    InvalidSelector(String),
    SelectorNotFound(String),
    AmbiguousSelector(String),
    Timeout(u8),
    ShortWrite {
        expected: usize,
        actual: usize,
    },
    UnexpectedMagic(u8),
    UnexpectedOffset {
        expected: u16,
        actual: u16,
    },
    ResponseTooLarge {
        command: u8,
        expected_len: usize,
        actual_end: usize,
    },
    IncompleteResponse(u8),
    OffsetOverflow,
    DeviceControl(u8),
    WrongDevice {
        expected: DeviceId,
        actual: DeviceId,
    },
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hid(error) => write!(formatter, "HID error: {error}"),
            Self::Protocol(error) => write!(formatter, "protocol error: {error}"),
            Self::Dump(error) => write!(formatter, "invalid backup: {error}"),
            Self::Model(error) => write!(formatter, "invalid configuration record: {error}"),
            Self::NoDevice => formatter.write_str(
                "no supported AULA configuration interface found; connect the keyboard or dongle",
            ),
            Self::MultipleDevices(count) => write!(
                formatter,
                "found {count} supported devices; select one with --device INDEX"
            ),
            Self::InvalidSelector(selector) => write!(
                formatter,
                "invalid device selector {selector:?}; use an index, hex VID:PID, or exact path"
            ),
            Self::SelectorNotFound(selector) => {
                write!(formatter, "no supported device matches {selector:?}")
            }
            Self::AmbiguousSelector(selector) => write!(
                formatter,
                "more than one device matches {selector:?}; use its displayed index or path"
            ),
            Self::Timeout(command) => {
                write!(formatter, "timed out waiting for command 0x{command:02x}")
            }
            Self::ShortWrite { expected, actual } => {
                write!(
                    formatter,
                    "HID short write: expected {expected} bytes, wrote {actual}"
                )
            }
            Self::UnexpectedMagic(magic) => {
                write!(formatter, "unexpected response marker 0x{magic:02x}")
            }
            Self::UnexpectedOffset { expected, actual } => {
                write!(
                    formatter,
                    "expected response offset {expected}, got {actual}"
                )
            }
            Self::ResponseTooLarge {
                command,
                expected_len,
                actual_end,
            } => write!(
                formatter,
                "command 0x{command:02x} response ends at {actual_end}, beyond {expected_len} bytes"
            ),
            Self::IncompleteResponse(command) => {
                write!(
                    formatter,
                    "command 0x{command:02x} returned incomplete data"
                )
            }
            Self::OffsetOverflow => formatter.write_str("response offset overflow"),
            Self::DeviceControl(code) => {
                write!(
                    formatter,
                    "device interrupted the session with control code {code}"
                )
            }
            Self::WrongDevice { expected, actual } => write!(
                formatter,
                "backup device {:04x}:{:04x} is incompatible with {:04x}:{:04x}",
                expected.vendor_id, expected.product_id, actual.vendor_id, actual.product_id
            ),
        }
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hid(error) => Some(error),
            Self::Protocol(error) => Some(error),
            Self::Dump(error) => Some(error),
            Self::Model(error) => Some(error),
            _ => None,
        }
    }
}

impl From<hidapi::HidError> for TransportError {
    fn from(value: hidapi::HidError) -> Self {
        Self::Hid(value)
    }
}

impl From<ProtocolError> for TransportError {
    fn from(value: ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

impl From<DumpError> for TransportError {
    fn from(value: DumpError) -> Self {
        Self::Dump(value)
    }
}

impl From<ModelError> for TransportError {
    fn from(value: ModelError) -> Self {
        Self::Model(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(path: &str, product_id: u16) -> DeviceDescriptor {
        DeviceDescriptor {
            vendor_id: AULA_VENDOR_ID,
            product_id,
            product_name: "test".to_owned(),
            manufacturer: None,
            serial_number: None,
            release_number: 0,
            usage_page: DONGLE_USAGE_PAGE,
            usage: CONFIG_USAGE,
            interface_number: 2,
            report_len: DONGLE_REPORT_LEN,
            path: path.to_owned(),
            bus: "Usb".to_owned(),
        }
    }

    #[test]
    fn selects_index_vid_pid_and_path() {
        let devices = vec![
            descriptor("/dev/hidraw1", MINI60_HE_PRO_PRODUCT_ID),
            descriptor("/dev/hidraw2", MINI60_HE_PRO_DONGLE_PRODUCT_ID),
        ];
        assert_eq!(select_device(&devices, Some("1")).unwrap(), 1);
        assert_eq!(select_device(&devices, Some("0c45:fefe")).unwrap(), 1);
        assert_eq!(select_device(&devices, Some("/dev/hidraw1")).unwrap(), 0);
        assert!(matches!(
            select_device(&devices, None),
            Err(TransportError::MultipleDevices(2))
        ));
    }

    #[test]
    fn host_magic_constant_remains_distinct() {
        assert_ne!(crate::protocol::HOST_MAGIC, DEVICE_MAGIC);
    }
}
