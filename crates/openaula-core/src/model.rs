//! Decoded configuration records used by both frontends.

use core::fmt;
use serde::{Deserialize, Serialize};

pub const DEVICE_INFO_LEN: usize = 32;
pub const KEYBOARD_SETTINGS_LEN: usize = 32;
pub const KEYMAP_LEN: usize = 512;
/// Semantic lighting record length.
pub const LIGHTING_LEN: usize = 16;
/// Largest lighting response observed from the dongle, including padding.
pub const LIGHTING_MAX_LEN: usize = 24;
pub const PER_KEY_LIGHTING_LEN: usize = 512;
pub const MACRO_BUFFER_LEN: usize = 1024;
pub const FN_KEYMAP_LEN: usize = 512;
pub const RAPID_TRIGGER_LEN: usize = 1024;
pub const DKS_BUFFER_LEN: usize = 1024;
pub const KEY_SLOT_COUNT: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{:02}", self.major, self.minor)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub firmware: FirmwareVersion,
    pub battery_percent: u8,
    pub axial_model: u16,
    pub max_frames: u16,
    pub gif_frames: u16,
    pub led_frames: u16,
    pub rotated_screen: bool,
}

impl DeviceInfo {
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        require_len(bytes, DEVICE_INFO_LEN)?;
        let packed = bytes[8];
        Ok(Self {
            firmware: FirmwareVersion {
                major: bytes[9],
                minor: 10 * (packed >> 4) + (packed & 0x0f),
            },
            battery_percent: bytes[17],
            axial_model: u16::from_le_bytes([bytes[20], bytes[21]]),
            max_frames: u16::from_le_bytes([bytes[22], bytes[23]]),
            gif_frames: u16::from_le_bytes([bytes[24], bytes[25]]),
            led_frames: u16::from_le_bytes([bytes[26], bytes[27]]),
            rotated_screen: bytes[28] != 0,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReportRate {
    Hz1000,
    Hz4000,
    Hz8000,
    Unknown(u8),
}

impl ReportRate {
    #[must_use]
    pub const fn decode(value: u8) -> Self {
        match value {
            5 => Self::Hz4000,
            6 => Self::Hz8000,
            value => Self::Unknown(value),
        }
    }

    #[must_use]
    pub const fn encode(self) -> u8 {
        match self {
            Self::Hz1000 => 0,
            Self::Hz4000 => 5,
            Self::Hz8000 => 6,
            Self::Unknown(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyboardSettings {
    pub sleep_minutes: u8,
    pub response_time: u8,
    pub report_rate: ReportRate,
    pub os_mode: u8,
    pub tft_timeout: u8,
    /// Hundredths of a millimeter.
    pub top_dead_zone: u8,
    /// Hundredths of a millimeter.
    pub bottom_dead_zone: u8,
    pub stability: bool,
    pub adaptive_calibration: bool,
    pub wake: bool,
}

impl KeyboardSettings {
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        require_len(bytes, 16)?;
        Ok(Self {
            sleep_minutes: bytes[3],
            response_time: bytes[4],
            report_rate: ReportRate::decode(bytes[5]),
            os_mode: bytes[6],
            tft_timeout: bytes[7],
            top_dead_zone: bytes[8],
            bottom_dead_zone: bytes[9],
            stability: bytes[11] != 0,
            adaptive_calibration: bytes[14] != 0,
            wake: bytes[15] != 0,
        })
    }

    #[must_use]
    pub fn encode(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[3] = self.sleep_minutes;
        bytes[4] = self.response_time;
        bytes[5] = self.report_rate.encode();
        bytes[6] = self.os_mode;
        bytes[7] = self.tft_timeout;
        bytes[8] = self.top_dead_zone;
        bytes[9] = self.bottom_dead_zone;
        bytes[11] = u8::from(self.stability);
        bytes[14] = u8::from(self.adaptive_calibration);
        bytes[15] = u8::from(self.wake);
        bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LightingConfig {
    pub effect: u8,
    pub primary: [u8; 3],
    pub secondary: [u8; 3],
    pub colorful: bool,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
    pub gear: u8,
}

impl LightingConfig {
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        require_len(bytes, LIGHTING_LEN)?;
        Ok(Self {
            effect: bytes[0],
            primary: [bytes[1], bytes[2], bytes[3]],
            secondary: [bytes[5], bytes[6], bytes[7]],
            colorful: bytes[8] != 0,
            brightness: bytes[9],
            speed: bytes[10],
            direction: bytes[11],
            gear: bytes[12],
        })
    }

    /// Encode the 16-byte form written by the official configurator.
    #[must_use]
    pub fn encode(self) -> [u8; LIGHTING_LEN] {
        let mut bytes = [0; LIGHTING_LEN];
        bytes[0] = self.effect;
        bytes[1..4].copy_from_slice(&self.primary);
        bytes[4] = 0xff;
        bytes[5..8].copy_from_slice(&self.secondary);
        bytes[8] = u8::from(self.colorful);
        bytes[9] = self.brightness;
        bytes[10] = self.speed;
        bytes[11] = self.direction;
        bytes[12] = self.gear;
        bytes[14] = 0xaa;
        bytes[15] = 0x55;
        bytes
    }

    /// Encode the writer form while retaining the reserved byte and any
    /// transport padding returned by the device.
    pub fn encode_preserving(self, original: &[u8]) -> Result<Vec<u8>, ModelError> {
        require_len(original, LIGHTING_LEN)?;
        let mut bytes = original.to_vec();
        let reserved = bytes[13];
        bytes[..LIGHTING_LEN].copy_from_slice(&self.encode());
        bytes[13] = reserved;
        Ok(bytes)
    }
}

/// One four-byte entry in the normal or Fn keymap.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyAssignment {
    Empty,
    Mouse { kind: u8, code: u8 },
    Keyboard { code: u8, special: bool },
    Consumer { usage: u16 },
    Macro { index: u8, mode: u8, repeat: u8 },
    Combo { key1: u8, key2: u8, key3: u8 },
    Dks { slot: u8 },
    ModTap { hold: u8, tap: u8, hold_ms: u16 },
    Toggle { key: u8 },
    Socd { mode: u8, key1: u8, key2: u8 },
    RappySnappy { key1: u8, key2: u8 },
    SpecialFunction { value: u32 },
    Unknown([u8; 4]),
}

impl KeyAssignment {
    #[must_use]
    pub const fn decode(bytes: [u8; 4]) -> Self {
        match bytes {
            [0, 0, 0, 0] => Self::Empty,
            [1, kind, code, _] => Self::Mouse { kind, code },
            [2, 0, code, _] => Self::Keyboard {
                code,
                special: false,
            },
            [2, code, _, _] => Self::Keyboard {
                code,
                special: true,
            },
            [3, low, high, _] => Self::Consumer {
                usage: u16::from_le_bytes([low, high]),
            },
            [6, index, mode, repeat] => Self::Macro {
                index,
                mode,
                repeat,
            },
            [7, key1, key2, key3] => Self::Combo { key1, key2, key3 },
            [8, slot, _, _] => Self::Dks { slot },
            [9, hold, tap, time] => Self::ModTap {
                hold,
                tap,
                hold_ms: time as u16 * 10,
            },
            [10, key, _, _] => Self::Toggle { key },
            [11, mode, key1, key2] => Self::Socd { mode, key1, key2 },
            [12, _, key1, key2] => Self::RappySnappy { key1, key2 },
            [13, high, middle, low] => Self::SpecialFunction {
                value: ((high as u32) << 16) | ((middle as u32) << 8) | low as u32,
            },
            bytes => Self::Unknown(bytes),
        }
    }

    #[must_use]
    pub fn encode(self) -> [u8; 4] {
        match self {
            Self::Empty => [0; 4],
            Self::Mouse { kind, code } => [1, kind, code, 0],
            Self::Keyboard {
                code,
                special: false,
            } => [2, 0, code, 0],
            Self::Keyboard {
                code,
                special: true,
            } => [2, code, 0, 0],
            Self::Consumer { usage } => {
                let [low, high] = usage.to_le_bytes();
                [3, low, high, 0]
            }
            Self::Macro {
                index,
                mode,
                repeat,
            } => [6, index, mode, repeat],
            Self::Combo { key1, key2, key3 } => [7, key1, key2, key3],
            Self::Dks { slot } => [8, slot, 0, 0],
            Self::ModTap { hold, tap, hold_ms } => {
                [9, hold, tap, (hold_ms / 10).min(u16::from(u8::MAX)) as u8]
            }
            Self::Toggle { key } => [10, key, 0, 0],
            Self::Socd { mode, key1, key2 } => [11, mode, key1, key2],
            Self::RappySnappy { key1, key2 } => [12, 0, key1, key2],
            Self::SpecialFunction { value } => [
                13,
                ((value >> 16) & 0xff) as u8,
                ((value >> 8) & 0xff) as u8,
                (value & 0xff) as u8,
            ],
            Self::Unknown(bytes) => bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RapidTriggerFlags(pub u8);

impl RapidTriggerFlags {
    pub const FULL_TRAVEL: u8 = 1 << 0;
    pub const BOTTOM_OPTIMIZATION: u8 = 1 << 1;
    pub const RAMPAGE: u8 = 1 << 2;

    #[must_use]
    pub const fn full_travel(self) -> bool {
        self.0 & Self::FULL_TRAVEL != 0
    }

    #[must_use]
    pub const fn bottom_optimization(self) -> bool {
        self.0 & Self::BOTTOM_OPTIMIZATION != 0
    }

    #[must_use]
    pub const fn rampage(self) -> bool {
        self.0 & Self::RAMPAGE != 0
    }
}

/// One eight-byte rapid-trigger record. Distance values are hundredths of a
/// millimeter when nonzero; an all-zero record selects firmware defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RapidTriggerConfig {
    pub switch_type: u8,
    pub flags: RapidTriggerFlags,
    pub actuation: u16,
    pub press_sensitivity: u16,
    pub release_sensitivity: u16,
}

impl RapidTriggerConfig {
    #[must_use]
    pub const fn decode(bytes: [u8; 8]) -> Self {
        Self {
            switch_type: bytes[0],
            flags: RapidTriggerFlags(bytes[1]),
            actuation: u16::from_le_bytes([bytes[2], bytes[3]]),
            press_sensitivity: u16::from_le_bytes([bytes[4], bytes[5]]),
            release_sensitivity: u16::from_le_bytes([bytes[6], bytes[7]]),
        }
    }

    #[must_use]
    pub fn encode(self) -> [u8; 8] {
        let mut bytes = [self.switch_type, self.flags.0, 0, 0, 0, 0, 0, 0];
        bytes[2..4].copy_from_slice(&self.actuation.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.press_sensitivity.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.release_sensitivity.to_le_bytes());
        bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SocdMode {
    Key1Priority = 1,
    Key2Priority = 2,
    LastInputPriority = 3,
    Neutral = 4,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DksRecord {
    pub trigger_points: [u8; 4],
    pub keycodes: [u8; 4],
    /// Four phase bitfields. Bits 0..3 are one-shot actions and bits 4..7 are
    /// held/dragged actions for the four keycodes.
    pub phase_actions: [u8; 4],
}

impl DksRecord {
    #[must_use]
    pub const fn decode(bytes: [u8; 16]) -> Self {
        Self {
            trigger_points: [bytes[0], bytes[1], bytes[2], bytes[3]],
            keycodes: [bytes[5], bytes[7], bytes[9], bytes[11]],
            phase_actions: [bytes[12], bytes[13], bytes[14], bytes[15]],
        }
    }

    #[must_use]
    pub fn encode(&self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[..4].copy_from_slice(&self.trigger_points);
        bytes[5] = self.keycodes[0];
        bytes[7] = self.keycodes[1];
        bytes[9] = self.keycodes[2];
        bytes[11] = self.keycodes[3];
        bytes[12..16].copy_from_slice(&self.phase_actions);
        bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelError {
    BufferTooShort { expected: usize, actual: usize },
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooShort { expected, actual } => {
                write!(
                    formatter,
                    "expected at least {expected} bytes, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for ModelError {}

fn require_len(bytes: &[u8], expected: usize) -> Result<(), ModelError> {
    if bytes.len() < expected {
        Err(ModelError::BufferTooShort {
            expected,
            actual: bytes.len(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_captured_device_info() {
        let bytes = [
            0, 0, 0, 0x92, 0x45, 0x0c, 0xa2, 0x80, 0x52, 1, 0, 0, 0x66, 1, 12, 17, 2, 61, 0, 0, 16,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let info = DeviceInfo::decode(&bytes).unwrap();
        assert_eq!(info.firmware.to_string(), "1.52");
        assert_eq!(info.battery_percent, 61);
        assert_eq!(info.axial_model, 16);
    }

    #[test]
    fn settings_round_trip_captured_values() {
        let bytes = [0, 0, 0, 1, 0, 6, 0, 0, 30, 30, 0, 1, 0, 0, 1, 0];
        let settings = KeyboardSettings::decode(&bytes).unwrap();
        assert_eq!(settings.report_rate, ReportRate::Hz8000);
        assert_eq!(settings.top_dead_zone, 30);
        assert!(settings.stability);
        assert!(settings.adaptive_calibration);
        assert_eq!(settings.encode(), bytes);
    }

    #[test]
    fn lighting_write_adds_driver_signature() {
        let config = LightingConfig {
            effect: 0x0b,
            primary: [255; 3],
            secondary: [0; 3],
            colorful: true,
            brightness: 5,
            speed: 3,
            direction: 0,
            gear: 0,
        };
        let bytes = config.encode();
        assert_eq!(&bytes[14..], &[0xaa, 0x55]);
        assert_eq!(LightingConfig::decode(&bytes).unwrap(), config);

        let mut returned = vec![0; LIGHTING_MAX_LEN];
        returned[13] = 0x7e;
        returned[20] = 0x99;
        let preserving = config.encode_preserving(&returned).unwrap();
        assert_eq!(preserving.len(), LIGHTING_MAX_LEN);
        assert_eq!(preserving[13], 0x7e);
        assert_eq!(preserving[20], 0x99);
        assert_eq!(preserving[4], 0xff);
        assert_eq!(&preserving[14..16], &[0xaa, 0x55]);
    }

    #[test]
    fn all_known_key_assignments_round_trip() {
        let assignments = [
            KeyAssignment::Empty,
            KeyAssignment::Mouse { kind: 1, code: 2 },
            KeyAssignment::Keyboard {
                code: 4,
                special: false,
            },
            KeyAssignment::Keyboard {
                code: 5,
                special: true,
            },
            KeyAssignment::Consumer { usage: 0x0223 },
            KeyAssignment::Macro {
                index: 2,
                mode: 1,
                repeat: 9,
            },
            KeyAssignment::Combo {
                key1: 224,
                key2: 225,
                key3: 4,
            },
            KeyAssignment::Dks { slot: 3 },
            KeyAssignment::ModTap {
                hold: 224,
                tap: 79,
                hold_ms: 400,
            },
            KeyAssignment::Toggle { key: 6 },
            KeyAssignment::Socd {
                mode: 3,
                key1: 4,
                key2: 7,
            },
            KeyAssignment::RappySnappy { key1: 4, key2: 7 },
            KeyAssignment::SpecialFunction { value: 22 },
        ];

        for assignment in assignments {
            assert_eq!(KeyAssignment::decode(assignment.encode()), assignment);
        }
    }

    #[test]
    fn rapid_trigger_round_trip() {
        let config = RapidTriggerConfig {
            switch_type: 6,
            flags: RapidTriggerFlags(RapidTriggerFlags::FULL_TRAVEL | RapidTriggerFlags::RAMPAGE),
            actuation: 120,
            press_sensitivity: 10,
            release_sensitivity: 12,
        };
        assert_eq!(RapidTriggerConfig::decode(config.encode()), config);
        assert!(config.flags.full_travel());
        assert!(config.flags.rampage());
    }

    #[test]
    fn dks_round_trip() {
        let record = DksRecord {
            trigger_points: [16, 30, 30, 16],
            keycodes: [4, 225, 0, 0],
            phase_actions: [0x01, 0x12, 0x20, 0x00],
        };
        assert_eq!(DksRecord::decode(record.encode()), record);
    }
}
