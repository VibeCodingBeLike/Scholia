use crate::theme::ThemeConfig;
use chrono::{Datelike, Local, NaiveDate};
use egui::{RichText, Window};

#[derive(Debug, Clone)]
pub struct CalendarModalState {
    pub is_open: bool,
    pub current_year: i32,
    pub current_month: u32,
    pub popup_pos: Option<egui::Pos2>,
}

impl Default for CalendarModalState {
    fn default() -> Self {
        let now = Local::now();
        Self {
            is_open: false,
            current_year: now.year(),
            current_month: now.month(),
            popup_pos: None,
        }
    }
}

impl CalendarModalState {
    pub fn open_at(&mut self, pos: egui::Pos2, current_date_str: &str) {
        self.is_open = true;
        self.popup_pos = Some(pos);
        if let Some((_, m, y)) = parse_mla_date(current_date_str) {
            self.current_year = y;
            self.current_month = m;
        } else {
            let now = Local::now();
            self.current_year = now.year();
            self.current_month = now.month();
        }
    }

    pub fn prev_month(&mut self) {
        if self.current_month <= 1 {
            self.current_month = 12;
            self.current_year -= 1;
        } else {
            self.current_month -= 1;
        }
    }

    pub fn next_month(&mut self) {
        if self.current_month >= 12 {
            self.current_month = 1;
            self.current_year += 1;
        } else {
            self.current_month += 1;
        }
    }
}

pub fn render_calendar_popup(
    ctx: &egui::Context,
    state: &mut CalendarModalState,
    target_date: &mut String,
    is_dirty: &mut bool,
    theme: &ThemeConfig,
) {
    if !state.is_open {
        return;
    }

    let mut open = true;
    let mut close_modal = false;
    let mut chosen_date = None;
    let accent_col = theme.accent_color();
    let text_col = theme.text_color();
    let muted_col = theme.muted_text_color();

    let mut window = Window::new("📅 Select Date (MLA 9)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(260.0);

    if let Some(pos) = state.popup_pos {
        window = window.current_pos(pos);
    } else {
        window = window.anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);
    }

    window.show(ctx, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        // Month / Year Navigation Header
        ui.horizontal(|ui| {
            if ui.button("◀").on_hover_text("Previous Month").clicked() {
                state.prev_month();
            }

            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        month_number_to_name(state.current_month),
                        state.current_year
                    ))
                    .strong()
                    .size(13.5)
                    .color(text_col),
                );
            });

            if ui.button("▶").on_hover_text("Next Month").clicked() {
                state.next_month();
            }
        });

        ui.separator();

        // Day of Week Column Headers (Su Mo Tu We Th Fr Sa)
        let days_of_week = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for dow in days_of_week {
                ui.allocate_ui(egui::vec2(28.0, 18.0), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(dow).size(10.5).color(muted_col).strong());
                    });
                });
            }
        });

        // Compute starting day of week for current month (0 = Sun, 1 = Mon, ..., 6 = Sat)
        let start_weekday = NaiveDate::from_ymd_opt(state.current_year, state.current_month, 1)
            .map(|d| d.weekday().num_days_from_sunday())
            .unwrap_or(0);

        let total_days = days_in_month(state.current_year, state.current_month);

        let now = Local::now();
        let today_day = now.day();
        let today_month = now.month();
        let today_year = now.year();

        // Days Grid
        let mut day_counter = 1;
        let mut slot = 0;

        while day_counter <= total_days {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for _ in 0..7 {
                    if slot < start_weekday || day_counter > total_days {
                        ui.allocate_ui(egui::vec2(28.0, 26.0), |_ui| {});
                    } else {
                        let is_today = day_counter == today_day
                            && state.current_month == today_month
                            && state.current_year == today_year;

                        let text = RichText::new(format!("{}", day_counter))
                            .size(11.5)
                            .color(if is_today { accent_col } else { text_col });

                        let btn = egui::Button::new(if is_today { text.strong() } else { text })
                            .min_size(egui::vec2(28.0, 26.0));

                        let resp = ui.add(btn);
                        if resp.clicked() {
                            chosen_date = Some(format!(
                                "{} {} {}",
                                day_counter,
                                month_number_to_name(state.current_month),
                                state.current_year
                            ));
                        }

                        day_counter += 1;
                    }
                    slot += 1;
                }
            });
        }

        ui.separator();

        // Footer with Today and Cancel
        ui.horizontal(|ui| {
            if ui.button("📅 Today").clicked() {
                chosen_date = Some(crate::model::format_current_mla_date());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    close_modal = true;
                }
            });
        });
    });

    if let Some(date_str) = chosen_date {
        *target_date = date_str;
        *is_dirty = true;
        state.is_open = false;
    }

    if !open || close_modal {
        state.is_open = false;
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn parse_mla_date(s: &str) -> Option<(u32, u32, i32)> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 3 {
        let day = parts[0].parse::<u32>().ok()?;
        let month = month_name_to_number(parts[1])?;
        let year = parts[2].parse::<i32>().ok()?;
        Some((day, month, year))
    } else {
        None
    }
}

fn month_name_to_number(name: &str) -> Option<u32> {
    let name_lower = name.to_lowercase();
    match name_lower.as_str() {
        "january" | "jan" => Some(1),
        "february" | "feb" => Some(2),
        "march" | "mar" => Some(3),
        "april" | "apr" => Some(4),
        "may" => Some(5),
        "june" | "jun" => Some(6),
        "july" | "jul" => Some(7),
        "august" | "aug" => Some(8),
        "september" | "sep" | "sept" => Some(9),
        "october" | "oct" => Some(10),
        "november" | "nov" => Some(11),
        "december" | "dec" => Some(12),
        _ => None,
    }
}

pub fn month_number_to_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "January",
    }
}
