use crate::keybinds::{Action, KeyName, KeybindConfig};
use crate::theme::{BlurMode, MlaFontChoice, ThemeConfig, ThemePreset};
use egui::{Color32, RichText, Window};

pub struct SettingsModalState {
    pub is_open: bool,
    pub selected_tab: SettingsTab,
    pub editing_action: Option<Action>,
    pub new_theme_name: String,
    pub theme_message: Option<(String, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    TransparencyAndTheme,
    Keybindings,
}

impl Default for SettingsModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_tab: SettingsTab::TransparencyAndTheme,
            editing_action: None,
            new_theme_name: String::new(),
            theme_message: None,
        }
    }
}

pub fn render_settings_modal(
    ctx: &egui::Context,
    state: &mut SettingsModalState,
    theme: &mut ThemeConfig,
    keybinds: &mut KeybindConfig,
    undo_limit: &mut usize,
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
        .frame(theme.modal_frame())
        .default_width(640.0)
        .default_height(540.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            // Tab bar with version display
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

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .size(12.5)
                            .strong()
                            .color(theme.accent_color()),
                    );
                });
            });

            ui.separator();

            match state.selected_tab {
                SettingsTab::TransparencyAndTheme => {
                    render_transparency_tab(ui, state, theme, undo_limit, vibrancy_dirty);
                }
                SettingsTab::Keybindings => {
                    render_keybindings_tab(ui, keybinds, theme);
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    close_modal = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Scholia v{}", env!("CARGO_PKG_VERSION")))
                            .size(11.5)
                            .italics()
                            .color(theme.muted_text_color()),
                    );
                });
            });
        });

    if !open || close_modal {
        state.is_open = false;
    }
}

fn render_transparency_tab(
    ui: &mut egui::Ui,
    state: &mut SettingsModalState,
    theme: &mut ThemeConfig,
    undo_limit: &mut usize,
    vibrancy_dirty: &mut bool,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 12.0;

        // Theme Preset Cards Grid
        ui.group(|ui| {
            ui.label(RichText::new("Theme Presets:").strong());
            ui.label(
                RichText::new("Select a curated aesthetic palette with transparent writing glass:")
                    .size(11.5)
                    .color(theme.muted_text_color()),
            );
            ui.add_space(4.0);

            egui::Grid::new("presets_card_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    for (i, &preset) in ThemePreset::all_presets().iter().enumerate() {
                        let is_active = theme.preset == preset && theme.custom_name.is_none();
                        let (win_rgb, page_rgb, txt_rgb, acc_rgb) = preset.preview_palette();

                        let border_color = if is_active {
                            theme.accent_color()
                        } else {
                            theme.text_color().gamma_multiply(0.20)
                        };
                        let bg_color = if is_active {
                            theme.accent_color().gamma_multiply(0.18)
                        } else {
                            theme.modal_fill_color().gamma_multiply(0.60)
                        };

                        egui::Frame::new()
                            .fill(bg_color)
                            .stroke(egui::Stroke::new(if is_active { 2.0 } else { 1.0 }, border_color))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.set_width(265.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(preset.icon()).size(18.0));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            let name_col = if is_active {
                                                theme.accent_color()
                                            } else {
                                                theme.text_color()
                                            };
                                            ui.label(RichText::new(preset.display_name()).size(12.0).strong().color(name_col));
                                            let badge = if preset.is_dark() { "Dark" } else { "Light" };
                                            ui.label(RichText::new(format!("({})", badge)).size(10.0).color(theme.muted_text_color()));
                                        });

                                        ui.horizontal(|ui| {
                                            // Palette swatches: Window, Page, Text, Accent
                                            for rgb in [win_rgb, page_rgb, txt_rgb, acc_rgb] {
                                                let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                                                ui.painter().rect_filled(rect, 3.0, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                                            }

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if is_active {
                                                    ui.label(RichText::new("✓ Active").size(11.0).strong().color(theme.accent_color()));
                                                } else if ui.small_button("Apply").clicked() {
                                                    theme.apply_preset(preset);
                                                    theme.custom_name = None;
                                                    *vibrancy_dirty = true;
                                                    state.theme_message = None;
                                                }
                                            });
                                        });
                                    });
                                });
                            });

                        if i % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
        });

        // Themes Folder & Sharing Section
        ui.group(|ui| {
            ui.label(RichText::new("📁 Themes Folder & Sharing:").strong());
            ui.label(
                RichText::new("Save current theme palette, import shared .json themes, or manage installed custom styles:")
                    .size(11.5)
                    .color(theme.muted_text_color()),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("📂 Open Themes Folder").on_hover_text("Open the themes directory in your system file explorer to share or add files").clicked() {
                    crate::theme::open_themes_folder();
                }

                if ui.button("📥 Import Theme File...").on_hover_text("Load and install a theme JSON file").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("Theme JSON (*.json)", &["json"]).pick_file() {
                        match crate::theme::load_theme_file(&path) {
                            Ok(loaded) => {
                                let name = loaded.custom_name.clone().unwrap_or_else(|| {
                                    path.file_stem().and_then(|s| s.to_str()).unwrap_or("Imported Theme").to_string()
                                });
                                let _ = crate::theme::save_custom_theme(&loaded, &name);
                                *theme = loaded;
                                *vibrancy_dirty = true;
                                state.theme_message = Some((format!("Successfully imported & applied theme: {}", name), false));
                            }
                            Err(e) => {
                                state.theme_message = Some((format!("Error importing theme: {}", e), true));
                            }
                        }
                    }
                }
            });

            ui.add_space(4.0);

            // Save current theme row
            ui.horizontal(|ui| {
                ui.label("Save current theme as:");
                ui.add(egui::TextEdit::singleline(&mut state.new_theme_name).hint_text("My Theme").desired_width(140.0));
                if ui.button("💾 Save Theme").clicked() {
                    if state.new_theme_name.trim().is_empty() {
                        state.theme_message = Some(("Please enter a theme name first.".to_string(), true));
                    } else {
                        match crate::theme::save_custom_theme(theme, &state.new_theme_name) {
                            Ok(path) => {
                                let saved_name = state.new_theme_name.trim().to_string();
                                theme.preset = ThemePreset::Custom;
                                theme.custom_name = Some(saved_name.clone());
                                state.theme_message = Some((format!("Theme '{}' saved to {}", saved_name, path.file_name().unwrap_or_default().to_string_lossy()), false));
                                state.new_theme_name.clear();
                            }
                            Err(e) => {
                                state.theme_message = Some((format!("Failed to save theme: {}", e), true));
                            }
                        }
                    }
                }
            });

            if let Some((msg, is_err)) = &state.theme_message {
                let col = if *is_err {
                    Color32::from_rgb(230, 80, 80)
                } else {
                    Color32::from_rgb(80, 200, 120)
                };
                ui.label(RichText::new(msg).color(col).size(11.5).strong());
            }

            ui.separator();

            ui.label(RichText::new("Installed Custom Themes:").size(12.0).strong());

            let custom_themes = crate::theme::list_custom_themes();
            if custom_themes.is_empty() {
                ui.label(
                    RichText::new("No custom theme files found in themes/ folder.")
                        .italics()
                        .color(theme.muted_text_color()),
                );
            } else {
                egui::Grid::new("custom_themes_list_grid")
                    .num_columns(3)
                    .spacing([12.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (name, path) in custom_themes {
                            let is_current = theme.custom_name.as_deref() == Some(&name);
                            let label_text = if is_current {
                                RichText::new(format!("★ {}", name)).strong().color(theme.accent_color())
                            } else {
                                RichText::new(&name).color(theme.text_color())
                            };
                            ui.label(label_text);

                            if is_current {
                                ui.label(RichText::new("✓ Active").size(11.0).strong().color(theme.accent_color()));
                            } else if ui.button("Apply").clicked() {
                                match crate::theme::load_theme_file(&path) {
                                    Ok(loaded) => {
                                        *theme = loaded;
                                        *vibrancy_dirty = true;
                                        state.theme_message = Some((format!("Applied theme: {}", name), false));
                                    }
                                    Err(e) => {
                                        state.theme_message = Some((format!("Failed to load {}: {}", name, e), true));
                                    }
                                }
                            }

                            if ui.button("🗑 Delete").clicked() {
                                let _ = std::fs::remove_file(&path);
                                state.theme_message = Some((format!("Deleted theme: {}", name), false));
                            }
                            ui.end_row();
                        }
                    });
            }
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

        // Undo & Redo History Limit
        ui.group(|ui| {
            ui.label(RichText::new("Undo & Redo History Limit:").strong());
            ui.label(
                RichText::new("Configure the maximum number of actions stored in undo/redo history (16 to 8,192). Default: 512.")
                    .size(11.5)
                    .color(theme.muted_text_color()),
            );
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                let mut val = *undo_limit as u32;
                let slider = egui::Slider::new(&mut val, 16..=8192)
                    .text("actions")
                    .logarithmic(true);
                if ui.add(slider).changed() {
                    *undo_limit = val as usize;
                }
                if ui.button("Reset (512)").clicked() {
                    *undo_limit = 512;
                }
            });
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

                    if action.is_in_text_action() {
                        let hint = action.in_text_hint().unwrap_or("In-text");
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("In-Text Action ({})", hint))
                                    .italics()
                                    .color(theme.accent_color()),
                            );
                        });
                    } else {
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
                                    for &key in KeyName::all() {
                                        ui.selectable_value(&mut sc.key, key, key.label());
                                    }
                                });
                        });

                        if changed {
                            keybinds.set_shortcut(action, sc);
                        }
                    }

                    ui.end_row();
                }
            });
    });
}

