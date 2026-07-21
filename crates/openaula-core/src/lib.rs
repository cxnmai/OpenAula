//! Device protocol and configuration types for OpenAula frontends.
//!
//! This crate deliberately does not contain UI code. It owns the wire format,
//! decoded configuration records, and validation shared by the `aula` CLI and
//! the future desktop frontend.

pub mod dump;
pub mod model;
pub mod protocol;
pub mod transport;

/// USB vendor ID used by the currently supported Mini60 HE Pro.
pub const AULA_VENDOR_ID: u16 = 0x0c45;

/// USB product ID of a wired Mini60 HE Pro.
pub const MINI60_HE_PRO_PRODUCT_ID: u16 = 0x80a2;

/// Product ID used by the official configurator for the wireless dongle.
pub const MINI60_HE_PRO_DONGLE_PRODUCT_ID: u16 = 0xfefe;

/// Identifies a USB HID device without coupling callers to a HID backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeviceId {
    pub vendor_id: u16,
    pub product_id: u16,
}

impl DeviceId {
    pub const MINI60_HE_PRO: Self = Self {
        vendor_id: AULA_VENDOR_ID,
        product_id: MINI60_HE_PRO_PRODUCT_ID,
    };

    pub const MINI60_HE_PRO_DONGLE: Self = Self {
        vendor_id: AULA_VENDOR_ID,
        product_id: MINI60_HE_PRO_DONGLE_PRODUCT_ID,
    };

    /// Returns whether this ID is supported by the current protocol work.
    #[must_use]
    pub const fn is_supported(self) -> bool {
        self.vendor_id == AULA_VENDOR_ID
            && matches!(
                self.product_id,
                MINI60_HE_PRO_PRODUCT_ID | MINI60_HE_PRO_DONGLE_PRODUCT_ID
            )
    }

    /// Whether two endpoints represent the same supported keyboard family.
    /// A wired Mini60 and its wireless dongle are intentionally compatible.
    #[must_use]
    pub const fn is_compatible_with(self, other: Self) -> bool {
        self.is_supported() && other.is_supported()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_known_devices() {
        assert!(DeviceId::MINI60_HE_PRO.is_supported());
        assert!(DeviceId::MINI60_HE_PRO_DONGLE.is_supported());
        assert!(DeviceId::MINI60_HE_PRO.is_compatible_with(DeviceId::MINI60_HE_PRO_DONGLE));
    }

    #[test]
    fn rejects_unknown_devices() {
        assert!(
            !DeviceId {
                vendor_id: AULA_VENDOR_ID,
                product_id: 0xffff,
            }
            .is_supported()
        );
    }
}
