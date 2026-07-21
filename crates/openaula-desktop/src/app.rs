use gpui::{
    AppContext as _, Context, Div, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, Disableable, Selectable, Sizable,
    button::{Button, ButtonVariants},
    scroll::ScrollableElement,
    slider::{Slider, SliderEvent, SliderState},
    switch::Switch,
};

use crate::{
    catalog::{KEY_ROWS, LIGHTING_EFFECTS},
    theme,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    CustomKeys,
    Lighting,
    Macro,
    Performance,
    Advanced,
    Settings,
}

impl Page {
    const ALL: [Self; 6] = [
        Self::CustomKeys,
        Self::Lighting,
        Self::Macro,
        Self::Performance,
        Self::Advanced,
        Self::Settings,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::CustomKeys => "Custom Keys",
            Self::Lighting => "Lighting Settings",
            Self::Macro => "Macro Manager",
            Self::Performance => "Performance",
            Self::Advanced => "Advanced Keys",
            Self::Settings => "Settings",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::CustomKeys => "Assign actions to either layer of the 61-key layout.",
            Self::Lighting => "Choose an effect and tune the controls supported by it.",
            Self::Macro => "Build reusable keyboard and mouse action sequences.",
            Self::Performance => "Tune Hall-effect actuation and rapid-trigger behavior.",
            Self::Advanced => "Create RS, SOCD, DKS, Mod-Tap, and Toggle bindings.",
            Self::Settings => "Host appearance, device behavior, and update information.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsPage {
    Interface,
    Device,
    Update,
}

impl SettingsPage {
    const ALL: [Self; 3] = [Self::Interface, Self::Device, Self::Update];

    fn label(self) -> &'static str {
        match self {
            Self::Interface => "Interface Settings",
            Self::Device => "Device settings",
            Self::Update => "Update",
        }
    }
}

pub struct OpenAulaApp {
    page: Page,
    settings_page: SettingsPage,
    dirty: bool,
    status: SharedString,
    selected_key: u8,
    fn_layer: bool,
    assignment_group: usize,
    lighting_on: bool,
    lighting_effect: usize,
    colorful: bool,
    color_index: usize,
    macro_index: usize,
    macro_recording: bool,
    macro_timing: usize,
    performance_preset: usize,
    fast_trigger: bool,
    shared_sensitivity: bool,
    full_distance_rt: bool,
    advanced_kind: usize,
    testing_bindings: bool,
    dark_theme: bool,
    sleep_mode: bool,
    single_key_wake: bool,
    stability_mode: bool,
    adaptive_calibration: bool,
    report_rate: usize,
    brightness: Entity<SliderState>,
    speed: Entity<SliderState>,
    trigger_distance: Entity<SliderState>,
    sensitivity: Entity<SliderState>,
    top_dead_zone: Entity<SliderState>,
    bottom_dead_zone: Entity<SliderState>,
    advanced_distance: Entity<SliderState>,
    sleep_minutes: Entity<SliderState>,
}

impl OpenAulaApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let brightness = cx.new(|_| {
            SliderState::new()
                .min(1.)
                .max(5.)
                .step(1.)
                .default_value(4.)
        });
        let speed = cx.new(|_| {
            SliderState::new()
                .min(1.)
                .max(5.)
                .step(1.)
                .default_value(3.)
        });
        let trigger_distance = cx.new(|_| {
            SliderState::new()
                .min(0.1)
                .max(3.4)
                .step(0.01)
                .default_value(1.2)
        });
        let sensitivity = cx.new(|_| {
            SliderState::new()
                .min(0.01)
                .max(3.4)
                .step(0.01)
                .default_value(0.16)
        });
        let top_dead_zone = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(0.5)
                .step(0.01)
                .default_value(0.02)
        });
        let bottom_dead_zone = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(0.5)
                .step(0.01)
                .default_value(0.05)
        });
        let advanced_distance = cx.new(|_| {
            SliderState::new()
                .min(0.1)
                .max(3.4)
                .step(0.01)
                .default_value(1.2)
        });
        let sleep_minutes = cx.new(|_| {
            SliderState::new()
                .min(1.)
                .max(30.)
                .step(1.)
                .default_value(5.)
        });

        for slider in [
            &brightness,
            &speed,
            &trigger_distance,
            &sensitivity,
            &top_dead_zone,
            &bottom_dead_zone,
            &advanced_distance,
            &sleep_minutes,
        ] {
            cx.subscribe(slider, |this, _, event, cx| {
                if matches!(event, SliderEvent::Change(_)) {
                    this.mark_dirty("Slider value staged");
                    cx.notify();
                }
            })
            .detach();
        }

        Self {
            page: Page::CustomKeys,
            settings_page: SettingsPage::Interface,
            dirty: false,
            status: "Preview session · device writes are disabled".into(),
            selected_key: 49,
            fn_layer: false,
            assignment_group: 0,
            lighting_on: true,
            lighting_effect: 0,
            colorful: false,
            color_index: 1,
            macro_index: 0,
            macro_recording: false,
            macro_timing: 0,
            performance_preset: 0,
            fast_trigger: true,
            shared_sensitivity: true,
            full_distance_rt: false,
            advanced_kind: 0,
            testing_bindings: false,
            dark_theme: true,
            sleep_mode: true,
            single_key_wake: true,
            stability_mode: true,
            adaptive_calibration: true,
            report_rate: 0,
            brightness,
            speed,
            trigger_distance,
            sensitivity,
            top_dead_zone,
            bottom_dead_zone,
            advanced_distance,
            sleep_minutes,
        }
    }

    fn mark_dirty(&mut self, message: &'static str) {
        self.dirty = true;
        self.status = message.into();
    }

    fn nav(&self, cx: &mut Context<Self>) -> Div {
        let mut navigation = div().flex().flex_col().gap_1().mt_4();
        for page in Page::ALL {
            navigation = navigation.child(
                Button::new(("nav", page as usize))
                    .label(page.label())
                    .selected(self.page == page)
                    .ghost()
                    .w_full()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.page = page;
                        cx.notify();
                    })),
            );
        }
        navigation
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .w(px(232.))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .justify_between()
            .p_4()
            .bg(theme::color(theme::SIDEBAR))
            .border_r_1()
            .border_color(theme::color(theme::BORDER))
            .child(
                div()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size(px(38.))
                                    .rounded_lg()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(theme::color(theme::PRIMARY))
                                    .text_color(theme::color(theme::BACKGROUND))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("OA"),
                            )
                            .child(
                                div()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .child("OpenAula"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::color(theme::MUTED))
                                            .child("Native configurator"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .mt_5()
                            .p_3()
                            .rounded_lg()
                            .border_1()
                            .border_color(theme::color(theme::BORDER))
                            .bg(theme::color(theme::SURFACE))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("MINI 60 HE PRO"),
                            )
                            .child(
                                div()
                                    .mt_1()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(theme::color(theme::MUTED))
                                    .child(
                                        div()
                                            .size_2()
                                            .rounded_full()
                                            .bg(theme::color(theme::WARNING)),
                                    )
                                    .child("Preview data"),
                            )
                            .child(
                                div()
                                    .mt_3()
                                    .flex()
                                    .justify_between()
                                    .text_xs()
                                    .text_color(theme::color(theme::MUTED))
                                    .child("Dongle")
                                    .child("FW 1.52"),
                            ),
                    )
                    .child(
                        div()
                            .mt_5()
                            .text_xs()
                            .text_color(theme::color(theme::MUTED))
                            .child("CONFIGURE"),
                    )
                    .child(self.nav(cx)),
            )
            .child(
                div()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::color(theme::MUTED))
                            .child("ACTIVE PROFILE"),
                    )
                    .child(
                        div()
                            .mt_2()
                            .p_3()
                            .rounded_lg()
                            .bg(theme::color(theme::SURFACE))
                            .border_1()
                            .border_color(theme::color(theme::BORDER))
                            .child(div().flex().justify_between().child("Default").when(
                                self.dirty,
                                |this| {
                                    this.child(
                                        div()
                                            .text_color(theme::color(theme::WARNING))
                                            .child("Edited"),
                                    )
                                },
                            ))
                            .child(
                                div()
                                    .mt_1()
                                    .text_xs()
                                    .text_color(theme::color(theme::MUTED))
                                    .child("Local profile"),
                            ),
                    ),
            )
    }

    fn page_header(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .pb_5()
            .child(
                div()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(self.page.label()),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_sm()
                            .text_color(theme::color(theme::MUTED))
                            .child(self.page.subtitle()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .text_color(if self.dirty {
                                theme::color(theme::WARNING)
                            } else {
                                theme::color(theme::MUTED)
                            })
                            .child(self.status.clone()),
                    )
                    .child(
                        Button::new("save-working-state")
                            .label(if self.dirty { "Save changes" } else { "Saved" })
                            .primary()
                            .disabled(!self.dirty)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dirty = false;
                                this.status = "Preview state saved locally".into();
                                cx.notify();
                            })),
                    ),
            )
    }

    fn section(title: &'static str, subtitle: &'static str) -> Div {
        div()
            .rounded_lg()
            .border_1()
            .border_color(theme::color(theme::BORDER))
            .bg(theme::color(theme::SURFACE))
            .p_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(title),
            )
            .when(!subtitle.is_empty(), |this| {
                this.child(
                    div()
                        .mt_1()
                        .text_sm()
                        .text_color(theme::color(theme::MUTED))
                        .child(subtitle),
                )
            })
    }

    fn keyboard(&self, cx: &mut Context<Self>) -> Div {
        let mut keyboard = div()
            .mt_4()
            .flex()
            .flex_col()
            .gap_1()
            .p_3()
            .rounded_lg()
            .bg(theme::color(theme::BACKGROUND))
            .border_1()
            .border_color(theme::color(theme::BORDER));

        for row in KEY_ROWS {
            let mut row_element = div().flex().gap_1();
            for key in *row {
                let slot = key.slot;
                row_element = row_element.child(
                    Button::new(("key", slot as usize))
                        .label(key.label)
                        .small()
                        .selected(self.selected_key == slot)
                        .w(px(key.width * 43.))
                        .h(px(34.))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_key = slot;
                            this.status = "Physical key selected".into();
                            cx.notify();
                        })),
                );
            }
            keyboard = keyboard.child(row_element);
        }
        keyboard
    }

    fn layer_buttons(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .gap_2()
            .child(
                Button::new("normal-layer")
                    .label("Normal Layer")
                    .selected(!self.fn_layer)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.fn_layer = false;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("fn-layer")
                    .label("Fn Layer")
                    .selected(self.fn_layer)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.fn_layer = true;
                        cx.notify();
                    })),
            )
    }

    fn custom_keys_page(&self, cx: &mut Context<Self>) -> Div {
        const GROUPS: [&str; 5] = ["Basic", "Extended", "Special", "Function", "Macro"];
        const ACTIONS: [&[&str]; 5] = [
            &["A", "B", "C", "1", "2", "Esc", "Fn"],
            &["Left Ctrl", "Left Shift", "Home", "Page Up", "F1", "F12"],
            &[
                "Mouse Left",
                "Wheel Up",
                "Volume +",
                "Play / Pause",
                "Browser Back",
            ],
            &[
                "Lighting Mode",
                "Brightness +",
                "Win Lock",
                "Profile",
                "Calibration",
            ],
            &["Strafe Burst", "Quick Build", "New macro…"],
        ];

        let mut groups = div().flex().gap_2().mt_4();
        for (index, label) in GROUPS.iter().enumerate() {
            groups = groups.child(
                Button::new(("assignment-group", index))
                    .label(*label)
                    .small()
                    .selected(self.assignment_group == index)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.assignment_group = index;
                        cx.notify();
                    })),
            );
        }

        let mut actions = div().flex().flex_wrap().gap_2().mt_3();
        for (index, label) in ACTIONS[self.assignment_group].iter().enumerate() {
            actions = actions.child(
                Button::new(("assign-action", index))
                    .label(*label)
                    .outline()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.mark_dirty("Key assignment staged");
                        cx.notify();
                    })),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                Self::section(
                    "Keyboard",
                    "Select one physical key, then assign its action.",
                )
                .child(
                    div()
                        .mt_4()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(self.layer_buttons(cx))
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme::color(theme::MUTED))
                                .child(format!("Selected firmware slot {}", self.selected_key)),
                        ),
                )
                .child(self.keyboard(cx)),
            )
            .child(
                Self::section("Assignment", "Factory action: A · staged action: unchanged")
                    .child(groups)
                    .child(actions)
                    .child(
                        div()
                            .mt_4()
                            .pt_4()
                            .border_t_1()
                            .border_color(theme::color(theme::BORDER))
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                Button::new("reset-key")
                                    .label("Reset selected key")
                                    .outline(),
                            )
                            .child(
                                Button::new("reset-layer")
                                    .label("Reset current layer")
                                    .danger()
                                    .outline(),
                            ),
                    ),
            )
    }

    fn slider_row(label: &'static str, value: String, slider: &Entity<SliderState>) -> Div {
        div()
            .mt_4()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_sm()
                    .child(label)
                    .child(div().text_color(theme::color(theme::PRIMARY)).child(value)),
            )
            .child(div().mt_3().h(px(20.)).child(Slider::new(slider)))
    }

    fn lighting_page(&self, cx: &mut Context<Self>) -> Div {
        let mut effects = div().mt_4().flex().flex_wrap().gap_2();
        for (index, effect) in LIGHTING_EFFECTS.iter().enumerate() {
            effects = effects.child(
                Button::new(("lighting-effect", index))
                    .label(*effect)
                    .selected(self.lighting_effect == index)
                    .w(px(174.))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.lighting_effect = index;
                        this.mark_dirty("Lighting effect staged");
                        cx.notify();
                    })),
            );
        }

        let palette = [
            ("Black", 0x000000),
            ("Blue", 0x2166ff),
            ("Cyan", 0x00d9ff),
            ("Green", 0x36d87a),
            ("Yellow", 0xffd83d),
            ("Red", 0xff454f),
            ("Magenta", 0xff41dd),
            ("White", 0xf2f4f7),
        ];
        let mut colors = div().mt_3().flex().gap_2();
        for (index, (name, color)) in palette.iter().enumerate() {
            colors = colors.child(
                Button::new(("palette", index))
                    .child(div().size_4().rounded_full().bg(theme::color(*color)))
                    .tooltip(*name)
                    .selected(self.color_index == index)
                    .small()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.color_index = index;
                        this.mark_dirty("Lighting color staged");
                        cx.notify();
                    })),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                Self::section(
                    "Lighting",
                    "Effect zero turns lighting off; hidden fields are preserved.",
                )
                .child(
                    div()
                        .mt_4()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div().child("Keyboard lighting").child(
                                div()
                                    .text_sm()
                                    .text_color(theme::color(theme::MUTED))
                                    .child("Apply effects to the onboard LEDs."),
                            ),
                        )
                        .child(
                            Switch::new("lighting-enabled")
                                .checked(self.lighting_on)
                                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                    this.lighting_on = *checked;
                                    this.mark_dirty("Lighting power staged");
                                    cx.notify();
                                })),
                        ),
                )
                .child(effects),
            )
            .child(
                Self::section("Effect controls", LIGHTING_EFFECTS[self.lighting_effect])
                    .child(Self::slider_row(
                        "Brightness",
                        format!("{:.0} / 5", self.brightness.read(cx).value().end()),
                        &self.brightness,
                    ))
                    .child(Self::slider_row(
                        "Speed",
                        format!("{:.0} / 5", self.speed.read(cx).value().end()),
                        &self.speed,
                    ))
                    .child(
                        div()
                            .mt_4()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(div().child("Primary color").child(colors))
                            .child(
                                Switch::new("colorful-effect")
                                    .label("Colorful / rainbow")
                                    .checked(self.colorful)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.colorful = *checked;
                                        this.mark_dirty("Rainbow mode staged");
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
    }

    fn macro_page(&self, cx: &mut Context<Self>) -> Div {
        let macros = ["Strafe Burst", "Quick Build", "Media Toggle"];
        let mut macro_list = div().mt_3().flex().flex_col().gap_2();
        for (index, name) in macros.iter().enumerate() {
            macro_list = macro_list.child(
                Button::new(("macro", index))
                    .label(*name)
                    .selected(self.macro_index == index)
                    .w_full()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.macro_index = index;
                        cx.notify();
                    })),
            );
        }

        let actions = [
            ("01", "Key down", "A", "0 ms"),
            ("02", "Delay", "—", "42 ms"),
            ("03", "Key down", "D", "0 ms"),
            ("04", "Delay", "—", "42 ms"),
            ("05", "Key up", "A + D", "0 ms"),
        ];
        let mut table = div()
            .mt_3()
            .rounded_lg()
            .border_1()
            .border_color(theme::color(theme::BORDER));
        table = table.child(
            div()
                .flex()
                .p_3()
                .bg(theme::color(theme::ELEVATED))
                .text_xs()
                .text_color(theme::color(theme::MUTED))
                .child(div().w(px(48.)).child("STEP"))
                .child(div().w(px(150.)).child("ACTION"))
                .child(div().flex_1().child("VALUE"))
                .child(div().w(px(90.)).child("DELAY")),
        );
        for (step, kind, value, delay) in actions {
            table = table.child(
                div()
                    .flex()
                    .p_3()
                    .border_t_1()
                    .border_color(theme::color(theme::BORDER))
                    .text_sm()
                    .child(
                        div()
                            .w(px(48.))
                            .text_color(theme::color(theme::MUTED))
                            .child(step),
                    )
                    .child(div().w(px(150.)).child(kind))
                    .child(div().flex_1().child(value))
                    .child(
                        div()
                            .w(px(90.))
                            .text_color(theme::color(theme::PRIMARY))
                            .child(delay),
                    ),
            );
        }

        let timing_labels = ["Recorded interval", "No interval", "Default 10 ms"];
        let mut timings = div().flex().gap_2();
        for (index, label) in timing_labels.iter().enumerate() {
            timings = timings.child(
                Button::new(("timing", index))
                    .label(*label)
                    .small()
                    .selected(self.macro_timing == index)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.macro_timing = index;
                        this.mark_dirty("Macro timing staged");
                        cx.notify();
                    })),
            );
        }

        div()
            .flex()
            .gap_4()
            .child(
                Self::section("Macros", "3 definitions · 5 actions")
                    .w(px(250.))
                    .flex_shrink_0()
                    .child(
                        div()
                            .mt_4()
                            .flex()
                            .gap_2()
                            .child(Button::new("macro-new").label("New").primary().small())
                            .child(Button::new("macro-copy").label("Copy").small()),
                    )
                    .child(macro_list)
                    .child(
                        div()
                            .mt_4()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                Button::new("macro-import")
                                    .label("Import")
                                    .small()
                                    .outline(),
                            )
                            .child(
                                Button::new("macro-export")
                                    .label("Export")
                                    .small()
                                    .outline(),
                            )
                            .child(
                                Button::new("macro-delete")
                                    .label("Delete")
                                    .small()
                                    .danger()
                                    .outline(),
                            ),
                    ),
            )
            .child(
                Self::section(
                    macros[self.macro_index],
                    "Playback mode: Routine · Repeat count: 1",
                )
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .mt_4()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(timings)
                        .child(
                            Button::new("record-macro")
                                .label(if self.macro_recording {
                                    "Stop recording"
                                } else {
                                    "Record"
                                })
                                .when(self.macro_recording, |button| button.danger())
                                .when(!self.macro_recording, |button| button.primary())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.macro_recording = !this.macro_recording;
                                    this.status = if this.macro_recording {
                                        "Macro recording active"
                                    } else {
                                        "Macro recording stopped"
                                    }
                                    .into();
                                    cx.notify();
                                })),
                        ),
                )
                .child(table)
                .child(
                    div()
                        .mt_3()
                        .flex()
                        .gap_2()
                        .child(
                            Button::new("insert-keyboard")
                                .label("+ Keyboard action")
                                .outline(),
                        )
                        .child(
                            Button::new("insert-mouse")
                                .label("+ Mouse action")
                                .outline(),
                        )
                        .child(Button::new("insert-delay").label("+ Delay").outline()),
                ),
            )
    }

    fn performance_page(&self, cx: &mut Context<Self>) -> Div {
        let presets = ["Custom", "Office Mode", "Beginner Mode", "Game Mode"];
        let mut preset_buttons = div().mt_4().flex().gap_2();
        for (index, preset) in presets.iter().enumerate() {
            preset_buttons = preset_buttons.child(
                Button::new(("preset", index))
                    .label(*preset)
                    .selected(self.performance_preset == index)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.performance_preset = index;
                        this.mark_dirty("Performance preset staged");
                        cx.notify();
                    })),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                Self::section("Performance preset", "Presets stage per-key values and do not create firmware modes.")
                    .child(preset_buttons)
                    .child(
                        div()
                            .mt_4()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(div().text_sm().text_color(theme::color(theme::MUTED)).child("Selection helpers"))
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(Button::new("select-all").label("All").small().outline())
                                    .child(Button::new("select-wasd").label("WASD").small().outline())
                                    .child(Button::new("select-numbers").label("Number row").small().outline()),
                            ),
                    )
                    .child(self.keyboard(cx)),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        Self::section("Normal Mode", "Per-key switch travel")
                            .flex_1()
                            .child(Self::slider_row(
                                "Trigger Distance",
                                format!("{:.2} mm", self.trigger_distance.read(cx).value().end()),
                                &self.trigger_distance,
                            ))
                            .child(
                                div()
                                    .mt_4()
                                    .p_3()
                                    .rounded_lg()
                                    .bg(theme::color(theme::ELEVATED))
                                    .child(div().text_sm().child("Switch type"))
                                    .child(div().mt_1().text_sm().text_color(theme::color(theme::MUTED)).child("Smoke Cloud Switch · 3.4 mm · ID 6")),
                            ),
                    )
                    .child(
                        Self::section("RT Mode", "Rapid-trigger behavior")
                            .flex_1()
                            .child(
                                div()
                                    .mt_4()
                                    .flex()
                                    .justify_between()
                                    .child("Fast Trigger")
                                    .child(
                                        Switch::new("fast-trigger")
                                            .checked(self.fast_trigger)
                                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                                this.fast_trigger = *checked;
                                                this.mark_dirty("Fast Trigger staged");
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(Self::slider_row(
                                if self.shared_sensitivity { "Shared Sensitivity" } else { "Press Sensitivity" },
                                format!("{:.2} mm", self.sensitivity.read(cx).value().end()),
                                &self.sensitivity,
                            ))
                            .child(
                                div()
                                    .mt_4()
                                    .flex()
                                    .justify_between()
                                    .child(
                                        Switch::new("shared-sensitivity")
                                            .label("Link press and release")
                                            .checked(self.shared_sensitivity)
                                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                                this.shared_sensitivity = *checked;
                                                this.mark_dirty("Sensitivity linking staged");
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Switch::new("full-distance-rt")
                                            .label("Full Distance RT")
                                            .checked(self.full_distance_rt)
                                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                                this.full_distance_rt = *checked;
                                                this.mark_dirty("Full-distance RT staged");
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        Self::section("Advanced Settings", "Global dead zones")
                            .flex_1()
                            .child(Self::slider_row(
                                "Top Dead Zone",
                                format!("{:.2} mm", self.top_dead_zone.read(cx).value().end()),
                                &self.top_dead_zone,
                            ))
                            .child(Self::slider_row(
                                "Bottom Dead Zone",
                                format!("{:.2} mm", self.bottom_dead_zone.read(cx).value().end()),
                                &self.bottom_dead_zone,
                            )),
                    )
                    .child(
                        Self::section("Recalibrate", "Hazardous device command")
                            .flex_1()
                            .child(
                                div()
                                    .mt_4()
                                    .text_sm()
                                    .text_color(theme::color(theme::MUTED))
                                    .child("Calibration creates a backup and requires confirmation. Keyboard LEDs report progress on wireless."),
                            )
                            .child(
                                div()
                                    .mt_4()
                                    .flex()
                                    .gap_2()
                                    .child(Button::new("start-calibration").label("Start calibration").danger().outline())
                                    .child(Button::new("stop-calibration").label("Stop and save").outline()),
                            ),
                    ),
            )
    }

    fn advanced_page(&self, cx: &mut Context<Self>) -> Div {
        let kinds = ["RS", "SOCD", "DKS", "MT", "TGL"];
        let names = [
            "Rappy Snappy",
            "SOCD",
            "Dynamic Key Travel",
            "Dual Effect Click",
            "Toggle Switch",
        ];
        let descriptions = [
            "Pair two physical keys and favor the key pressed farther.",
            "Resolve two opposing inputs using a selectable priority rule.",
            "Trigger up to four actions at four travel phases.",
            "Use one action on tap and another when held.",
            "Tap to lock or unlock continuous activation.",
        ];
        let mut kind_buttons = div().mt_4().flex().gap_2();
        for (index, kind) in kinds.iter().enumerate() {
            kind_buttons = kind_buttons.child(
                Button::new(("advanced-kind", index))
                    .label(*kind)
                    .selected(self.advanced_kind == index)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.advanced_kind = index;
                        cx.notify();
                    })),
            );
        }

        let editor_body = if self.advanced_kind <= 1 {
            div()
                .child(Self::slider_row(
                    "Trigger Distance",
                    format!("{:.2} mm", self.advanced_distance.read(cx).value().end()),
                    &self.advanced_distance,
                ))
                .child(
                    div()
                        .mt_4()
                        .p_3()
                        .rounded_lg()
                        .bg(theme::color(theme::ELEVATED))
                        .text_sm()
                        .child(if self.advanced_kind == 0 {
                            "Key pair: A + D · select another key in the preview"
                        } else {
                            "Behavior: Last Input Priority · Key pair: A + D"
                        }),
                )
        } else if self.advanced_kind == 2 {
            div()
                .mt_4()
                .child("Pressure points")
                .child(
                    div()
                        .mt_3()
                        .flex()
                        .gap_2()
                        .child(Button::new("dks-p1").label("Start · 1.6 mm").outline())
                        .child(Button::new("dks-p2").label("Bottom · 3.0 mm").outline())
                        .child(Button::new("dks-p3").label("Lift · 3.0 mm").outline())
                        .child(Button::new("dks-p4").label("Complete · 1.6 mm").outline()),
                )
                .child(
                    div()
                        .mt_4()
                        .text_sm()
                        .text_color(theme::color(theme::MUTED))
                        .child(
                            "Action matrix: 4 targets × 4 phases · Basic and Extended actions only",
                        ),
                )
        } else if self.advanced_kind == 3 {
            div()
                .mt_4()
                .flex()
                .gap_3()
                .child(Button::new("mt-hold").label("Hold: Left Ctrl").outline())
                .child(Button::new("mt-tap").label("Tap: Esc").outline())
                .child(Button::new("mt-time").label("Hold Time: 400 ms").outline())
        } else {
            div()
                .mt_4()
                .child(Button::new("tgl-target").label("Target action: W").outline())
                .child(div().mt_3().text_sm().text_color(theme::color(theme::MUTED)).child("Tap toggles continuous activation; hold behaves as a normal key press."))
        };

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                Self::section("Advanced bindings", "0 / 40 bindings used")
                    .child(
                        div()
                            .mt_4()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(kind_buttons)
                            .child(
                                Button::new("test-bindings")
                                    .label(if self.testing_bindings {
                                        "Stop test"
                                    } else {
                                        "Test Your Bindings"
                                    })
                                    .when(self.testing_bindings, |button| button.danger())
                                    .when(!self.testing_bindings, |button| button.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.testing_bindings = !this.testing_bindings;
                                        this.status = if this.testing_bindings {
                                            "Binding test mode active"
                                        } else {
                                            "Binding test mode stopped"
                                        }
                                        .into();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(self.keyboard(cx)),
            )
            .child(
                Self::section(names[self.advanced_kind], descriptions[self.advanced_kind])
                    .child(editor_body)
                    .child(
                        div()
                            .mt_4()
                            .pt_4()
                            .border_t_1()
                            .border_color(theme::color(theme::BORDER))
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(Button::new("clear-advanced").label("Clear").outline())
                            .child(
                                Button::new("stage-advanced")
                                    .label("Stage binding")
                                    .primary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.mark_dirty("Advanced binding staged");
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
    }

    fn setting_switch(
        label: &'static str,
        description: &'static str,
        id: &'static str,
        checked: bool,
        switch: Switch,
    ) -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .py_4()
            .border_b_1()
            .border_color(theme::color(theme::BORDER))
            .child(
                div().child(label).child(
                    div()
                        .mt_1()
                        .text_sm()
                        .text_color(theme::color(theme::MUTED))
                        .child(description),
                ),
            )
            .child(switch.checked(checked).tooltip(id))
    }

    fn settings_page(&self, cx: &mut Context<Self>) -> Div {
        let mut tabs = div().flex().gap_2();
        for page in SettingsPage::ALL {
            tabs = tabs.child(
                Button::new(("settings", page as usize))
                    .label(page.label())
                    .selected(self.settings_page == page)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.settings_page = page;
                        cx.notify();
                    })),
            );
        }

        let content = match self.settings_page {
            SettingsPage::Interface => self.interface_settings(cx),
            SettingsPage::Device => self.device_settings(cx),
            SettingsPage::Update => self.update_settings(cx),
        };

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .pb_4()
                    .border_b_1()
                    .border_color(theme::color(theme::BORDER))
                    .child(tabs),
            )
            .child(content)
    }

    fn interface_settings(&self, cx: &mut Context<Self>) -> Div {
        Self::section("Appearance", "The interface preference is host-only and never writes to the keyboard.")
            .child(
                div()
                    .mt_4()
                    .flex()
                    .gap_3()
                    .child(
                        Button::new("dark-theme")
                            .label("Dark Theme")
                            .selected(self.dark_theme)
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dark_theme = true;
                                this.status = "Dark-blue theme selected".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("light-theme")
                            .label("Light Theme")
                            .disabled(true)
                            .tooltip("The OpenAula preview currently ships with its dark-blue identity"),
                    )
                    .child(
                        Button::new("system-theme")
                            .label("Sync with computer")
                            .disabled(true)
                            .tooltip("System theme support will reuse this host-only setting"),
                    ),
            )
            .child(
                div()
                    .mt_5()
                    .p_4()
                    .rounded_lg()
                    .bg(theme::color(theme::BACKGROUND))
                    .border_1()
                    .border_color(theme::color(theme::BORDER))
                    .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child("OpenAula Blue"))
                    .child(div().mt_1().text_sm().text_color(theme::color(theme::MUTED)).child("Dark blue surfaces with a light gray foreground and cool blue focus accents.")),
            )
    }

    fn device_settings(&self, cx: &mut Context<Self>) -> Div {
        let report_rates = ["1 kHz", "4 kHz", "8 kHz"];
        let mut rates = div().mt_3().flex().gap_2();
        for (index, label) in report_rates.iter().enumerate() {
            rates = rates.child(
                Button::new(("report-rate", index))
                    .label(*label)
                    .selected(self.report_rate == index)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.report_rate = index;
                        this.mark_dirty("Report rate staged");
                        cx.notify();
                    })),
            );
        }

        Self::section(
            "Device behavior",
            "Availability follows wired/dongle capabilities; controls stage changes until Save.",
        )
        .child(Self::setting_switch(
            "Sleep Mode",
            "Dongle only · enter low-power mode after the selected time.",
            "sleep-mode",
            self.sleep_mode,
            Switch::new("sleep-mode").on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.sleep_mode = *checked;
                this.mark_dirty("Sleep mode staged");
                cx.notify();
            })),
        ))
        .when(self.sleep_mode, |this| {
            this.child(Self::slider_row(
                "Turn Off Time",
                format!("{:.0} minutes", self.sleep_minutes.read(cx).value().end()),
                &self.sleep_minutes,
            ))
        })
        .child(Self::setting_switch(
            "All Key & Single Key Wakeup",
            "Single-key wake uses less power while sleeping.",
            "single-key-wake",
            self.single_key_wake,
            Switch::new("single-key-wake").on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.single_key_wake = *checked;
                this.mark_dirty("Wake behavior staged");
                cx.notify();
            })),
        ))
        .child(Self::setting_switch(
            "Stability Mode",
            "Reduces noisy switch readings at very low actuation distances.",
            "stability-mode",
            self.stability_mode,
            Switch::new("stability-mode").on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.stability_mode = *checked;
                this.mark_dirty("Stability mode staged");
                cx.notify();
            })),
        ))
        .child(Self::setting_switch(
            "Adaptive Dynamic Calibration (Beta)",
            "Disconnect keyboard power before replacing switches.",
            "adaptive-calibration",
            self.adaptive_calibration,
            Switch::new("adaptive-calibration").on_click(cx.listener(
                |this, checked: &bool, _, cx| {
                    this.adaptive_calibration = *checked;
                    this.mark_dirty("Adaptive calibration staged");
                    cx.notify();
                },
            )),
        ))
        .child(
            div()
                .py_4()
                .border_b_1()
                .border_color(theme::color(theme::BORDER))
                .child("Report Rate")
                .child(
                    div()
                        .mt_1()
                        .text_sm()
                        .text_color(theme::color(theme::MUTED))
                        .child("Wired model only · raw values 3, 5, and 6."),
                )
                .child(rates),
        )
        .child(
            div()
                .pt_5()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div().child("Reset all settings").child(
                        div()
                            .mt_1()
                            .text_sm()
                            .text_color(theme::color(theme::MUTED))
                            .child("Requires a fresh backup and explicit confirmation."),
                    ),
                )
                .child(
                    Button::new("reset-all-settings")
                        .label("Reset all settings")
                        .danger()
                        .outline(),
                ),
        )
    }

    fn update_settings(&self, cx: &mut Context<Self>) -> Div {
        Self::section(
            "Firmware update",
            "Metadata only · OpenAula does not flash firmware or execute vendor updaters.",
        )
        .child(
            div()
                .mt_5()
                .flex()
                .gap_4()
                .child(
                    div()
                        .flex_1()
                        .p_4()
                        .rounded_lg()
                        .bg(theme::color(theme::BACKGROUND))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::color(theme::MUTED))
                                .child("INSTALLED"),
                        )
                        .child(
                            div()
                                .mt_2()
                                .text_2xl()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child("1.52"),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_sm()
                                .text_color(theme::color(theme::MUTED))
                                .child("MINI 60 HE PRO Dongle"),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .p_4()
                        .rounded_lg()
                        .border_1()
                        .border_color(theme::color(theme::PRIMARY))
                        .bg(theme::color(theme::ELEVATED))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::color(theme::PRIMARY))
                                .child("VENDOR METADATA"),
                        )
                        .child(
                            div()
                                .mt_2()
                                .text_2xl()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child("1.55"),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_sm()
                                .text_color(theme::color(theme::MUTED))
                                .child("Windows-only package observed in survey"),
                        ),
                ),
        )
        .child(
            div().mt_5().flex().justify_end().child(
                Button::new("check-updates")
                    .label("Check for Updates")
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.status = "Update metadata refreshed (preview)".into();
                        cx.notify();
                    })),
            ),
        )
    }

    fn body(&self, cx: &mut Context<Self>) -> Div {
        match self.page {
            Page::CustomKeys => self.custom_keys_page(cx),
            Page::Lighting => self.lighting_page(cx),
            Page::Macro => self.macro_page(cx),
            Page::Performance => self.performance_page(cx),
            Page::Advanced => self.advanced_page(cx),
            Page::Settings => self.settings_page(cx),
        }
    }
}

impl Render for OpenAulaApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.sidebar(cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scrollbar()
                    .p_6()
                    .child(self.page_header(cx))
                    .child(self.body(cx)),
            )
    }
}
