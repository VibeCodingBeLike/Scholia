use crate::keybinds::{Action, KeyName, KeybindConfig};
use crate::theme::{BlurMode, MlaFontChoice, ThemeConfig, ThemePreset};
use egui::{Color32, RichText, Window};

pub struct SettingsModalState {
    pub is_open: bool,
    pub selected_tab: SettingsTab,
    pub editing_action: Option<Action>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    TransparencyAndTheme,
    Keybindings,
    AboutMla,
}

impl Default for SettingsModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_tab: SettingsTab::TransparencyAndTheme,
            editing_action: None,
        }
    }
}

pub fn render_settings_modal(
    ctx: &egui::Context,
    state: &mut SettingsModalState,
    theme: &mut ThemeConfig,
    keybinds: &mut KeybindConfig,
    vibrancy_dirty: &mut bool,
) {
    if !state.is_open {
        return;
    }

    let mut open = true;
    let mut close_modal = false;

    Window::new("Preferences & Customization")
        .open(&mut open)
        .resizable(true)
        .default_width(620.0)
        .default_height(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut state.selected_tab,
                    SettingsTab::TransparencyAndTheme,
                    "🎨 Transparency & Themes",
                );
                ui.selectable_value(
                    &mut state.selected_tab,
                    SettingsTab::Keybindings,
                    "⌨️ Custom Keybinds",
                );
                ui.selectable_value(
                    &mut state.selected_tab,
                    SettingsTab::AboutMla,
                    "📖 MLA 9 Standards Guide",
                );
            });

            ui.separator();

            match state.selected_tab {
                SettingsTab::TransparencyAndTheme => {
                    render_transparency_tab(ui, theme, vibrancy_dirty);
                }
                SettingsTab::Keybindings => {
                    render_keybindings_tab(ui, keybinds, theme);
                }
                SettingsTab::AboutMla => {
                    render_mla_guide_tab(ui, theme);
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    close_modal = true;
                }
            });
        });

    if !open || close_modal {
        state.is_open = false;
    }
}

fn render_transparency_tab(ui: &mut egui::Ui, theme: &mut ThemeConfig, vibrancy_dirty: &mut bool) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 12.0;

        // Theme Preset Quick Selector
        ui.group(|ui| {
            ui.label(RichText::new("Theme Presets:").strong());
            ui.horizontal(|ui| {
                if ui.button("🌙 Frosted Obsidian (Dark)").clicked() {
                    theme.apply_preset(ThemePreset::FrostedDark);
                    *vibrancy_dirty = true;
                }
                if ui.button("☀️ Frosted Parchment (Light)").clicked() {
                    theme.apply_preset(ThemePreset::FrostedLight);
                    *vibrancy_dirty = true;
                }
                if ui.button("❄️ Nordic Frost").clicked() {
                    theme.apply_preset(ThemePreset::NordicFrost);
                    *vibrancy_dirty = true;
                }
                if ui.button("🕯️ Amber Glass").clicked() {
                    theme.apply_preset(ThemePreset::AmberTerminal);
                    *vibrancy_dirty = true;
                }
            });
        });

        // Window Blur & Native Vibrancy
        ui.group(|ui| {
            ui.label(RichText::new("Native Background Blur / Vibrancy:").strong());
            ui.label(
                RichText::new("Enables OS-level acrylic, mica, or vibrancy behind the window.")
                    .size(11.5)
                    .color(theme.muted_text_color()),
            );

            let prev_blur = theme.blur_mode;
            egui::ComboBox::from_id_salt("blur_mode_cb")
                .selected_text(theme.blur_mode.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut theme.blur_mode, BlurMode::Mica, BlurMode::Mica.display_name());
                    ui.selectable_value(&mut theme.blur_mode, BlurMode::Acrylic, BlurMode::Acrylic.display_name());
                    ui.selectable_value(&mut theme.blur_mode, BlurMode::SystemBlur, BlurMode::SystemBlur.display_name());
                    ui.selectable_value(&mut theme.blur_mode, BlurMode::TransparentOnly, BlurMode::TransparentOnly.display_name());
                });

            if prev_blur != theme.blur_mode {
                *vibrancy_dirty = true;
            }
        });

        // Transparency Sliders
        ui.group(|ui| {
            ui.label(RichText::new("Transparency Controls:").strong());

            let win_label = format!("{:.0}%", theme.window_opacity * 100.0);
            ui.horizontal(|ui| {
                ui.label("Window Background Opacity:");
                let slider = egui::Slider::new(&mut theme.window_opacity, 0.05..=1.0)
                    .text(win_label);
                if ui.add(slider).changed() {
                    *vibrancy_dirty = true;
                }
            });

            let page_label = format!("{:.0}%", theme.page_opacity * 100.0);
            ui.horizontal(|ui| {
                ui.label("Writing Page Sheet Opacity:");
                ui.add(
                    egui::Slider::new(&mut theme.page_opacity, 0.05..=1.0)
                        .text(page_label),
                );
            });
            ui.label(
                RichText::new("Tip: Lower the Page Sheet Opacity to make the paper you write on completely translucent with your desktop shining through!")
                    .size(11.0)
                    .italics()
                    .color(theme.muted_text_color()),
            );
        });

        // Custom Color Customization
        ui.group(|ui| {
            ui.label(RichText::new("Custom Color Palette:").strong());

            egui::Grid::new("color_palette_grid")
                .num_columns(2)
                .spacing([14.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Window Tint Color:");
                    color_picker_rgb(ui, &mut theme.window_tint_rgb);
                    ui.end_row();

                    ui.label("Page Sheet Tint Color:");
                    color_picker_rgb(ui, &mut theme.page_tint_rgb);
                    ui.end_row();

                    ui.label("Text Color:");
                    color_picker_rgb(ui, &mut theme.text_rgb);
                    ui.end_row();

                    ui.label("Accent / Highlight Color:");
                    color_picker_rgb(ui, &mut theme.accent_rgb);
                    ui.end_row();

                    ui.label("Page Border Color:");
                    color_picker_rgb(ui, &mut theme.page_border_rgb);
                    ui.end_row();
                });
        });

        // Font selection (strictly MLA approved serif fonts)
        ui.group(|ui| {
            ui.label(RichText::new("MLA Approved Typeface:").strong());
            ui.label(
                RichText::new("MLA 9 permits standard, legible serif fonts. Size is strictly locked to 12 pt.")
                    .size(11.5)
                    .color(theme.muted_text_color()),
            );

            egui::ComboBox::from_id_salt("font_choice_cb")
                .selected_text(theme.font_choice.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut theme.font_choice,
                        MlaFontChoice::TimesNewRoman,
                        MlaFontChoice::TimesNewRoman.display_name(),
                    );
                    ui.selectable_value(
                        &mut theme.font_choice,
                        MlaFontChoice::Georgia,
                        MlaFontChoice::Georgia.display_name(),
                    );
                    ui.selectable_value(
                        &mut theme.font_choice,
                        MlaFontChoice::Palatino,
                        MlaFontChoice::Palatino.display_name(),
                    );
                    ui.selectable_value(
                        &mut theme.font_choice,
                        MlaFontChoice::LiberationSerif,
                        MlaFontChoice::LiberationSerif.display_name(),
                    );
                });
        });

        // Interface Preferences
        ui.group(|ui| {
            ui.label(RichText::new("Interface & Layout Options:").strong());
            ui.checkbox(
                &mut theme.show_window_controls,
                "Show window navigation controls (Minimize, Maximize, Close) in top-right",
            );
            ui.checkbox(
                &mut theme.show_shortcuts_panel,
                "Show shortcuts helper panel alongside document page",
            );
        });
    });
}

fn color_picker_rgb(ui: &mut egui::Ui, rgb: &mut [u8; 3]) {
    let mut color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    if ui.color_edit_button_srgba(&mut color).changed() {
        rgb[0] = color.r();
        rgb[1] = color.g();
        rgb[2] = color.b();
    }
}

fn render_keybindings_tab(ui: &mut egui::Ui, keybinds: &mut KeybindConfig, theme: &ThemeConfig) {
    ui.label(RichText::new("Custom Keybindings:").strong());
    ui.label(
        RichText::new("All major editor actions have dedicated, remappable key combinations.")
            .size(11.5)
            .color(theme.muted_text_color()),
    );

    ui.add_space(4.0);

    if ui.button("↺ Reset All Keybinds to Defaults").clicked() {
        *keybinds = KeybindConfig::default();
    }

    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("keybindings_grid")
            .num_columns(3)
            .spacing([16.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Action").strong());
                ui.label(RichText::new("Category").strong());
                ui.label(RichText::new("Shortcut").strong());
                ui.end_row();

                for &action in Action::all() {
                    ui.label(action.display_name());
                    ui.label(
                        RichText::new(action.category())
                            .size(11.0)
                            .color(theme.muted_text_color()),
                    );

                    let mut sc = keybinds.get_shortcut(action);
                    let mut changed = false;

                    ui.horizontal(|ui| {
                        changed |= ui.checkbox(&mut sc.ctrl, "Ctrl").changed();
                        changed |= ui.checkbox(&mut sc.shift, "Shift").changed();
                        changed |= ui.checkbox(&mut sc.alt, "Alt").changed();

                        egui::ComboBox::from_id_salt(format!("kb_{:?}", action))
                            .selected_text(sc.key.label())
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                for key in [
                                    KeyName::N,
                                    KeyName::O,
                                    KeyName::S,
                                    KeyName::E,
                                    KeyName::B,
                                    KeyName::C,
                                    KeyName::W,
                                    KeyName::T,
                                    KeyName::V,
                                    KeyName::P,
                                    KeyName::Comma,
                                    KeyName::Enter,
                                    KeyName::F11,
                                    KeyName::Num1,
                                    KeyName::Num2,
                                ] {
                                    ui.selectable_value(&mut sc.key, key, key.label());
                                }
                            });
                    });

                    if changed {
                        keybinds.set_shortcut(action, sc);
                    }

                    ui.end_row();
                }
            });
    });
}

fn render_mla_guide_tab(ui: &mut egui::Ui, theme: &ThemeConfig) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        ui.heading("Modern Language Association (MLA) 9th Edition Quick Rules");

        ui.label(RichText::new("1. Document Geometry").strong().color(theme.accent_color()));
        ui.label("• Margins: Exactly 1.0 inch (72 points) on all four sides.\n• Spacing: Strict double spacing throughout the entire paper—no extra space between paragraphs or headings.\n• Font: Legible standard serif (e.g. Times New Roman) strictly 12 pt throughout.");

        ui.label(RichText::new("2. First-Page Identification & Header").strong().color(theme.accent_color()));
        ui.label("• Student Name, Instructor Name, Course Title, and Date aligned flush left on four consecutive double-spaced lines.\n• Date format: Day Month Year (e.g., 20 September 2026).\n• Running Head: Student's last name + Page Number at top-right 0.5 in from top edge.");

        ui.label(RichText::new("3. Paper Title").strong().color(theme.accent_color()));
        ui.label("• Centered, standard 12pt font.\n• Not bolded, underlined, italicized, or placed in quotation marks.\n• Follows MLA Title Capitalization rules.");

        ui.label(RichText::new("4. Body Paragraphs & Block Quotes").strong().color(theme.accent_color()));
        ui.label("• First line of every body paragraph indented exactly 0.5 inches.\n• Quotations exceeding 4 lines of prose or 3 lines of verse are formatted as Block Quotes: indented 0.5 in from left margin, quotation marks omitted, and parenthetical citation outside the final period.");

        ui.label(RichText::new("5. Works Cited Page").strong().color(theme.accent_color()));
        ui.label("• Starts on a separate page at the end of the manuscript.\n• Centered heading 'Works Cited' (or 'Work Cited' if only one source).\n• 0.5-inch hanging indent for each entry.\n• Alphabetized by author's last name or title.");
    });
}
