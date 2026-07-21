//! Versioned, lossless configuration backups.

use core::fmt;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::{
    DEVICE_INFO_LEN, DKS_BUFFER_LEN, FN_KEYMAP_LEN, KEYBOARD_SETTINGS_LEN, KEYMAP_LEN,
    LIGHTING_MAX_LEN, MACRO_BUFFER_LEN, PER_KEY_LIGHTING_LEN, RAPID_TRIGGER_LEN,
};
use crate::transport::DeviceDescriptor;

pub const DUMP_SCHEMA_VERSION: u32 = 1;

/// Bytes serialized as a compact lowercase hexadecimal string.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HexBytes(pub Vec<u8>);

impl HexBytes {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for HexBytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Serialize for HexBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut text = String::with_capacity(self.0.len() * 2);
        for byte in &self.0 {
            use core::fmt::Write;
            write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
        }
        serializer.serialize_str(&text)
    }
}

impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HexVisitor;

        impl Visitor<'_> for HexVisitor {
            type Value = HexBytes;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an even-length hexadecimal string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value.len() % 2 != 0 {
                    return Err(E::custom("hexadecimal string has odd length"));
                }
                let mut bytes = Vec::with_capacity(value.len() / 2);
                for index in (0..value.len()).step_by(2) {
                    let byte = u8::from_str_radix(&value[index..index + 2], 16)
                        .map_err(|_| E::custom(format!("invalid hex at character {index}")))?;
                    bytes.push(byte);
                }
                Ok(HexBytes(bytes))
            }
        }

        deserializer.deserialize_str(HexVisitor)
    }
}

/// Every Mini60 configuration section needed for an exact round trip.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigurationDump {
    pub schema_version: u32,
    pub captured_unix_ms: u64,
    pub device: DeviceDescriptor,
    pub device_info: HexBytes,
    pub keyboard_settings: HexBytes,
    pub keymap: HexBytes,
    pub lighting: HexBytes,
    pub per_key_lighting: HexBytes,
    pub macros: HexBytes,
    pub fn_keymap: HexBytes,
    pub rapid_trigger: HexBytes,
    pub dks: HexBytes,
}

impl ConfigurationDump {
    pub fn validate(&self) -> Result<(), DumpError> {
        if self.schema_version != DUMP_SCHEMA_VERSION {
            return Err(DumpError::UnsupportedSchema(self.schema_version));
        }
        if !self.device.id().is_supported() {
            return Err(DumpError::UnsupportedDevice {
                vendor_id: self.device.vendor_id,
                product_id: self.device.product_id,
            });
        }
        require_section("device_info", &self.device_info, DEVICE_INFO_LEN)?;
        require_section(
            "keyboard_settings",
            &self.keyboard_settings,
            KEYBOARD_SETTINGS_LEN,
        )?;
        require_section("keymap", &self.keymap, KEYMAP_LEN)?;
        if !matches!(self.lighting.0.len(), 16 | LIGHTING_MAX_LEN) {
            return Err(DumpError::InvalidSectionLengthSet {
                section: "lighting",
                first: 16,
                second: LIGHTING_MAX_LEN,
                actual: self.lighting.0.len(),
            });
        }
        require_section(
            "per_key_lighting",
            &self.per_key_lighting,
            PER_KEY_LIGHTING_LEN,
        )?;
        require_section("macros", &self.macros, MACRO_BUFFER_LEN)?;
        require_section("fn_keymap", &self.fn_keymap, FN_KEYMAP_LEN)?;
        require_section("rapid_trigger", &self.rapid_trigger, RAPID_TRIGGER_LEN)?;
        require_section("dks", &self.dks, DKS_BUFFER_LEN)?;
        Ok(())
    }
}

fn require_section(
    section: &'static str,
    bytes: &HexBytes,
    expected: usize,
) -> Result<(), DumpError> {
    if bytes.0.len() == expected {
        Ok(())
    } else {
        Err(DumpError::InvalidSectionLength {
            section,
            expected,
            actual: bytes.0.len(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DumpError {
    UnsupportedSchema(u32),
    UnsupportedDevice {
        vendor_id: u16,
        product_id: u16,
    },
    InvalidSectionLength {
        section: &'static str,
        expected: usize,
        actual: usize,
    },
    InvalidSectionLengthSet {
        section: &'static str,
        first: usize,
        second: usize,
        actual: usize,
    },
}

impl fmt::Display for DumpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported dump schema version {version}")
            }
            Self::UnsupportedDevice {
                vendor_id,
                product_id,
            } => write!(
                formatter,
                "unsupported backup device {vendor_id:04x}:{product_id:04x}"
            ),
            Self::InvalidSectionLength {
                section,
                expected,
                actual,
            } => write!(
                formatter,
                "section {section} must be {expected} bytes, got {actual}"
            ),
            Self::InvalidSectionLengthSet {
                section,
                first,
                second,
                actual,
            } => write!(
                formatter,
                "section {section} must be {first} or {second} bytes, got {actual}"
            ),
        }
    }
}

impl std::error::Error for DumpError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_bytes_round_trip_json() {
        let bytes = HexBytes(vec![0, 1, 0xaa, 0xff]);
        let json = serde_json::to_string(&bytes).unwrap();
        assert_eq!(json, "\"0001aaff\"");
        assert_eq!(serde_json::from_str::<HexBytes>(&json).unwrap(), bytes);
    }

    #[test]
    fn rejects_odd_hex() {
        assert!(serde_json::from_str::<HexBytes>("\"abc\"").is_err());
    }
}
