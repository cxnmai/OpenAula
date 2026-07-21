#[derive(Clone, Copy)]
pub struct KeySpec {
    pub slot: u8,
    pub label: &'static str,
    pub width: f32,
}

pub const KEY_ROWS: &[&[KeySpec]] = &[
    &[
        KeySpec {
            slot: 0,
            label: "Esc",
            width: 1.0,
        },
        KeySpec {
            slot: 17,
            label: "1 !",
            width: 1.0,
        },
        KeySpec {
            slot: 18,
            label: "2 @",
            width: 1.0,
        },
        KeySpec {
            slot: 19,
            label: "3 #",
            width: 1.0,
        },
        KeySpec {
            slot: 20,
            label: "4 $",
            width: 1.0,
        },
        KeySpec {
            slot: 21,
            label: "5 %",
            width: 1.0,
        },
        KeySpec {
            slot: 22,
            label: "6 ^",
            width: 1.0,
        },
        KeySpec {
            slot: 23,
            label: "7 &",
            width: 1.0,
        },
        KeySpec {
            slot: 24,
            label: "8 *",
            width: 1.0,
        },
        KeySpec {
            slot: 25,
            label: "9 (",
            width: 1.0,
        },
        KeySpec {
            slot: 26,
            label: "0 )",
            width: 1.0,
        },
        KeySpec {
            slot: 27,
            label: "- _",
            width: 1.0,
        },
        KeySpec {
            slot: 28,
            label: "= +",
            width: 1.0,
        },
        KeySpec {
            slot: 92,
            label: "Backspace",
            width: 2.0,
        },
    ],
    &[
        KeySpec {
            slot: 32,
            label: "Tab",
            width: 1.5,
        },
        KeySpec {
            slot: 33,
            label: "Q",
            width: 1.0,
        },
        KeySpec {
            slot: 34,
            label: "W",
            width: 1.0,
        },
        KeySpec {
            slot: 35,
            label: "E",
            width: 1.0,
        },
        KeySpec {
            slot: 36,
            label: "R",
            width: 1.0,
        },
        KeySpec {
            slot: 37,
            label: "T",
            width: 1.0,
        },
        KeySpec {
            slot: 38,
            label: "Y",
            width: 1.0,
        },
        KeySpec {
            slot: 39,
            label: "U",
            width: 1.0,
        },
        KeySpec {
            slot: 40,
            label: "I",
            width: 1.0,
        },
        KeySpec {
            slot: 41,
            label: "O",
            width: 1.0,
        },
        KeySpec {
            slot: 42,
            label: "P",
            width: 1.0,
        },
        KeySpec {
            slot: 43,
            label: "[ {",
            width: 1.0,
        },
        KeySpec {
            slot: 44,
            label: "] }",
            width: 1.0,
        },
        KeySpec {
            slot: 60,
            label: "\\ |",
            width: 1.5,
        },
    ],
    &[
        KeySpec {
            slot: 48,
            label: "Caps",
            width: 1.75,
        },
        KeySpec {
            slot: 49,
            label: "A",
            width: 1.0,
        },
        KeySpec {
            slot: 50,
            label: "S",
            width: 1.0,
        },
        KeySpec {
            slot: 51,
            label: "D",
            width: 1.0,
        },
        KeySpec {
            slot: 52,
            label: "F",
            width: 1.0,
        },
        KeySpec {
            slot: 53,
            label: "G",
            width: 1.0,
        },
        KeySpec {
            slot: 54,
            label: "H",
            width: 1.0,
        },
        KeySpec {
            slot: 55,
            label: "J",
            width: 1.0,
        },
        KeySpec {
            slot: 56,
            label: "K",
            width: 1.0,
        },
        KeySpec {
            slot: 57,
            label: "L",
            width: 1.0,
        },
        KeySpec {
            slot: 58,
            label: "; :",
            width: 1.0,
        },
        KeySpec {
            slot: 59,
            label: "' quote",
            width: 1.0,
        },
        KeySpec {
            slot: 76,
            label: "Enter",
            width: 2.25,
        },
    ],
    &[
        KeySpec {
            slot: 64,
            label: "L-Shift",
            width: 2.25,
        },
        KeySpec {
            slot: 65,
            label: "Z",
            width: 1.0,
        },
        KeySpec {
            slot: 66,
            label: "X",
            width: 1.0,
        },
        KeySpec {
            slot: 67,
            label: "C",
            width: 1.0,
        },
        KeySpec {
            slot: 68,
            label: "V",
            width: 1.0,
        },
        KeySpec {
            slot: 69,
            label: "B",
            width: 1.0,
        },
        KeySpec {
            slot: 70,
            label: "N",
            width: 1.0,
        },
        KeySpec {
            slot: 71,
            label: "M",
            width: 1.0,
        },
        KeySpec {
            slot: 72,
            label: ", <",
            width: 1.0,
        },
        KeySpec {
            slot: 73,
            label: ". >",
            width: 1.0,
        },
        KeySpec {
            slot: 74,
            label: "/ ?",
            width: 1.0,
        },
        KeySpec {
            slot: 75,
            label: "R-Shift",
            width: 2.75,
        },
    ],
    &[
        KeySpec {
            slot: 80,
            label: "L-Ctrl",
            width: 1.25,
        },
        KeySpec {
            slot: 81,
            label: "L-Win",
            width: 1.25,
        },
        KeySpec {
            slot: 82,
            label: "L-Alt",
            width: 1.25,
        },
        KeySpec {
            slot: 83,
            label: "Spacebar",
            width: 6.25,
        },
        KeySpec {
            slot: 84,
            label: "R-Alt",
            width: 1.25,
        },
        KeySpec {
            slot: 86,
            label: "Super",
            width: 1.25,
        },
        KeySpec {
            slot: 87,
            label: "R-Ctrl",
            width: 1.25,
        },
        KeySpec {
            slot: 85,
            label: "Fn",
            width: 1.25,
        },
    ],
];

pub const LIGHTING_EFFECTS: &[&str] = &[
    "Static Bright",
    "Single Point On",
    "Single Point Off",
    "Starry Sky",
    "Snowfall",
    "Floral Competition",
    "Dynamic Breathing",
    "Spectrum Cycle",
    "Color Fountain",
    "Colorful Interchange",
    "Flowing with the Waves",
    "Turning Peaks",
    "One Touch to Fire",
    "Two Birds with One Stone",
    "Ripples Spread",
    "Endless Flow",
    "Layered Mountains",
    "Gentle Rain and Wind",
    "Back and Forth",
    "Custom",
];
