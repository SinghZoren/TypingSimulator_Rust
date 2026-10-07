use super::*;
use eframe::egui::{
    Align, CornerRadius, Frame, Id, Layout, Margin, Modal, RichText, Sense, Stroke,
};
use shortcut::KEYS;

const SUCCESS: Color32 = Color32::from_rgb(111, 220, 166);
const ERROR: Color32 = Color32::from_rgb(255, 116, 116);
const INSET: Color32 = Color32::from_rgb(17, 17, 20);

impl App {
    pub(super) fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header")
            .frame(
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(24, 13)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.image((self.icon_texture.id(), egui::vec2(34.0, 34.0)));
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("TYPING / SIMULATOR")
                                .size(13.0)
                                .strong()
                                .color(TEXT),
                        );
                        ui.label(
                            RichText::new("A calmer way to type")
                                .size(11.0)
                                .color(MUTED),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("How it works  ↗").clicked() {
                            self.guide_step = 0;
                            self.show_guide = true;
                        }
                    });
                });
            });
    }

    pub(super) fn main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(Frame::new().fill(BG)).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let width = ui.available_width();
                let page_width = (width - 40.0).clamp(300.0, 900.0);
                ui.horizontal(|ui| {
                    ui.add_space(((width - page_width) / 2.0).max(0.0));
                    ui.vertical(|ui| {
                        ui.set_width(page_width);
                        ui.add_space(38.0);
                        ui.label(RichText::new("YOUR TYPING STUDIO").size(11.0).strong().color(ACCENT));
                        ui.add_space(3.0);
                        ui.label(RichText::new("Make every keystroke feel human.").size(31.0).strong().color(TEXT));
                        ui.label(RichText::new("Paste your words, tune the rhythm, then focus any app to begin.").size(14.0).color(MUTED));
                        ui.add_space(28.0);
                        self.editor_card(ui);
                        ui.add_space(14.0);
                        self.behavior_card(ui);
                        ui.add_space(14.0);
                        self.shortcut_card(ui);
                        ui.add_space(24.0);
                        ui.label(RichText::new("Tip: Test a short sentence in Notepad or TextEdit before using a longer passage.").size(12.0).color(MUTED));
                        ui.add_space(28.0);
                    });
                });
            });
        });
    }

    fn editor_card(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Text to type")
                        .size(18.0)
                        .strong()
                        .color(TEXT),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} characters", self.text.chars().count()))
                            .size(12.0)
                            .color(MUTED),
                    );
                });
            });
            ui.label(
                RichText::new("Your text stays in this window and is cleared when you close it.")
                    .size(12.0)
                    .color(MUTED),
            );
            ui.add_space(12.0);
            let active = self.active.load(Ordering::SeqCst);
            ui.add_enabled(
                !active,
                egui::TextEdit::multiline(&mut self.text)
                    .hint_text("Paste or write your text here...")
                    .desired_rows(8)
                    .desired_width(ui.available_width())
                    .background_color(INSET),
            );
            ui.add_space(15.0);
            ui.horizontal_wrapped(|ui| {
                let time = if self.text.is_empty() {
                    "--".to_owned()
                } else {
                    format_duration(self.estimated_seconds())
                };
                let title = if active {
                    "Approx. remaining"
                } else {
                    "Estimated completion"
                };
                ui.label(RichText::new(title).size(12.0).color(MUTED));
                ui.label(RichText::new(time).size(16.0).strong().color(TEXT));
                ui.label(
                    RichText::new("with countdown + pauses")
                        .size(12.0)
                        .color(MUTED),
                );
            });
            if active {
                let total = self.text.replace("\r\n", "\n").chars().count().max(1);
                let progress = (self.completed_chars as f32 / total as f32).clamp(0.0, 1.0);
                ui.add(
                    egui::ProgressBar::new(progress)
                        .fill(ACCENT)
                        .desired_width(ui.available_width()),
                );
            }
            ui.add_space(16.0);
            ui.horizontal_wrapped(|ui| {
                let label = if active {
                    "Stop typing"
                } else {
                    "Start typing"
                };
                if ui
                    .add_sized(
                        [160.0, 42.0],
                        egui::Button::new(
                            RichText::new(format!("{label}  →")).strong().color(INSET),
                        )
                        .fill(ACCENT)
                        .corner_radius(8),
                    )
                    .clicked()
                {
                    self.toggle();
                }
                ui.label(
                    RichText::new(format!("or press {}", self.config.shortcut.label()))
                        .size(12.0)
                        .color(MUTED),
                );
            });
            let status_color = if self.status_error {
                ERROR
            } else if active {
                ACCENT
            } else {
                SUCCESS
            };
            ui.add_space(3.0);
            ui.label(RichText::new(&self.status).size(12.0).color(status_color));
        });
    }

    fn behavior_card(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.label(
                RichText::new("Shape the rhythm")
                    .size(18.0)
                    .strong()
                    .color(TEXT),
            );
            ui.label(
                RichText::new(
                    "Tune the pace and natural variation. Changes apply to the next run.",
                )
                .size(12.0)
                .color(MUTED),
            );
            ui.add_space(22.0);
            if ui.available_width() >= 630.0 {
                ui.columns(2, |columns| {
                    self.pace_controls(&mut columns[0]);
                    self.variation_controls(&mut columns[1]);
                });
            } else {
                self.pace_controls(ui);
                ui.separator();
                self.variation_controls(ui);
            }
        });
    }

    fn pace_controls(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("PACE").size(11.0).strong().color(ACCENT));
        ui.add_space(5.0);
        slider_row(
            ui,
            "Typing speed",
            "Words per minute; one word equals five characters.",
            &mut self.config.wpm,
            20.0,
            250.0,
            "WPM",
        );
        slider_row(
            ui,
            "Start countdown",
            "Time to focus the destination after you press Start.",
            &mut self.config.delay_seconds,
            0.0,
            10.0,
            "sec",
        );
        slider_row(
            ui,
            "Thinking chance",
            "Chance of a pause after each finished word, never mid-word.",
            &mut self.config.thinking_chance_percent,
            0.0,
            50.0,
            "%",
        );
        slider_row(
            ui,
            "Thinking pause",
            "How long each thinking break lasts; set chance to 0% to disable.",
            &mut self.config.thinking_pause_seconds,
            0.5,
            10.0,
            "sec",
        );
    }

    fn variation_controls(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("REALISM").size(11.0).strong().color(ACCENT));
        ui.add_space(5.0);
        slider_row(
            ui,
            "Corrected typos",
            "Chance per letter of typing a wrong key, then backspacing it.",
            &mut self.config.typo_percent,
            0.0,
            25.0,
            "%",
        );
        slider_row(
            ui,
            "Timing variation",
            "Varies the delay between keys around your chosen speed.",
            &mut self.config.variation_percent,
            0.0,
            50.0,
            "%",
        );
    }

    fn shortcut_card(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.label(
                RichText::new("Your shortcut")
                    .size(18.0)
                    .strong()
                    .color(TEXT),
            );
            ui.label(
                RichText::new("Start or stop without returning to this window.")
                    .size(12.0)
                    .color(MUTED),
            );
            ui.add_space(14.0);
            Frame::new()
                .fill(INSET)
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(9)
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new("ACTIVE SHORTCUT")
                                .size(10.0)
                                .strong()
                                .color(MUTED),
                        );
                        ui.label(
                            RichText::new(self.config.shortcut.label())
                                .size(17.0)
                                .strong()
                                .color(TEXT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Change shortcut  →").clicked() {
                                self.draft_shortcut = self.config.shortcut;
                                self.capture_error = None;
                                self.show_shortcut_capture = true;
                            }
                        });
                    });
                });
        });
    }

    pub(super) fn shortcut_modal(&mut self, ctx: &egui::Context) {
        if !self.show_shortcut_capture {
            return;
        }
        let keys: Vec<_> = ctx.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        Some((*key, *modifiers))
                    } else {
                        None
                    }
                })
                .collect()
        });
        for (key, modifiers) in keys {
            if key == egui::Key::Escape {
                self.show_shortcut_capture = false;
                return;
            }
            if let Some(index) = KEYS.iter().position(|(name, _)| *name == key.name()) {
                self.draft_shortcut = ShortcutConfig {
                    key_index: index,
                    control: modifiers.ctrl,
                    alt: modifiers.alt,
                    shift: modifiers.shift,
                    super_key: modifiers.mac_cmd,
                };
                self.capture_error = self.draft_shortcut.hotkey().err().map(str::to_owned);
            }
        }
        let response = Modal::new(Id::new("shortcut_capture"))
            .backdrop_color(Color32::from_black_alpha(190))
            .frame(
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(16)
                    .inner_margin(Margin::same(26)),
            )
            .show(ctx, |ui| {
                ui.set_width((ctx.available_rect().width() - 70.0).clamp(280.0, 420.0));
                ui.label(
                    RichText::new("KEYBOARD SHORTCUT")
                        .size(11.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.add_space(9.0);
                ui.label(
                    RichText::new("Press your new shortcut")
                        .size(24.0)
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    RichText::new(
                        "Use F1–F12, or Ctrl/Alt/Cmd plus a letter. Press Escape to cancel.",
                    )
                    .size(13.0)
                    .color(MUTED),
                );
                ui.add_space(20.0);
                Frame::new()
                    .fill(INSET)
                    .stroke(Stroke::new(1.0, ACCENT))
                    .corner_radius(9)
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(self.draft_shortcut.label())
                                    .size(22.0)
                                    .strong()
                                    .color(TEXT),
                            );
                        });
                    });
                if let Some(error) = &self.capture_error {
                    ui.label(RichText::new(error).size(12.0).color(ERROR));
                }
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_shortcut_capture = false;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                self.capture_error.is_none(),
                                egui::Button::new(RichText::new("Save shortcut").color(INSET))
                                    .fill(ACCENT),
                            )
                            .clicked()
                        {
                            self.apply_shortcut();
                            if self.status_error {
                                self.capture_error = Some(self.status.clone());
                            } else {
                                self.show_shortcut_capture = false;
                            }
                        }
                    });
                });
            });
        if response.should_close() {
            self.show_shortcut_capture = false;
        }
    }

    pub(super) fn guide_modal(&mut self, ctx: &egui::Context) {
        if !self.show_guide {
            return;
        }
        let response = Modal::new(Id::new("setup_guide"))
            .backdrop_color(Color32::from_black_alpha(125))
            .frame(
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(16))
                    .inner_margin(Margin::same(24)),
            )
            .show(ctx, |ui| {
                let width = (ctx.available_rect().width() - 96.0).clamp(320.0, 520.0);
                ui.set_width(width);
                ui.set_max_width(width);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("SETUP GUIDE")
                            .size(11.0)
                            .strong()
                            .color(ACCENT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            self.close_guide();
                        }
                    });
                });
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("Step {} of 4", self.guide_step + 1))
                        .size(12.0)
                        .color(MUTED),
                );
                ui.add(
                    egui::ProgressBar::new((self.guide_step + 1) as f32 / 4.0)
                        .fill(ACCENT)
                        .desired_width(ui.available_width()),
                );
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.guide_platform, Platform::Windows, "Windows");
                    ui.selectable_value(&mut self.guide_platform, Platform::Mac, "macOS");
                });
                ui.add_space(12.0);
                let max_height = (ctx.available_rect().height() - 290.0).max(110.0);
                egui::ScrollArea::vertical()
                    .max_height(max_height)
                    .show(ui, |ui| {
                        let (title, body) = self.guide_copy();
                        ui.label(RichText::new(title).size(23.0).strong().color(TEXT));
                        ui.add_space(9.0);
                        ui.label(RichText::new(body).size(14.0).color(MUTED));
                        if self.guide_step == 3 {
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(format!(
                                    "Current shortcut: {}",
                                    self.config.shortcut.label()
                                ))
                                .size(13.0)
                                .strong()
                                .color(ACCENT),
                            );
                        }
                    });
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.guide_step > 0, egui::Button::new("Back"))
                        .clicked()
                    {
                        self.guide_step -= 1;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.guide_step < 3 {
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("Next").color(INSET))
                                        .fill(ACCENT),
                                )
                                .clicked()
                            {
                                self.guide_step += 1;
                            }
                        } else if ui
                            .add(
                                egui::Button::new(RichText::new("Finish setup").color(INSET))
                                    .fill(ACCENT),
                            )
                            .clicked()
                        {
                            self.close_guide();
                        }
                    });
                });
            });
        if self.show_guide && response.should_close() {
            self.close_guide();
        }
    }

    fn guide_copy(&self) -> (&'static str, &'static str) {
        match (self.guide_step, self.guide_platform) {
            (0, _) => ("Welcome", "Paste the text you want to type in the editor. Pick a speed and adjust how often the app makes corrections or pauses to think."),
            (1, Platform::Mac) => ("Allow keyboard access", "Open System Settings > Privacy & Security > Accessibility. Enable Typing Simulator. If it is missing, add the app and restart it. On first launch of an unsigned download, right-click the app and choose Open. Some Mac keyboards need Fn with function-key shortcuts."),
            (1, Platform::Windows) => ("Windows permissions", "The app can type into normal windows immediately. To type into a program running as administrator, run Typing Simulator as administrator too. Windows may ask you to confirm an unsigned download on first launch."),
            (2, _) => ("Try a short sentence", "Open Notepad or TextEdit. Add a short sentence in Typing Simulator and press Start. During the countdown, click inside the text editor. The app types wherever the cursor is focused."),
            _ => ("Control it from anywhere", "Use the global shortcut to start or stop when another window is focused. You can change it in the Keyboard shortcut section below. A letter key needs Ctrl, Alt, or Win/Cmd."),
        }
    }
}

fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

trait SliderValue {
    fn as_f32(&self) -> f32;
    fn set_f32(&mut self, value: f32);
}

impl SliderValue for f32 {
    fn as_f32(&self) -> f32 {
        *self
    }
    fn set_f32(&mut self, value: f32) {
        *self = (value * 10.0).round() / 10.0;
    }
}

impl SliderValue for u32 {
    fn as_f32(&self) -> f32 {
        *self as f32
    }
    fn set_f32(&mut self, value: f32) {
        *self = value.round() as u32;
    }
}

fn slider_row(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    value: &mut impl SliderValue,
    min: f32,
    max: f32,
    suffix: &str,
) {
    let number = value.as_f32();
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).size(13.0).strong().color(TEXT));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let number_text = if number.fract().abs() < 0.05 {
                format!("{}", number as i32)
            } else {
                format!("{number:.1}")
            };
            ui.label(
                RichText::new(format!("{number_text} {suffix}"))
                    .size(12.0)
                    .strong()
                    .color(ACCENT),
            );
        });
    });
    ui.label(RichText::new(description).size(11.0).color(MUTED));
    ui.add_space(8.0);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(100.0), 24.0),
        Sense::click_and_drag(),
    );
    if let Some(pointer) = response
        .interact_pointer_pos()
        .filter(|_| response.clicked() || response.dragged())
    {
        let fraction = ((pointer.x - rect.left() - 7.0) / (rect.width() - 14.0)).clamp(0.0, 1.0);
        value.set_f32(min + fraction * (max - min));
        response.request_focus();
    }
    if response.has_focus() {
        let delta = ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowRight) {
                1.0
            } else if input.key_pressed(egui::Key::ArrowLeft) {
                -1.0
            } else {
                0.0
            }
        });
        if delta != 0.0 {
            value.set_f32((value.as_f32() + delta * (max - min) / 50.0).clamp(min, max));
        }
    }
    let painter = ui.painter();
    let track = egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width() - 14.0, 5.0));
    painter.rect_filled(track, 3.0, Color32::from_rgb(65, 64, 68));
    let t = ((value.as_f32() - min) / (max - min)).clamp(0.0, 1.0);
    let x = track.left() + track.width() * t;
    painter.rect_filled(
        egui::Rect::from_min_max(track.min, egui::pos2(x, track.max.y)),
        3.0,
        ACCENT,
    );
    painter.circle_filled(egui::pos2(x, rect.center().y), 8.0, TEXT);
    painter.circle_filled(egui::pos2(x, rect.center().y), 4.0, ACCENT);
    ui.add_space(14.0);
}

fn format_duration(seconds: f32) -> String {
    let seconds = seconds.max(0.0).ceil() as u64;
    match seconds {
        0..=59 => format!("{seconds}s"),
        60..=3599 => format!("{}m {}s", seconds / 60, seconds % 60),
        _ => format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60),
    }
}
