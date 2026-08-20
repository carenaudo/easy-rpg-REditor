use eframe::egui;
use image::RgbaImage;
use std::collections::VecDeque;
use crate::dialogs::event_dialog::EventDialogState;
use crate::dialogs::map_target_picker::MapTargetPickerState;
use crate::lcf_bridge::{self, EventCommandInfo, EventInfo, EventPageInfo, Passability, StartPointInfo};
use crate::tilemap;
use crate::views::map_palette::MapPalette;
use crate::widgets::asset_viewer::AssetPreviewCache;
use crate::widgets::undo_stack::{MapEditAction, TileChange, UndoStack};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MapLayerMode {
    Lower,
    Upper,
    Events,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MapDrawTool {
    Pen,
    Rectangle,
    Ellipse,
    Fill,
    Eyedropper,
}

#[derive(Clone, Debug, PartialEq)]
pub enum QuickEventPending {
    Door { x: i32, y: i32 },
    Transfer { x: i32, y: i32 },
}

pub struct MapDims {
    pub width: i32,
    pub height: i32,
    pub lower: Vec<i32>,
    pub upper: Vec<i32>,
}

pub struct MapViewState {
    pub map_texture: Option<egui::TextureHandle>,
    pub map_dims: Option<MapDims>,
    pub events: Vec<EventInfo>,
    pub start_points: StartPointInfo,
    pub events_dirty: bool,
    pub map_dirty: bool,
    pub save_message: Option<Result<String, String>>,
    pub layer_mode: MapLayerMode,
    pub draw_tool: MapDrawTool,
    pub zoom: f32,
    pub show_grid: bool,
    pub show_lower_layer: bool,
    pub show_upper_layer: bool,
    pub show_events: bool,
    pub show_passability: bool,
    pub dim_inactive_layers: bool,
    pub palette: MapPalette,
    pub undo_stack: UndoStack,
    pub event_dialog: EventDialogState,
    pub target_picker: MapTargetPickerState,
    pub quick_event_pending: Option<QuickEventPending>,
    pub drag_start_tile: Option<(i32, i32)>,
    pub active_stroke_changes: Vec<TileChange>,
    pub active_chipset_id: Option<i32>,
    /// Tile coordinate the pointer is currently over, for the status bar's
    /// Cursor readout. `None` when the pointer is off the canvas or no map
    /// is loaded.
    pub hover_tile: Option<(i32, i32)>,
    /// Clipboard for the event right-click menu's Cut/Copy/Paste.
    pub copied_event: Option<EventInfo>,
    /// Event currently being dragged: (index into `events`, original x, original y).
    /// The event's live x/y are updated directly during the drag; this just
    /// remembers where it started, for the undo action and for snap-back if
    /// the drop lands on another event.
    pub dragging_event: Option<(usize, i32, i32)>,
    /// Tile coordinate where the right-click context menu was opened.
    pub context_menu_tile: Option<(i32, i32)>,
    pub show_shift_dialog: bool,
    pub shift_dx: i32,
    pub shift_dy: i32,
    pub shift_wrap_x: bool,
    pub shift_wrap_y: bool,
}

impl Default for MapViewState {
    fn default() -> Self {
        Self {
            map_texture: None,
            map_dims: None,
            events: Vec::new(),
            start_points: StartPointInfo::default(),
            events_dirty: false,
            map_dirty: false,
            save_message: None,
            layer_mode: MapLayerMode::Lower,
            draw_tool: MapDrawTool::Pen,
            zoom: 1.0,
            show_grid: true,
            show_lower_layer: true,
            show_upper_layer: true,
            show_events: true,
            show_passability: false,
            dim_inactive_layers: false,
            palette: MapPalette::default(),
            undo_stack: UndoStack::new(),
            event_dialog: EventDialogState::default(),
            target_picker: MapTargetPickerState::default(),
            quick_event_pending: None,
            drag_start_tile: None,
            active_stroke_changes: Vec::new(),
            active_chipset_id: None,
            hover_tile: None,
            copied_event: None,
            dragging_event: None,
            context_menu_tile: None,
            show_shift_dialog: false,
            shift_dx: 0,
            shift_dy: 0,
            shift_wrap_x: true,
            shift_wrap_y: true,
        }
    }
}

impl MapViewState {
    pub fn refresh_texture(&mut self, ctx: &egui::Context, chipset: &RgbaImage) {
        if let Some(dims) = &self.map_dims {
            let empty_lower = vec![0; (dims.width * dims.height) as usize];
            let empty_upper = vec![10000; (dims.width * dims.height) as usize];
            let lower_ref = if self.show_lower_layer { &dims.lower } else { &empty_lower };
            let upper_ref = if self.show_upper_layer { &dims.upper } else { &empty_upper };

            let map_img = tilemap::render_map(chipset, dims.width, dims.height, lower_ref, upper_ref);
            let size = [map_img.width() as usize, map_img.height() as usize];
            let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &map_img);
            self.map_texture = Some(ctx.load_texture("map_canvas", color_img, egui::TextureOptions::NEAREST));
        }
    }

    /// Saves the current map's tile layers and event list to disk. Shared by
    /// the map toolbar's Save button and the global Ctrl+S shortcut.
    pub fn save_current_map(&mut self, project_path: Option<&str>, map_id: Option<i32>) {
        if let (Some(proj), Some(mid), Some(dims)) = (project_path, map_id, &self.map_dims) {
            let res_layers = lcf_bridge::save_map_layers(proj, mid, &dims.lower, &dims.upper);
            let res_events = lcf_bridge::save_map_events_full(proj, mid, &self.events);

            if res_layers.is_ok() && res_events.is_ok() {
                self.save_message = Some(Ok("Map saved successfully.".to_string()));
                self.map_dirty = false;
                self.events_dirty = false;
            } else {
                let err_msg = format!("{:?} {:?}", res_layers.err(), res_events.err());
                self.save_message = Some(Err(err_msg));
            }
        }
    }

    /// Undoes the last map edit (tile stroke or event move/delete/paste), if
    /// any, refreshing the rendered map texture and marking things dirty.
    pub fn undo(&mut self, ctx: &egui::Context, chipset: Option<&RgbaImage>) {
        if let Some(dims) = &mut self.map_dims {
            if self.undo_stack.undo(&mut dims.lower, &mut dims.upper, &mut self.events) {
                if let Some(cs) = chipset {
                    self.refresh_texture(ctx, cs);
                }
                self.map_dirty = true;
                self.events_dirty = true;
            }
        }
    }

    /// Redoes the last undone map edit, if any.
    pub fn redo(&mut self, ctx: &egui::Context, chipset: Option<&RgbaImage>) {
        if let Some(dims) = &mut self.map_dims {
            if self.undo_stack.redo(&mut dims.lower, &mut dims.upper, &mut self.events) {
                if let Some(cs) = chipset {
                    self.refresh_texture(ctx, cs);
                }
                self.map_dirty = true;
                self.events_dirty = true;
            }
        }
    }

    pub fn paint_tile_at(&mut self, x: i32, y: i32, new_tile: i32, is_upper: bool) {
        if let Some(dims) = &mut self.map_dims {
            if x < 0 || x >= dims.width || y < 0 || y >= dims.height {
                return;
            }
            let idx = (y * dims.width + x) as usize;
            let target = if is_upper { &mut dims.upper } else { &mut dims.lower };
            if idx < target.len() {
                let old_val = target[idx];
                if old_val != new_tile {
                    target[idx] = new_tile;
                    self.map_dirty = true;
                    self.active_stroke_changes.push(TileChange {
                        index: idx,
                        old_val,
                        new_val: new_tile,
                    });

                    if !is_upper && tilemap::is_autotile_d(new_tile) {
                        tilemap::update_autotile_neighborhood(&mut dims.lower, dims.width, dims.height, x, y);
                    }
                }
            }
        }
    }

    pub fn fill_rect(&mut self, start_x: i32, start_y: i32, end_x: i32, end_y: i32, new_tile: i32, is_upper: bool) {
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);

        let mut changes = Vec::new();
        if let Some(dims) = &mut self.map_dims {
            let target = if is_upper { &mut dims.upper } else { &mut dims.lower };
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if x >= 0 && x < dims.width && y >= 0 && y < dims.height {
                        let idx = (y * dims.width + x) as usize;
                        if idx < target.len() {
                            let old_val = target[idx];
                            if old_val != new_tile {
                                target[idx] = new_tile;
                                changes.push(TileChange { index: idx, old_val, new_val: new_tile });
                            }
                        }
                    }
                }
            }
            if !is_upper && tilemap::is_autotile_d(new_tile) {
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        tilemap::update_autotile_neighborhood(&mut dims.lower, dims.width, dims.height, x, y);
                    }
                }
            }
        }
        if !changes.is_empty() {
            self.undo_stack.push(MapEditAction::TileStroke { is_upper, changes });
            self.map_dirty = true;
        }
    }

    pub fn fill_ellipse(&mut self, start_x: i32, start_y: i32, end_x: i32, end_y: i32, new_tile: i32, is_upper: bool) {
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);

        let rx = (max_x - min_x) as f32 / 2.0;
        let ry = (max_y - min_y) as f32 / 2.0;
        if rx <= 0.0 || ry <= 0.0 {
            self.fill_rect(start_x, start_y, end_x, end_y, new_tile, is_upper);
            return;
        }

        let cx = min_x as f32 + rx;
        let cy = min_y as f32 + ry;

        let mut changes = Vec::new();
        if let Some(dims) = &mut self.map_dims {
            let target = if is_upper { &mut dims.upper } else { &mut dims.lower };
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let dx = (x as f32 - cx) / rx;
                    let dy = (y as f32 - cy) / ry;
                    if dx * dx + dy * dy <= 1.05 && x >= 0 && x < dims.width && y >= 0 && y < dims.height {
                        let idx = (y * dims.width + x) as usize;
                        if idx < target.len() {
                            let old_val = target[idx];
                            if old_val != new_tile {
                                target[idx] = new_tile;
                                changes.push(TileChange { index: idx, old_val, new_val: new_tile });
                            }
                        }
                    }
                }
            }
            if !is_upper && tilemap::is_autotile_d(new_tile) {
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        tilemap::update_autotile_neighborhood(&mut dims.lower, dims.width, dims.height, x, y);
                    }
                }
            }
        }
        if !changes.is_empty() {
            self.undo_stack.push(MapEditAction::TileStroke { is_upper, changes });
            self.map_dirty = true;
        }
    }

    pub fn flood_fill(&mut self, start_x: i32, start_y: i32, new_tile: i32, is_upper: bool) {
        let mut changes = Vec::new();
        if let Some(dims) = &mut self.map_dims {
            if start_x < 0 || start_x >= dims.width || start_y < 0 || start_y >= dims.height {
                return;
            }

            let start_idx = (start_y * dims.width + start_x) as usize;
            let target_ref = if is_upper { &dims.upper } else { &dims.lower };
            let target_tile = target_ref[start_idx];

            if target_tile == new_tile {
                return;
            }

            let mut visited = vec![false; (dims.width * dims.height) as usize];
            let mut queue = VecDeque::new();
            queue.push_back((start_x, start_y));
            visited[start_idx] = true;

            let target = if is_upper { &mut dims.upper } else { &mut dims.lower };

            while let Some((cx, cy)) = queue.pop_front() {
                let idx = (cy * dims.width + cx) as usize;
                let old_val = target[idx];
                target[idx] = new_tile;
                changes.push(TileChange { index: idx, old_val, new_val: new_tile });

                let neighbors = [(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)];
                for (nx, ny) in neighbors {
                    if nx >= 0 && nx < dims.width && ny >= 0 && ny < dims.height {
                        let n_idx = (ny * dims.width + nx) as usize;
                        if !visited[n_idx] && target[n_idx] == target_tile {
                            visited[n_idx] = true;
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }

            if !is_upper && tilemap::is_autotile_d(new_tile) {
                for cy in 0..dims.height {
                    for cx in 0..dims.width {
                        tilemap::update_autotile_neighborhood(&mut dims.lower, dims.width, dims.height, cx, cy);
                    }
                }
            }
        }

        if !changes.is_empty() {
            self.undo_stack.push(MapEditAction::TileStroke { is_upper, changes });
            self.map_dirty = true;
        }
    }

    pub fn create_quick_door(&mut self, x: i32, y: i32, target_map: i32, target_x: i32, target_y: i32) {
        let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
        let event = EventInfo {
            id: new_id,
            name: format!("Door_{:04}", new_id),
            x,
            y,
            page_count: 1,
            graphic: "Door1".to_string(),
            trigger: "Action Button".to_string(),
            pages: vec![EventPageInfo {
                id: 1,
                character_name: "Door1".to_string(),
                character_index: 0,
                trigger: 0, // Action button
                layer: 1,
                commands: vec![
                    EventCommandInfo { code: 11140, string: "Open1".to_string(), ..Default::default() },
                    EventCommandInfo { code: 10610, parameters: vec![target_map, target_x, target_y], ..Default::default() },
                ],
                ..Default::default()
            }],
        };
        self.events.push(event);
        self.events_dirty = true;
    }

    pub fn create_quick_chest(&mut self, x: i32, y: i32) {
        let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
        let event = EventInfo {
            id: new_id,
            name: format!("Chest_{:04}", new_id),
            x,
            y,
            page_count: 2,
            graphic: "Chest".to_string(),
            trigger: "Action Button".to_string(),
            pages: vec![
                EventPageInfo {
                    id: 1,
                    character_name: "Chest".to_string(),
                    character_index: 0,
                    trigger: 0,
                    layer: 1,
                    commands: vec![
                        EventCommandInfo { code: 11140, string: "Chest1".to_string(), ..Default::default() },
                        EventCommandInfo { code: 10310, parameters: vec![0, 0, 50], ..Default::default() },
                        EventCommandInfo { code: 10110, string: "Found 50 Gold!".to_string(), ..Default::default() },
                        EventCommandInfo { code: 10210, parameters: vec![1, 0, 0], ..Default::default() },
                    ],
                    ..Default::default()
                },
                EventPageInfo {
                    id: 2,
                    character_name: "Chest".to_string(),
                    character_index: 1,
                    trigger: 0,
                    layer: 1,
                    commands: Vec::new(),
                    ..Default::default()
                },
            ],
        };
        self.events.push(event);
        self.events_dirty = true;
    }

    pub fn create_quick_inn(&mut self, x: i32, y: i32) {
        let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
        let event = EventInfo {
            id: new_id,
            name: format!("Inn_{:04}", new_id),
            x,
            y,
            page_count: 1,
            graphic: "People1".to_string(),
            trigger: "Action Button".to_string(),
            pages: vec![EventPageInfo {
                id: 1,
                character_name: "People1".to_string(),
                character_index: 0,
                trigger: 0,
                layer: 1,
                commands: vec![
                    EventCommandInfo { code: 10110, string: "Welcome to the Inn! Stay for 10 Gold?".to_string(), ..Default::default() },
                    EventCommandInfo { code: 10130, string: "Yes/No".to_string(), ..Default::default() },
                    EventCommandInfo { code: 10310, parameters: vec![1, 0, 10], ..Default::default() },
                    EventCommandInfo { code: 11110, string: "Inn".to_string(), ..Default::default() },
                    EventCommandInfo { code: 11030, parameters: vec![20], ..Default::default() },
                    EventCommandInfo { code: 10110, string: "Have a safe journey!".to_string(), ..Default::default() },
                ],
                ..Default::default()
            }],
        };
        self.events.push(event);
        self.events_dirty = true;
    }

    pub fn create_quick_transfer(&mut self, x: i32, y: i32, target_map: i32, target_x: i32, target_y: i32) {
        let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
        let event = EventInfo {
            id: new_id,
            name: format!("Transfer_{:04}", new_id),
            x,
            y,
            page_count: 1,
            graphic: String::new(),
            trigger: "Player Touch".to_string(),
            pages: vec![EventPageInfo {
                id: 1,
                trigger: 1, // Player touch
                layer: 0, // Below player
                commands: vec![
                    EventCommandInfo { code: 10610, parameters: vec![target_map, target_x, target_y], ..Default::default() },
                ],
                ..Default::default()
            }],
        };
        self.events.push(event);
        self.events_dirty = true;
    }

    pub fn create_quick_save_point(&mut self, x: i32, y: i32) {
        let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
        let event = EventInfo {
            id: new_id,
            name: format!("SavePoint_{:04}", new_id),
            x,
            y,
            page_count: 1,
            graphic: "Save Point".to_string(),
            trigger: "Action Button".to_string(),
            pages: vec![EventPageInfo {
                id: 1,
                character_name: "SavePoint".to_string(),
                character_index: 0,
                trigger: 0, // Action Button
                layer: 1, // Same level
                commands: vec![
                    EventCommandInfo { code: 10110, string: "Would you like to save your game?".to_string(), ..Default::default() },
                    EventCommandInfo { code: 10130, string: "Yes/No".to_string(), ..Default::default() },
                    EventCommandInfo { code: 20130, parameters: vec![0], ..Default::default() }, // Choice 1: Yes
                    EventCommandInfo { code: 11430, ..Default::default() }, // Open Save Menu
                    EventCommandInfo { code: 20130, parameters: vec![1], ..Default::default() }, // Choice 2: No
                    EventCommandInfo { code: 20132, ..Default::default() }, // End Choices
                ],
                ..Default::default()
            }],
        };
        self.events.push(event);
        self.events_dirty = true;
    }

    pub fn create_quick_recovery(&mut self, x: i32, y: i32) {
        let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
        let event = EventInfo {
            id: new_id,
            name: format!("Fountain_{:04}", new_id),
            x,
            y,
            page_count: 1,
            graphic: "Fountain".to_string(),
            trigger: "Action Button".to_string(),
            pages: vec![EventPageInfo {
                id: 1,
                character_name: "Fountain".to_string(),
                character_index: 0,
                trigger: 0, // Action Button
                layer: 1, // Same level
                commands: vec![
                    EventCommandInfo { code: 10730, parameters: vec![31, 31, 31, 20, 10], ..Default::default() }, // Flash screen white
                    EventCommandInfo { code: 10420, parameters: vec![0], ..Default::default() }, // Recover All Entire Party
                    EventCommandInfo { code: 10110, string: "The soothing spring waters have restored your party's HP and SP!".to_string(), ..Default::default() },
                ],
                ..Default::default()
            }],
        };
        self.events.push(event);
        self.events_dirty = true;
    }

    pub fn shift_map(
        &mut self,
        dx: i32,
        dy: i32,
        wrap_x: bool,
        wrap_y: bool,
        chipset: Option<&RgbaImage>,
        ctx: &egui::Context,
    ) {
        if let Some(dims) = &mut self.map_dims {
            let w = dims.width;
            let h = dims.height;
            if w <= 0 || h <= 0 {
                return;
            }
            let mut new_lower = vec![0; (w * h) as usize];
            let mut new_upper = vec![10000; (w * h) as usize];

            for y in 0..h {
                for x in 0..w {
                    let old_idx = (y * w + x) as usize;
                    let target_x = if wrap_x {
                        ((x + dx) % w + w) % w
                    } else {
                        x + dx
                    };
                    let target_y = if wrap_y {
                        ((y + dy) % h + h) % h
                    } else {
                        y + dy
                    };

                    if target_x >= 0 && target_x < w && target_y >= 0 && target_y < h {
                        let new_idx = (target_y * w + target_x) as usize;
                        if old_idx < dims.lower.len() && new_idx < new_lower.len() {
                            new_lower[new_idx] = dims.lower[old_idx];
                        }
                        if old_idx < dims.upper.len() && new_idx < new_upper.len() {
                            new_upper[new_idx] = dims.upper[old_idx];
                        }
                    }
                }
            }
            dims.lower = new_lower;
            dims.upper = new_upper;
            self.map_dirty = true;

            // Shift events
            for ev in &mut self.events {
                let target_x = if wrap_x {
                    ((ev.x + dx) % w + w) % w
                } else {
                    ev.x + dx
                };
                let target_y = if wrap_y {
                    ((ev.y + dy) % h + h) % h
                } else {
                    ev.y + dy
                };
                ev.x = target_x;
                ev.y = target_y;
            }
            self.events_dirty = true;

            if let Some(cs) = chipset {
                self.refresh_texture(ctx, cs);
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        project_path: Option<&str>,
        map_id: Option<i32>,
        chipset: Option<&RgbaImage>,
        passability: &Passability,
        asset_cache: &mut AssetPreviewCache,
    ) {
        let proj_str = project_path.unwrap_or("");

        // Handle Target Picker Dialog result for Quick Events
        if let Some((t_map, t_x, t_y)) = self.target_picker.show(ui.ctx(), proj_str) {
            if let Some(pending) = self.quick_event_pending.take() {
                match pending {
                    QuickEventPending::Door { x, y } => self.create_quick_door(x, y, t_map, t_x, t_y),
                    QuickEventPending::Transfer { x, y } => self.create_quick_transfer(x, y, t_map, t_x, t_y),
                }
            }
        }

        // Event Dialog modal
        if let Some(proj) = project_path {
            if let Some(saved_event) = self.event_dialog.show(ui.ctx(), proj, asset_cache) {
                if let Some(existing) = self.events.iter_mut().find(|e| e.id == saved_event.id) {
                    *existing = saved_event;
                } else {
                    self.events.push(saved_event);
                }
                self.events_dirty = true;
            }
        }

        // Shift Map Dialog modal
        if self.show_shift_dialog {
            let mut open = true;
            egui::Window::new("↔ Shift Map")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_size([300.0, 200.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Shift entire map contents & events by offset:");
                    egui::Grid::new("shift_map_grid")
                        .num_columns(2)
                        .spacing([8.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Horizontal (ΔX):");
                            ui.add(egui::DragValue::new(&mut self.shift_dx).range(-99..=99));
                            ui.end_row();

                            ui.label("Vertical (ΔY):");
                            ui.add(egui::DragValue::new(&mut self.shift_dy).range(-99..=99));
                            ui.end_row();
                        });

                    ui.checkbox(&mut self.shift_wrap_x, "Wrap Horizontally (Loop)");
                    ui.checkbox(&mut self.shift_wrap_y, "Wrap Vertically (Loop)");

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("✅ Apply Shift").clicked() {
                            let dx = self.shift_dx;
                            let dy = self.shift_dy;
                            let wx = self.shift_wrap_x;
                            let wy = self.shift_wrap_y;
                            self.shift_map(dx, dy, wx, wy, chipset, ui.ctx());
                            self.show_shift_dialog = false;
                        }
                        if ui.button("❌ Cancel").clicked() {
                            self.show_shift_dialog = false;
                        }
                    });
                });
            if !open {
                self.show_shift_dialog = false;
            }
        }

        // Main map editing canvas
        egui::ScrollArea::both()
            .id_salt("map_canvas_scroll")
            .show(ui, |ui| {
            if let (Some(dims), Some(tex), Some(cs)) = (&mut self.map_dims, &self.map_texture, chipset) {
                let tile_px = 16.0 * self.zoom;
                let canvas_w = dims.width as f32 * tile_px;
                let canvas_h = dims.height as f32 * tile_px;

                let (rect, resp) = ui.allocate_exact_size(egui::vec2(canvas_w, canvas_h), egui::Sense::click_and_drag());
                let painter = ui.painter().with_clip_rect(rect);

                // Track the hovered tile for the status bar's Cursor readout,
                // independent of click/drag interaction below.
                self.hover_tile = resp.hover_pos().and_then(|pos| {
                    let tx = ((pos.x - rect.min.x) / tile_px) as i32;
                    let ty = ((pos.y - rect.min.y) / tile_px) as i32;
                    if tx >= 0 && tx < dims.width && ty >= 0 && ty < dims.height {
                        Some((tx, ty))
                    } else {
                        None
                    }
                });

                // Capture tile on right-click for the context menu
                if resp.secondary_clicked() {
                    if let Some(pos) = resp.interact_pointer_pos().or(resp.hover_pos()) {
                        let tx = ((pos.x - rect.min.x) / tile_px) as i32;
                        let ty = ((pos.y - rect.min.y) / tile_px) as i32;
                        if tx >= 0 && tx < dims.width && ty >= 0 && ty < dims.height {
                            self.context_menu_tile = Some((tx, ty));
                        }
                    }
                }

                // Render Map Image
                let tint = if self.dim_inactive_layers && self.layer_mode == MapLayerMode::Events {
                    egui::Color32::from_rgba_unmultiplied(160, 160, 160, 180)
                } else {
                    egui::Color32::WHITE
                };
                painter.image(
                    tex.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    tint,
                );

                // Passability Overlay
                if self.show_passability {
                    for y in 0..dims.height {
                        for x in 0..dims.width {
                            let idx = (y * dims.width + x) as usize;
                            let l_id = dims.lower.get(idx).copied().unwrap_or(0);
                            let u_id = dims.upper.get(idx).copied().unwrap_or(10000);
                            if tilemap::is_blocked(l_id, u_id, &passability.lower, &passability.upper) {
                                let cell_rect = egui::Rect::from_min_size(
                                    egui::pos2(rect.min.x + x as f32 * tile_px, rect.min.y + y as f32 * tile_px),
                                    egui::vec2(tile_px, tile_px),
                                );
                                painter.rect_filled(cell_rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 0, 0, 80));
                                painter.line_segment(
                                    [cell_rect.min, cell_rect.max],
                                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 0, 0, 160)),
                                );
                            }
                        }
                    }
                }

                // Precision Grid lines overlay
                if self.show_grid {
                    let stroke_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 35);
                    for x in 0..=dims.width {
                        let px = rect.min.x + x as f32 * tile_px;
                        painter.line_segment(
                            [egui::pos2(px, rect.min.y), egui::pos2(px, rect.max.y)],
                            egui::Stroke::new(1.0, stroke_color),
                        );
                    }
                    for y in 0..=dims.height {
                        let py = rect.min.y + y as f32 * tile_px;
                        painter.line_segment(
                            [egui::pos2(rect.min.x, py), egui::pos2(rect.max.x, py)],
                            egui::Stroke::new(1.0, stroke_color),
                        );
                    }
                }

                // Render Start Points
                if let Some(cur_map) = map_id {
                    if self.start_points.party_map_id == cur_map {
                        let p_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + self.start_points.party_x as f32 * tile_px, rect.min.y + self.start_points.party_y as f32 * tile_px),
                            egui::vec2(tile_px, tile_px),
                        );
                        painter.rect_filled(p_rect, 2.0, egui::Color32::from_rgba_unmultiplied(240, 200, 0, 180));
                        painter.text(p_rect.center(), egui::Align2::CENTER_CENTER, "🧍", egui::FontId::proportional(12.0), egui::Color32::BLACK);
                    }
                    if self.start_points.boat_map_id == cur_map {
                        let b_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + self.start_points.boat_x as f32 * tile_px, rect.min.y + self.start_points.boat_y as f32 * tile_px),
                            egui::vec2(tile_px, tile_px),
                        );
                        painter.rect_filled(b_rect, 2.0, egui::Color32::from_rgba_unmultiplied(0, 150, 240, 180));
                        painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, "⛵", egui::FontId::proportional(12.0), egui::Color32::WHITE);
                    }
                }

                // Render Events Overlay
                if self.show_events {
                    for event in &self.events {
                        let ev_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + event.x as f32 * tile_px, rect.min.y + event.y as f32 * tile_px),
                            egui::vec2(tile_px, tile_px),
                        );

                        let fill_color = if self.layer_mode == MapLayerMode::Events {
                            egui::Color32::from_rgba_unmultiplied(0, 180, 255, 160)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(0, 140, 220, 90)
                        };
                        painter.rect_filled(ev_rect, 2.0, fill_color);
                        painter.rect_stroke(ev_rect, 2.0, egui::Stroke::new(1.5, egui::Color32::WHITE), egui::StrokeKind::Outside);
                        painter.text(
                            ev_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("E{}", event.id),
                            egui::FontId::monospace(10.0 * self.zoom.clamp(0.8, 1.4)),
                            egui::Color32::WHITE,
                        );
                    }
                }

                // Interactive Mouse Input Handling
                if let Some(pos) = resp.interact_pointer_pos() {
                    let tile_x = ((pos.x - rect.min.x) / tile_px) as i32;
                    let tile_y = ((pos.y - rect.min.y) / tile_px) as i32;

                    if tile_x >= 0 && tile_x < dims.width && tile_y >= 0 && tile_y < dims.height {
                        // Hover box
                        let hover_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + tile_x as f32 * tile_px, rect.min.y + tile_y as f32 * tile_px),
                            egui::vec2(tile_px, tile_px),
                        );
                        painter.rect_stroke(
                            hover_rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 220, 0)),
                            egui::StrokeKind::Outside,
                        );

                        // Alt+Click or Middle-Click Eyedropper shortcut
                        let is_upper = self.layer_mode == MapLayerMode::Upper;
                        if ui.input(|i| (i.modifiers.alt && i.pointer.primary_clicked()) || i.pointer.button_down(egui::PointerButton::Middle)) {
                            let idx = (tile_y * dims.width + tile_x) as usize;
                            let t_id = if is_upper { dims.upper.get(idx) } else { dims.lower.get(idx) };
                            if let Some(&id) = t_id {
                                self.palette.selected_tile_id = id;
                            }
                        }

                        if self.layer_mode == MapLayerMode::Events {
                            // Drag-to-move: grab whichever event sits on the tile a
                            // primary-button drag starts on. Must check the button
                            // specifically - `drag_started()` alone fires for any
                            // button, and real mouse input almost always has a
                            // pixel or two of jitter during a right-click's
                            // press-release, which was tipping egui's interaction
                            // resolution into "drag" and suppressing the
                            // secondary-click the context menu depends on.
                            if resp.drag_started_by(egui::PointerButton::Primary) {
                                if let Some(idx) = self.events.iter().position(|e| e.x == tile_x && e.y == tile_y) {
                                    self.dragging_event = Some((idx, tile_x, tile_y));
                                }
                            }

                            if let Some((idx, orig_x, orig_y)) = self.dragging_event {
                                if resp.dragged_by(egui::PointerButton::Primary) {
                                    if let Some(ev) = self.events.get_mut(idx) {
                                        ev.x = tile_x;
                                        ev.y = tile_y;
                                    }
                                }

                                if resp.drag_stopped() {
                                    let blocked = self.events.iter().enumerate()
                                        .any(|(i, e)| i != idx && e.x == tile_x && e.y == tile_y);
                                    if blocked {
                                        // Another event already occupies the drop tile -
                                        // snap back rather than stack two events together.
                                        if let Some(ev) = self.events.get_mut(idx) {
                                            ev.x = orig_x;
                                            ev.y = orig_y;
                                        }
                                    } else if (orig_x, orig_y) != (tile_x, tile_y) {
                                        self.undo_stack.push(MapEditAction::EventMove {
                                            index: idx,
                                            old_pos: (orig_x, orig_y),
                                            new_pos: (tile_x, tile_y),
                                        });
                                        self.events_dirty = true;
                                    }
                                    self.dragging_event = None;
                                }
                            }

                            // Double-click still opens/creates events - a click that
                            // never became a real drag falls through to here.
                            if resp.double_clicked() {
                                if let Some(ev) = self.events.iter().find(|e| e.x == tile_x && e.y == tile_y) {
                                    self.event_dialog.open(ev);
                                } else {
                                    let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
                                    let new_ev = EventInfo {
                                        id: new_id,
                                        name: format!("EV{:04}", new_id),
                                        x: tile_x,
                                        y: tile_y,
                                        page_count: 1,
                                        ..Default::default()
                                    };
                                    self.event_dialog.open(&new_ev);
                                }
                            }
                        } else {
                            // Tile painting mode
                            let selected_tile = self.palette.selected_tile_id;

                            // Primary-button only, same reasoning as the event
                            // drag guard above - `dragged()` alone fires for any
                            // button and would let right-click jitter paint a
                            // tile instead of opening the context menu.
                            if resp.clicked() || resp.dragged_by(egui::PointerButton::Primary) {
                                match self.draw_tool {
                                    MapDrawTool::Pen => {
                                        self.paint_tile_at(tile_x, tile_y, selected_tile, is_upper);
                                        self.refresh_texture(ui.ctx(), cs);
                                    }
                                    MapDrawTool::Fill => {
                                        if resp.clicked() {
                                            self.flood_fill(tile_x, tile_y, selected_tile, is_upper);
                                            self.refresh_texture(ui.ctx(), cs);
                                        }
                                    }
                                    MapDrawTool::Eyedropper => {
                                        let idx = (tile_y * dims.width + tile_x) as usize;
                                        let t_id = if is_upper { dims.upper.get(idx) } else { dims.lower.get(idx) };
                                        if let Some(&id) = t_id {
                                            self.palette.selected_tile_id = id;
                                        }
                                    }
                                    MapDrawTool::Rectangle | MapDrawTool::Ellipse => {
                                        if self.drag_start_tile.is_none() {
                                            self.drag_start_tile = Some((tile_x, tile_y));
                                        }
                                    }
                                }
                            }

                            // Shape Tool Previews
                            if let Some((sx, sy)) = self.drag_start_tile {
                                let min_x = sx.min(tile_x);
                                let max_x = sx.max(tile_x);
                                let min_y = sy.min(tile_y);
                                let max_y = sy.max(tile_y);

                                let box_rect = egui::Rect::from_min_max(
                                    egui::pos2(rect.min.x + min_x as f32 * tile_px, rect.min.y + min_y as f32 * tile_px),
                                    egui::pos2(rect.min.x + (max_x + 1) as f32 * tile_px, rect.min.y + (max_y + 1) as f32 * tile_px),
                                );
                                painter.rect_filled(box_rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 255, 0, 60));
                                painter.rect_stroke(box_rect, 0.0, egui::Stroke::new(2.0, egui::Color32::YELLOW), egui::StrokeKind::Outside);

                                if resp.drag_stopped() {
                                    match self.draw_tool {
                                        MapDrawTool::Rectangle => self.fill_rect(sx, sy, tile_x, tile_y, selected_tile, is_upper),
                                        MapDrawTool::Ellipse => self.fill_ellipse(sx, sy, tile_x, tile_y, selected_tile, is_upper),
                                        _ => {}
                                    }
                                    self.refresh_texture(ui.ctx(), cs);
                                    self.drag_start_tile = None;
                                }
                            }

                            // Commit active stroke into undo stack
                            if resp.drag_stopped() && !self.active_stroke_changes.is_empty() {
                                let changes = std::mem::take(&mut self.active_stroke_changes);
                                self.undo_stack.push(MapEditAction::TileStroke { is_upper, changes });
                            }
                        }
                    }
                }

                // Right-Click Context Menu on Canvas
                if let Some((tile_x, tile_y)) = self.context_menu_tile {
                    let cur_map_id = map_id.unwrap_or(1);
                    let event_here_idx = if self.layer_mode == MapLayerMode::Events {
                        self.events.iter().position(|e| e.x == tile_x && e.y == tile_y)
                    } else {
                        None
                    };

                    resp.context_menu(|ui| {
                        // One consistent menu: event-specific actions when
                        // the tile already holds an event, then the
                        // creation actions (New/Paste/Quick Events) - which
                        // don't make sense on an occupied tile, since two
                        // events can't share a tile (same rule the drag-to-
                        // move block enforces) - only when it's empty. Set
                        // Starting Position is always available either way.
                        if let Some(idx) = event_here_idx {
                            let ev_id = self.events[idx].id;
                            let ev_name = self.events[idx].name.clone();
                            let ev_pages = self.events[idx].page_count;
                            ui.label(format!("#{ev_id} {ev_name} — {ev_pages} page(s)"));
                            ui.separator();

                            if ui.button(format!("✏ {}", rust_i18n::t!("map.edit_event"))).clicked() {
                                self.event_dialog.open(&self.events[idx]);
                                ui.close();
                            }
                            if ui.button(format!("🗑 {}", rust_i18n::t!("map.delete_event"))).clicked() {
                                let removed = self.events.remove(idx);
                                self.undo_stack.push(MapEditAction::EventDelete { index: idx, event: removed });
                                self.events_dirty = true;
                                ui.close();
                            }
                            if ui.button(format!("✂ {}", rust_i18n::t!("map.cut_event"))).clicked() {
                                self.copied_event = Some(self.events[idx].clone());
                                let removed = self.events.remove(idx);
                                self.undo_stack.push(MapEditAction::EventDelete { index: idx, event: removed });
                                self.events_dirty = true;
                                ui.close();
                            }
                            if ui.button(format!("📋 {}", rust_i18n::t!("map.copy_event"))).clicked() {
                                self.copied_event = Some(self.events[idx].clone());
                                ui.close();
                            }
                        } else {
                            if ui.button(format!("✨ {}", rust_i18n::t!("map.new_event_here"))).clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                let new_id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
                                let new_ev = EventInfo {
                                    id: new_id,
                                    name: format!("EV{:04}", new_id),
                                    x: tile_x,
                                    y: tile_y,
                                    page_count: 1,
                                    ..Default::default()
                                };
                                self.event_dialog.open(&new_ev);
                                ui.close();
                            }

                            if self.copied_event.is_some() && ui.button(format!("📋 {}", rust_i18n::t!("map.paste_event"))).clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                let mut pasted = self.copied_event.clone().unwrap();
                                pasted.id = (self.events.iter().map(|e| e.id).max().unwrap_or(0)) + 1;
                                pasted.x = tile_x;
                                pasted.y = tile_y;
                                let idx = self.events.len();
                                self.events.push(pasted.clone());
                                self.undo_stack.push(MapEditAction::EventAdd { index: idx, event: pasted });
                                self.events_dirty = true;
                                ui.close();
                            }
                        }

                        ui.separator();
                        ui.label("Set Starting Position:");
                        if ui.button(format!("🧍 {}", rust_i18n::t!("map.set_start_party"))).clicked() {
                            self.start_points.party_map_id = cur_map_id;
                            self.start_points.party_x = tile_x;
                            self.start_points.party_y = tile_y;
                            if let Some(proj) = project_path {
                                let _ = lcf_bridge::save_start_points(proj, &self.start_points);
                            }
                            ui.close();
                        }
                        if ui.button(format!("⛵ {}", rust_i18n::t!("map.set_start_boat"))).clicked() {
                            self.start_points.boat_map_id = cur_map_id;
                            self.start_points.boat_x = tile_x;
                            self.start_points.boat_y = tile_y;
                            if let Some(proj) = project_path {
                                let _ = lcf_bridge::save_start_points(proj, &self.start_points);
                            }
                            ui.close();
                        }

                        if event_here_idx.is_none() {
                            ui.separator();
                            ui.label("Quick Events:");
                            if ui.button(format!("🚪 {}", rust_i18n::t!("map.quick_event_door"))).clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                self.quick_event_pending = Some(QuickEventPending::Door { x: tile_x, y: tile_y });
                                self.target_picker.open(proj_str, cur_map_id, tile_x, tile_y);
                                ui.close();
                            }
                            if ui.button(format!("🎁 {}", rust_i18n::t!("map.quick_event_chest"))).clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                self.create_quick_chest(tile_x, tile_y);
                                ui.close();
                            }
                            if ui.button(format!("🏨 {}", rust_i18n::t!("map.quick_event_inn"))).clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                self.create_quick_inn(tile_x, tile_y);
                                ui.close();
                            }
                            if ui.button(format!("🚶 {}", rust_i18n::t!("map.quick_event_transfer"))).clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                self.quick_event_pending = Some(QuickEventPending::Transfer { x: tile_x, y: tile_y });
                                self.target_picker.open(proj_str, cur_map_id, tile_x, tile_y);
                                ui.close();
                            }
                            if ui.button("💾 Quick Save Point").clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                self.create_quick_save_point(tile_x, tile_y);
                                ui.close();
                            }
                            if ui.button("⛲ Quick Recovery Spring").clicked() {
                                self.layer_mode = MapLayerMode::Events;
                                self.show_events = true;
                                self.create_quick_recovery(tile_x, tile_y);
                                ui.close();
                            }
                        }
                    });
                }
            }
        });
    }
}
