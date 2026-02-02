//! iTerm2 theme support for Anteater
//!
//! This module provides support for loading and applying iTerm2 color schemes
//! to the Anteater debugger UI, similar to how Ghostty does it.

use egui::Color32;
use plist::{Dictionary, Value};

/// An iTerm2 color scheme
#[derive(Debug, Clone)]
pub struct ITerm2Theme {
    pub name: String,
    pub foreground: Color32,
    pub background: Color32,
    pub cursor: Color32,
    pub selection: Color32,

    // ANSI colors (0-15)
    pub ansi_black: Color32,
    pub ansi_red: Color32,
    pub ansi_green: Color32,
    pub ansi_yellow: Color32,
    pub ansi_blue: Color32,
    pub ansi_magenta: Color32,
    pub ansi_cyan: Color32,
    pub ansi_white: Color32,

    pub ansi_bright_black: Color32,
    pub ansi_bright_red: Color32,
    pub ansi_bright_green: Color32,
    pub ansi_bright_yellow: Color32,
    pub ansi_bright_blue: Color32,
    pub ansi_bright_magenta: Color32,
    pub ansi_bright_cyan: Color32,
    pub ansi_bright_white: Color32,
}

impl ITerm2Theme {
    /// Parse an iTerm2 color scheme from a plist file
    pub fn from_plist(data: &[u8]) -> Result<Self, String> {
        let plist = Value::from_reader(std::io::Cursor::new(data))
            .map_err(|e| format!("Failed to parse plist: {}", e))?;

        let dict = plist.as_dictionary()
            .ok_or("Root element is not a dictionary")?;

        Ok(ITerm2Theme {
            name: "Custom".to_string(),
            foreground: parse_color(dict, "Foreground Color")?,
            background: parse_color(dict, "Background Color")?,
            cursor: parse_color(dict, "Cursor Color")?,
            selection: parse_color(dict, "Selection Color")?,

            ansi_black: parse_color(dict, "Ansi 0 Color")?,
            ansi_red: parse_color(dict, "Ansi 1 Color")?,
            ansi_green: parse_color(dict, "Ansi 2 Color")?,
            ansi_yellow: parse_color(dict, "Ansi 3 Color")?,
            ansi_blue: parse_color(dict, "Ansi 4 Color")?,
            ansi_magenta: parse_color(dict, "Ansi 5 Color")?,
            ansi_cyan: parse_color(dict, "Ansi 6 Color")?,
            ansi_white: parse_color(dict, "Ansi 7 Color")?,

            ansi_bright_black: parse_color(dict, "Ansi 8 Color")?,
            ansi_bright_red: parse_color(dict, "Ansi 9 Color")?,
            ansi_bright_green: parse_color(dict, "Ansi 10 Color")?,
            ansi_bright_yellow: parse_color(dict, "Ansi 11 Color")?,
            ansi_bright_blue: parse_color(dict, "Ansi 12 Color")?,
            ansi_bright_magenta: parse_color(dict, "Ansi 13 Color")?,
            ansi_bright_cyan: parse_color(dict, "Ansi 14 Color")?,
            ansi_bright_white: parse_color(dict, "Ansi 15 Color")?,
        })
    }

    /// Apply this theme to egui visuals
    pub fn apply_to_egui(&self, visuals: &mut egui::Visuals) {
        // Background colors
        visuals.widgets.noninteractive.bg_fill = self.background;
        visuals.widgets.inactive.bg_fill = self.background;
        visuals.extreme_bg_color = self.background;
        visuals.faint_bg_color = blend_colors(self.background, self.foreground, 0.05);

        // Foreground/text colors
        visuals.widgets.noninteractive.fg_stroke.color = self.foreground;
        visuals.widgets.inactive.fg_stroke.color = self.foreground;
        visuals.widgets.active.fg_stroke.color = self.foreground;
        visuals.text_cursor.stroke.color = self.cursor;

        // Selection
        visuals.selection.bg_fill = self.selection;
        visuals.selection.stroke.color = self.foreground;

        // Accents and highlights
        visuals.hyperlink_color = self.ansi_blue;
        visuals.warn_fg_color = self.ansi_yellow;
        visuals.error_fg_color = self.ansi_red;

        // Window and panel colors
        visuals.window_fill = self.background;
        visuals.panel_fill = blend_colors(self.background, self.foreground, 0.03);

        // Widget backgrounds
        visuals.widgets.inactive.weak_bg_fill = blend_colors(self.background, self.foreground, 0.08);
        visuals.widgets.hovered.weak_bg_fill = blend_colors(self.background, self.foreground, 0.12);
        visuals.widgets.active.weak_bg_fill = blend_colors(self.background, self.foreground, 0.15);

        // Strokes
        visuals.widgets.noninteractive.bg_stroke.color = blend_colors(self.background, self.foreground, 0.2);
        visuals.widgets.inactive.bg_stroke.color = blend_colors(self.background, self.foreground, 0.25);
        visuals.widgets.hovered.bg_stroke.color = self.ansi_blue;
        visuals.widgets.active.bg_stroke.color = self.ansi_bright_blue;
    }

    /// Get a default dark theme (similar to base16-ocean)
    pub fn default_dark() -> Self {
        ITerm2Theme {
            name: "Default Dark".to_string(),
            foreground: Color32::from_rgb(192, 197, 206),
            background: Color32::from_rgb(43, 48, 59),
            cursor: Color32::from_rgb(192, 197, 206),
            selection: Color32::from_rgba_premultiplied(76, 86, 106, 128),

            ansi_black: Color32::from_rgb(43, 48, 59),
            ansi_red: Color32::from_rgb(191, 97, 106),
            ansi_green: Color32::from_rgb(163, 190, 140),
            ansi_yellow: Color32::from_rgb(235, 203, 139),
            ansi_blue: Color32::from_rgb(129, 161, 193),
            ansi_magenta: Color32::from_rgb(180, 142, 173),
            ansi_cyan: Color32::from_rgb(136, 192, 208),
            ansi_white: Color32::from_rgb(216, 222, 233),

            ansi_bright_black: Color32::from_rgb(76, 86, 106),
            ansi_bright_red: Color32::from_rgb(191, 97, 106),
            ansi_bright_green: Color32::from_rgb(163, 190, 140),
            ansi_bright_yellow: Color32::from_rgb(235, 203, 139),
            ansi_bright_blue: Color32::from_rgb(129, 161, 193),
            ansi_bright_magenta: Color32::from_rgb(180, 142, 173),
            ansi_bright_cyan: Color32::from_rgb(143, 191, 189),
            ansi_bright_white: Color32::from_rgb(236, 239, 244),
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        ITerm2Theme {
            name: "Dracula".to_string(),
            foreground: Color32::from_rgb(248, 248, 242),
            background: Color32::from_rgb(40, 42, 54),
            cursor: Color32::from_rgb(248, 248, 242),
            selection: Color32::from_rgba_premultiplied(68, 71, 90, 128),

            ansi_black: Color32::from_rgb(33, 34, 44),
            ansi_red: Color32::from_rgb(255, 85, 85),
            ansi_green: Color32::from_rgb(80, 250, 123),
            ansi_yellow: Color32::from_rgb(241, 250, 140),
            ansi_blue: Color32::from_rgb(189, 147, 249),
            ansi_magenta: Color32::from_rgb(255, 121, 198),
            ansi_cyan: Color32::from_rgb(139, 233, 253),
            ansi_white: Color32::from_rgb(248, 248, 242),

            ansi_bright_black: Color32::from_rgb(98, 114, 164),
            ansi_bright_red: Color32::from_rgb(255, 110, 103),
            ansi_bright_green: Color32::from_rgb(90, 247, 142),
            ansi_bright_yellow: Color32::from_rgb(244, 249, 157),
            ansi_bright_blue: Color32::from_rgb(206, 147, 216),
            ansi_bright_magenta: Color32::from_rgb(255, 146, 208),
            ansi_bright_cyan: Color32::from_rgb(154, 237, 254),
            ansi_bright_white: Color32::from_rgb(255, 255, 255),
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        ITerm2Theme {
            name: "Nord".to_string(),
            foreground: Color32::from_rgb(216, 222, 233),
            background: Color32::from_rgb(46, 52, 64),
            cursor: Color32::from_rgb(216, 222, 233),
            selection: Color32::from_rgba_premultiplied(67, 76, 94, 128),

            ansi_black: Color32::from_rgb(59, 66, 82),
            ansi_red: Color32::from_rgb(191, 97, 106),
            ansi_green: Color32::from_rgb(163, 190, 140),
            ansi_yellow: Color32::from_rgb(235, 203, 139),
            ansi_blue: Color32::from_rgb(129, 161, 193),
            ansi_magenta: Color32::from_rgb(180, 142, 173),
            ansi_cyan: Color32::from_rgb(136, 192, 208),
            ansi_white: Color32::from_rgb(229, 233, 240),

            ansi_bright_black: Color32::from_rgb(76, 86, 106),
            ansi_bright_red: Color32::from_rgb(191, 97, 106),
            ansi_bright_green: Color32::from_rgb(163, 190, 140),
            ansi_bright_yellow: Color32::from_rgb(235, 203, 139),
            ansi_bright_blue: Color32::from_rgb(129, 161, 193),
            ansi_bright_magenta: Color32::from_rgb(180, 142, 173),
            ansi_bright_cyan: Color32::from_rgb(143, 188, 187),
            ansi_bright_white: Color32::from_rgb(236, 239, 244),
        }
    }

    /// One Dark theme (similar to Atom/VSCode)
    pub fn one_dark() -> Self {
        ITerm2Theme {
            name: "One Dark".to_string(),
            foreground: Color32::from_rgb(171, 178, 191),
            background: Color32::from_rgb(40, 44, 52),
            cursor: Color32::from_rgb(171, 178, 191),
            selection: Color32::from_rgba_premultiplied(56, 61, 71, 128),

            ansi_black: Color32::from_rgb(40, 44, 52),
            ansi_red: Color32::from_rgb(224, 108, 117),
            ansi_green: Color32::from_rgb(152, 195, 121),
            ansi_yellow: Color32::from_rgb(229, 192, 123),
            ansi_blue: Color32::from_rgb(97, 175, 239),
            ansi_magenta: Color32::from_rgb(198, 120, 221),
            ansi_cyan: Color32::from_rgb(86, 182, 194),
            ansi_white: Color32::from_rgb(171, 178, 191),

            ansi_bright_black: Color32::from_rgb(92, 99, 112),
            ansi_bright_red: Color32::from_rgb(224, 108, 117),
            ansi_bright_green: Color32::from_rgb(152, 195, 121),
            ansi_bright_yellow: Color32::from_rgb(229, 192, 123),
            ansi_bright_blue: Color32::from_rgb(97, 175, 239),
            ansi_bright_magenta: Color32::from_rgb(198, 120, 221),
            ansi_bright_cyan: Color32::from_rgb(86, 182, 194),
            ansi_bright_white: Color32::from_rgb(255, 255, 255),
        }
    }

    /// Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        ITerm2Theme {
            name: "Gruvbox Dark".to_string(),
            foreground: Color32::from_rgb(235, 219, 178),
            background: Color32::from_rgb(40, 40, 40),
            cursor: Color32::from_rgb(235, 219, 178),
            selection: Color32::from_rgba_premultiplied(80, 73, 69, 128),

            ansi_black: Color32::from_rgb(40, 40, 40),
            ansi_red: Color32::from_rgb(204, 36, 29),
            ansi_green: Color32::from_rgb(152, 151, 26),
            ansi_yellow: Color32::from_rgb(215, 153, 33),
            ansi_blue: Color32::from_rgb(69, 133, 136),
            ansi_magenta: Color32::from_rgb(177, 98, 134),
            ansi_cyan: Color32::from_rgb(104, 157, 106),
            ansi_white: Color32::from_rgb(168, 153, 132),

            ansi_bright_black: Color32::from_rgb(146, 131, 116),
            ansi_bright_red: Color32::from_rgb(251, 73, 52),
            ansi_bright_green: Color32::from_rgb(184, 187, 38),
            ansi_bright_yellow: Color32::from_rgb(250, 189, 47),
            ansi_bright_blue: Color32::from_rgb(131, 165, 152),
            ansi_bright_magenta: Color32::from_rgb(211, 134, 155),
            ansi_bright_cyan: Color32::from_rgb(142, 192, 124),
            ansi_bright_white: Color32::from_rgb(235, 219, 178),
        }
    }
}

/// Parse a color from an iTerm2 plist dictionary
fn parse_color(dict: &Dictionary, key: &str) -> Result<Color32, String> {
    let color_dict = dict.get(key)
        .and_then(|v| v.as_dictionary())
        .ok_or(format!("Missing color: {}", key))?;

    // iTerm2 colors are stored as floats from 0.0 to 1.0
    let r = color_dict.get("Red Component")
        .and_then(|v| v.as_real().or_else(|| v.as_unsigned_integer().map(|i| i as f64)))
        .ok_or(format!("Missing red component for {}", key))?;

    let g = color_dict.get("Green Component")
        .and_then(|v| v.as_real().or_else(|| v.as_unsigned_integer().map(|i| i as f64)))
        .ok_or(format!("Missing green component for {}", key))?;

    let b = color_dict.get("Blue Component")
        .and_then(|v| v.as_real().or_else(|| v.as_unsigned_integer().map(|i| i as f64)))
        .ok_or(format!("Missing blue component for {}", key))?;

    Ok(Color32::from_rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    ))
}

/// Blend two colors together
fn blend_colors(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
        (a.a() as f32 * (1.0 - t) + b.a() as f32 * t) as u8,
    )
}
