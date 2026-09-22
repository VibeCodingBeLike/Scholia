use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlurMode {
    Mica,
    Acrylic,
    SystemBlur,
    TransparentOnly,
}

impl BlurMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            BlurMode::Mica => "Windows 11 Mica / macOS Vibrancy",
            BlurMode::Acrylic => "Windows Acrylic Frosted Glass",
            BlurMode::SystemBlur => "System Window Blur",
            BlurMode::TransparentOnly => "Transparent (No Blur)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    FrostedDark,
    FrostedLight,
    NordicFrost,
    AmberTerminal,
    RosePine,
    RosePineMoon,
    RosePineDawn,
    Custom,
}

impl ThemePreset {
    pub fn all_presets() -> &'static [ThemePreset] {
        &[
            ThemePreset::FrostedDark,
            ThemePreset::FrostedLight,
            ThemePreset::NordicFrost,
            ThemePreset::AmberTerminal,
            ThemePreset::RosePine,
            ThemePreset::RosePineMoon,
            ThemePreset::RosePineDawn,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ThemePreset::FrostedDark => "Frosted Obsidian (Dark)",
            ThemePreset::FrostedLight => "Frosted Parchment (Light)",
            ThemePreset::NordicFrost => "Nordic Frost (Deep Blue)",
            ThemePreset::AmberTerminal => "Amber Glass (Warm Retro)",
            ThemePreset::RosePine => "Rosé Pine (Dark Rose)",
            ThemePreset::RosePineMoon => "Rosé Pine Moon (Deep Violet)",
            ThemePreset::RosePineDawn => "Rosé Pine Dawn (Warm Pastel)",
            ThemePreset::Custom => "Custom Theme",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ThemePreset::FrostedDark => "🌙",
            ThemePreset::FrostedLight => "☀️",
            ThemePreset::NordicFrost => "❄️",
            ThemePreset::AmberTerminal => "🕯️",
            ThemePreset::RosePine => "🌹",
            ThemePreset::RosePineMoon => "🌑",
            ThemePreset::RosePineDawn => "🌅",
            ThemePreset::Custom => "🎨",
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            ThemePreset::FrostedDark
            | ThemePreset::NordicFrost
            | ThemePreset::AmberTerminal
            | ThemePreset::RosePine
            | ThemePreset::RosePineMoon => true,
            ThemePreset::FrostedLight | ThemePreset::RosePineDawn => false,
            ThemePreset::Custom => true,
        }
    }

    pub fn preview_palette(&self) -> ([u8; 3], [u8; 3], [u8; 3], [u8; 3]) {
        match self {
            ThemePreset::FrostedDark => {
                ([18, 20, 26], [28, 32, 42], [240, 243, 248], [70, 145, 235])
            }
            ThemePreset::FrostedLight => (
                [235, 240, 245],
                [255, 255, 255],
                [20, 25, 30],
                [30, 110, 210],
            ),
            ThemePreset::NordicFrost => {
                ([15, 23, 34], [24, 34, 48], [236, 242, 248], [136, 192, 208])
            }
            ThemePreset::AmberTerminal => {
                ([22, 18, 14], [35, 28, 22], [255, 205, 120], [240, 150, 40])
            }
            ThemePreset::RosePine => ([25, 23, 36], [31, 29, 46], [224, 222, 244], [235, 188, 186]),
            ThemePreset::RosePineMoon => {
                ([35, 33, 54], [42, 39, 63], [224, 222, 244], [234, 154, 151])
            }
            ThemePreset::RosePineDawn => (
                [250, 244, 237],
                [255, 250, 243],
                [87, 82, 121],
                [215, 130, 126],
            ),
            ThemePreset::Custom => ([20, 20, 25], [30, 30, 40], [240, 240, 245], [150, 150, 200]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub preset: ThemePreset,
    #[serde(default)]
    pub custom_name: Option<String>,
    pub blur_mode: BlurMode,

    /// Opacity of the background application window (0.05 to 1.0)
    pub window_opacity: f32,
    /// Opacity of the paper sheet canvas you write on (0.05 to 1.0)
    pub page_opacity: f32,

    /// RGB for main window background tint
    pub window_tint_rgb: [u8; 3],
    /// RGB for paper sheet background
    pub page_tint_rgb: [u8; 3],
    /// RGB for primary text (e.g. paper content)
    pub text_rgb: [u8; 3],
    /// RGB for secondary/muted labels and icons
    pub muted_text_rgb: [u8; 3],
    /// RGB for accent buttons, focus highlights, badges
    pub accent_rgb: [u8; 3],
    /// RGB for paper border / guidelines
    pub page_border_rgb: [u8; 3],
    /// Border alpha (0.0 to 1.0)
    pub page_border_alpha: f32,

    pub font_choice: MlaFontChoice,

    /// Whether to show window navigation controls (Minimize, Maximize, Close) on the top right
    #[serde(default = "default_true")]
    pub show_window_controls: bool,

    /// Whether to show the shortcuts helper panel alongside the document
    #[serde(default = "default_true")]
    pub show_shortcuts_panel: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MlaFontChoice {
    TimesNewRoman,
    Georgia,
    Palatino,
    LiberationSerif,
}

impl MlaFontChoice {
    pub fn display_name(&self) -> &'static str {
        match self {
            MlaFontChoice::TimesNewRoman => "Times New Roman (Standard MLA)",
            MlaFontChoice::Georgia => "Georgia (Approved Modern Serif)",
            MlaFontChoice::Palatino => "Palatino Linotype (Classic Serif)",
            MlaFontChoice::LiberationSerif => "Liberation Serif (Cross-platform Serif)",
        }
    }

    pub fn css_font_family(&self) -> &'static str {
        match self {
            MlaFontChoice::TimesNewRoman => "'Times New Roman', Times, serif",
            MlaFontChoice::Georgia => "Georgia, serif",
            MlaFontChoice::Palatino => "'Palatino Linotype', 'Book Antiqua', Palatino, serif",
            MlaFontChoice::LiberationSerif => "'Liberation Serif', Times, serif",
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::preset_frosted_dark()
    }
}

impl ThemeConfig {
    pub fn preset_frosted_dark() -> Self {
        Self {
            preset: ThemePreset::FrostedDark,
            custom_name: None,
            blur_mode: BlurMode::Acrylic,
            window_opacity: 0.65,
            page_opacity: 0.40, // Elegant translucent frosted writing sheet!
            window_tint_rgb: [18, 20, 26],
            page_tint_rgb: [28, 32, 42],
            text_rgb: [240, 243, 248],
            muted_text_rgb: [150, 160, 175],
            accent_rgb: [70, 145, 235], // Frost Blue
            page_border_rgb: [90, 110, 140],
            page_border_alpha: 0.35,
            font_choice: MlaFontChoice::TimesNewRoman,
            show_window_controls: true,
            show_shortcuts_panel: true,
        }
    }

    pub fn preset_frosted_light() -> Self {
        Self {
            preset: ThemePreset::FrostedLight,
            custom_name: None,
            blur_mode: BlurMode::Acrylic,
            window_opacity: 0.75,
            page_opacity: 0.55,
            window_tint_rgb: [235, 240, 245],
            page_tint_rgb: [255, 255, 255],
            text_rgb: [20, 25, 30],
            muted_text_rgb: [100, 110, 125],
            accent_rgb: [30, 110, 210],
            page_border_rgb: [180, 195, 210],
            page_border_alpha: 0.50,
            font_choice: MlaFontChoice::TimesNewRoman,
            show_window_controls: true,
            show_shortcuts_panel: true,
        }
    }

    pub fn preset_nordic_frost() -> Self {
        Self {
            preset: ThemePreset::NordicFrost,
            custom_name: None,
            blur_mode: BlurMode::Mica,
            window_opacity: 0.60,
            page_opacity: 0.38,
            window_tint_rgb: [15, 23, 34],
            page_tint_rgb: [24, 34, 48],
            text_rgb: [236, 242, 248],
            muted_text_rgb: [142, 160, 180],
            accent_rgb: [136, 192, 208], // Nord Frost cyan
            page_border_rgb: [76, 86, 106],
            page_border_alpha: 0.40,
            font_choice: MlaFontChoice::TimesNewRoman,
            show_window_controls: true,
            show_shortcuts_panel: true,
        }
    }

    pub fn preset_amber_terminal() -> Self {
        Self {
            preset: ThemePreset::AmberTerminal,
            custom_name: None,
            blur_mode: BlurMode::Acrylic,
            window_opacity: 0.70,
            page_opacity: 0.45,
            window_tint_rgb: [22, 18, 14],
            page_tint_rgb: [35, 28, 22],
            text_rgb: [255, 205, 120],
            muted_text_rgb: [185, 145, 95],
            accent_rgb: [240, 150, 40],
            page_border_rgb: [120, 90, 50],
            page_border_alpha: 0.40,
            font_choice: MlaFontChoice::Georgia,
            show_window_controls: true,
            show_shortcuts_panel: true,
        }
    }

    pub fn preset_rose_pine() -> Self {
        Self {
            preset: ThemePreset::RosePine,
            custom_name: None,
            blur_mode: BlurMode::Acrylic,
            window_opacity: 0.65,
            page_opacity: 0.40,
            window_tint_rgb: [25, 23, 36],   // Base
            page_tint_rgb: [31, 29, 46],     // Surface
            text_rgb: [224, 222, 244],       // Text
            muted_text_rgb: [144, 140, 170], // Muted
            accent_rgb: [235, 188, 186],     // Rose
            page_border_rgb: [64, 61, 82],   // Highlight Med
            page_border_alpha: 0.40,
            font_choice: MlaFontChoice::TimesNewRoman,
            show_window_controls: true,
            show_shortcuts_panel: true,
        }
    }

    pub fn preset_rose_pine_moon() -> Self {
        Self {
            preset: ThemePreset::RosePineMoon,
            custom_name: None,
            blur_mode: BlurMode::Acrylic,
            window_opacity: 0.65,
            page_opacity: 0.40,
            window_tint_rgb: [35, 33, 54],   // Base
            page_tint_rgb: [42, 39, 63],     // Surface
            text_rgb: [224, 222, 244],       // Text
            muted_text_rgb: [144, 140, 170], // Muted
            accent_rgb: [234, 154, 151],     // Rose
            page_border_rgb: [68, 65, 90],   // Highlight Med
            page_border_alpha: 0.40,
            font_choice: MlaFontChoice::TimesNewRoman,
            show_window_controls: true,
            show_shortcuts_panel: true,
        }
    }

    pub fn preset_rose_pine_dawn() -> Self {
        Self {
            preset: ThemePreset::RosePineDawn,
            custom_name: None,
            blur_mode: BlurMode::Acrylic,
            window_opacity: 0.75,
            page_opacity: 0.55,
            window_tint_rgb: [250, 244, 237], // Base
            page_tint_rgb: [255, 250, 243],   // Surface
            text_rgb: [87, 82, 121],          // Text
            muted_text_rgb: [152, 147, 165],  // Muted
            accent_rgb: [215, 130, 126],      // Rose
            page_border_rgb: [223, 218, 217], // Highlight Med
            page_border_alpha: 0.50,
            font_choice: MlaFontChoice::TimesNewRoman,
            show_window_controls: true,
            show_shortcuts_panel: true,
        }
    }

    pub fn apply_preset(&mut self, preset: ThemePreset) {
        let font = self.font_choice;
        let win_ctrls = self.show_window_controls;
        let sc_panel = self.show_shortcuts_panel;
        *self = match preset {
            ThemePreset::FrostedDark => Self::preset_frosted_dark(),
            ThemePreset::FrostedLight => Self::preset_frosted_light(),
            ThemePreset::NordicFrost => Self::preset_nordic_frost(),
            ThemePreset::AmberTerminal => Self::preset_amber_terminal(),
            ThemePreset::RosePine => Self::preset_rose_pine(),
            ThemePreset::RosePineMoon => Self::preset_rose_pine_moon(),
            ThemePreset::RosePineDawn => Self::preset_rose_pine_dawn(),
            ThemePreset::Custom => {
                let mut c = self.clone();
                c.preset = ThemePreset::Custom;
                c
            }
        };
        self.font_choice = font;
        self.show_window_controls = win_ctrls;
        self.show_shortcuts_panel = sc_panel;
    }

    /// Color for the main outer app window fill
    pub fn window_fill_color(&self) -> egui::Color32 {
        let a = (self.window_opacity.clamp(0.05, 1.0) * 255.0) as u8;
        egui::Color32::from_rgba_premultiplied(
            ((self.window_tint_rgb[0] as f32) * (a as f32 / 255.0)) as u8,
            ((self.window_tint_rgb[1] as f32) * (a as f32 / 255.0)) as u8,
            ((self.window_tint_rgb[2] as f32) * (a as f32 / 255.0)) as u8,
            a,
        )
    }

    /// Color for the transparent paper page you write on
    pub fn page_fill_color(&self) -> egui::Color32 {
        let a = (self.page_opacity.clamp(0.05, 1.0) * 255.0) as u8;
        egui::Color32::from_rgba_premultiplied(
            ((self.page_tint_rgb[0] as f32) * (a as f32 / 255.0)) as u8,
            ((self.page_tint_rgb[1] as f32) * (a as f32 / 255.0)) as u8,
            ((self.page_tint_rgb[2] as f32) * (a as f32 / 255.0)) as u8,
            a,
        )
    }

    /// Color for floating cards, sidebars, and popups
    pub fn card_fill_color(&self) -> egui::Color32 {
        let a = ((self.page_opacity * 0.85).clamp(0.1, 0.95) * 255.0) as u8;
        egui::Color32::from_rgba_premultiplied(
            ((self.page_tint_rgb[0] as f32) * (a as f32 / 255.0)) as u8,
            ((self.page_tint_rgb[1] as f32) * (a as f32 / 255.0)) as u8,
            ((self.page_tint_rgb[2] as f32) * (a as f32 / 255.0)) as u8,
            a,
        )
    }

    /// Color for modal dialogs and popup windows (10% transparent, 90% opaque)
    pub fn modal_fill_color(&self) -> egui::Color32 {
        let a = 230u8; // 90% opaque (10% transparent)
        egui::Color32::from_rgba_premultiplied(
            ((self.page_tint_rgb[0] as f32) * (a as f32 / 255.0)) as u8,
            ((self.page_tint_rgb[1] as f32) * (a as f32 / 255.0)) as u8,
            ((self.page_tint_rgb[2] as f32) * (a as f32 / 255.0)) as u8,
            a,
        )
    }

    /// Modal dialog window frame with 90% opaque fill and subtle border
    pub fn modal_frame(&self) -> egui::Frame {
        let text_col = self.text_color();
        egui::Frame::new()
            .fill(self.modal_fill_color())
            .stroke(egui::Stroke::new(1.0, text_col.gamma_multiply(0.25)))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(16))
    }

    pub fn text_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.text_rgb[0], self.text_rgb[1], self.text_rgb[2])
    }

    pub fn muted_text_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.muted_text_rgb[0],
            self.muted_text_rgb[1],
            self.muted_text_rgb[2],
        )
    }

    pub fn accent_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.accent_rgb[0], self.accent_rgb[1], self.accent_rgb[2])
    }

    pub fn page_stroke(&self) -> egui::Stroke {
        let a = (self.page_border_alpha.clamp(0.0, 1.0) * 255.0) as u8;
        egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_premultiplied(
                ((self.page_border_rgb[0] as f32) * (a as f32 / 255.0)) as u8,
                ((self.page_border_rgb[1] as f32) * (a as f32 / 255.0)) as u8,
                ((self.page_border_rgb[2] as f32) * (a as f32 / 255.0)) as u8,
                a,
            ),
        )
    }

    /// Apply native OS window vibrancy/blur effects
    pub fn apply_vibrancy_to_window(&self, window: &winit::window::Window) {
        let tint_a = (self.window_opacity.clamp(0.05, 1.0) * 255.0) as u8;
        let tint_tuple = Some((
            self.window_tint_rgb[0],
            self.window_tint_rgb[1],
            self.window_tint_rgb[2],
            tint_a,
        ));

        #[cfg(target_os = "windows")]
        {
            match self.blur_mode {
                BlurMode::Mica => {
                    let _ = window_vibrancy::apply_mica(window, None)
                        .or_else(|_| window_vibrancy::apply_acrylic(window, tint_tuple))
                        .or_else(|_| window_vibrancy::apply_blur(window, tint_tuple));
                }
                BlurMode::Acrylic => {
                    let _ = window_vibrancy::apply_acrylic(window, tint_tuple)
                        .or_else(|_| window_vibrancy::apply_blur(window, tint_tuple));
                }
                BlurMode::SystemBlur => {
                    let _ = window_vibrancy::apply_blur(window, tint_tuple);
                }
                BlurMode::TransparentOnly => {
                    let _ = window_vibrancy::clear_acrylic(window);
                    let _ = window_vibrancy::clear_blur(window);
                    let _ = window_vibrancy::clear_mica(window);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            match self.blur_mode {
                BlurMode::TransparentOnly => {
                    let _ = window_vibrancy::clear_vibrancy(window);
                }
                _ => {
                    let _ = window_vibrancy::apply_vibrancy(
                        window,
                        window_vibrancy::NSVisualEffectMaterial::UnderWindowBackground,
                        None,
                        None,
                    );
                }
            }
        }
    }

    pub fn is_dark(&self) -> bool {
        match self.preset {
            ThemePreset::FrostedDark
            | ThemePreset::NordicFrost
            | ThemePreset::AmberTerminal
            | ThemePreset::RosePine
            | ThemePreset::RosePineMoon => true,
            ThemePreset::FrostedLight | ThemePreset::RosePineDawn => false,
            ThemePreset::Custom => {
                let [r, g, b] = self.window_tint_rgb;
                let lum = 0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);
                lum < 128.0
            }
        }
    }

    pub fn create_egui_visuals(&self) -> egui::Visuals {
        let mut visuals = if self.is_dark() {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        let text_col = self.text_color();
        let accent_col = self.accent_color();

        // Inactive widgets: transparent background so frosted glass shines through
        visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, text_col.gamma_multiply(0.18));
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_col);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(5);

        // Hovered widgets: soft frosted highlight with accent stroke
        visuals.widgets.hovered.bg_fill = accent_col.gamma_multiply(0.20);
        visuals.widgets.hovered.weak_bg_fill = accent_col.gamma_multiply(0.15);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent_col.gamma_multiply(0.65));
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text_col);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(5);

        // Active (clicked/pressed) widgets: deeper accent highlight
        visuals.widgets.active.bg_fill = accent_col.gamma_multiply(0.35);
        visuals.widgets.active.weak_bg_fill = accent_col.gamma_multiply(0.30);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent_col);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, text_col);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(5);

        // Open widgets (e.g. open dropdowns/menus): translucent accent tint
        visuals.widgets.open.bg_fill = accent_col.gamma_multiply(0.22);
        visuals.widgets.open.weak_bg_fill = accent_col.gamma_multiply(0.18);
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, accent_col.gamma_multiply(0.70));
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text_col);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(5);

        // Noninteractive widgets (e.g. disabled buttons, label backgrounds): transparent
        visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0, text_col.gamma_multiply(0.10));
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.muted_text_color());
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5);

        // Window & dialog styling: 90% opaque modal fill for readable popups, transparent main background
        visuals.window_fill = self.modal_fill_color();
        visuals.panel_fill = self.window_fill_color();
        visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
        visuals.window_stroke = egui::Stroke::new(1.0, text_col.gamma_multiply(0.20));

        visuals
    }
}

// ---------------------------------------------------------------------------
// Theme Folder Management (Save, Import, Share Themes)
// ---------------------------------------------------------------------------

pub fn themes_dir() -> std::path::PathBuf {
    // Check if themes directory exists next to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("themes");
            if p.exists() {
                return p;
            }
        }
    }
    // Default to ./themes in current working directory
    std::path::PathBuf::from("themes")
}

pub fn ensure_sample_themes() {
    let dir = themes_dir();
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
}

pub fn list_custom_themes() -> Vec<(String, std::path::PathBuf)> {
    let dir = themes_dir();
    let mut themes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unnamed")
                    .to_string();
                themes.push((stem, path));
            }
        }
    }
    themes.sort_by(|a, b| a.0.cmp(&b.0));
    themes
}

pub fn save_custom_theme(theme: &ThemeConfig, name: &str) -> Result<std::path::PathBuf, String> {
    let dir = themes_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create themes directory: {e}"))?;
    }
    let safe_name: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    let trimmed = safe_name.trim();
    if trimmed.is_empty() {
        return Err("Theme name cannot be empty".to_string());
    }
    let mut saved_theme = theme.clone();
    saved_theme.preset = ThemePreset::Custom;
    saved_theme.custom_name = Some(trimmed.to_string());
    let json = serde_json::to_string_pretty(&saved_theme)
        .map_err(|e| format!("Failed to serialize theme: {e}"))?;
    let file_path = dir.join(format!("{}.json", trimmed));
    std::fs::write(&file_path, json).map_err(|e| format!("Failed to write theme file: {e}"))?;
    Ok(file_path)
}

pub fn load_theme_file(path: &std::path::Path) -> Result<ThemeConfig, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read theme file: {e}"))?;
    let mut theme: ThemeConfig =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse theme JSON: {e}"))?;
    if theme.custom_name.is_none() {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            theme.custom_name = Some(stem.to_string());
        }
    }
    Ok(theme)
}

pub fn open_themes_folder() {
    let dir = themes_dir();
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&dir).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
    }
}
