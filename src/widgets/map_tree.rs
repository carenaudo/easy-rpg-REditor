use eframe::egui;
use crate::lcf_bridge::MapTreeItem;

pub enum MapTreeAction {
    Select(usize),
    OpenProperties(i32),
    NewMap { parent_id: i32 },
    Duplicate(i32),
    Delete(i32),
}

pub struct MapTreeWidget {
    pub search_query: String,
}

impl Default for MapTreeWidget {
    fn default() -> Self {
        Self {
            search_query: String::new(),
        }
    }
}

impl MapTreeWidget {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        maps: &[MapTreeItem],
        selected_map_idx: Option<usize>,
    ) -> Option<MapTreeAction> {
        let mut action = None;

        // Pane head: compact uppercase title, muted count, right-aligned actions.
        ui.horizontal(|ui| {
            ui.strong(rust_i18n::t!("pane.maps").to_uppercase());
            let is_dark = ui.visuals().dark_mode;
            ui.colored_label(crate::theme::colors::muted(is_dark), format!("({})", maps.len()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("＋").on_hover_text("New Map").clicked() {
                    action = Some(MapTreeAction::NewMap { parent_id: 0 });
                }
            });
        });

        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);
            if !self.search_query.is_empty() && ui.small_button("x").clicked() {
                self.search_query.clear();
            }
        });

        ui.separator();

        let q = self.search_query.trim().to_lowercase();

        egui::ScrollArea::vertical()
            .id_salt("map_tree_scroll")
            .show(ui, |ui| {
                for (idx, map) in maps.iter().enumerate() {
                    if !q.is_empty() && !map.name.to_lowercase().contains(&q) && !map.id.to_string().contains(&q) {
                        continue;
                    }

                    let indent_spaces = "  ".repeat(map.indentation.max(0) as usize);
                    let label_text = format!("{}{:04}: {}", indent_spaces, map.id, map.name);
                    let is_selected = selected_map_idx == Some(idx);

                    let resp = ui.selectable_label(is_selected, label_text);
                    if resp.clicked() {
                        action = Some(MapTreeAction::Select(idx));
                    }

                    // Context Menu
                    resp.context_menu(|ui| {
                        if ui.button(format!("⚙ {}", rust_i18n::t!("map.properties"))).clicked() {
                            action = Some(MapTreeAction::OpenProperties(map.id));
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(format!("+ {}", rust_i18n::t!("map.new_submap"))).clicked() {
                            action = Some(MapTreeAction::NewMap { parent_id: map.id });
                            ui.close();
                        }
                        if ui.button(format!("📄 {}", rust_i18n::t!("map.duplicate_map"))).clicked() {
                            action = Some(MapTreeAction::Duplicate(map.id));
                            ui.close();
                        }
                        if maps.len() > 1 && ui.button(format!("🗑 {}", rust_i18n::t!("map.delete_map"))).clicked() {
                            action = Some(MapTreeAction::Delete(map.id));
                            ui.close();
                        }
                    });
                }
            });

        action
    }
}
