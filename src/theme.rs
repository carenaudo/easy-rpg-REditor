use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePalette {
    Zinc,
    Slate,
    Stone,
    Gray,
    Neutral,
    Red,
    Rose,
    Orange,
    Green,
    Blue,
    Yellow,
    Violet,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self::Zinc
    }
}

impl ThemePalette {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Zinc => "Zinc",
            Self::Slate => "Slate",
            Self::Stone => "Stone",
            Self::Gray => "Gray",
            Self::Neutral => "Neutral",
            Self::Red => "Red",
            Self::Rose => "Rose",
            Self::Orange => "Orange",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Yellow => "Yellow",
            Self::Violet => "Violet",
        }
    }

    pub fn accent_color(&self, is_dark: bool) -> egui::Color32 {
        match self {
            Self::Zinc => if is_dark { egui::Color32::from_rgb(63, 63, 70) } else { egui::Color32::from_rgb(212, 212, 216) },
            Self::Slate => if is_dark { egui::Color32::from_rgb(51, 65, 85) } else { egui::Color32::from_rgb(203, 213, 225) },
            Self::Stone => if is_dark { egui::Color32::from_rgb(68, 64, 60) } else { egui::Color32::from_rgb(214, 211, 209) },
            Self::Gray => if is_dark { egui::Color32::from_rgb(55, 65, 81) } else { egui::Color32::from_rgb(209, 213, 219) },
            Self::Neutral => if is_dark { egui::Color32::from_rgb(64, 64, 64) } else { egui::Color32::from_rgb(212, 212, 212) },
            Self::Red => if is_dark { egui::Color32::from_rgb(239, 68, 68) } else { egui::Color32::from_rgb(220, 38, 38) },
            Self::Rose => if is_dark { egui::Color32::from_rgb(244, 63, 94) } else { egui::Color32::from_rgb(225, 29, 72) },
            Self::Orange => if is_dark { egui::Color32::from_rgb(249, 115, 22) } else { egui::Color32::from_rgb(234, 88, 12) },
            Self::Green => if is_dark { egui::Color32::from_rgb(34, 197, 94) } else { egui::Color32::from_rgb(22, 163, 74) },
            Self::Blue => if is_dark { egui::Color32::from_rgb(59, 130, 246) } else { egui::Color32::from_rgb(37, 99, 235) },
            Self::Yellow => if is_dark { egui::Color32::from_rgb(234, 179, 8) } else { egui::Color32::from_rgb(202, 138, 4) },
            Self::Violet => if is_dark { egui::Color32::from_rgb(139, 92, 246) } else { egui::Color32::from_rgb(124, 58, 237) },
        }
    }

    pub fn accent_fg(&self, is_dark: bool) -> egui::Color32 {
        match self {
            Self::Zinc | Self::Slate | Self::Stone | Self::Gray | Self::Neutral => {
                if is_dark {
                    egui::Color32::from_rgb(250, 250, 250)
                } else {
                    egui::Color32::from_rgb(24, 24, 27)
                }
            }
            Self::Yellow => {
                if is_dark {
                    egui::Color32::from_rgb(15, 15, 18)
                } else {
                    egui::Color32::WHITE
                }
            }
            _ => egui::Color32::WHITE,
        }
    }
}

pub mod colors {
    use eframe::egui;

    // Status / Feedback
    pub fn success(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(34, 197, 94) } else { egui::Color32::from_rgb(22, 163, 74) }
    }

    pub fn warning(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(234, 179, 8) } else { egui::Color32::from_rgb(202, 138, 4) }
    }

    pub fn danger(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(239, 68, 68) } else { egui::Color32::from_rgb(220, 38, 38) }
    }

    pub fn info(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(56, 189, 248) } else { egui::Color32::from_rgb(2, 132, 199) }
    }

    pub fn muted(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(156, 163, 175) } else { egui::Color32::from_rgb(107, 114, 128) }
    }

    pub fn dim(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgba_unmultiplied(200, 210, 220, 180) } else { egui::Color32::from_rgba_unmultiplied(60, 70, 80, 200) }
    }

    // Category / Group Sections
    pub fn category_characters(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(129, 140, 248) } else { egui::Color32::from_rgb(67, 56, 202) }
    }

    pub fn category_combat(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(248, 113, 113) } else { egui::Color32::from_rgb(185, 28, 28) }
    }

    pub fn category_world(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(52, 211, 153) } else { egui::Color32::from_rgb(4, 120, 87) }
    }

    pub fn category_logic(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(192, 132, 252) } else { egui::Color32::from_rgb(126, 34, 206) }
    }

    // Stats
    pub fn stat_hp(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(248, 113, 113) } else { egui::Color32::from_rgb(220, 38, 38) }
    }

    pub fn stat_sp(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(96, 165, 250) } else { egui::Color32::from_rgb(37, 99, 235) }
    }

    pub fn stat_atk(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(251, 146, 60) } else { egui::Color32::from_rgb(217, 119, 6) }
    }

    pub fn stat_def(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(74, 222, 128) } else { egui::Color32::from_rgb(22, 163, 74) }
    }

    pub fn stat_spi(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(192, 132, 252) } else { egui::Color32::from_rgb(147, 51, 234) }
    }

    pub fn stat_agi(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(250, 204, 21) } else { egui::Color32::from_rgb(202, 138, 4) }
    }

    pub fn gold(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(250, 204, 21) } else { egui::Color32::from_rgb(180, 120, 0) }
    }

    // Resistance Ranks (A, B, C, D, E)
    pub fn rank_a(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(248, 113, 113) } else { egui::Color32::from_rgb(220, 38, 38) }
    }

    pub fn rank_b(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(251, 146, 60) } else { egui::Color32::from_rgb(217, 119, 6) }
    }

    pub fn rank_c(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(250, 204, 21) } else { egui::Color32::from_rgb(161, 140, 0) }
    }

    pub fn rank_d(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(56, 189, 248) } else { egui::Color32::from_rgb(2, 132, 199) }
    }

    pub fn rank_e(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(74, 222, 128) } else { egui::Color32::from_rgb(22, 163, 74) }
    }

    // Item types
    pub fn item_type(item_type_id: i32, is_dark: bool) -> egui::Color32 {
        if is_dark {
            match item_type_id {
                1 => egui::Color32::from_rgb(248, 113, 113), // Weapon
                2..=4 => egui::Color32::from_rgb(96, 165, 250), // Shield/Armor/Helmet
                5 => egui::Color32::from_rgb(251, 146, 60), // Accessory
                6 => egui::Color32::from_rgb(74, 222, 128), // Medicine
                7 => egui::Color32::from_rgb(192, 132, 252), // Book
                10 => egui::Color32::from_rgb(244, 114, 182), // Switch
                _ => egui::Color32::from_rgb(156, 163, 175),
            }
        } else {
            match item_type_id {
                1 => egui::Color32::from_rgb(220, 38, 38), // Weapon
                2..=4 => egui::Color32::from_rgb(37, 99, 235), // Shield/Armor/Helmet
                5 => egui::Color32::from_rgb(217, 119, 6), // Accessory
                6 => egui::Color32::from_rgb(22, 163, 74), // Medicine
                7 => egui::Color32::from_rgb(147, 51, 234), // Book
                10 => egui::Color32::from_rgb(219, 39, 119), // Switch
                _ => egui::Color32::from_rgb(75, 85, 99),
            }
        }
    }

    // Event script command highlights
    pub fn event_command(code: i32, is_dark: bool) -> egui::Color32 {
        if is_dark {
            match code {
                10110..=10150 => egui::Color32::from_rgb(250, 204, 21), // Messages & Choices (Gold)
                12010..=12400 => egui::Color32::from_rgb(56, 189, 248), // Flow control & Branches (Cyan)
                10210..=10230 => egui::Color32::from_rgb(74, 222, 128), // Switches & Variables (Green)
                10610..=10630 | 11310..=11340 => egui::Color32::from_rgb(251, 146, 60), // Movement & Teleport (Orange)
                11510..=11560 => egui::Color32::from_rgb(232, 121, 249), // Audio & Music (Magenta)
                10710..=10740 | 11010..=11040 => egui::Color32::from_rgb(248, 113, 113), // Battle & Combat (Red)
                3001..=3032 => egui::Color32::from_rgb(200, 140, 255), // Maniac Patch (Purple) - matches the accent used elsewhere for Maniac UI
                _ => egui::Color32::from_rgb(226, 232, 240),
            }
        } else {
            match code {
                10110..=10150 => egui::Color32::from_rgb(180, 120, 0), // Messages & Choices (Dark Gold)
                12010..=12400 => egui::Color32::from_rgb(2, 132, 199), // Flow control & Branches (Dark Cyan)
                10210..=10230 => egui::Color32::from_rgb(22, 163, 74), // Switches & Variables (Dark Green)
                10610..=10630 | 11310..=11340 => egui::Color32::from_rgb(195, 90, 10), // Movement & Teleport (Dark Orange)
                11510..=11560 => egui::Color32::from_rgb(147, 40, 160), // Audio & Music (Dark Magenta)
                10710..=10740 | 11010..=11040 => egui::Color32::from_rgb(210, 35, 35), // Battle & Combat (Dark Red)
                3001..=3032 => egui::Color32::from_rgb(147, 51, 178), // Maniac Patch (Dark Purple)
                _ => egui::Color32::from_rgb(30, 45, 65),
            }
        }
    }

    // Canvas & Preview backgrounds
    pub fn stage_bg(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgb(14, 18, 24) } else { egui::Color32::from_rgb(240, 243, 246) }
    }

    pub fn troop_stage_bg(perspective: i32, is_dark: bool) -> egui::Color32 {
        if is_dark {
            if perspective == 0 { egui::Color32::from_rgb(22, 30, 40) } else { egui::Color32::from_rgb(26, 36, 30) }
        } else {
            if perspective == 0 { egui::Color32::from_rgb(238, 242, 246) } else { egui::Color32::from_rgb(238, 245, 240) }
        }
    }

    pub fn checkerboard(is_dark: bool) -> (egui::Color32, egui::Color32) {
        if is_dark {
            (egui::Color32::from_rgb(28, 34, 42), egui::Color32::from_rgb(38, 46, 56))
        } else {
            (egui::Color32::from_rgb(245, 245, 247), egui::Color32::from_rgb(228, 230, 235))
        }
    }

    pub fn grid_line(is_dark: bool) -> egui::Color32 {
        if is_dark { egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25) } else { egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppTheme {
    pub mode: ThemeMode,
    pub palette: ThemePalette,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            palette: ThemePalette::Zinc,
        }
    }
}

impl AppTheme {
    pub fn apply(&self, ctx: &egui::Context) {
        let is_dark = match self.mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => ctx.system_theme().map(|t| t == egui::Theme::Dark).unwrap_or(true),
        };

        let (bg, card, muted, border, fg) = match self.palette {
            ThemePalette::Slate => {
                if is_dark {
                    (
                        egui::Color32::from_rgb(2, 8, 23),       // bg
                        egui::Color32::from_rgb(15, 23, 42),     // card
                        egui::Color32::from_rgb(30, 41, 59),     // muted
                        egui::Color32::from_rgb(51, 65, 85),     // border
                        egui::Color32::from_rgb(248, 250, 252),  // fg
                    )
                } else {
                    (
                        egui::Color32::from_rgb(255, 255, 255),  // bg
                        egui::Color32::from_rgb(248, 250, 252),  // card
                        egui::Color32::from_rgb(241, 245, 249),  // muted
                        egui::Color32::from_rgb(226, 232, 240),  // border
                        egui::Color32::from_rgb(15, 23, 42),     // fg
                    )
                }
            }
            ThemePalette::Stone => {
                if is_dark {
                    (
                        egui::Color32::from_rgb(12, 10, 9),      // bg
                        egui::Color32::from_rgb(28, 25, 23),     // card
                        egui::Color32::from_rgb(41, 37, 36),     // muted
                        egui::Color32::from_rgb(68, 64, 60),     // border
                        egui::Color32::from_rgb(250, 250, 249),  // fg
                    )
                } else {
                    (
                        egui::Color32::from_rgb(255, 255, 255),  // bg
                        egui::Color32::from_rgb(250, 250, 249),  // card
                        egui::Color32::from_rgb(245, 245, 244),  // muted
                        egui::Color32::from_rgb(231, 229, 228),  // border
                        egui::Color32::from_rgb(28, 25, 23),     // fg
                    )
                }
            }
            ThemePalette::Gray => {
                if is_dark {
                    (
                        egui::Color32::from_rgb(3, 7, 18),       // bg
                        egui::Color32::from_rgb(17, 24, 39),     // card
                        egui::Color32::from_rgb(31, 41, 55),     // muted
                        egui::Color32::from_rgb(55, 65, 81),     // border
                        egui::Color32::from_rgb(249, 250, 251),  // fg
                    )
                } else {
                    (
                        egui::Color32::from_rgb(255, 255, 255),  // bg
                        egui::Color32::from_rgb(249, 250, 251),  // card
                        egui::Color32::from_rgb(243, 244, 246),  // muted
                        egui::Color32::from_rgb(229, 231, 235),  // border
                        egui::Color32::from_rgb(17, 24, 39),     // fg
                    )
                }
            }
            ThemePalette::Neutral => {
                if is_dark {
                    (
                        egui::Color32::from_rgb(10, 10, 10),     // bg
                        egui::Color32::from_rgb(23, 23, 23),     // card
                        egui::Color32::from_rgb(38, 38, 38),     // muted
                        egui::Color32::from_rgb(64, 64, 64),     // border
                        egui::Color32::from_rgb(250, 250, 250),  // fg
                    )
                } else {
                    (
                        egui::Color32::from_rgb(255, 255, 255),  // bg
                        egui::Color32::from_rgb(250, 250, 250),  // card
                        egui::Color32::from_rgb(245, 245, 245),  // muted
                        egui::Color32::from_rgb(229, 229, 229),  // border
                        egui::Color32::from_rgb(23, 23, 23),     // fg
                    )
                }
            }
            _ => {
                // Zinc & Colored Palettes base
                if is_dark {
                    (
                        egui::Color32::from_rgb(9, 9, 11),       // bg
                        egui::Color32::from_rgb(24, 24, 27),     // card
                        egui::Color32::from_rgb(39, 39, 42),     // muted
                        egui::Color32::from_rgb(63, 63, 70),     // border
                        egui::Color32::from_rgb(250, 250, 250),  // fg
                    )
                } else {
                    (
                        egui::Color32::from_rgb(255, 255, 255),  // bg
                        egui::Color32::from_rgb(244, 244, 245),  // card
                        egui::Color32::from_rgb(244, 244, 245),  // muted
                        egui::Color32::from_rgb(228, 228, 231),  // border
                        egui::Color32::from_rgb(9, 9, 11),       // fg
                    )
                }
            }
        };

        let primary_accent = self.palette.accent_color(is_dark);
        let accent_fg = self.palette.accent_fg(is_dark);

        let mut visuals = if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.override_text_color = None;
        visuals.panel_fill = bg;
        visuals.window_fill = card;
        visuals.faint_bg_color = muted;
        visuals.code_bg_color = muted;
        visuals.extreme_bg_color = if is_dark { egui::Color32::from_rgb(15, 15, 18) } else { egui::Color32::WHITE };
        visuals.window_stroke = egui::Stroke::new(1.0, border);

        // Selection
        visuals.selection.bg_fill = primary_accent;
        visuals.selection.stroke = egui::Stroke::new(1.0, accent_fg);

        // Corner radius (shadcn-style 6px rounded cards and buttons)
        visuals.window_corner_radius = egui::CornerRadius::same(6);
        visuals.menu_corner_radius = egui::CornerRadius::same(6);

        let cr = egui::CornerRadius::same(5);

        // Non-interactive (groups, cards)
        visuals.widgets.noninteractive.bg_fill = card;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);
        visuals.widgets.noninteractive.corner_radius = cr;

        // Inactive (buttons, input fields)
        visuals.widgets.inactive.bg_fill = muted;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);
        visuals.widgets.inactive.corner_radius = cr;

        // Hovered
        let hovered_bg = if is_dark {
            egui::Color32::from_rgba_unmultiplied(primary_accent.r(), primary_accent.g(), primary_accent.b(), 50)
        } else {
            egui::Color32::from_rgba_unmultiplied(primary_accent.r(), primary_accent.g(), primary_accent.b(), 30)
        };
        visuals.widgets.hovered.bg_fill = hovered_bg;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, primary_accent);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg);
        visuals.widgets.hovered.corner_radius = cr;

        // Active / Clicked
        visuals.widgets.active.bg_fill = primary_accent;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, primary_accent);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, accent_fg);
        visuals.widgets.active.corner_radius = cr;

        // Open menu items
        visuals.widgets.open.bg_fill = card;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, border);
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, fg);
        visuals.widgets.open.corner_radius = cr;

        ctx.set_visuals(visuals);
    }
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let windows_fonts = [
        "C:\\Windows\\Fonts\\msyh.ttc",     // Microsoft YaHei (Chinese)
        "C:\\Windows\\Fonts\\meiryo.ttc",   // Meiryo (Japanese)
        "C:\\Windows\\Fonts\\msgothic.ttc", // MS Gothic (Japanese)
        "C:\\Windows\\Fonts\\malgun.ttf",   // Malgun Gothic (Korean)
    ];

    for font_path in &windows_fonts {
        if std::path::Path::new(font_path).exists() {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "system_cjk".to_owned(),
                    egui::FontData::from_owned(font_data).into(),
                );
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.push("system_cjk".to_owned());
                }
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    family.push("system_cjk".to_owned());
                }
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
}

