use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "aula",
    version,
    about = "Local CLI configurator for supported AULA keyboards",
    after_help = "Writes are dry-runs unless --apply is supplied. Every applied configuration write creates a full JSON backup first."
)]
pub struct Cli {
    /// Device index, hexadecimal VID:PID, or exact HID path.
    #[arg(long, global = true)]
    pub device: Option<String>,

    /// HID response timeout in milliseconds.
    #[arg(long, global = true, default_value_t = 5_000)]
    pub timeout_ms: u64,

    /// Emit machine-readable JSON where supported.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: TopCommand,
}

#[derive(Debug, Subcommand)]
pub enum TopCommand {
    /// List supported configuration interfaces without opening them.
    Devices,
    /// Read identity, firmware, battery, settings, and lighting.
    Info,
    /// Export every configuration section as a versioned JSON backup.
    Dump {
        /// Output path; omit or use `-` for stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Validate or restore a complete JSON backup.
    Restore {
        file: PathBuf,
        #[command(flatten)]
        guard: WriteGuard,
    },
    /// Read or edit device settings.
    Settings {
        #[command(subcommand)]
        command: SettingsCommand,
    },
    /// Read or edit global lighting.
    Lighting {
        #[command(subcommand)]
        command: LightingCommand,
    },
    /// Read or edit normal/Fn key assignments.
    Keymap {
        #[command(subcommand)]
        command: KeymapCommand,
    },
    /// Read or edit per-key Hall-effect and Rapid Trigger records.
    Performance {
        #[command(subcommand)]
        command: PerformanceCommand,
    },
    /// Read or replace an exact binary protocol section.
    Section {
        #[command(subcommand)]
        command: SectionCommand,
    },
    /// Start or save travel calibration.
    Calibration {
        #[command(subcommand)]
        command: CalibrationCommand,
    },
    /// Enable or disable firmware key-test mode.
    KeyTest {
        #[command(subcommand)]
        command: KeyTestCommand,
    },
    /// Reset key configuration or every device setting.
    Reset {
        #[arg(value_enum)]
        target: ResetTarget,
        /// Actually send the reset command.
        #[arg(long)]
        apply: bool,
        /// Required exact phrase: `RESET KEYS` or `RESET ALL`.
        #[arg(long)]
        confirm: Option<String>,
        /// Path for the automatic pre-reset backup.
        #[arg(long)]
        backup: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, Args)]
pub struct WriteGuard {
    /// Perform the write. Without this flag, validate and preview only.
    #[arg(long)]
    pub apply: bool,

    /// Path for the automatic pre-write backup.
    #[arg(long)]
    pub backup: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum SettingsCommand {
    /// Read and decode the current keyboard settings.
    Get,
    /// Change selected settings, preserving all unspecified bytes.
    Set {
        /// Sleep timeout: 0 disables sleep; otherwise 1-30 minutes.
        #[arg(long)]
        sleep_minutes: Option<u8>,
        /// Device-specific response-time value.
        #[arg(long)]
        response_time: Option<u8>,
        /// Report rate: 1000, 4000, or 8000 Hz.
        #[arg(long)]
        report_rate: Option<u16>,
        /// Device OS-mode byte.
        #[arg(long)]
        os_mode: Option<u8>,
        /// Screen timeout byte (preserved on screenless models).
        #[arg(long)]
        tft_timeout: Option<u8>,
        /// Top dead zone in millimeters (0.00-0.50).
        #[arg(long)]
        top_dead_zone: Option<f32>,
        /// Bottom dead zone in millimeters (0.00-0.50).
        #[arg(long)]
        bottom_dead_zone: Option<f32>,
        /// Enable or disable stability mode.
        #[arg(long, value_parser = clap::value_parser!(bool))]
        stability: Option<bool>,
        /// Enable or disable adaptive dynamic calibration.
        #[arg(long, value_parser = clap::value_parser!(bool))]
        adaptive_calibration: Option<bool>,
        /// Wake mode: true for single-key wake; false for all-key wake.
        #[arg(long, value_parser = clap::value_parser!(bool))]
        wake: Option<bool>,
        #[command(flatten)]
        guard: WriteGuard,
    },
}

#[derive(Debug, Subcommand)]
pub enum LightingCommand {
    /// Read and decode global lighting state.
    Get,
    /// Change selected global lighting fields.
    Set {
        /// Effect number in decimal or `0x` hexadecimal.
        #[arg(long)]
        effect: Option<String>,
        /// Primary color as `RRGGBB` or `#RRGGBB`.
        #[arg(long)]
        color: Option<String>,
        /// Secondary color as `RRGGBB` or `#RRGGBB`.
        #[arg(long)]
        secondary: Option<String>,
        /// Enable or disable the effect's rainbow/colorful mode.
        #[arg(long, value_parser = clap::value_parser!(bool))]
        colorful: Option<bool>,
        /// Brightness level, 1-5.
        #[arg(long)]
        brightness: Option<u8>,
        /// Animation speed, 1-5.
        #[arg(long)]
        speed: Option<u8>,
        /// Animation direction, 0 or 1.
        #[arg(long)]
        direction: Option<u8>,
        /// Effect-specific gear/mode byte.
        #[arg(long)]
        gear: Option<u8>,
        #[command(flatten)]
        guard: WriteGuard,
    },
}

#[derive(Debug, Subcommand)]
pub enum KeymapCommand {
    /// Show assignments for the physical layout or one sparse slot.
    Get {
        /// Keymap layer to read.
        #[arg(long, value_enum, default_value_t = Layer::Normal)]
        layer: Layer,
        /// Show one sparse firmware slot instead of the physical layout.
        #[arg(long)]
        slot: Option<u8>,
    },
    /// Assign one normal or Fn-layer slot.
    Set {
        /// Keymap layer to modify.
        #[arg(long, value_enum, default_value_t = Layer::Normal)]
        layer: Layer,
        /// Sparse firmware slot, 0-127.
        #[arg(long)]
        slot: u8,
        /// Second sparse slot for SOCD or Rappy Snappy; both records are updated.
        #[arg(long)]
        pair_slot: Option<u8>,
        /// Assignment expression; run `aula keymap set --help` for forms.
        ///
        /// Examples: `key:a`, `extended:224`, `mouse:1:1`,
        /// `consumer:205`, `macro:0:0:1`, `mt:224:79:400`,
        /// `toggle:6`, `socd:3:4:7`, `rs:4:7`, `special:22`,
        /// `raw:02000400`, or `factory`.
        #[arg(long, verbatim_doc_comment)]
        action: String,
        #[command(flatten)]
        guard: WriteGuard,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Layer {
    Normal,
    Fn,
}

#[derive(Debug, Subcommand)]
pub enum PerformanceCommand {
    /// Show Rapid Trigger records for the layout or one sparse slot.
    Get {
        /// Show one sparse firmware slot.
        #[arg(long)]
        slot: Option<u8>,
    },
    /// Change selected Hall-effect fields for one or more slots.
    Set {
        /// One or more slots, separated by commas.
        #[arg(long, required = true, value_delimiter = ',')]
        slots: Vec<u8>,
        /// Switch type byte; zero selects the device default.
        #[arg(long)]
        switch_type: Option<u8>,
        /// Actuation in millimeters (0.00-3.40; zero inherits default).
        #[arg(long)]
        actuation: Option<f32>,
        /// Press sensitivity in millimeters.
        #[arg(long)]
        press: Option<f32>,
        /// Release sensitivity in millimeters.
        #[arg(long)]
        release: Option<f32>,
        /// Enable or disable full-distance Rapid Trigger.
        #[arg(long, value_parser = clap::value_parser!(bool))]
        full_travel: Option<bool>,
        /// Enable or disable bottom optimization.
        #[arg(long, value_parser = clap::value_parser!(bool))]
        bottom_optimization: Option<bool>,
        /// Enable or disable Rampage Mode.
        #[arg(long, value_parser = clap::value_parser!(bool))]
        rampage: Option<bool>,
        #[command(flatten)]
        guard: WriteGuard,
    },
}

#[derive(Debug, Subcommand)]
pub enum SectionCommand {
    /// Read one exact protocol buffer.
    Read {
        #[arg(value_enum)]
        section: SectionName,
        /// Write raw bytes to a file instead of hexadecimal stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Replace one exact writable protocol buffer.
    Write {
        #[arg(value_enum)]
        section: SectionName,
        /// Exact section bytes as hexadecimal.
        #[arg(long, conflicts_with = "file")]
        hex: Option<String>,
        /// Read exact section bytes from a binary file.
        #[arg(long, conflicts_with = "hex")]
        file: Option<PathBuf>,
        #[command(flatten)]
        guard: WriteGuard,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum SectionName {
    DeviceInfo,
    Settings,
    Keymap,
    Lighting,
    PerKeyLighting,
    Macros,
    FnKeymap,
    RapidTrigger,
    Dks,
}

#[derive(Debug, Subcommand)]
pub enum CalibrationCommand {
    /// Enter calibration mode. Follow with `calibration save` when done.
    Start {
        /// Actually send the calibration command.
        #[arg(long)]
        apply: bool,
        /// Required exact phrase: `CALIBRATE`.
        #[arg(long)]
        confirm: Option<String>,
        /// Path for the automatic pre-calibration backup.
        #[arg(long)]
        backup: Option<PathBuf>,
    },
    /// Save calibration and leave calibration mode.
    Save {
        /// Actually send the save command.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum KeyTestCommand {
    /// Enter firmware key-test mode.
    Start {
        /// Actually send the key-test command.
        #[arg(long)]
        apply: bool,
    },
    /// Leave firmware key-test mode.
    Stop {
        /// Actually send the key-test command.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ResetTarget {
    Keys,
    All,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_command_tree_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_representative_guarded_commands() {
        let cli = Cli::try_parse_from([
            "aula",
            "--device",
            "0c45:fefe",
            "keymap",
            "set",
            "--slot",
            "49",
            "--pair-slot",
            "51",
            "--action",
            "socd:3:a:d",
            "--apply",
        ])
        .unwrap();
        assert_eq!(cli.device.as_deref(), Some("0c45:fefe"));
        assert!(matches!(
            cli.command,
            TopCommand::Keymap {
                command: KeymapCommand::Set {
                    slot: 49,
                    pair_slot: Some(51),
                    guard: WriteGuard { apply: true, .. },
                    ..
                }
            }
        ));
    }
}
