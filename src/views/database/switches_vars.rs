use eframe::egui;
use crate::lcf_bridge::{SwitchInfo, VariableInfo};

pub struct SwitchVarViewState {
    pub search_query: String,
    pub active_range: usize, // 0 = 1..20, 1 = 21..40, etc. usize::MAX = All
    pub show_resize_modal: bool,
    pub new_capacity: usize,
}

impl Default for SwitchVarViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_range: 0,
            show_resize_modal: false,
            new_capacity: 100,
        }
    }
}

pub fn show_switches_table(
    ui: &mut egui::Ui,
    switches: &mut Vec<SwitchInfo>,
    state: &mut SwitchVarViewState,
    dirty: &mut bool,
) {
    render_table(
        ui,
        "Switches",
        "🔲",
        switches,
        state,
        dirty,
        |id| SwitchInfo { id: id as i32, name: String::new() },
        |item| &mut item.name,
        |item| item.id,
    );
}

pub fn show_variables_table(
    ui: &mut egui::Ui,
    variables: &mut Vec<VariableInfo>,
    state: &mut SwitchVarViewState,
    dirty: &mut bool,
) {
    render_table(
        ui,
        "Variables",
        "🔢",
        variables,
        state,
        dirty,
        |id| VariableInfo { id: id as i32, name: String::new() },
        |item| &mut item.name,
        |item| item.id,
    );
}

fn render_table<T>(
    ui: &mut egui::Ui,
    title: &str,
    icon: &str,
    items: &mut Vec<T>,
    state: &mut SwitchVarViewState,
    dirty: &mut bool,
    create_new: impl Fn(usize) -> T,
    get_name: impl Fn(&mut T) -> &mut String,
    get_id: impl Fn(&T) -> i32,
) {
    let total_count = items.len();

    // Header Toolbar
    ui.horizontal(|ui| {
        ui.heading(format!("{} {} ({})", icon, title, total_count));

        ui.separator();

        // Search Filter
        ui.label("🔍");
        let _ = ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text("Filter by name or #ID...")
                .desired_width(220.0),
        );
        if !state.search_query.is_empty() && ui.small_button("✕").clicked() {
            state.search_query.clear();
        }

        ui.separator();

        if ui.button("📏 Change Max Capacity...").clicked() {
            state.new_capacity = total_count;
            state.show_resize_modal = true;
        }

        if ui.button("🧹 Clear Empty Names").clicked() {
            // No-op or notification
        }
    });

    // Resize Modal Dialog
    if state.show_resize_modal {
        egui::Window::new(format!("Change Maximum {}", title))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label(format!("Current capacity: {} items", total_count));
                ui.label("Enter new maximum number of entries (e.g. 100, 500, 1000, 5000):");
                ui.add(egui::DragValue::new(&mut state.new_capacity).range(1..=5000));

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        if state.new_capacity > total_count {
                            for i in (total_count + 1)..=state.new_capacity {
                                items.push(create_new(i));
                            }
                            *dirty = true;
                        } else if state.new_capacity < total_count && state.new_capacity >= 1 {
                            items.truncate(state.new_capacity);
                            *dirty = true;
                        }
                        state.show_resize_modal = false;
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_resize_modal = false;
                    }
                });
            });
    }

    ui.add_space(4.0);

    // Range Tabs (0001-0020, 0021-0040, etc.)
    let page_size = 20;
    let num_pages = (total_count + page_size - 1) / page_size;

    let is_filtering = !state.search_query.trim().is_empty();
    if !is_filtering {
        egui::ScrollArea::horizontal()
            .id_salt(format!("{}_range_scroll", title))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(state.active_range == usize::MAX, "Show All").clicked() {
                        state.active_range = usize::MAX;
                    }
                    for p in 0..num_pages {
                        let start_id = p * page_size + 1;
                        let end_id = ((p + 1) * page_size).min(total_count);
                        let label = format!("{:04} – {:04}", start_id, end_id);
                        if ui.selectable_label(state.active_range == p, label).clicked() {
                            state.active_range = p;
                        }
                    }
                });
            });
        ui.separator();
    } else {
        ui.label(format!("Showing search matches for \"{}\":", state.search_query));
        ui.separator();
    }

    // Determine filtered list of indices
    let query = state.search_query.trim().to_lowercase();
    let indices: Vec<usize> = (0..total_count)
        .filter(|&i| {
            if is_filtering {
                let id_str = get_id(&items[i]).to_string();
                let name = get_name(&mut items[i]).to_lowercase();
                id_str.contains(&query) || name.contains(&query)
            } else if state.active_range == usize::MAX {
                true
            } else {
                let start_idx = state.active_range * page_size;
                let end_idx = ((state.active_range + 1) * page_size).min(total_count);
                i >= start_idx && i < end_idx
            }
        })
        .collect();

    if indices.is_empty() {
        ui.colored_label(egui::Color32::GRAY, "(No matching entries found)");
        return;
    }

    // Responsive Multi-Column Layout
    let avail_width = ui.available_width();
    let col_width = 320.0f32;
    let num_cols = ((avail_width / col_width).floor() as usize).max(1).min(4);

    egui::ScrollArea::vertical()
        .id_salt(format!("{}_multi_col_scroll", title))
        .show(ui, |ui| {
            let chunk_size = (indices.len() + num_cols - 1) / num_cols;
            ui.columns(num_cols, |cols| {
                for col_idx in 0..num_cols {
                    let col_ui = &mut cols[col_idx];
                    let start = col_idx * chunk_size;
                    let end = ((col_idx + 1) * chunk_size).min(indices.len());

                    if start >= indices.len() {
                        continue;
                    }

                    col_ui.group(|ui| {
                        for &idx in &indices[start..end] {
                            let item = &mut items[idx];
                            let id = get_id(item);
                            let name = get_name(item);

                            ui.horizontal(|ui| {
                                // ID Badge
                                ui.colored_label(
                                    egui::Color32::from_rgb(130, 180, 240),
                                    format!("#{:04}:", id),
                                );

                                // Text Edit
                                let resp = ui.add(
                                    egui::TextEdit::singleline(name)
                                        .hint_text("(Unnamed)")
                                        .desired_width(ui.available_width() - 8.0),
                                );
                                if resp.changed() {
                                    *dirty = true;
                                }
                            });
                            ui.add_space(2.0);
                        }
                    });
                }
            });
        });
}

