use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail, ensure};
use openaula_core::dump::ConfigurationDump;
use openaula_core::model::{
    DEVICE_INFO_LEN, DKS_BUFFER_LEN, DeviceInfo, FN_KEYMAP_LEN, KEYBOARD_SETTINGS_LEN, KEYMAP_LEN,
    KeyAssignment, KeyboardSettings, LIGHTING_LEN, LIGHTING_MAX_LEN, LightingConfig,
    MACRO_BUFFER_LEN, PER_KEY_LIGHTING_LEN, RAPID_TRIGGER_LEN, RapidTriggerConfig,
    RapidTriggerFlags, ReportRate,
};
use openaula_core::protocol::Command;
use openaula_core::transport::{Keyboard, discover_devices, open_device};
use serde::Serialize;
use serde_json::json;

use crate::args::{
    CalibrationCommand, Cli, KeyTestCommand, KeymapCommand, Layer, LightingCommand,
    PerformanceCommand, ResetTarget, SectionCommand, SectionName, SettingsCommand, TopCommand,
    WriteGuard,
};
use crate::parse::{parse_assignment, parse_hex, parse_rgb, parse_u8};

const PHYSICAL_KEYS: &[(u8, &str)] = &[
    (0, "Esc"),
    (17, "1"),
    (18, "2"),
    (19, "3"),
    (20, "4"),
    (21, "5"),
    (22, "6"),
    (23, "7"),
    (24, "8"),
    (25, "9"),
    (26, "0"),
    (27, "-"),
    (28, "="),
    (92, "Backspace"),
    (32, "Tab"),
    (33, "Q"),
    (34, "W"),
    (35, "E"),
    (36, "R"),
    (37, "T"),
    (38, "Y"),
    (39, "U"),
    (40, "I"),
    (41, "O"),
    (42, "P"),
    (43, "["),
    (44, "]"),
    (60, "Backslash"),
    (48, "Caps"),
    (49, "A"),
    (50, "S"),
    (51, "D"),
    (52, "F"),
    (53, "G"),
    (54, "H"),
    (55, "J"),
    (56, "K"),
    (57, "L"),
    (58, ";"),
    (59, "Quote"),
    (76, "Enter"),
    (64, "Left Shift"),
    (65, "Z"),
    (66, "X"),
    (67, "C"),
    (68, "V"),
    (69, "B"),
    (70, "N"),
    (71, "M"),
    (72, "Comma"),
    (73, "Period"),
    (74, "Slash"),
    (75, "Right Shift"),
    (80, "Left Ctrl"),
    (81, "Left Win"),
    (82, "Left Alt"),
    (83, "Space"),
    (84, "Right Alt"),
    (86, "Menu/Super"),
    (87, "Right Ctrl"),
    (85, "Fn"),
];

struct Runtime {
    selector: Option<String>,
    timeout_ms: u64,
    json: bool,
}

pub fn run(cli: Cli) -> Result<()> {
    let runtime = Runtime {
        selector: cli.device,
        timeout_ms: cli.timeout_ms,
        json: cli.json,
    };
    match cli.command {
        TopCommand::Devices => devices(&runtime),
        TopCommand::Info => info(&runtime),
        TopCommand::Dump { output } => dump(&runtime, output.as_deref()),
        TopCommand::Restore { file, guard } => restore(&runtime, &file, &guard),
        TopCommand::Settings { command } => settings(&runtime, command),
        TopCommand::Lighting { command } => lighting(&runtime, command),
        TopCommand::Keymap { command } => keymap(&runtime, command),
        TopCommand::Performance { command } => performance(&runtime, command),
        TopCommand::Section { command } => section(&runtime, command),
        TopCommand::Calibration { command } => calibration(&runtime, command),
        TopCommand::KeyTest { command } => key_test(&runtime, command),
        TopCommand::Reset {
            target,
            apply,
            confirm,
            backup,
        } => reset(
            &runtime,
            target,
            apply,
            confirm.as_deref(),
            backup.as_deref(),
        ),
    }
}

fn keyboard(runtime: &Runtime) -> Result<Keyboard> {
    open_device(runtime.selector.as_deref(), runtime.timeout_ms).map_err(Into::into)
}

fn devices(runtime: &Runtime) -> Result<()> {
    let devices = discover_devices()?;
    if runtime.json {
        print_json(&devices)?;
        return Ok(());
    }
    if devices.is_empty() {
        println!("No supported AULA configuration interfaces found.");
        return Ok(());
    }
    println!("INDEX  VID:PID    REPORT  USAGE       PRODUCT  PATH");
    for (index, device) in devices.iter().enumerate() {
        println!(
            "{index:<5}  {:04x}:{:04x}  {:>3} B   {:04x}:{:02x}  {}  {}",
            device.vendor_id,
            device.product_id,
            device.report_len,
            device.usage_page,
            device.usage,
            device.product_name,
            device.path
        );
    }
    Ok(())
}

fn info(runtime: &Runtime) -> Result<()> {
    let keyboard = keyboard(runtime)?;
    let device_raw = keyboard.read_bulk(Command::ReadDeviceInfo, DEVICE_INFO_LEN)?;
    let settings_raw = keyboard.read_bulk(Command::ReadKeyboardSettings, KEYBOARD_SETTINGS_LEN)?;
    let lighting_raw = keyboard.read_bulk(Command::ReadLighting, LIGHTING_LEN)?;
    keyboard.commit()?;
    let device = DeviceInfo::decode(&device_raw)?;
    let settings = KeyboardSettings::decode(&settings_raw)?;
    let lighting = LightingConfig::decode(&lighting_raw)?;

    if runtime.json {
        print_json(&json!({
            "device": keyboard.descriptor(),
            "device_info": device,
            "settings": settings,
            "lighting": lighting,
        }))?;
    } else {
        let descriptor = keyboard.descriptor();
        println!("{}", descriptor.product_name);
        println!(
            "  endpoint: {:04x}:{:04x} {}-byte report ({})",
            descriptor.vendor_id, descriptor.product_id, descriptor.report_len, descriptor.path
        );
        println!("  firmware: {}", device.firmware);
        println!("  battery: {}%", device.battery_percent);
        println!("  axial model: {}", device.axial_model);
        print_settings(&settings);
        print_lighting(&lighting);
    }
    Ok(())
}

fn dump(runtime: &Runtime, output: Option<&Path>) -> Result<()> {
    let keyboard = keyboard(runtime)?;
    let dump = keyboard.dump()?;
    let text = serde_json::to_string_pretty(&dump)?;
    match output {
        Some(path) if path.as_os_str() != "-" => {
            write_new(path, text.as_bytes())?;
            if runtime.json {
                print_json(&json!({"output": path, "bytes": text.len()}))?;
            } else {
                println!("Saved {}-byte backup to {}", text.len(), path.display());
            }
        }
        _ => println!("{text}"),
    }
    Ok(())
}

fn restore(runtime: &Runtime, file: &Path, guard: &WriteGuard) -> Result<()> {
    let text = fs::read_to_string(file)
        .with_context(|| format!("failed to read backup {}", file.display()))?;
    let expected: ConfigurationDump = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse backup {}", file.display()))?;
    expected.validate()?;

    if !guard.apply {
        let summary = json!({
            "dry_run": true,
            "schema_version": expected.schema_version,
            "source_device": expected.device,
            "sections": dump_section_lengths(&expected),
        });
        if runtime.json {
            print_json(&summary)?;
        } else {
            println!("Dry run: backup is valid; no device was modified.");
            println!(
                "Source: {} ({:04x}:{:04x})",
                expected.device.product_name, expected.device.vendor_id, expected.device.product_id
            );
            println!(
                "Run again with --apply to back up the current keyboard, restore, and verify."
            );
        }
        return Ok(());
    }

    let keyboard = keyboard(runtime)?;
    let backup = automatic_backup(&keyboard, guard.backup.as_deref())?;
    keyboard.restore(&expected)?;
    let actual = keyboard.dump()?;
    let differences = compare_dumps(&expected, &actual)?;
    if !differences.is_empty() {
        bail!(
            "restore readback differed in {}; original keyboard backup is {}",
            differences.join(", "),
            backup.display()
        );
    }
    write_result(runtime, "restore", &backup, json!({"source": file}))
}

fn settings(runtime: &Runtime, command: SettingsCommand) -> Result<()> {
    let keyboard = keyboard(runtime)?;
    let mut raw = keyboard.read_bulk(Command::ReadKeyboardSettings, KEYBOARD_SETTINGS_LEN)?;
    keyboard.commit()?;
    match command {
        SettingsCommand::Get => {
            let decoded = KeyboardSettings::decode(&raw)?;
            if runtime.json {
                print_json(&decoded)?;
            } else {
                print_settings(&decoded);
            }
        }
        SettingsCommand::Set {
            sleep_minutes,
            response_time,
            report_rate,
            os_mode,
            tft_timeout,
            top_dead_zone,
            bottom_dead_zone,
            stability,
            adaptive_calibration,
            wake,
            guard,
        } => {
            let before = raw.clone();
            if let Some(value) = sleep_minutes {
                ensure!(value <= 30, "sleep timeout must be 0-30 minutes");
                raw[3] = value;
            }
            if let Some(value) = response_time {
                raw[4] = value;
            }
            if let Some(value) = report_rate {
                raw[5] = match value {
                    1_000 => ReportRate::Hz1000.encode(),
                    4_000 => ReportRate::Hz4000.encode(),
                    8_000 => ReportRate::Hz8000.encode(),
                    _ => bail!("report rate must be 1000, 4000, or 8000"),
                };
            }
            if let Some(value) = os_mode {
                raw[6] = value;
            }
            if let Some(value) = tft_timeout {
                raw[7] = value;
            }
            if let Some(value) = top_dead_zone {
                raw[8] = mm_u8(value, 0.50, "top dead zone")?;
            }
            if let Some(value) = bottom_dead_zone {
                raw[9] = mm_u8(value, 0.50, "bottom dead zone")?;
            }
            if let Some(value) = stability {
                raw[11] = u8::from(value);
            }
            if let Some(value) = adaptive_calibration {
                raw[14] = u8::from(value);
            }
            if let Some(value) = wake {
                raw[15] = u8::from(value);
            }
            ensure!(before[..16] != raw[..16], "no setting would change");
            let preview = KeyboardSettings::decode(&raw)?;
            if !guard.apply {
                return dry_run(runtime, "settings", &preview);
            }
            let backup = automatic_backup(&keyboard, guard.backup.as_deref())?;
            keyboard.write_bulk(Command::WriteKeyboardSettings, &raw[..16])?;
            let readback =
                keyboard.read_bulk(Command::ReadKeyboardSettings, KEYBOARD_SETTINGS_LEN)?;
            keyboard.commit()?;
            ensure!(
                readback[..16] == raw[..16],
                "settings verification failed; backup is {}",
                backup.display()
            );
            write_result(runtime, "settings", &backup, json!({"settings": preview}))?;
        }
    }
    Ok(())
}

fn lighting(runtime: &Runtime, command: LightingCommand) -> Result<()> {
    let keyboard = keyboard(runtime)?;
    let raw = keyboard.read_bulk(Command::ReadLighting, LIGHTING_LEN)?;
    keyboard.commit()?;
    match command {
        LightingCommand::Get => {
            let decoded = LightingConfig::decode(&raw)?;
            if runtime.json {
                print_json(&decoded)?;
            } else {
                print_lighting(&decoded);
            }
        }
        LightingCommand::Set {
            effect,
            color,
            secondary,
            colorful,
            brightness,
            speed,
            direction,
            gear,
            guard,
        } => {
            let mut decoded = LightingConfig::decode(&raw)?;
            let before = decoded;
            if let Some(value) = effect {
                decoded.effect = parse_u8(&value)?;
            }
            if let Some(value) = color {
                decoded.primary = parse_rgb(&value)?;
            }
            if let Some(value) = secondary {
                decoded.secondary = parse_rgb(&value)?;
            }
            if let Some(value) = colorful {
                decoded.colorful = value;
            }
            if let Some(value) = brightness {
                ensure!((1..=5).contains(&value), "brightness must be 1-5");
                decoded.brightness = value;
            }
            if let Some(value) = speed {
                ensure!((1..=5).contains(&value), "speed must be 1-5");
                decoded.speed = value;
            }
            if let Some(value) = direction {
                ensure!(value <= 1, "direction must be 0 or 1");
                decoded.direction = value;
            }
            if let Some(value) = gear {
                decoded.gear = value;
            }
            ensure!(before != decoded, "no lighting value would change");
            if !guard.apply {
                return dry_run(runtime, "lighting", &decoded);
            }
            let backup = automatic_backup(&keyboard, guard.backup.as_deref())?;
            let encoded = decoded.encode_preserving(&raw)?;
            keyboard.write_bulk(Command::WriteLighting, &encoded)?;
            let readback = keyboard.read_bulk(Command::ReadLighting, LIGHTING_LEN)?;
            keyboard.commit()?;
            ensure!(
                LightingConfig::decode(&readback)? == decoded,
                "lighting verification failed; backup is {}",
                backup.display()
            );
            write_result(runtime, "lighting", &backup, json!({"lighting": decoded}))?;
        }
    }
    Ok(())
}

fn keymap(runtime: &Runtime, command: KeymapCommand) -> Result<()> {
    let keyboard = keyboard(runtime)?;
    match command {
        KeymapCommand::Get { layer, slot } => {
            let (read_command, _) = layer_commands(layer);
            let raw = keyboard.read_bulk(read_command, KEYMAP_LEN)?;
            keyboard.commit()?;
            let slots: Vec<_> = if let Some(slot) = slot {
                ensure!(slot < 128, "slot must be 0-127");
                vec![(slot, slot_name(slot))]
            } else {
                PHYSICAL_KEYS.to_vec()
            };
            let records: Vec<_> = slots
                .iter()
                .map(|(slot, name)| {
                    let assignment = assignment_at(&raw, *slot);
                    json!({"slot": slot, "key": name, "assignment": assignment, "raw": hex(&assignment.encode())})
                })
                .collect();
            if runtime.json {
                print_json(
                    &json!({"layer": format!("{layer:?}").to_ascii_lowercase(), "records": records}),
                )?;
            } else {
                println!("{:>4}  {:<14}  ASSIGNMENT", "SLOT", "KEY");
                for (slot, name) in slots {
                    println!(
                        "{slot:>4}  {name:<14}  {}",
                        format_assignment(assignment_at(&raw, slot))
                    );
                }
            }
        }
        KeymapCommand::Set {
            layer,
            slot,
            pair_slot,
            action,
            guard,
        } => {
            ensure!(slot < 128, "slot must be 0-127");
            let assignment = parse_assignment(&action)?;
            validate_assignment(assignment)?;
            let mut targets = vec![slot];
            if matches!(
                assignment,
                KeyAssignment::Socd { .. } | KeyAssignment::RappySnappy { .. }
            ) {
                let pair_slot = pair_slot
                    .ok_or_else(|| anyhow!("SOCD and Rappy Snappy require --pair-slot"))?;
                ensure!(pair_slot < 128, "pair slot must be 0-127");
                ensure!(pair_slot != slot, "pair slot must differ from slot");
                targets.push(pair_slot);
            } else {
                ensure!(
                    pair_slot.is_none(),
                    "--pair-slot is only valid for SOCD or Rappy Snappy"
                );
            }
            let (read_command, write_command) = layer_commands(layer);
            let mut raw = keyboard.read_bulk(read_command, KEYMAP_LEN)?;
            keyboard.commit()?;
            let encoded = assignment.encode();
            ensure!(
                targets.iter().any(|target| {
                    let start = usize::from(*target) * 4;
                    raw[start..start + 4] != encoded
                }),
                "assignment is already set on every target slot"
            );
            for target in &targets {
                let start = usize::from(*target) * 4;
                raw[start..start + 4].copy_from_slice(&encoded);
            }
            let target_preview: Vec<_> = targets
                .iter()
                .map(|target| json!({"slot": target, "key": slot_name(*target)}))
                .collect();
            let preview = json!({"layer": format!("{layer:?}").to_ascii_lowercase(), "targets": target_preview, "assignment": assignment});
            if !guard.apply {
                return dry_run(runtime, "keymap", &preview);
            }
            let backup = automatic_backup(&keyboard, guard.backup.as_deref())?;
            keyboard.write_bulk(write_command, &raw)?;
            let readback = keyboard.read_bulk(read_command, KEYMAP_LEN)?;
            keyboard.commit()?;
            ensure!(
                readback == raw,
                "keymap verification failed; backup is {}",
                backup.display()
            );
            write_result(runtime, "keymap", &backup, preview)?;
        }
    }
    Ok(())
}

fn performance(runtime: &Runtime, command: PerformanceCommand) -> Result<()> {
    let keyboard = keyboard(runtime)?;
    let mut raw = keyboard.read_bulk(Command::ReadRapidTrigger, RAPID_TRIGGER_LEN)?;
    keyboard.commit()?;
    match command {
        PerformanceCommand::Get { slot } => {
            let slots: Vec<_> = if let Some(slot) = slot {
                ensure!(slot < 128, "slot must be 0-127");
                vec![(slot, slot_name(slot))]
            } else {
                PHYSICAL_KEYS.to_vec()
            };
            let records: Vec<_> = slots
                .iter()
                .map(|(slot, name)| {
                    let record = rt_at(&raw, *slot);
                    json!({"slot": slot, "key": name, "record": record, "millimeters": rt_mm(record)})
                })
                .collect();
            if runtime.json {
                print_json(&records)?;
            } else {
                println!(
                    "{:>4}  {:<14}  SWITCH FLAGS  ACTUATION PRESS RELEASE",
                    "SLOT", "KEY"
                );
                for (slot, name) in slots {
                    let record = rt_at(&raw, slot);
                    println!(
                        "{slot:>4}  {name:<14}  {:>6}  0x{:02x}   {:>5.2} mm  {:>5.2}  {:>5.2}",
                        record.switch_type,
                        record.flags.0,
                        f32::from(record.actuation) / 100.0,
                        f32::from(record.press_sensitivity) / 100.0,
                        f32::from(record.release_sensitivity) / 100.0,
                    );
                }
            }
        }
        PerformanceCommand::Set {
            slots,
            switch_type,
            actuation,
            press,
            release,
            full_travel,
            bottom_optimization,
            rampage,
            guard,
        } => {
            ensure!(slots.iter().all(|slot| *slot < 128), "slots must be 0-127");
            ensure!(
                switch_type.is_some()
                    || actuation.is_some()
                    || press.is_some()
                    || release.is_some()
                    || full_travel.is_some()
                    || bottom_optimization.is_some()
                    || rampage.is_some(),
                "no performance value was supplied"
            );
            let before = raw.clone();
            let mut preview = Vec::new();
            for slot in &slots {
                let mut record = rt_at(&raw, *slot);
                if let Some(value) = switch_type {
                    ensure!(value <= 8, "Mini60 switch type must be 0-8");
                    record.switch_type = value;
                }
                if let Some(value) = actuation {
                    record.actuation = mm_u16(value, 3.40, "actuation")?;
                }
                if let Some(value) = press {
                    record.press_sensitivity = mm_u16(value, 3.40, "press sensitivity")?;
                }
                if let Some(value) = release {
                    record.release_sensitivity = mm_u16(value, 3.40, "release sensitivity")?;
                }
                set_flag(
                    &mut record.flags.0,
                    RapidTriggerFlags::FULL_TRAVEL,
                    full_travel,
                );
                set_flag(
                    &mut record.flags.0,
                    RapidTriggerFlags::BOTTOM_OPTIMIZATION,
                    bottom_optimization,
                );
                set_flag(&mut record.flags.0, RapidTriggerFlags::RAMPAGE, rampage);
                let start = usize::from(*slot) * 8;
                raw[start..start + 8].copy_from_slice(&record.encode());
                preview.push(json!({"slot": slot, "key": slot_name(*slot), "record": record, "millimeters": rt_mm(record)}));
            }
            ensure!(before != raw, "performance records are already set");
            if !guard.apply {
                return dry_run(runtime, "performance", &preview);
            }
            let backup = automatic_backup(&keyboard, guard.backup.as_deref())?;
            keyboard.write_bulk(Command::WriteRapidTrigger, &raw)?;
            let readback = keyboard.read_bulk(Command::ReadRapidTrigger, RAPID_TRIGGER_LEN)?;
            keyboard.commit()?;
            ensure!(
                readback == raw,
                "performance verification failed; backup is {}",
                backup.display()
            );
            write_result(runtime, "performance", &backup, json!({"records": preview}))?;
        }
    }
    Ok(())
}

fn section(runtime: &Runtime, command: SectionCommand) -> Result<()> {
    let keyboard = keyboard(runtime)?;
    match command {
        SectionCommand::Read { section, output } => {
            let spec = section_spec(section);
            let bytes = keyboard.read_bulk(spec.read, spec.read_len)?;
            keyboard.commit()?;
            if let Some(path) = output {
                write_new(&path, &bytes)?;
                if runtime.json {
                    print_json(
                        &json!({"section": spec.name, "output": path, "bytes": bytes.len()}),
                    )?;
                } else {
                    println!(
                        "Saved {} bytes from {} to {}",
                        bytes.len(),
                        spec.name,
                        path.display()
                    );
                }
            } else if runtime.json {
                print_json(
                    &json!({"section": spec.name, "bytes": bytes.len(), "hex": hex(&bytes)}),
                )?;
            } else {
                println!("{}", hex(&bytes));
            }
        }
        SectionCommand::Write {
            section,
            hex: hex_input,
            file,
            guard,
        } => {
            let spec = section_spec(section);
            let write_command = spec
                .write
                .ok_or_else(|| anyhow!("{} is read-only", spec.name))?;
            let input = match (hex_input, file) {
                (Some(value), None) => parse_hex(&value)?,
                (None, Some(path)) => {
                    fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?
                }
                _ => bail!("supply exactly one of --hex or --file"),
            };
            let payload =
                if section == SectionName::Settings && input.len() == KEYBOARD_SETTINGS_LEN {
                    &input[..16]
                } else {
                    &input
                };
            if section == SectionName::Lighting {
                ensure!(
                    matches!(payload.len(), LIGHTING_LEN | LIGHTING_MAX_LEN),
                    "lighting write requires {LIGHTING_LEN} or {LIGHTING_MAX_LEN} bytes, got {}",
                    payload.len()
                );
            } else {
                ensure!(
                    payload.len() == spec.write_len,
                    "{} write requires {} bytes, got {}",
                    spec.name,
                    spec.write_len,
                    payload.len()
                );
            }
            if !guard.apply {
                let preview =
                    json!({"section": spec.name, "bytes": payload.len(), "hex": hex(payload)});
                return dry_run(runtime, "section", &preview);
            }
            let backup = automatic_backup(&keyboard, guard.backup.as_deref())?;
            keyboard.write_bulk(write_command, payload)?;
            let readback = keyboard.read_bulk(spec.read, spec.read_len)?;
            keyboard.commit()?;
            let verified = if section == SectionName::Lighting {
                LightingConfig::decode(&readback)? == LightingConfig::decode(payload)?
            } else {
                readback[..spec.write_len] == *payload
            };
            ensure!(
                verified,
                "{} verification failed; backup is {}",
                spec.name,
                backup.display()
            );
            write_result(runtime, "section", &backup, json!({"section": spec.name}))?;
        }
    }
    Ok(())
}

fn calibration(runtime: &Runtime, command: CalibrationCommand) -> Result<()> {
    match command {
        CalibrationCommand::Start {
            apply,
            confirm,
            backup,
        } => {
            if !apply {
                return control_dry_run(runtime, "start calibration", "CALIBRATE");
            }
            ensure!(
                confirm.as_deref() == Some("CALIBRATE"),
                "calibration requires --confirm CALIBRATE"
            );
            let keyboard = keyboard(runtime)?;
            let backup = automatic_backup(&keyboard, backup.as_deref())?;
            keyboard.send_short(Command::StartCalibration, &[0, 0, 0, 0], false)?;
            if runtime.json {
                print_json(
                    &json!({"applied": true, "operation": "calibration-start", "backup": backup}),
                )?;
            } else {
                println!("Calibration started. Fully press every key, then run:");
                println!("  aula calibration save --apply");
                println!("Backup: {}", backup.display());
            }
        }
        CalibrationCommand::Save { apply } => {
            if !apply {
                return control_dry_run(runtime, "save calibration", "--apply");
            }
            let keyboard = keyboard(runtime)?;
            keyboard.send_short(Command::StopCalibration, &[0, 0, 0, 0], false)?;
            if runtime.json {
                print_json(&json!({"applied": true, "operation": "calibration-save"}))?;
            } else {
                println!("Calibration save command sent.");
            }
        }
    }
    Ok(())
}

fn key_test(runtime: &Runtime, command: KeyTestCommand) -> Result<()> {
    let (start, apply) = match command {
        KeyTestCommand::Start { apply } => (true, apply),
        KeyTestCommand::Stop { apply } => (false, apply),
    };
    if !apply {
        return control_dry_run(
            runtime,
            if start {
                "start key test"
            } else {
                "stop key test"
            },
            "--apply",
        );
    }
    let keyboard = keyboard(runtime)?;
    let command = if start {
        Command::StartKeyTest
    } else {
        Command::StopKeyTest
    };
    keyboard.send_short(command, &[0, 0, 0, 0], false)?;
    if runtime.json {
        print_json(&json!({"applied": true, "key_test": start}))?;
    } else {
        println!(
            "Key-test mode {} command sent.",
            if start { "start" } else { "stop" }
        );
    }
    Ok(())
}

fn reset(
    runtime: &Runtime,
    target: ResetTarget,
    apply: bool,
    confirm: Option<&str>,
    backup_path: Option<&Path>,
) -> Result<()> {
    let (subcode, phrase, name) = match target {
        ResetTarget::Keys => (0x05, "RESET KEYS", "key configuration"),
        ResetTarget::All => (0xff, "RESET ALL", "all device settings"),
    };
    if !apply {
        return control_dry_run(runtime, &format!("reset {name}"), phrase);
    }
    ensure!(
        confirm == Some(phrase),
        "reset requires --confirm {phrase:?}"
    );
    let keyboard = keyboard(runtime)?;
    let backup = automatic_backup(&keyboard, backup_path)?;
    keyboard.send_short(Command::Reset, &[subcode, 0, 0, 0], false)?;
    if runtime.json {
        print_json(
            &json!({"applied": true, "reset": format!("{target:?}").to_ascii_lowercase(), "backup": backup}),
        )?;
    } else {
        println!("Reset command sent for {name}.");
        println!("Backup: {}", backup.display());
    }
    Ok(())
}

fn automatic_backup(keyboard: &Keyboard, requested: Option<&Path>) -> Result<PathBuf> {
    let dump = keyboard.dump()?;
    let text = serde_json::to_string_pretty(&dump)?;
    if let Some(path) = requested {
        write_new(path, text.as_bytes())?;
        return Ok(path.to_owned());
    }

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    for suffix in 0..100_u8 {
        let name = if suffix == 0 {
            format!("aula-backup-{millis}.json")
        } else {
            format!("aula-backup-{millis}-{suffix}.json")
        };
        let path = PathBuf::from(name);
        match write_new(&path, text.as_bytes()) {
            Ok(()) => return Ok(path),
            Err(error)
                if error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|io| io.kind() == std::io::ErrorKind::AlreadyExists) =>
            {
                continue;
            }
            Err(error) => return Err(error),
        }
    }
    bail!("could not allocate a unique automatic backup filename")
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("refusing to overwrite {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("failed to write {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to flush {}", path.display()))?;
    Ok(())
}

fn compare_dumps(
    expected: &ConfigurationDump,
    actual: &ConfigurationDump,
) -> Result<Vec<&'static str>> {
    let mut differences = Vec::new();
    if expected.keyboard_settings.as_slice()[..16] != actual.keyboard_settings.as_slice()[..16] {
        differences.push("settings");
    }
    if expected.keymap != actual.keymap {
        differences.push("keymap");
    }
    if LightingConfig::decode(expected.lighting.as_slice())?
        != LightingConfig::decode(actual.lighting.as_slice())?
    {
        differences.push("lighting");
    }
    if expected.per_key_lighting != actual.per_key_lighting {
        differences.push("per-key-lighting");
    }
    if expected.macros != actual.macros {
        differences.push("macros");
    }
    if expected.fn_keymap != actual.fn_keymap {
        differences.push("fn-keymap");
    }
    if expected.rapid_trigger != actual.rapid_trigger {
        differences.push("rapid-trigger");
    }
    if expected.dks != actual.dks {
        differences.push("dks");
    }
    Ok(differences)
}

fn print_settings(settings: &KeyboardSettings) {
    println!("Settings:");
    if settings.sleep_minutes == 0 {
        println!("  sleep: off");
    } else {
        println!("  sleep: {} minute(s)", settings.sleep_minutes);
    }
    println!("  response time: {}", settings.response_time);
    println!("  report rate: {}", report_rate_name(settings.report_rate));
    println!("  OS mode: {}", settings.os_mode);
    println!("  TFT timeout: {}", settings.tft_timeout);
    println!(
        "  dead zones: {:.2} mm top / {:.2} mm bottom",
        f32::from(settings.top_dead_zone) / 100.0,
        f32::from(settings.bottom_dead_zone) / 100.0
    );
    println!("  stability: {}", on_off(settings.stability));
    println!(
        "  adaptive calibration: {}",
        on_off(settings.adaptive_calibration)
    );
    println!(
        "  wake mode: {}",
        if settings.wake {
            "single-key"
        } else {
            "all-key"
        }
    );
}

fn print_lighting(lighting: &LightingConfig) {
    println!("Lighting:");
    println!("  effect: 0x{:02x}", lighting.effect);
    println!(
        "  primary: #{:02x}{:02x}{:02x}",
        lighting.primary[0], lighting.primary[1], lighting.primary[2]
    );
    println!(
        "  secondary: #{:02x}{:02x}{:02x}",
        lighting.secondary[0], lighting.secondary[1], lighting.secondary[2]
    );
    println!("  colorful: {}", on_off(lighting.colorful));
    println!("  brightness: {}", lighting.brightness);
    println!("  speed: {}", lighting.speed);
    println!("  direction: {}", lighting.direction);
    println!("  gear: {}", lighting.gear);
}

fn report_rate_name(rate: ReportRate) -> String {
    match rate {
        ReportRate::Hz1000 => "1000 Hz".to_owned(),
        ReportRate::Hz4000 => "4000 Hz".to_owned(),
        ReportRate::Hz8000 => "8000 Hz".to_owned(),
        ReportRate::Unknown(value) => format!("unknown code {value}"),
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

fn layer_commands(layer: Layer) -> (Command, Command) {
    match layer {
        Layer::Normal => (Command::ReadKeymap, Command::WriteKeymap),
        Layer::Fn => (Command::ReadFnKeymap, Command::WriteFnKeymap),
    }
}

fn assignment_at(raw: &[u8], slot: u8) -> KeyAssignment {
    let start = usize::from(slot) * 4;
    KeyAssignment::decode(
        raw[start..start + 4]
            .try_into()
            .expect("validated keymap length"),
    )
}

fn rt_at(raw: &[u8], slot: u8) -> RapidTriggerConfig {
    let start = usize::from(slot) * 8;
    RapidTriggerConfig::decode(
        raw[start..start + 8]
            .try_into()
            .expect("validated RT length"),
    )
}

fn rt_mm(record: RapidTriggerConfig) -> serde_json::Value {
    json!({
        "actuation": f32::from(record.actuation) / 100.0,
        "press": f32::from(record.press_sensitivity) / 100.0,
        "release": f32::from(record.release_sensitivity) / 100.0,
    })
}

fn set_flag(flags: &mut u8, mask: u8, value: Option<bool>) {
    if let Some(value) = value {
        if value {
            *flags |= mask;
        } else {
            *flags &= !mask;
        }
    }
}

fn mm_u8(value: f32, maximum: f32, name: &str) -> Result<u8> {
    let value = mm_u16(value, maximum, name)?;
    u8::try_from(value).with_context(|| format!("{name} is outside byte range"))
}

fn mm_u16(value: f32, maximum: f32, name: &str) -> Result<u16> {
    ensure!(value.is_finite(), "{name} must be finite");
    ensure!(
        (0.0..=maximum).contains(&value),
        "{name} must be 0.00-{maximum:.2} mm"
    );
    Ok((value * 100.0).round() as u16)
}

fn validate_assignment(assignment: KeyAssignment) -> Result<()> {
    match assignment {
        KeyAssignment::Dks { slot } => ensure!(slot < 64, "DKS slot must be 0-63"),
        KeyAssignment::Socd { mode, .. } => {
            ensure!((1..=4).contains(&mode), "SOCD mode must be 1-4")
        }
        KeyAssignment::ModTap { hold_ms, .. } => {
            ensure!(hold_ms <= 2_550, "Mod-Tap hold time must be 0-2550 ms");
            ensure!(
                hold_ms % 10 == 0,
                "Mod-Tap hold time must be divisible by 10 ms"
            );
        }
        KeyAssignment::SpecialFunction { value } => {
            ensure!(
                (1..=22).contains(&value),
                "Mini60 special function must be 1-22"
            );
        }
        _ => {}
    }
    Ok(())
}

fn format_assignment(assignment: KeyAssignment) -> String {
    match assignment {
        KeyAssignment::Empty => "factory/default".to_owned(),
        other => format!("{other:?}"),
    }
}

fn slot_name(slot: u8) -> &'static str {
    PHYSICAL_KEYS
        .iter()
        .find(|(value, _)| *value == slot)
        .map(|(_, name)| *name)
        .unwrap_or("non-physical slot")
}

#[derive(Clone, Copy)]
struct SectionSpec {
    name: &'static str,
    read: Command,
    write: Option<Command>,
    read_len: usize,
    write_len: usize,
}

fn section_spec(section: SectionName) -> SectionSpec {
    match section {
        SectionName::DeviceInfo => SectionSpec {
            name: "device-info",
            read: Command::ReadDeviceInfo,
            write: None,
            read_len: DEVICE_INFO_LEN,
            write_len: 0,
        },
        SectionName::Settings => SectionSpec {
            name: "settings",
            read: Command::ReadKeyboardSettings,
            write: Some(Command::WriteKeyboardSettings),
            read_len: KEYBOARD_SETTINGS_LEN,
            write_len: 16,
        },
        SectionName::Keymap => SectionSpec {
            name: "keymap",
            read: Command::ReadKeymap,
            write: Some(Command::WriteKeymap),
            read_len: KEYMAP_LEN,
            write_len: KEYMAP_LEN,
        },
        SectionName::Lighting => SectionSpec {
            name: "lighting",
            read: Command::ReadLighting,
            write: Some(Command::WriteLighting),
            read_len: LIGHTING_LEN,
            write_len: LIGHTING_LEN,
        },
        SectionName::PerKeyLighting => SectionSpec {
            name: "per-key-lighting",
            read: Command::ReadPerKeyLighting,
            write: Some(Command::WritePerKeyLighting),
            read_len: PER_KEY_LIGHTING_LEN,
            write_len: PER_KEY_LIGHTING_LEN,
        },
        SectionName::Macros => SectionSpec {
            name: "macros",
            read: Command::ReadMacros,
            write: Some(Command::WriteMacros),
            read_len: MACRO_BUFFER_LEN,
            write_len: MACRO_BUFFER_LEN,
        },
        SectionName::FnKeymap => SectionSpec {
            name: "fn-keymap",
            read: Command::ReadFnKeymap,
            write: Some(Command::WriteFnKeymap),
            read_len: FN_KEYMAP_LEN,
            write_len: FN_KEYMAP_LEN,
        },
        SectionName::RapidTrigger => SectionSpec {
            name: "rapid-trigger",
            read: Command::ReadRapidTrigger,
            write: Some(Command::WriteRapidTrigger),
            read_len: RAPID_TRIGGER_LEN,
            write_len: RAPID_TRIGGER_LEN,
        },
        SectionName::Dks => SectionSpec {
            name: "dks",
            read: Command::ReadDks,
            write: Some(Command::WriteDks),
            read_len: DKS_BUFFER_LEN,
            write_len: DKS_BUFFER_LEN,
        },
    }
}

fn dump_section_lengths(dump: &ConfigurationDump) -> serde_json::Value {
    json!({
        "device_info": dump.device_info.0.len(),
        "settings": dump.keyboard_settings.0.len(),
        "keymap": dump.keymap.0.len(),
        "lighting": dump.lighting.0.len(),
        "per_key_lighting": dump.per_key_lighting.0.len(),
        "macros": dump.macros.0.len(),
        "fn_keymap": dump.fn_keymap.0.len(),
        "rapid_trigger": dump.rapid_trigger.0.len(),
        "dks": dump.dks.0.len(),
    })
}

fn dry_run<T: Serialize>(runtime: &Runtime, operation: &str, preview: &T) -> Result<()> {
    if runtime.json {
        print_json(&json!({"dry_run": true, "operation": operation, "preview": preview}))?;
    } else {
        println!("Dry run: no device was modified.");
        println!("Operation: {operation}");
        println!("{}", serde_json::to_string_pretty(preview)?);
        println!("Run again with --apply to back up, write, and verify.");
    }
    Ok(())
}

fn control_dry_run(runtime: &Runtime, operation: &str, confirmation: &str) -> Result<()> {
    if runtime.json {
        print_json(
            &json!({"dry_run": true, "operation": operation, "required_confirmation": confirmation}),
        )?;
    } else {
        println!("Dry run: would {operation}; no command was sent.");
        println!("Required confirmation/apply value: {confirmation}");
    }
    Ok(())
}

fn write_result(
    runtime: &Runtime,
    operation: &str,
    backup: &Path,
    details: serde_json::Value,
) -> Result<()> {
    if runtime.json {
        print_json(
            &json!({"applied": true, "verified": true, "operation": operation, "backup": backup, "details": details}),
        )?;
    } else {
        println!("Applied and verified {operation}.");
        println!("Backup: {}", backup.display());
    }
    Ok(())
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").expect("writing to String cannot fail");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_physical_key_to_a_valid_slot() {
        assert_eq!(PHYSICAL_KEYS.len(), 61);
        let mut slots: Vec<_> = PHYSICAL_KEYS.iter().map(|(slot, _)| *slot).collect();
        slots.sort_unstable();
        slots.dedup();
        assert_eq!(slots.len(), 61);
        assert!(slots.iter().all(|slot| *slot < 128));
    }

    #[test]
    fn converts_hundredth_millimeter_units() {
        assert_eq!(mm_u8(0.30, 0.50, "zone").unwrap(), 30);
        assert_eq!(mm_u16(1.20, 3.40, "actuation").unwrap(), 120);
        assert!(mm_u16(3.41, 3.40, "actuation").is_err());
    }

    #[test]
    fn section_sizes_match_protocol_buffers() {
        assert_eq!(section_spec(SectionName::Keymap).read_len, 512);
        assert_eq!(section_spec(SectionName::Settings).write_len, 16);
        assert!(section_spec(SectionName::DeviceInfo).write.is_none());
    }
}
