use crate::mla_rules::{MlaLinter, RuleSeverity};
use crate::model::MlaDocument;
use crate::theme::ThemeConfig;
use egui::{Color32, RichText, Window};

#[derive(Default)]
pub struct ComplianceModalState {
    pub is_open: bool,
}

pub fn render_compliance_modal(
    ctx: &egui::Context,
    state: &mut ComplianceModalState,
    doc: &mut MlaDocument,
    theme: &ThemeConfig,
) {
    if !state.is_open {
        return;
    }

    let mut open = true;
    let mut close_modal = false;
    let report = MlaLinter::inspect(doc);

    Window::new("MLA 9th Edition Compliance Inspector")
        .open(&mut open)
        .resizable(true)
        .default_width(520.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            // Header Score Banner
            let (score_col, score_text) = if report.score_percentage >= 95 {
                (Color32::from_rgb(40, 190, 90), "✨ 100% MLA Compliant")
            } else if report.score_percentage >= 75 {
                (Color32::from_rgb(230, 160, 30), "⚠️ Minor MLA Inconsistencies")
            } else {
                (Color32::from_rgb(235, 70, 70), "❌ Action Required (MLA Non-Compliant)")
            };

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Score: {}%", report.score_percentage)).size(22.0).strong().color(score_col));
                    ui.add_space(10.0);
                    ui.label(RichText::new(score_text).size(14.0).color(theme.text_color()));
                });
                ui.label(
                    RichText::new(format!(
                        "Passed {} of {} MLA structure checks.",
                        report.passed_rules_count, report.total_rules_checked
                    ))
                    .size(11.5)
                    .color(theme.muted_text_color()),
                );
            });

            ui.separator();

            if report.issues.is_empty() {
                ui.label(RichText::new("🎉 Perfect! Your document strictly satisfies every MLA 9 formatting standard.").color(Color32::from_rgb(50, 190, 100)).size(13.0));
            } else {
                ui.label(RichText::new("Identified Formatting Issues:").strong().color(theme.text_color()));

                egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                    for issue in &report.issues {
                        ui.group(|ui| {
                            let (badge_col, badge_txt) = match issue.severity {
                                RuleSeverity::Error => (Color32::from_rgb(235, 60, 60), "VIOLATION"),
                                RuleSeverity::Warning => (Color32::from_rgb(230, 150, 20), "WARNING"),
                                RuleSeverity::Suggestion => (Color32::from_rgb(70, 150, 230), "RECOMMENDED"),
                            };

                            ui.horizontal(|ui| {
                                ui.colored_label(badge_col, format!("[{}]", badge_txt));
                                ui.label(RichText::new(issue.title).strong().color(theme.text_color()));

                                if issue.can_auto_fix {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(RichText::new("⚡ Auto-Fix").color(Color32::from_rgb(50, 180, 100))).clicked() {
                                            MlaLinter::auto_fix(doc, issue.rule_id);
                                        }
                                    });
                                }
                            });

                            ui.label(RichText::new(&issue.description).size(12.0).color(theme.muted_text_color()));
                        });
                        ui.add_space(4.0);
                    }
                });
            }

            ui.separator();

            ui.horizontal(|ui| {
                if report.issues.iter().any(|i| i.can_auto_fix)
                    && ui.button(RichText::new("⚡ Auto-Fix All Compatible Issues").strong()).clicked()
                {
                    let fixable_ids: Vec<String> = report.issues.iter().filter(|i| i.can_auto_fix).map(|i| i.rule_id.to_string()).collect();
                    for id in fixable_ids {
                        MlaLinter::auto_fix(doc, &id);
                    }
                }

                if ui.button("Close").clicked() {
                    close_modal = true;
                }
            });
        });

    if !open || close_modal {
        state.is_open = false;
    }
}
