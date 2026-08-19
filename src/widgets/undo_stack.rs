use crate::lcf_bridge::EventInfo;

#[derive(Clone, Debug)]
pub struct TileChange {
    pub index: usize,
    pub old_val: i32,
    pub new_val: i32,
}

#[derive(Clone, Debug)]
pub enum MapEditAction {
    TileStroke {
        is_upper: bool,
        changes: Vec<TileChange>,
    },
    /// An existing event was dragged from `old_pos` to `new_pos`.
    EventMove {
        index: usize,
        old_pos: (i32, i32),
        new_pos: (i32, i32),
    },
    /// An event was removed (via Delete or Cut). Carries the full event so
    /// undo can reinsert it at the same list index.
    EventDelete {
        index: usize,
        event: EventInfo,
    },
    /// An event was added (via Paste). Undo removes it by index; redo
    /// reinserts the same event, so the pasted data is kept here rather
    /// than assumed to still be sitting in the caller's Vec.
    EventAdd {
        index: usize,
        event: EventInfo,
    },
}

#[derive(Default)]
pub struct UndoStack {
    undo_list: Vec<MapEditAction>,
    redo_list: Vec<MapEditAction>,
    max_history: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            undo_list: Vec::new(),
            redo_list: Vec::new(),
            max_history: 50,
        }
    }

    pub fn push(&mut self, action: MapEditAction) {
        self.undo_list.push(action);
        if self.undo_list.len() > self.max_history {
            self.undo_list.remove(0);
        }
        self.redo_list.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_list.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_list.is_empty()
    }

    pub fn undo(&mut self, lower: &mut [i32], upper: &mut [i32], events: &mut Vec<EventInfo>) -> bool {
        if let Some(action) = self.undo_list.pop() {
            match &action {
                MapEditAction::TileStroke { is_upper, changes } => {
                    let target = if *is_upper { &mut *upper } else { &mut *lower };
                    for change in changes {
                        if change.index < target.len() {
                            target[change.index] = change.old_val;
                        }
                    }
                }
                MapEditAction::EventMove { index, old_pos, .. } => {
                    if let Some(ev) = events.get_mut(*index) {
                        ev.x = old_pos.0;
                        ev.y = old_pos.1;
                    }
                }
                MapEditAction::EventDelete { index, event } => {
                    let idx = (*index).min(events.len());
                    events.insert(idx, event.clone());
                }
                MapEditAction::EventAdd { index, .. } => {
                    if *index < events.len() {
                        events.remove(*index);
                    }
                }
            }
            self.redo_list.push(action);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, lower: &mut [i32], upper: &mut [i32], events: &mut Vec<EventInfo>) -> bool {
        if let Some(action) = self.redo_list.pop() {
            match &action {
                MapEditAction::TileStroke { is_upper, changes } => {
                    let target = if *is_upper { &mut *upper } else { &mut *lower };
                    for change in changes {
                        if change.index < target.len() {
                            target[change.index] = change.new_val;
                        }
                    }
                }
                MapEditAction::EventMove { index, new_pos, .. } => {
                    if let Some(ev) = events.get_mut(*index) {
                        ev.x = new_pos.0;
                        ev.y = new_pos.1;
                    }
                }
                MapEditAction::EventDelete { index, .. } => {
                    if *index < events.len() {
                        events.remove(*index);
                    }
                }
                MapEditAction::EventAdd { index, event } => {
                    let idx = (*index).min(events.len());
                    events.insert(idx, event.clone());
                }
            }
            self.undo_list.push(action);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.undo_list.clear();
        self.redo_list.clear();
    }
}
