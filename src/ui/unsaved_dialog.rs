use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use egui::{Color32, RichText, Window};

#[derive(Default, Debug, Clone)]
pub struct UnsavedDialogState {
    pub is_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedDialogResponse {
    None,
    Save,
    Discard,
    Cancel,
}

pub fn render_unsaved_dialog(
    ctx: &egui::Context,
    state: &mut UnsavedDialogState,
    doc: &MlaDocument,
    theme: &ThemeConfig,
) -> UnsavedDialogResponse {
    if !state.is_open {
        return UnsavedDialogResponse::None;
    }

    let mut open = true;
    let mut response = UnsavedDialogResponse::None;

    // Check keyboard shortcuts while modal is open
    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) {
            response = UnsavedDialogResponse::Cancel;
        } else if i.key_pressed(egui::Key::Enter) {
            response = UnsavedDialogResponse::Save;
        } else if i.key_pressed(egui::Key::D) && !i.modifiers.command && !i.modifiers.ctrl {
            response = UnsavedDialogResponse::Discard;
        }
    });

    Window::new("Unsaved Changes")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .frame(theme.modal_frame())
        .default_width(440.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 12.0;

            // Warning icon & heading
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚠️").size(24.0));
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Save changes to document before closing?")
                            .strong()
                            .size(15.0)
                            .color(theme.text_color()),
                    );
                    ui.label(
                        RichText::new("If you close without saving, your recent modifications will be permanently lost.")
                            .size(12.0)
                            .color(theme.muted_text_color()),
                    );
                });
            });

            // Document info card
            let doc_name = if let Some(path) = &doc.file_path {
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path.as_str())
            } else if !doc.title.trim().is_empty() {
                doc.title.as_str()
            } else {
                "Untitled MLA Document"
            };

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📄").size(14.0));
                    ui.label(
                        RichText::new(doc_name)
                            .strong()
                            .color(theme.accent_color()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Unsaved")
                                .size(11.0)
                                .color(Color32::from_rgb(235, 120, 40))
                                .strong(),
                        );
                    });
                });
            });

            ui.separator();

            // Action Buttons
            ui.horizontal(|ui| {
                // Save button (default action)
                let save_btn = ui.add(
                    egui::Button::new(
                        RichText::new("💾 Save (Enter)")
                            .strong()
                            .color(theme.accent_color()),
                    )
                    .min_size(egui::vec2(110.0, 28.0)),
                );
                if save_btn.clicked() {
                    response = UnsavedDialogResponse::Save;
                }

                // Don't Save / Discard
                let discard_btn = ui.add(
                    egui::Button::new(
                        RichText::new("Don't Save (D)")
                            .color(Color32::from_rgb(235, 80, 80)),
                    )
                    .min_size(egui::vec2(110.0, 28.0)),
                );
                if discard_btn.clicked() {
                    response = UnsavedDialogResponse::Discard;
                }

                // Cancel button (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let cancel_btn = ui.add(
                        egui::Button::new(RichText::new("Cancel (Esc)").color(theme.text_color()))
                            .min_size(egui::vec2(80.0, 28.0)),
                    );
                    if cancel_btn.clicked() {
                        response = UnsavedDialogResponse::Cancel;
                    }
                });
            });
        });

    if !open {
        response = UnsavedDialogResponse::Cancel;
    }

    if response != UnsavedDialogResponse::None {
        state.is_open = false;
    }

    response
}
