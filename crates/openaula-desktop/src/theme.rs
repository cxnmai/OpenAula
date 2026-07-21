use gpui::{App, Hsla, rgb};
use gpui_component::Theme;

pub const BACKGROUND: u32 = 0x07111f;
pub const SIDEBAR: u32 = 0x0a1728;
pub const SURFACE: u32 = 0x0e1d30;
pub const ELEVATED: u32 = 0x14263c;
pub const BORDER: u32 = 0x243a55;
pub const FOREGROUND: u32 = 0xd9dde4;
pub const MUTED: u32 = 0x91a0b3;
pub const PRIMARY: u32 = 0x54a8ff;
pub const PRIMARY_HOVER: u32 = 0x70b7ff;
pub const SUCCESS: u32 = 0x43d19e;
pub const WARNING: u32 = 0xf2b84b;
pub const DANGER: u32 = 0xee6b73;

pub fn color(value: u32) -> Hsla {
    rgb(value).into()
}

pub fn apply(cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.shadow = false;
    theme.radius = gpui::px(8.);
    theme.radius_lg = gpui::px(12.);

    let colors = &mut theme.colors;
    colors.background = color(BACKGROUND);
    colors.foreground = color(FOREGROUND);
    colors.border = color(BORDER);
    colors.input = color(BORDER);
    colors.muted = color(ELEVATED);
    colors.muted_foreground = color(MUTED);
    colors.primary = color(PRIMARY);
    colors.primary_hover = color(PRIMARY_HOVER);
    colors.primary_active = color(0x368bdc);
    colors.primary_foreground = color(BACKGROUND);
    colors.secondary = color(ELEVATED);
    colors.secondary_hover = color(0x1b3451);
    colors.secondary_active = color(0x213e60);
    colors.secondary_foreground = color(FOREGROUND);
    colors.accent = color(0x173252);
    colors.accent_foreground = color(FOREGROUND);
    colors.sidebar = color(SIDEBAR);
    colors.sidebar_foreground = color(FOREGROUND);
    colors.sidebar_border = color(BORDER);
    colors.sidebar_accent = color(0x173252);
    colors.sidebar_accent_foreground = color(FOREGROUND);
    colors.sidebar_primary = color(PRIMARY);
    colors.sidebar_primary_foreground = color(BACKGROUND);
    colors.switch = color(0x2a3e56);
    colors.switch_thumb = color(0xe8ebef);
    colors.slider_bar = color(0x29435f);
    colors.slider_thumb = color(PRIMARY);
    colors.list = color(SURFACE);
    colors.list_hover = color(ELEVATED);
    colors.list_active = color(0x173252);
    colors.list_active_border = color(PRIMARY);
    colors.table = color(SURFACE);
    colors.table_head = color(ELEVATED);
    colors.table_head_foreground = color(FOREGROUND);
    colors.table_hover = color(ELEVATED);
    colors.table_row_border = color(BORDER);
    colors.tab = color(SURFACE);
    colors.tab_bar = color(SURFACE);
    colors.tab_bar_segmented = color(ELEVATED);
    colors.tab_foreground = color(MUTED);
    colors.tab_active = color(0x173252);
    colors.tab_active_foreground = color(FOREGROUND);
    colors.popover = color(ELEVATED);
    colors.popover_foreground = color(FOREGROUND);
    colors.title_bar = color(SIDEBAR);
    colors.title_bar_border = color(BORDER);
    colors.ring = color(PRIMARY);
    colors.selection = color(0x24588b);
    colors.info = color(PRIMARY);
    colors.info_hover = color(PRIMARY_HOVER);
    colors.info_active = color(0x368bdc);
    colors.info_foreground = color(BACKGROUND);
    colors.success = color(SUCCESS);
    colors.success_hover = color(0x5ddbad);
    colors.success_active = color(0x31ad80);
    colors.success_foreground = color(BACKGROUND);
    colors.warning = color(WARNING);
    colors.warning_hover = color(0xffc861);
    colors.warning_active = color(0xd69b2d);
    colors.warning_foreground = color(BACKGROUND);
    colors.danger = color(DANGER);
    colors.danger_hover = color(0xf28289);
    colors.danger_active = color(0xd7545d);
    colors.danger_foreground = color(BACKGROUND);
    colors.scrollbar = color(SURFACE);
    colors.scrollbar_thumb = color(0x34506d);
    colors.scrollbar_thumb_hover = color(0x426789);
    colors.window_border = color(BORDER);
}
