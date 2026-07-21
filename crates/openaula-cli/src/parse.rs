use anyhow::{Context, Result, anyhow, bail, ensure};
use openaula_core::model::KeyAssignment;

pub fn parse_u8(value: &str) -> Result<u8> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix("0x") {
        u8::from_str_radix(hex, 16).with_context(|| format!("invalid byte {value:?}"))
    } else {
        value
            .parse::<u8>()
            .with_context(|| format!("invalid byte {value:?}"))
    }
}

pub fn parse_u16(value: &str) -> Result<u16> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).with_context(|| format!("invalid value {value:?}"))
    } else {
        value
            .parse::<u16>()
            .with_context(|| format!("invalid value {value:?}"))
    }
}

pub fn parse_rgb(value: &str) -> Result<[u8; 3]> {
    let value = value.trim().trim_start_matches('#');
    if value.len() != 6 {
        bail!("RGB color must contain exactly six hexadecimal digits");
    }
    Ok([
        u8::from_str_radix(&value[0..2], 16).context("invalid red channel")?,
        u8::from_str_radix(&value[2..4], 16).context("invalid green channel")?,
        u8::from_str_radix(&value[4..6], 16).context("invalid blue channel")?,
    ])
}

pub fn parse_hex(value: &str) -> Result<Vec<u8>> {
    let compact: String = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && *character != '_')
        .collect();
    if compact.len() % 2 != 0 {
        bail!("hex data must contain an even number of digits");
    }
    (0..compact.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&compact[index..index + 2], 16)
                .with_context(|| format!("invalid hex at character {index}"))
        })
        .collect()
}

pub fn parse_assignment(value: &str) -> Result<KeyAssignment> {
    let value = value.trim();
    if matches!(
        value.to_ascii_lowercase().as_str(),
        "factory" | "default" | "empty" | "clear"
    ) {
        return Ok(KeyAssignment::Empty);
    }
    let parts: Vec<_> = value.split(':').collect();
    let kind = parts[0].to_ascii_lowercase();
    let argument_count = match kind.as_str() {
        "key" | "keyboard" | "extended" | "ext" | "consumer" | "media" | "dks" | "toggle"
        | "tgl" | "special" | "function" | "raw" => 1,
        "mouse" | "rs" => 2,
        "macro" | "combo" | "mt" | "modtap" | "socd" => 3,
        _ => bail!(
            "unknown assignment {kind:?}; use factory, key, extended, mouse, consumer, macro, combo, dks, mt, toggle, socd, rs, special, or raw"
        ),
    };
    ensure!(
        parts.len() == argument_count + 1,
        "assignment {kind:?} requires {argument_count} argument(s), got {}",
        parts.len().saturating_sub(1)
    );
    let arg = |index: usize| {
        parts
            .get(index)
            .copied()
            .ok_or_else(|| anyhow!("missing argument {} in {value:?}", index))
    };

    match kind.as_str() {
        "key" | "keyboard" => Ok(KeyAssignment::Keyboard {
            code: parse_key(arg(1)?)?,
            special: false,
        }),
        "extended" | "ext" => Ok(KeyAssignment::Keyboard {
            code: parse_key(arg(1)?)?,
            special: true,
        }),
        "mouse" => Ok(KeyAssignment::Mouse {
            kind: parse_u8(arg(1)?)?,
            code: parse_u8(arg(2)?)?,
        }),
        "consumer" | "media" => Ok(KeyAssignment::Consumer {
            usage: parse_u16(arg(1)?)?,
        }),
        "macro" => Ok(KeyAssignment::Macro {
            index: parse_u8(arg(1)?)?,
            mode: parse_u8(arg(2)?)?,
            repeat: parse_u8(arg(3)?)?,
        }),
        "combo" => Ok(KeyAssignment::Combo {
            key1: parse_key(arg(1)?)?,
            key2: parse_key(arg(2)?)?,
            key3: parse_key(arg(3)?)?,
        }),
        "dks" => Ok(KeyAssignment::Dks {
            slot: parse_u8(arg(1)?)?,
        }),
        "mt" | "modtap" => Ok(KeyAssignment::ModTap {
            hold: parse_key(arg(1)?)?,
            tap: parse_key(arg(2)?)?,
            hold_ms: parse_u16(arg(3)?)?,
        }),
        "toggle" | "tgl" => Ok(KeyAssignment::Toggle {
            key: parse_key(arg(1)?)?,
        }),
        "socd" => Ok(KeyAssignment::Socd {
            mode: parse_u8(arg(1)?)?,
            key1: parse_key(arg(2)?)?,
            key2: parse_key(arg(3)?)?,
        }),
        "rs" => Ok(KeyAssignment::RappySnappy {
            key1: parse_key(arg(1)?)?,
            key2: parse_key(arg(2)?)?,
        }),
        "special" | "function" => Ok(KeyAssignment::SpecialFunction {
            value: u32::from(parse_u16(arg(1)?)?),
        }),
        "raw" => {
            let bytes = parse_hex(arg(1)?)?;
            let bytes: [u8; 4] = bytes.try_into().map_err(|bytes: Vec<u8>| {
                anyhow!("raw assignment must be 4 bytes, got {}", bytes.len())
            })?;
            Ok(KeyAssignment::Unknown(bytes))
        }
        _ => unreachable!("assignment kinds were validated above"),
    }
}

pub fn parse_key(value: &str) -> Result<u8> {
    let normalized = value
        .trim()
        .to_ascii_lowercase()
        .replace(['_', ' ', '-'], "");
    if normalized.len() == 1 && normalized.as_bytes()[0].is_ascii_digit() {
        return Ok(match normalized.as_bytes()[0] {
            b'1'..=b'9' => normalized.as_bytes()[0] - b'1' + 30,
            b'0' => 39,
            _ => unreachable!("checked ASCII digit"),
        });
    }
    if let Ok(number) = parse_u8(value) {
        return Ok(number);
    }
    let usage = match normalized.as_str() {
        letter if letter.len() == 1 && letter.as_bytes()[0].is_ascii_lowercase() => {
            letter.as_bytes()[0] - b'a' + 4
        }
        "enter" | "return" => 40,
        "esc" | "escape" => 41,
        "backspace" => 42,
        "tab" => 43,
        "space" | "spacebar" => 44,
        "minus" => 45,
        "equal" | "equals" => 46,
        "leftbracket" => 47,
        "rightbracket" => 48,
        "backslash" => 49,
        "semicolon" => 51,
        "quote" | "apostrophe" => 52,
        "backtick" | "grave" => 53,
        "comma" => 54,
        "period" | "dot" => 55,
        "slash" => 56,
        "caps" | "capslock" => 57,
        function
            if function.len() == 2
                && function.as_bytes()[0] == b'f'
                && (b'1'..=b'9').contains(&function.as_bytes()[1]) =>
        {
            function.as_bytes()[1] - b'1' + 58
        }
        "f10" => 67,
        "f11" => 68,
        "f12" => 69,
        "print" | "printscreen" => 70,
        "scroll" | "scrolllock" => 71,
        "pause" => 72,
        "insert" | "ins" => 73,
        "home" => 74,
        "pageup" | "pgup" => 75,
        "delete" | "del" => 76,
        "end" => 77,
        "pagedown" | "pgdn" => 78,
        "right" | "arrowright" => 79,
        "left" | "arrowleft" => 80,
        "down" | "arrowdown" => 81,
        "up" | "arrowup" => 82,
        "menu" | "super" => 101,
        "leftctrl" | "lctrl" => 224,
        "leftshift" | "lshift" => 225,
        "leftalt" | "lalt" => 226,
        "leftwin" | "lwin" | "leftmeta" => 227,
        "rightctrl" | "rctrl" => 228,
        "rightshift" | "rshift" => 229,
        "rightalt" | "ralt" => 230,
        _ => bail!("unknown key name {value:?}; use a USB HID usage number"),
    };
    Ok(usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_colors_and_flexible_numbers() {
        assert_eq!(parse_rgb("#12aBff").unwrap(), [0x12, 0xab, 0xff]);
        assert_eq!(parse_u8("0x80").unwrap(), 128);
    }

    #[test]
    fn parses_named_assignments() {
        assert_eq!(
            parse_assignment("key:a").unwrap(),
            KeyAssignment::Keyboard {
                code: 4,
                special: false
            }
        );
        assert_eq!(
            parse_assignment("mt:lctrl:right:400").unwrap(),
            KeyAssignment::ModTap {
                hold: 224,
                tap: 79,
                hold_ms: 400
            }
        );
        assert_eq!(parse_key("1").unwrap(), 30);
        assert_eq!(parse_key("0x01").unwrap(), 1);
        assert!(parse_assignment("key:a:extra").is_err());
    }

    #[test]
    fn parses_raw_without_changing_unknown_bytes() {
        assert_eq!(
            parse_assignment("RAW:de_ad_be_ef").unwrap(),
            KeyAssignment::Unknown([0xde, 0xad, 0xbe, 0xef])
        );
    }
}
