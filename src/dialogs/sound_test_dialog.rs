use eframe::egui;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use crate::app_state::AppPersistentData;
use crate::audio::AudioPlayer;

pub struct SoundTestDialog {
    pub is_open: bool,
    pub active_tab: usize, // 0: Music, 1: Sound
    pub search_query: String,
    pub selected_track: Option<String>,
    pub volume: u32,
    pub pitch: u32,
    pub pan: i32,
    pub is_playing: bool,
    pub tracks: Vec<String>,
    pub status_message: Option<Result<String, String>>,
}

impl Default for SoundTestDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: 0,
            search_query: String::new(),
            selected_track: None,
            volume: 100,
            pitch: 100,
            pan: 0,
            is_playing: false,
            tracks: Vec::new(),
            status_message: None,
        }
    }
}

impl SoundTestDialog {
    pub fn open(&mut self, project_path: Option<&str>) {
        self.is_open = true;
        self.is_playing = false;
        self.status_message = None;
        self.scan_tracks(project_path);
    }

    /// Finds the real file (with its real extension) for a track stem shown
    /// in the list - `scan_tracks` only records stems in a set, discarding
    /// which directory each came from, so playback needs its own lookup.
    /// Project assets take priority over RTP ones of the same name, matching
    /// how RPG Maker itself resolves overrides.
    fn resolve_track_path(&self, project_path: Option<&str>, stem: &str) -> Option<PathBuf> {
        let cat = self.category_name();
        let search_dirs: Vec<PathBuf> = [
            project_path.map(|p| Path::new(p).join(cat)),
            AppPersistentData::load().get_effective_rtp_path().map(|p| p.join(cat)),
        ]
        .into_iter()
        .flatten()
        .collect();

        for dir in search_dirs {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() && p.file_stem().and_then(|s| s.to_str()) == Some(stem) {
                        return Some(p);
                    }
                }
            }
        }
        None
    }

    fn category_name(&self) -> &'static str {
        if self.active_tab == 0 { "Music" } else { "Sound" }
    }

    pub fn scan_tracks(&mut self, project_path: Option<&str>) {
        let mut set = BTreeSet::new();
        let cat = self.category_name();

        // 1. Scan project directory
        if let Some(proj) = project_path {
            let cat_dir = Path::new(proj).join(cat);
            if let Ok(entries) = fs::read_dir(cat_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            set.insert(stem.to_string());
                        }
                    }
                }
            }
        }

        // 2. Scan RTP directory
        let config = AppPersistentData::load();
        if let Some(rtp_dir) = config.get_effective_rtp_path() {
            let cat_dir = rtp_dir.join(cat);
            if let Ok(entries) = fs::read_dir(cat_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            set.insert(stem.to_string());
                        }
                    }
                }
            }
        }

        self.tracks = set.into_iter().collect();
    }

    pub fn show(&mut self, ctx: &egui::Context, project_path: Option<&str>, audio: Option<&AudioPlayer>) {
        if !self.is_open {
            return;
        }

        // MIDI playback runs on a background worker and can't report
        // failure synchronously from the Play button click - pick up
        // whatever it reported since we last checked.
        if let Some(a) = audio {
            if let Some(err) = a.take_midi_error() {
                self.is_playing = false;
                self.status_message = Some(Err(err));
            }
        }

        let mut is_open = self.is_open;

        egui::Window::new("🎵 Sound Test / Jukebox")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([580.0, 420.0])
            .show(ctx, |ui| {
                // Category Tabs
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.active_tab == 0, "🎵 Music (BGM)").clicked() {
                        self.active_tab = 0;
                        self.selected_track = None;
                        self.is_playing = false;
                        self.status_message = None;
                        if let Some(a) = audio { a.stop(); }
                        self.scan_tracks(project_path);
                    }
                    if ui.selectable_label(self.active_tab == 1, "🔊 Sound Effects (SE)").clicked() {
                        self.active_tab = 1;
                        self.selected_track = None;
                        self.is_playing = false;
                        self.status_message = None;
                        if let Some(a) = audio { a.stop(); }
                        self.scan_tracks(project_path);
                    }
                });

                ui.separator();

                ui.columns(2, |cols| {
                    // Left Column: Track List
                    cols[0].group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Filter tracks...").desired_width(140.0));
                            if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                                self.search_query.clear();
                            }
                        });

                        ui.separator();
                        let q = self.search_query.to_lowercase();
                        let filtered: Vec<&String> = self.tracks.iter().filter(|t| q.is_empty() || t.to_lowercase().contains(&q)).collect();
                        ui.label(format!("Tracks ({}):", filtered.len()));

                        egui::ScrollArea::vertical()
                            .id_salt("sound_test_tracks_scroll")
                            .max_height(260.0)
                            .show(ui, |ui| {
                                for track in filtered {
                                    let is_sel = self.selected_track.as_deref() == Some(track.as_str());
                                    if ui.selectable_label(is_sel, track).clicked() {
                                        self.selected_track = Some(track.clone());
                                        self.is_playing = false;
                                        self.status_message = None;
                                        if let Some(a) = audio { a.stop(); }
                                    }
                                }
                            });
                    });

                    // Right Column: Controls & Inspector
                    cols[1].group(|ui| {
                        ui.heading("Track Controls");

                        if let Some(track) = self.selected_track.clone() {
                            let resolved_path = self.resolve_track_path(project_path, &track);
                            let is_midi_track = resolved_path.as_deref().map(AudioPlayer::is_midi).unwrap_or(false);

                            ui.label(format!("Selected: {}", track));
                            ui.label(format!("Type: {}{}", self.category_name(), if is_midi_track { " (MIDI)" } else { "" }));
                            ui.separator();

                            egui::Grid::new("sound_test_sliders")
                                .num_columns(2)
                                .spacing([8.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Volume:");
                                    if ui.add(egui::Slider::new(&mut self.volume, 0..=100).suffix("%")).changed() {
                                        if let Some(a) = audio { a.set_volume(self.volume as f32 / 100.0); }
                                    }
                                    ui.end_row();

                                    ui.label("Pitch / Tempo:");
                                    ui.add_enabled_ui(!is_midi_track, |ui| {
                                        let resp = ui.add(egui::Slider::new(&mut self.pitch, 50..=150).suffix("%"));
                                        if resp.changed() {
                                            if let Some(a) = audio { a.set_speed(self.pitch as f32 / 100.0); }
                                        }
                                        if is_midi_track {
                                            resp.on_disabled_hover_text("Tempo control isn't available for MIDI playback.");
                                        }
                                    });
                                    ui.end_row();

                                    ui.label("Pan (Balance):");
                                    ui.add(egui::Slider::new(&mut self.pan, -100..=100))
                                        .on_hover_text("Pan preview isn't wired up to playback yet.");
                                    ui.end_row();
                                });

                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button(if self.is_playing { "🔄 Replay" } else { "▶ Play" }).clicked() {
                                    self.status_message = None;
                                    match &resolved_path {
                                        Some(path) => match audio {
                                            Some(a) => match a.play_file(path) {
                                                Ok(()) => {
                                                    self.is_playing = true;
                                                    a.set_volume(self.volume as f32 / 100.0);
                                                    if !is_midi_track {
                                                        a.set_speed(self.pitch as f32 / 100.0);
                                                    }
                                                }
                                                Err(e) => {
                                                    self.is_playing = false;
                                                    self.status_message = Some(Err(e));
                                                }
                                            },
                                            None => {
                                                self.is_playing = false;
                                                self.status_message = Some(Err("No audio output device available.".to_string()));
                                            }
                                        },
                                        None => {
                                            self.is_playing = false;
                                            self.status_message = Some(Err("Couldn't find that track's file.".to_string()));
                                        }
                                    }
                                }
                                if ui.button("⏹ Stop").clicked() {
                                    self.is_playing = false;
                                    if let Some(a) = audio { a.stop(); }
                                }
                            });

                            let is_dark = ui.visuals().dark_mode;
                            match &self.status_message {
                                Some(Ok(msg)) => { ui.colored_label(crate::theme::colors::success(is_dark), msg); }
                                Some(Err(msg)) => { ui.colored_label(crate::theme::colors::danger(is_dark), msg); }
                                None if self.is_playing => {
                                    ui.colored_label(crate::theme::colors::success(is_dark), "▶ Playing track...");
                                }
                                None => {
                                    ui.colored_label(crate::theme::colors::muted(is_dark), "⏹ Stopped");
                                }
                            }
                        } else {
                            let is_dark = ui.visuals().dark_mode;
                            ui.colored_label(crate::theme::colors::muted(is_dark), "Select a track on the left to preview.");
                        }
                    });
                });

                ui.separator();
                if ui.button("Close").clicked() {
                    self.is_playing = false;
                    if let Some(a) = audio { a.stop(); }
                    self.is_open = false;
                }
            });

        if !is_open {
            self.is_playing = false;
            if let Some(a) = audio { a.stop(); }
            self.is_open = false;
        }
    }
}
