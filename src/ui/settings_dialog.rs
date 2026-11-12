use crate::keybinds::{Action, KeyName, KeybindConfig};
use crate::theme::{BlurMode, MlaFontChoice, ThemeConfig, ThemePreset};
use crate::writing_editor::{EditorSettings, EnglishLanguage};
use egui::{Color32, RichText, Window};

pub struct SettingsModalState {
    pub is_open: bool,
    pub selected_tab: SettingsTab,
    pub editing_action: Option<Action>,
    pub new_theme_name: String,
    pub theme_message: Option<(String, bool)>,
    pub new_dict_word: String,
    pub dict_search_query: String,
    pub dict_message: Option<(String, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    TransparencyAndTheme,
    EditorSettings,
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
            new_dict_word: String::new(),
            dict_search_query: String::new(),
            dict_message: None,
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
    let mut dummy_editor_settings = EditorSettings::default();
    render_settings_modal_with_editor(
        ctx,
        state,
        theme,
        keybinds,
        &mut dummy_editor_settings,
        undo_limit,
        vibrancy_dirty,
    );
}

pub fn render_settings_modal_with_editor(
    ctx: &egui::Context,
    state: &mut SettingsModalState,
    theme: &mut ThemeConfig,
    keybinds: &mut KeybindConfig,
    editor_settings: &mut EditorSettings,
    undo_limit: &mut usize,
    vibrancy_dirty: &mut bool,
) {
    if !state.is_open {
        return;
    }

    let mut open = true;
    let mut close_modal = false;

    let screen_h = ctx.input(|i| i.viewport().inner_rect.map(|r| r.height()).unwrap_or(700.0));
    let default_h = (screen_h * 0.80).round();

    Window::new("Preferences & Customization")
        .open(&mut open)
        .resizable(true)
        .frame(theme.modal_frame())
        .default_width(660.0)
        .default_height(default_h)
        .max_height(default_h)
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
                    SettingsTab::EditorSettings,
                    "📝 Editor Settings",
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
                SettingsTab::EditorSettings => {
                    render_editor_settings_tab(ui, state, editor_settings, theme);
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

        // Themes
        ui.group(|ui| {
            ui.label(RichText::new("Themes:").strong());
            ui.label(
                RichText::new("Choose from built-in frosted glass themes or import, save, and share custom .json styles:")
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
                ui.add(egui::TextEdit::singleline(&mut state.new_theme_name).hint_text("My Custom Theme").desired_width(140.0));
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

            // Unified list of Themes (Default presets first without delete button, then custom themes with delete button)
            egui::Grid::new("unified_themes_list_grid")
                .num_columns(4)
                .spacing([14.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Theme").strong());
                    ui.label(RichText::new("Palette").strong());
                    ui.label(RichText::new("Status").strong());
                    ui.label(RichText::new("Options").strong());
                    ui.end_row();

                    // 1. Default built-in theme presets
                    for &preset in ThemePreset::all_presets() {
                        let is_active = theme.preset == preset && theme.custom_name.is_none();
                        let (win_rgb, page_rgb, txt_rgb, acc_rgb) = preset.preview_palette();

                        // Theme Name + Icon
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(preset.icon()).size(14.0));
                            let name_col = if is_active {
                                theme.accent_color()
                            } else {
                                theme.text_color()
                            };
                            ui.label(RichText::new(preset.display_name()).color(name_col).strong());
                            let badge = if preset.is_dark() { "Dark" } else { "Light" };
                            ui.label(RichText::new(format!("({})", badge)).size(10.0).color(theme.muted_text_color()));
                        });

                        // Swatches
                        ui.horizontal(|ui| {
                            for rgb in [win_rgb, page_rgb, txt_rgb, acc_rgb] {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(13.0, 13.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 2.5, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                            }
                        });

                        // Status / Apply
                        if is_active {
                            ui.label(RichText::new("✓ Active").size(11.5).strong().color(theme.accent_color()));
                        } else if ui.button("Apply").clicked() {
                            theme.apply_preset(preset);
                            theme.custom_name = None;
                            *vibrancy_dirty = true;
                            state.theme_message = None;
                        }

                        // Built-in presets do not have a delete button
                        ui.label(RichText::new("Preset").size(10.5).color(theme.muted_text_color()));
                        ui.end_row();
                    }

                    // 2. Installed custom themes (from themes/ directory)
                    let custom_themes = crate::theme::list_custom_themes();
                    for (name, path) in custom_themes {
                        let is_current = theme.custom_name.as_deref() == Some(&name);

                        // Theme Name + Icon
                        ui.horizontal(|ui| {
                            ui.label("🎨");
                            let name_col = if is_current {
                                theme.accent_color()
                            } else {
                                theme.text_color()
                            };
                            ui.label(RichText::new(&name).color(name_col).strong());
                            ui.label(RichText::new("(Custom)").size(10.0).color(theme.muted_text_color()));
                        });

                        // Swatches preview
                        let preview_colors = crate::theme::load_theme_file(&path).ok().map(|t| {
                            (t.window_tint_rgb, t.page_tint_rgb, t.text_rgb, t.accent_rgb)
                        });
                        ui.horizontal(|ui| {
                            if let Some((w, p, t, a)) = preview_colors {
                                for rgb in [w, p, t, a] {
                                    let (rect, _) = ui.allocate_exact_size(egui::vec2(13.0, 13.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, 2.5, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                                }
                            } else {
                                ui.label("-");
                            }
                        });

                        // Status / Apply
                        if is_current {
                            ui.label(RichText::new("✓ Active").size(11.5).strong().color(theme.accent_color()));
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

                        // Custom themes have a delete button
                        if ui.button("🗑 Delete").clicked() {
                            let _ = std::fs::remove_file(&path);
                            if is_current {
                                theme.custom_name = None;
                            }
                            state.theme_message = Some((format!("Deleted theme: {}", name), false));
                        }
                        ui.end_row();
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

            let border_label = format!("{:.0}%", theme.page_border_alpha * 100.0);
            ui.horizontal(|ui| {
                ui.label("Page Border Opacity:");
                ui.add(
                    egui::Slider::new(&mut theme.page_border_alpha, 0.0..=1.0)
                        .text(border_label),
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
                    if color_picker_rgb(ui, &mut theme.window_tint_rgb) {
                        *vibrancy_dirty = true;
                    }
                    ui.end_row();

                    ui.label("Page Sheet Tint Color:");
                    color_picker_rgb(ui, &mut theme.page_tint_rgb);
                    ui.end_row();

                    ui.label("Text Color:");
                    color_picker_rgb(ui, &mut theme.text_rgb);
                    ui.end_row();

                    ui.label("Muted / Secondary Text Color:");
                    color_picker_rgb(ui, &mut theme.muted_text_rgb);
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

fn color_picker_rgb(ui: &mut egui::Ui, rgb: &mut [u8; 3]) -> bool {
    let mut color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    if ui.color_edit_button_srgba(&mut color).changed() {
        rgb[0] = color.r();
        rgb[1] = color.g();
        rgb[2] = color.b();
        true
    } else {
        false
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

    let ctrl_label = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };
    let alt_label = if cfg!(target_os = "macos") {
        "Option"
    } else {
        "Alt"
    };

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
                            changed |= ui.checkbox(&mut sc.ctrl, ctrl_label).changed();
                            changed |= ui.checkbox(&mut sc.shift, "Shift").changed();
                            changed |= ui.checkbox(&mut sc.alt, alt_label).changed();

                            egui::ComboBox::from_id_salt(format!("kb_{:?}", action))
                                .selected_text(sc.key.label())
                                .width(60.0)
                                .show_ui(ui, |ui| {
                                    for &key in KeyName::all() {
                                        changed |= ui
                                            .selectable_value(&mut sc.key, key, key.label())
                                            .changed();
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

fn render_editor_settings_tab(
    ui: &mut egui::Ui,
    state: &mut SettingsModalState,
    editor_settings: &mut EditorSettings,
    theme: &ThemeConfig,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 12.0;

        // 1. Language & Dialect Selection
        ui.group(|ui| {
            ui.label(RichText::new("🌐 Proofing Language & Regional Spelling:").strong());
            ui.label(
                RichText::new(
                    "Select the English regional dialect used for spell checking and grammar suggestions.",
                )
                .size(11.5)
                .color(theme.muted_text_color()),
            );

            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("editor_language_cb")
                .selected_text(editor_settings.language.display_name())
                .width(300.0)
                .show_ui(ui, |ui| {
                    for &lang in EnglishLanguage::all() {
                        ui.selectable_value(
                            &mut editor_settings.language,
                            lang,
                            lang.display_name(),
                        );
                    }
                });

            ui.add_space(4.0);
            ui.label(
                RichText::new(editor_settings.language.description())
                    .size(11.5)
                    .italics()
                    .color(theme.accent_color()),
            );
        });

        // 2. Spell Check, Grammar & Style Rules
        ui.group(|ui| {
            ui.label(RichText::new("🔍 Writing Editor Categories & Rules:").strong());
            ui.label(
                RichText::new(
                    "Choose which categories of suggestions are counted in the bottom bar and shown in the Editor.",
                )
                .size(11.5)
                .color(theme.muted_text_color()),
            );

            ui.add_space(4.0);

            ui.checkbox(
                &mut editor_settings.enable_spell_check,
                "Check Spelling (flag misspelled words and dialect mismatches)",
            );
            ui.checkbox(
                &mut editor_settings.enable_grammar_check,
                "Check Grammar & Punctuation (agreement, articles, confused words, punctuation)",
            );
            ui.checkbox(
                &mut editor_settings.enable_style_check,
                "Check Clarity, Conciseness & Readability (wordiness, redundancy, phrasing)",
            );
            ui.checkbox(
                &mut editor_settings.flag_contractions,
                "Flag Contractions in Academic Prose (suggest expanding don't → do not, can't → cannot, etc.)",
            );

            ui.separator();

            ui.checkbox(
                &mut editor_settings.ignore_all_caps,
                "Ignore words in ALL CAPS (e.g., MLA, NASA, UNESCO, ISBN)",
            );
            ui.checkbox(
                &mut editor_settings.ignore_words_with_numbers,
                "Ignore words containing numbers (e.g., COVID19, B12, MP3)",
            );
        });

        // 3. Custom Dictionary Manager
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📖 Custom Dictionary:").strong());
                ui.label(
                    RichText::new(format!(
                        "({} word{})",
                        editor_settings.custom_dictionary.len(),
                        if editor_settings.custom_dictionary.len() == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ))
                    .size(11.5)
                    .color(theme.muted_text_color()),
                );
            });

            ui.label(
                RichText::new(
                    "Words added to your custom dictionary are never flagged as spelling mistakes.",
                )
                .size(11.5)
                .color(theme.muted_text_color()),
            );

            ui.add_space(4.0);

            // Add new word row
            ui.horizontal(|ui| {
                ui.label("Add word:");
                let edit_resp = ui.add(
                    egui::TextEdit::singleline(&mut state.new_dict_word)
                        .hint_text("e.g., Bildungsroman")
                        .desired_width(180.0),
                );
                let enter_pressed =
                    edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("➕ Add to Dictionary").clicked() || enter_pressed {
                    let trimmed = state.new_dict_word.trim().to_string();
                    if trimmed.is_empty() {
                        state.dict_message =
                            Some(("Please enter a word to add.".to_string(), true));
                    } else if editor_settings.add_to_dictionary(&trimmed) {
                        state.dict_message = Some((
                            format!("Added \"{}\" to custom dictionary.", trimmed),
                            false,
                        ));
                        state.new_dict_word.clear();
                    } else {
                        state.dict_message = Some((
                            format!("\"{}\" is already in your dictionary.", trimmed),
                            true,
                        ));
                    }
                }
            });

            // Import / Export / Clear row
            ui.horizontal(|ui| {
                if ui
                    .button("📥 Import Word List (.txt)...")
                    .on_hover_text("Import newline- or whitespace-separated words from a text file")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Text Dictionary (*.txt)", &["txt"])
                        .pick_file()
                    {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                let mut added = 0usize;
                                for token in content.split_whitespace() {
                                    if editor_settings.add_to_dictionary(token) {
                                        added += 1;
                                    }
                                }
                                state.dict_message = Some((
                                    format!("Imported {} new word(s) into custom dictionary.", added),
                                    false,
                                ));
                            }
                            Err(e) => {
                                state.dict_message =
                                    Some((format!("Failed to read file: {}", e), true));
                            }
                        }
                    }
                }

                if ui
                    .button("📤 Export Dictionary (.txt)...")
                    .on_hover_text("Save your custom dictionary words to a text file")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("scholia_custom_dictionary.txt")
                        .add_filter("Text Dictionary (*.txt)", &["txt"])
                        .save_file()
                    {
                        let body: String = editor_settings
                            .custom_dictionary
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join("\n");
                        match std::fs::write(&path, body) {
                            Ok(_) => {
                                state.dict_message = Some((
                                    format!(
                                        "Exported {} word(s) to {}",
                                        editor_settings.custom_dictionary.len(),
                                        path.file_name().unwrap_or_default().to_string_lossy()
                                    ),
                                    false,
                                ));
                            }
                            Err(e) => {
                                state.dict_message =
                                    Some((format!("Failed to export dictionary: {}", e), true));
                            }
                        }
                    }
                }

                if !editor_settings.custom_dictionary.is_empty()
                    && ui
                        .button("🗑 Clear All")
                        .on_hover_text("Remove all words from your custom dictionary")
                        .clicked()
                {
                    editor_settings.custom_dictionary.clear();
                    state.dict_message =
                        Some(("Cleared all words from custom dictionary.".to_string(), false));
                }
            });

            if let Some((msg, is_err)) = &state.dict_message {
                let col = if *is_err {
                    Color32::from_rgb(230, 80, 80)
                } else {
                    Color32::from_rgb(80, 200, 120)
                };
                ui.label(RichText::new(msg).color(col).size(11.5).strong());
            }

            ui.separator();

            if editor_settings.custom_dictionary.is_empty() {
                ui.label(
                    RichText::new(
                        "Your custom dictionary is currently empty. Add words above or click '📖 + Add to Dictionary' in the Writing Editor.",
                    )
                    .size(11.5)
                    .italics()
                    .color(theme.muted_text_color()),
                );
            } else {
                ui.horizontal(|ui| {
                    ui.label("Filter dictionary:");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.dict_search_query)
                            .hint_text("Search words...")
                            .desired_width(180.0),
                    );
                    if !state.dict_search_query.is_empty() && ui.small_button("✕").clicked() {
                        state.dict_search_query.clear();
                    }
                });

                let query = state.dict_search_query.trim().to_lowercase();
                let words: Vec<String> = editor_settings
                    .custom_dictionary
                    .iter()
                    .filter(|w| query.is_empty() || w.contains(&query))
                    .cloned()
                    .collect();

                let mut word_to_remove: Option<String> = None;

                egui::ScrollArea::vertical()
                    .id_salt("custom_dict_scroll")
                    .max_height(180.0)
                    .show(ui, |ui| {
                        egui::Grid::new("custom_dict_grid")
                            .num_columns(2)
                            .spacing([24.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for w in &words {
                                    ui.label(RichText::new(w).strong().color(theme.text_color()));
                                    if ui.small_button("🗑 Remove").clicked() {
                                        word_to_remove = Some(w.clone());
                                    }
                                    ui.end_row();
                                }
                            });
                    });

                if let Some(rem) = word_to_remove {
                    editor_settings.remove_from_dictionary(&rem);
                    state.dict_message = Some((
                        format!("Removed \"{}\" from custom dictionary.", rem),
                        false,
                    ));
                }
            }

            let ignored_count =
                editor_settings.ignored_issues.len() + editor_settings.ignored_words_session.len();
            if ignored_count > 0 {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Ignored in current session: {} item(s)",
                            ignored_count
                        ))
                        .size(11.5)
                        .color(theme.muted_text_color()),
                    );
                    if ui.button("↺ Reset Ignored Suggestions").clicked() {
                        editor_settings.clear_ignored();
                    }
                });
            }
        });
    });
}
