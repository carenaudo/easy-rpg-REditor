use eframe::egui;
use crate::app_state::EditorAppState;
use crate::audio::SoundFontManager;

#[derive(Default)]
pub struct SoundFontDialog {
    pub is_open: bool,
    pub status_message: Option<Result<String, String>>,
}

impl SoundFontDialog {
    pub fn open(&mut self) {
        self.is_open = true;
        self.status_message = None;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        app: &mut EditorAppState,
        soundfont_manager: &SoundFontManager,
    ) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;

        egui::Window::new("🎵 MIDI SoundFont Setup")
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([580.0, 480.0])
            .show(ctx, |ui| {
                ui.label(
                    "MIDI (.mid) files do not contain recorded audio. To play MIDI music, \
                     a SoundFont (.sf2) sound bank is required for cross-platform synthesis.",
                );

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.heading("Current SoundFont Status");
                    ui.separator();

                    if let Some(loaded_path) = soundfont_manager.get_path() {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(0, 180, 0),
                                "✅ Active SoundFont:",
                            );
                            ui.strong(
                                loaded_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("SoundFont.sf2"),
                            );
                        });

                        if let Ok(metadata) = std::fs::metadata(&loaded_path) {
                            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                            ui.label(format!("File Size: {:.2} MB", size_mb));
                        }
                        ui.label(format!("Location: {}", loaded_path.display()));
                    } else {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(240, 140, 0),
                                "⚠️ No SoundFont is currently loaded.",
                            );
                        });
                        ui.label("MIDI playback is disabled until an .sf2 SoundFont is selected.");
                    }

                    if let Some(msg) = &self.status_message {
                        ui.separator();
                        match msg {
                            Ok(ok) => {
                                ui.colored_label(egui::Color32::from_rgb(0, 180, 0), ok);
                            }
                            Err(err) => {
                                ui.colored_label(egui::Color32::from_rgb(240, 80, 80), err);
                            }
                        }
                    }

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("📂 Browse for .sf2 File...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select SoundFont (.sf2)")
                                .add_filter("SoundFont 2 (*.sf2)", &["sf2", "SF2"])
                                .pick_file()
                            {
                                match soundfont_manager.load(&path) {
                                    Ok(_) => {
                                        let path_str = path.to_string_lossy().to_string();
                                        app.config.soundfont_path = Some(path_str.clone());
                                        app.config.save();
                                        self.status_message = Some(Ok(format!(
                                            "Loaded SoundFont: {}",
                                            path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                                        )));
                                    }
                                    Err(e) => {
                                        self.status_message = Some(Err(e));
                                    }
                                }
                            }
                        }

                        if ui.button("🔍 Auto-Detect").on_hover_text("Search RTP, project, and standard OS directories").clicked() {
                            let detected = SoundFontManager::detect_soundfont(
                                None,
                                app.config.rtp_path.as_deref(),
                                app.project_path.as_deref(),
                            );
                            if let Some(p) = detected {
                                match soundfont_manager.load(&p) {
                                    Ok(_) => {
                                        app.config.soundfont_path = Some(p.to_string_lossy().to_string());
                                        app.config.save();
                                        self.status_message = Some(Ok(format!("Auto-detected and loaded: {}", p.display())));
                                    }
                                    Err(e) => {
                                        self.status_message = Some(Err(e));
                                    }
                                }
                            } else {
                                self.status_message = Some(Err("No SoundFont (.sf2) found in RTP or system paths.".to_string()));
                            }
                        }

                        if soundfont_manager.is_loaded() && ui.button("❌ Clear").clicked() {
                            soundfont_manager.unload();
                            app.config.soundfont_path = None;
                            app.config.save();
                            self.status_message = Some(Ok("SoundFont unloaded.".to_string()));
                        }
                    });
                });

                ui.add_space(10.0);
                ui.heading("Recommended Legal / Free SoundFonts");
                ui.label("Download any of these high-quality General MIDI SoundFonts, then click 'Browse for .sf2 File...':");

                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("soundfont_links_scroll")
                    .max_height(220.0)
                    .show(ui, |ui| {
                        soundfont_card(
                            ui,
                            "⭐ GeneralUser GS (Recommended for RPG Maker)",
                            "~30 MB • Roland GS balance • Free for personal & commercial use",
                            "Authentic Roland SC-55 balance, matching what RPG Maker 2000/2003 games were originally composed for.",
                            "https://www.schristiancollins.com/generaluser.php",
                        );

                        soundfont_card(
                            ui,
                            "⚡ TimGM6mb (Lightweight & Fast)",
                            "~5.7 MB • GPL v2 (with font exception) • Minimal download",
                            "Lightweight and standard General MIDI bank used by MuseScore, TuxGuitar, and EasyRPG Player.",
                            "https://raw.githubusercontent.com/RoboCodeRobo/TimGM6mb/master/TimGM6mb.sf2",
                        );

                        soundfont_card(
                            ui,
                            "🎼 Fluid (R3) General MIDI",
                            "~140 MB • MIT / Expat License • Standard Linux SoundFont",
                            "Rich high-quality soundfont standard on Debian/Ubuntu/Fedora (package: fluid-soundfont-gm).",
                            "https://github.com/urish/cinto/blob/master/soundfonts/FluidR3_GM.sf2",
                        );

                        soundfont_card(
                            ui,
                            "🎹 FatBoy GM/GS",
                            "~320 MB • CC-BY 4.0 & GPL v3 • Studio Fidelity",
                            "High-fidelity soundbank sampled from physical studio hardware.",
                            "https://fatboy.site/",
                        );
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.is_open = false;
                    }
                });
            });

        if !is_open {
            self.is_open = false;
        }
    }
}

fn soundfont_card(ui: &mut egui::Ui, title: &str, meta: &str, description: &str, url: &str) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.strong(title);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("🌐 Download", url);
            });
        });
        ui.label(egui::RichText::new(meta).small().color(ui.visuals().weak_text_color()));
        ui.label(egui::RichText::new(description).small());
    });
}
