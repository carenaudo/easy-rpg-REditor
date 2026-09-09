use eframe::egui;
use crate::dialogs::asset_picker::AssetPickerState;
use crate::lcf_bridge::{AnimationInfo, BattlerAnimationInfo, BattlerAnimationPoseInfo, BattlerAnimationWeaponInfo, STANDARD_POSE_NAMES};
use crate::widgets::asset_viewer::{draw_checkerboard, AssetPreviewCache};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BattlerAnimTab {
    Poses,
    Weapons,
}

pub struct BattlerAnimationsView {
    pub selected_idx: usize,
    pub selected_pose_idx: usize,
    pub selected_weapon_idx: usize,
    pub active_tab: BattlerAnimTab,
    pub search_query: String,
    pub resize_dialog_open: bool,
    pub resize_target_count: usize,
}

impl Default for BattlerAnimationsView {
    fn default() -> Self {
        Self {
            selected_idx: 0,
            selected_pose_idx: 0,
            selected_weapon_idx: 0,
            active_tab: BattlerAnimTab::Poses,
            search_query: String::new(),
            resize_dialog_open: false,
            resize_target_count: 20,
        }
    }
}

impl BattlerAnimationsView {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        battler_animations: &mut Vec<BattlerAnimationInfo>,
        animations: &[AnimationInfo],
        project_path: Option<&str>,
        picker: &mut AssetPickerState,
        cache: &mut AssetPreviewCache,
        dirty: &mut bool,
    ) {
        if battler_animations.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.heading("No Battle Characters Configured");
                ui.label("RPG Maker 2003 uses Battle Characters to define hero side-view poses and weapon overlays.");
                ui.add_space(10.0);
                if ui.button("➕ Add Default Battle Character").clicked() {
                    battler_animations.push(BattlerAnimationInfo::default_template(1, "Hero".to_string()));
                    self.selected_idx = 0;
                    *dirty = true;
                }
            });
            return;
        }

        ui.columns(2, |cols| {
            // Master list (Left)
            cols[0].group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading("Battle Characters");
                    if ui.small_button("➕ Add").clicked() {
                        let new_id = (battler_animations.len() + 1) as i32;
                        battler_animations.push(BattlerAnimationInfo::default_template(new_id, format!("Character {:04}", new_id)));
                        self.selected_idx = battler_animations.len() - 1;
                        *dirty = true;
                    }
                    if ui.small_button("📄 Duplicate").clicked() && self.selected_idx < battler_animations.len() {
                        let mut copy = battler_animations[self.selected_idx].clone();
                        copy.id = (battler_animations.len() + 1) as i32;
                        copy.name = format!("{} (Copy)", copy.name);
                        battler_animations.push(copy);
                        self.selected_idx = battler_animations.len() - 1;
                        *dirty = true;
                    }
                    if ui.add_enabled(battler_animations.len() > 1 && self.selected_idx < battler_animations.len(), egui::Button::new("🗑 Del").small()).clicked() {
                        battler_animations.remove(self.selected_idx);
                        for (i, entry) in battler_animations.iter_mut().enumerate() {
                            entry.id = (i + 1) as i32;
                        }
                        if self.selected_idx >= battler_animations.len() {
                            self.selected_idx = battler_animations.len().saturating_sub(1);
                        }
                        *dirty = true;
                    }
                    if ui.small_button("📏 Resize").clicked() {
                        self.resize_target_count = battler_animations.len();
                        self.resize_dialog_open = true;
                    }
                });

                if self.resize_dialog_open {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Batch count:");
                        ui.add(egui::DragValue::new(&mut self.resize_target_count).range(1..=1000));
                        if ui.button("Apply").clicked() {
                            if self.resize_target_count > battler_animations.len() {
                                for i in battler_animations.len()..self.resize_target_count {
                                    let new_id = (i + 1) as i32;
                                    battler_animations.push(BattlerAnimationInfo::default_template(new_id, format!("Character {:04}", new_id)));
                                }
                            } else if self.resize_target_count < battler_animations.len() {
                                battler_animations.truncate(self.resize_target_count);
                                if self.selected_idx >= battler_animations.len() {
                                    self.selected_idx = battler_animations.len().saturating_sub(1);
                                }
                            }
                            *dirty = true;
                            self.resize_dialog_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.resize_dialog_open = false;
                        }
                    });
                }

                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Filter characters...").desired_width(120.0));
                    if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                        self.search_query.clear();
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("ba_master_scroll")
                    .max_height(550.0)
                    .show(ui, |ui| {
                        let q = self.search_query.trim().to_lowercase();
                        for (idx, ba) in battler_animations.iter().enumerate() {
                            if !q.is_empty() && !ba.name.to_lowercase().contains(&q) && !ba.id.to_string().contains(&q) {
                                continue;
                            }
                            let label = format!("{:04}: {}", ba.id, ba.name);
                            if ui.selectable_label(self.selected_idx == idx, label).clicked() {
                                self.selected_idx = idx;
                            }
                        }
                    });
            });

            // Detail view (Right)
            cols[1].group(|ui| {
                if let Some(ba) = battler_animations.get_mut(self.selected_idx) {
                    ui.heading(format!("Edit Battle Character #{:04}: {}", ba.id, ba.name));

                    egui::Grid::new("ba_general_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            if ui.text_edit_singleline(&mut ba.name).changed() { *dirty = true; }
                            ui.end_row();

                            ui.label("Animation Speed:");
                            ui.horizontal(|ui| {
                                if ui.selectable_label(ba.speed == 20, "Slow (20)").clicked() {
                                    ba.speed = 20;
                                    *dirty = true;
                                }
                                if ui.selectable_label(ba.speed == 14, "Medium (14)").clicked() {
                                    ba.speed = 14;
                                    *dirty = true;
                                }
                                if ui.selectable_label(ba.speed == 8, "Fast (8)").clicked() {
                                    ba.speed = 8;
                                    *dirty = true;
                                }
                                ui.label("Frames:");
                                if ui.add(egui::DragValue::new(&mut ba.speed).range(1..=50)).changed() {
                                    *dirty = true;
                                }
                            });
                            ui.end_row();
                        });

                    ui.separator();

                    // Tab selector: Poses vs Weapons
                    ui.horizontal(|ui| {
                        let poses_label = format!("🏃 Poses ({})", ba.poses.len());
                        let weapons_label = format!("⚔ Weapons ({})", ba.weapons.len());
                        ui.selectable_value(&mut self.active_tab, BattlerAnimTab::Poses, poses_label);
                        ui.selectable_value(&mut self.active_tab, BattlerAnimTab::Weapons, weapons_label);
                    });

                    ui.separator();

                    match self.active_tab {
                        BattlerAnimTab::Poses => {
                            self.show_poses_tab(ui, ba, animations, project_path, picker, cache, dirty);
                        }
                        BattlerAnimTab::Weapons => {
                            self.show_weapons_tab(ui, ba, project_path, picker, cache, dirty);
                        }
                    }
                } else {
                    ui.colored_label(egui::Color32::GRAY, "(Select a Battle Character from the list)");
                }
            });
        });
    }
}

pub fn apply_standard_2003_poses(ba: &mut BattlerAnimationInfo, base_sheet: &str, available_files: &[String]) {
    if base_sheet.is_empty() {
        return;
    }

    // Determine Sheet A and Sheet B
    // Standard RM2003 naming convention: "Name A" and "Name B"
    let (sheet_a, sheet_b) = if base_sheet.ends_with(" B") {
        let prefix = &base_sheet[..base_sheet.len() - 2];
        let candidate_a = format!("{} A", prefix);
        let a = if available_files.iter().any(|f| f == &candidate_a) { candidate_a } else { base_sheet.to_string() };
        (a, base_sheet.to_string())
    } else if base_sheet.ends_with(" A") {
        let prefix = &base_sheet[..base_sheet.len() - 2];
        let candidate_b = format!("{} B", prefix);
        let b = if available_files.iter().any(|f| f == &candidate_b) { candidate_b } else { base_sheet.to_string() };
        (base_sheet.to_string(), b)
    } else {
        let candidate_a = format!("{} A", base_sheet);
        let candidate_b = format!("{} B", base_sheet);
        let a = if available_files.iter().any(|f| f == &candidate_a) { candidate_a } else { base_sheet.to_string() };
        let b = if available_files.iter().any(|f| f == &candidate_b) { candidate_b } else { a.clone() };
        (a, b)
    };

    // RTP Sheet A row mapping:
    // Row 0: Skill / Magic (Cast)
    // Row 1: Defend / Guard
    // Row 2: Abnormal Status (Dazed)
    // Row 3: Dead / Defeated
    // Row 4: Damage / Low HP
    // Row 5: Right Hand Attack
    // Row 6: Left Hand Attack
    // Row 7: Idle / Ready Stance
    //
    // RTP Sheet B row mapping:
    // Row 0: Walk Left / Approach
    // Row 1: Walk Right / Retreat
    // Row 2: Victory / Cheer
    // Row 3: Item / Custom
    let standard_poses = [
        (1, "Idle", &sheet_a, 7),
        (2, "Right Hand Attack", &sheet_a, 5),
        (3, "Left Hand Attack", &sheet_a, 6),
        (4, "Skill", &sheet_a, 0),
        (5, "Dead", &sheet_a, 3),
        (6, "Damage", &sheet_a, 4),
        (7, "Abnormal Status", &sheet_a, 2),
        (8, "Defend", &sheet_a, 1),
        (9, "Walk Left", &sheet_b, 0),
        (10, "Walk Right", &sheet_b, 1),
        (11, "Victory", &sheet_b, 2),
        (12, "Item", &sheet_b, 3),
    ];

    ba.poses.clear();
    for (id, name, sheet, row) in standard_poses {
        ba.poses.push(BattlerAnimationPoseInfo {
            id,
            name: name.to_string(),
            battler_name: sheet.to_string(),
            battler_index: row,
            animation_type: 0,
            battle_animation_id: 1,
        });
    }
}

impl BattlerAnimationsView {
    fn show_poses_tab(
        &mut self,
        ui: &mut egui::Ui,
        ba: &mut BattlerAnimationInfo,
        animations: &[AnimationInfo],
        project_path: Option<&str>,
        picker: &mut AssetPickerState,
        cache: &mut AssetPreviewCache,
        dirty: &mut bool,
    ) {
        ui.horizontal(|ui| {
            if ui.small_button("➕ Add Pose").clicked() {
                let next_id = (ba.poses.len() + 1) as i32;
                let name = STANDARD_POSE_NAMES.get(ba.poses.len()).copied().unwrap_or("Custom Pose").to_string();
                ba.poses.push(BattlerAnimationPoseInfo {
                    id: next_id,
                    name,
                    battler_name: String::new(),
                    battler_index: 0,
                    animation_type: 0,
                    battle_animation_id: 1,
                });
                self.selected_pose_idx = ba.poses.len() - 1;
                *dirty = true;
            }
            if ui.button("⚡ Auto-Setup Standard 2003 Poses...").on_hover_text("Select a BattleCharSet to automatically generate and assign all 12 standard RM2003 poses (A: rows 0-7, B: rows 0-3)").clicked() {
                if let Some(proj) = project_path {
                    let cur = ba.poses.first().map(|p| p.battler_name.as_str()).unwrap_or("");
                    picker.open(proj, "BattleCharSet", cur, 0);
                    picker.auto_apply_standard_poses = true;
                }
            }
            if ui.small_button("↺ Reset 12 Poses").clicked() {
                let template = BattlerAnimationInfo::default_template(ba.id, ba.name.clone());
                ba.poses = template.poses;
                self.selected_pose_idx = 0;
                *dirty = true;
            }
        });

        ui.add_space(4.0);

        let mut auto_apply_sheet: Option<String> = None;
        ui.columns(2, |sub_cols| {
            // Pose list (Sub-left)
            sub_cols[0].group(|ui| {
                ui.label("Pose Actions:");
                egui::ScrollArea::vertical()
                    .id_salt("pose_list_scroll")
                    .max_height(360.0)
                    .show(ui, |ui| {
                        for (idx, pose) in ba.poses.iter().enumerate() {
                            let label = format!("{:02}: {}", pose.id, pose.name);
                            if ui.selectable_label(self.selected_pose_idx == idx, label).clicked() {
                                self.selected_pose_idx = idx;
                            }
                        }
                    });
            });

            // Pose editor (Sub-right)
            sub_cols[1].group(|ui| {
                if let Some(pose) = ba.poses.get_mut(self.selected_pose_idx) {
                    ui.label(egui::RichText::new(format!("Pose #{:02}: {}", pose.id, pose.name)).strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Action Name:");
                        if ui.text_edit_singleline(&mut pose.name).changed() { *dirty = true; }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Animation Type:");
                        if ui.radio_value(&mut pose.animation_type, 0, "Character Motion").changed() { *dirty = true; }
                        if ui.radio_value(&mut pose.animation_type, 1, "Battle Animation Overlay").changed() { *dirty = true; }
                    });

                    if pose.animation_type == 0 {
                        ui.group(|ui| {
                            ui.label("Character Graphic:");
                            ui.horizontal(|ui| {
                                let display_name = if pose.battler_name.is_empty() { "(None)" } else { &pose.battler_name };
                                ui.label(display_name);
                                if ui.button("📁 Choose...").clicked() {
                                    picker.open(project_path.unwrap_or(""), "BattleCharSet", &pose.battler_name, pose.battler_index);
                                }
                                if !pose.battler_name.is_empty() {
                                    if ui.small_button("⚡ Apply 12 Poses").on_hover_text("Populate all 12 standard poses using this character set (A: rows 0-7, B: rows 0-3)").clicked() {
                                        auto_apply_sheet = Some(pose.battler_name.clone());
                                    }
                                    if ui.small_button("✕").clicked() {
                                        pose.battler_name.clear();
                                        *dirty = true;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Sprite Index (0..7):");
                                if ui.add(egui::Slider::new(&mut pose.battler_index, 0..=7)).changed() {
                                    *dirty = true;
                                }
                            });

                            // Graphic preview thumbnail (animated single 48x48 pose)
                            if !pose.battler_name.is_empty() {
                                if let Some(proj) = project_path {
                                    let tex_opt = cache.get_or_load(ui.ctx(), proj, "BattleCharSet", &pose.battler_name)
                                        .or_else(|| cache.get_or_load(ui.ctx(), proj, "CharSet", &pose.battler_name));
                                    if let Some(tex) = tex_opt {
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
                                        let is_dark = ui.visuals().dark_mode;
                                        draw_checkerboard(&ui.painter(), rect, 6.0, is_dark);

                                        let anim_step = (ui.input(|i| i.time * 6.0).floor() as usize) % 4;
                                        let anim_col = match anim_step { 0 => 0, 1 => 1, 2 => 2, _ => 1 };
                                        let sel_row = pose.battler_index.clamp(0, 7) as usize;

                                        let u_min = anim_col as f32 / 3.0;
                                        let u_max = (anim_col + 1) as f32 / 3.0;
                                        let v_min = sel_row as f32 / 8.0;
                                        let v_max = (sel_row + 1) as f32 / 8.0;
                                        let uv = egui::Rect::from_min_max(egui::pos2(u_min, v_min), egui::pos2(u_max, v_max));

                                        ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
                                    }
                                }
                            }
                        });
                    } else {
                        ui.group(|ui| {
                            ui.label("Battle Animation Link:");
                            let current_anim_name = animations.iter()
                                .find(|a| a.id == pose.battle_animation_id)
                                .map(|a| format!("{:04}: {}", a.id, a.name))
                                .unwrap_or_else(|| format!("{:04}: (None)", pose.battle_animation_id));

                            egui::ComboBox::from_id_salt("pose_battle_anim_combo")
                                .selected_text(current_anim_name)
                                .show_ui(ui, |ui| {
                                    for a in animations {
                                        if ui.selectable_value(&mut pose.battle_animation_id, a.id, format!("{:04}: {}", a.id, a.name)).clicked() {
                                            *dirty = true;
                                        }
                                    }
                                });
                        });
                    }
                } else {
                    ui.colored_label(egui::Color32::GRAY, "(Select a pose from the left)");
                }
            });
        });

        if let Some(sheet) = auto_apply_sheet {
            apply_standard_2003_poses(ba, &sheet, &picker.available_files);
            *dirty = true;
        }
    }

    fn show_weapons_tab(
        &mut self,
        ui: &mut egui::Ui,
        ba: &mut BattlerAnimationInfo,
        project_path: Option<&str>,
        picker: &mut AssetPickerState,
        cache: &mut AssetPreviewCache,
        dirty: &mut bool,
    ) {
        ui.horizontal(|ui| {
            if ui.small_button("➕ Add Weapon").clicked() {
                let next_id = (ba.weapons.len() + 1) as i32;
                ba.weapons.push(BattlerAnimationWeaponInfo {
                    id: next_id,
                    name: format!("Weapon {:02}", next_id),
                    weapon_name: String::new(),
                    weapon_index: 0,
                });
                self.selected_weapon_idx = ba.weapons.len() - 1;
                *dirty = true;
            }
        });

        ui.add_space(4.0);

        if ba.weapons.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No weapon overlays configured for this character.");
            return;
        }

        ui.columns(2, |sub_cols| {
            // Weapon list
            sub_cols[0].group(|ui| {
                ui.label("Weapon Overlays:");
                egui::ScrollArea::vertical()
                    .id_salt("weapon_list_scroll")
                    .max_height(360.0)
                    .show(ui, |ui| {
                        let mut remove_idx = None;
                        for (idx, weapon) in ba.weapons.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let label = format!("{:02}: {}", weapon.id, weapon.name);
                                if ui.selectable_label(self.selected_weapon_idx == idx, label).clicked() {
                                    self.selected_weapon_idx = idx;
                                }
                                if ui.small_button("✕").clicked() {
                                    remove_idx = Some(idx);
                                }
                            });
                        }
                        if let Some(r_idx) = remove_idx {
                            ba.weapons.remove(r_idx);
                            for (i, w) in ba.weapons.iter_mut().enumerate() {
                                w.id = (i + 1) as i32;
                            }
                            if self.selected_weapon_idx >= ba.weapons.len() {
                                self.selected_weapon_idx = ba.weapons.len().saturating_sub(1);
                            }
                            *dirty = true;
                        }
                    });
            });

            // Weapon detail
            sub_cols[1].group(|ui| {
                if let Some(weapon) = ba.weapons.get_mut(self.selected_weapon_idx) {
                    ui.label(egui::RichText::new(format!("Weapon #{:02}: {}", weapon.id, weapon.name)).strong());
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(&mut weapon.name).changed() { *dirty = true; }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Graphic:");
                        let display_name = if weapon.weapon_name.is_empty() { "(None)" } else { &weapon.weapon_name };
                        ui.label(display_name);
                        if ui.button("📁 Choose...").clicked() {
                            picker.open(project_path.unwrap_or(""), "BattleWeapon", &weapon.weapon_name, weapon.weapon_index);
                        }
                        if !weapon.weapon_name.is_empty() && ui.small_button("✕").clicked() {
                            weapon.weapon_name.clear();
                            *dirty = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Weapon Index:");
                        if ui.add(egui::DragValue::new(&mut weapon.weapon_index).range(0..=100)).changed() {
                            *dirty = true;
                        }
                    });

                    // Graphic preview thumbnail (animated single 64x64 weapon motion)
                    if !weapon.weapon_name.is_empty() {
                        if let Some(proj) = project_path {
                            if let Some(tex) = cache.get_or_load(ui.ctx(), proj, "BattleWeapon", &weapon.weapon_name) {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
                                let is_dark = ui.visuals().dark_mode;
                                draw_checkerboard(&ui.painter(), rect, 6.0, is_dark);

                                let anim_step = (ui.input(|i| i.time * 6.0).floor() as usize) % 4;
                                let anim_col = match anim_step { 0 => 0, 1 => 1, 2 => 2, _ => 1 };
                                let sel_row = weapon.weapon_index.clamp(0, 7) as usize;

                                let u_min = anim_col as f32 / 3.0;
                                let u_max = (anim_col + 1) as f32 / 3.0;
                                let v_min = sel_row as f32 / 8.0;
                                let v_max = (sel_row + 1) as f32 / 8.0;
                                let uv = egui::Rect::from_min_max(egui::pos2(u_min, v_min), egui::pos2(u_max, v_max));

                                ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
                            }
                        }
                    }
                } else {
                    ui.colored_label(egui::Color32::GRAY, "(Select a weapon from the left)");
                }
            });
        });
    }
}
