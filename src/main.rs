#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shortcut;
mod typing;
mod ui;

use eframe::egui::{self, Color32};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde::{Deserialize, Serialize};
use shortcut::ShortcutConfig;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    time::{Duration, Instant},
};
use typing::{TypingEvent, TypingSettings};

const STORAGE_KEY: &str = "typing_simulator_settings";
const BG: Color32 = Color32::from_rgb(12, 12, 14);
const SURFACE: Color32 = Color32::from_rgb(24, 24, 27);
const TEXT: Color32 = Color32::from_rgb(247, 246, 243);
const MUTED: Color32 = Color32::from_rgb(156, 155, 161);
const BORDER: Color32 = Color32::from_rgb(50, 50, 54);
const ACCENT: Color32 = Color32::from_rgb(255, 130, 71);

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Typing Simulator")
            .with_icon(load_icon())
            .with_inner_size([980.0, 780.0])
            .with_min_inner_size([520.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Typing Simulator",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("embedded app icon is a valid PNG")
        .thumbnail(256, 256)
        .to_rgba8();
    egui::IconData {
        rgba: image.into_raw(),
        width: 256,
        height: 256,
    }
}

fn set_style(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Inter.ttf")).into(),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".into());
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = BG;
    style.visuals.window_fill = SURFACE;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 38, 42);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 52, 57);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(69, 53, 45);
    style.visuals.selection.bg_fill = Color32::from_rgb(133, 73, 46);
    style.visuals.slider_trailing_fill = true;
    style.visuals.hyperlink_color = ACCENT;
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(13.0, 8.0);
    style.spacing.slider_rail_height = 6.0;
    ctx.set_style(style);
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    wpm: u32,
    delay_seconds: f32,
    typo_percent: f32,
    variation_percent: f32,
    thinking_chance_percent: f32,
    thinking_pause_seconds: f32,
    shortcut: ShortcutConfig,
    guide_seen: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wpm: 120,
            delay_seconds: 3.0,
            typo_percent: 10.0,
            variation_percent: 15.0,
            thinking_chance_percent: 8.0,
            thinking_pause_seconds: 2.5,
            shortcut: ShortcutConfig::default(),
            guide_seen: false,
        }
    }
}

impl Config {
    fn typing_settings(&self) -> TypingSettings {
        TypingSettings {
            wpm: self.wpm,
            delay_seconds: self.delay_seconds,
            typo_chance: self.typo_percent / 100.0,
            timing_variation: self.variation_percent / 100.0,
            thinking_chance: self.thinking_chance_percent / 100.0,
            thinking_pause_seconds: self.thinking_pause_seconds,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Platform {
    Windows,
    Mac,
}

struct App {
    text: String,
    config: Config,
    draft_shortcut: ShortcutConfig,
    active: Arc<AtomicBool>,
    status: String,
    status_error: bool,
    events: Option<Receiver<TypingEvent>>,
    hotkey_manager: Option<GlobalHotKeyManager>,
    registered_hotkey: Option<HotKey>,
    show_guide: bool,
    guide_step: usize,
    guide_platform: Platform,
    icon_texture: egui::TextureHandle,
    run_settings: Option<TypingSettings>,
    completed_chars: usize,
    countdown_left: Option<u32>,
    thinking_until: Option<Instant>,
    show_shortcut_capture: bool,
    capture_error: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        set_style(&cc.egui_ctx);
        let icon = load_icon();
        let icon_image = egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        );
        let icon_texture =
            cc.egui_ctx
                .load_texture("app-icon", icon_image, egui::TextureOptions::LINEAR);
        let mut config: Config = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, STORAGE_KEY))
            .unwrap_or_default();
        config.wpm = config.wpm.clamp(20, 250);
        config.delay_seconds = config.delay_seconds.clamp(0.0, 10.0);
        config.typo_percent = config.typo_percent.clamp(0.0, 25.0);
        config.variation_percent = config.variation_percent.clamp(0.0, 50.0);
        config.thinking_chance_percent = config.thinking_chance_percent.clamp(0.0, 50.0);
        config.thinking_pause_seconds = config.thinking_pause_seconds.clamp(0.5, 10.0);
        if config.shortcut.hotkey().is_err() {
            config.shortcut = ShortcutConfig::default();
        }

        let (hotkey_manager, registered_hotkey, status, status_error) =
            match GlobalHotKeyManager::new() {
                Ok(manager) => {
                    let hotkey = config.shortcut.hotkey().expect("validated shortcut");
                    match manager.register(hotkey) {
                        Ok(()) => (
                            Some(manager),
                            Some(hotkey),
                            format!("Ready. Shortcut: {}", config.shortcut.label()),
                            false,
                        ),
                        Err(error) => (
                            Some(manager),
                            None,
                            format!("Shortcut unavailable: {error}. The Start button still works."),
                            true,
                        ),
                    }
                }
                Err(error) => (
                    None,
                    None,
                    format!("Global shortcuts unavailable: {error}. The Start button still works."),
                    true,
                ),
            };

        Self {
            text: String::new(),
            draft_shortcut: config.shortcut,
            show_guide: !config.guide_seen,
            guide_step: 0,
            guide_platform: if cfg!(target_os = "macos") {
                Platform::Mac
            } else {
                Platform::Windows
            },
            config,
            active: Arc::new(AtomicBool::new(false)),
            status,
            status_error,
            events: None,
            hotkey_manager,
            registered_hotkey,
            icon_texture,
            run_settings: None,
            completed_chars: 0,
            countdown_left: None,
            thinking_until: None,
            show_shortcut_capture: false,
            capture_error: None,
        }
    }

    fn toggle(&mut self) {
        if self.active.load(Ordering::SeqCst) {
            self.active.store(false, Ordering::SeqCst);
            self.status = "Stopping...".into();
            return;
        }
        if self.show_guide {
            self.status = "Close the setup guide before starting.".into();
            self.status_error = true;
            return;
        }
        if self.text.trim().is_empty() {
            self.status = "Add some text before starting.".into();
            self.status_error = true;
            return;
        }
        self.active = Arc::new(AtomicBool::new(true));
        self.completed_chars = 0;
        self.run_settings = Some(self.config.typing_settings());
        self.countdown_left = Some(self.config.delay_seconds.ceil() as u32);
        self.thinking_until = None;
        let (sender, receiver) = mpsc::channel();
        self.events = Some(receiver);
        self.status = "Get ready to focus the target window...".into();
        self.status_error = false;
        typing::spawn(
            self.text.clone(),
            self.run_settings.expect("set above"),
            Arc::clone(&self.active),
            sender,
        );
    }

    fn apply_shortcut(&mut self) {
        let new_hotkey = match self.draft_shortcut.hotkey() {
            Ok(hotkey) => hotkey,
            Err(message) => {
                self.status = message.into();
                self.status_error = true;
                return;
            }
        };
        let Some(manager) = &self.hotkey_manager else {
            self.status = "System-wide shortcuts are unavailable on this device.".into();
            self.status_error = true;
            return;
        };
        if self.registered_hotkey == Some(new_hotkey) {
            self.config.shortcut = self.draft_shortcut;
            self.status = format!("Shortcut is already {}", self.draft_shortcut.label());
            self.status_error = false;
            return;
        }
        if let Err(error) = manager.register(new_hotkey) {
            self.status = format!(
                "Could not use {}: {error}. Your previous shortcut is still active.",
                self.draft_shortcut.label()
            );
            self.status_error = true;
            return;
        }
        if let Some(old_hotkey) = self.registered_hotkey {
            if let Err(error) = manager.unregister(old_hotkey) {
                let _ = manager.unregister(new_hotkey);
                self.status = format!("Could not replace the old shortcut: {error}");
                self.status_error = true;
                return;
            }
        }
        self.registered_hotkey = Some(new_hotkey);
        self.config.shortcut = self.draft_shortcut;
        self.status = format!("Shortcut changed to {}", self.config.shortcut.label());
        self.status_error = false;
    }

    fn poll(&mut self) {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if self.show_shortcut_capture {
                continue;
            }
            if self
                .registered_hotkey
                .is_some_and(|key| key.id() == event.id)
                && event.state == HotKeyState::Pressed
            {
                self.toggle();
            }
        }
        let pending: Vec<_> = self
            .events
            .as_ref()
            .map(|receiver| receiver.try_iter().collect())
            .unwrap_or_default();
        for event in pending {
            match event {
                TypingEvent::Countdown(seconds) => {
                    self.countdown_left = Some(seconds);
                    self.status = format!("Focus the target window. Starting in {seconds}...");
                }
                TypingEvent::Typing => {
                    self.countdown_left = None;
                    self.thinking_until = None;
                    self.status =
                        format!("Typing. Press {} to stop.", self.config.shortcut.label());
                }
                TypingEvent::Thinking => {
                    let seconds = self
                        .run_settings
                        .map_or(0.0, |settings| settings.thinking_pause_seconds);
                    self.thinking_until = Some(Instant::now() + Duration::from_secs_f32(seconds));
                    self.status = "Thinking pause after a word...".into();
                }
                TypingEvent::Progress(count) => self.completed_chars = count,
                TypingEvent::Finished => {
                    self.completed_chars = self.text.chars().count();
                    self.status = "Finished typing.".into();
                    self.thinking_until = None;
                }
                TypingEvent::Stopped => {
                    self.active.store(false, Ordering::SeqCst);
                    self.status = "Stopped.".into();
                    self.thinking_until = None;
                }
                TypingEvent::Error(message) => {
                    self.status = message;
                    self.status_error = true;
                    self.thinking_until = None;
                }
            }
        }
    }

    fn close_guide(&mut self) {
        self.show_guide = false;
        self.config.guide_seen = true;
        if self.status == "Close the setup guide before starting." {
            self.status = format!("Ready. Shortcut: {}", self.config.shortcut.label());
            self.status_error = false;
        }
    }

    fn estimated_seconds(&self) -> f32 {
        let active = self.active.load(Ordering::SeqCst);
        let settings = if active {
            self.run_settings
                .unwrap_or_else(|| self.config.typing_settings())
        } else {
            self.config.typing_settings()
        };
        let completed = if active { self.completed_chars } else { 0 };
        let typing = typing::estimate_seconds(&self.text, settings, completed);
        let countdown = if active {
            self.countdown_left.unwrap_or(0) as f32
        } else {
            settings.delay_seconds
        };
        let current_pause = self.thinking_until.map_or(0.0, |end| {
            end.saturating_duration_since(Instant::now()).as_secs_f32()
        });
        typing + countdown + current_pause
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        ctx.request_repaint_after(Duration::from_millis(50));
        self.header(ctx);
        self.main_content(ctx);
        self.guide_modal(ctx);
        self.shortcut_modal(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.config);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.active.store(false, Ordering::SeqCst);
        if let (Some(manager), Some(hotkey)) = (&self.hotkey_manager, self.registered_hotkey) {
            let _ = manager.unregister(hotkey);
        }
    }
}
