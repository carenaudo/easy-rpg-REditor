use eframe::egui;
use crate::lcf_bridge::AttributeInfo;

pub fn show_attribute_form(ui: &mut egui::Ui, attr: &mut AttributeInfo, dirty: &mut bool) {
    ui.heading(format!("{:04}: {}", attr.id, attr.name));
    ui.separator();

    egui::Grid::new("attr_general_grid")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Name:");
            let name_edit = ui.text_edit_singleline(&mut attr.name);
            if name_edit.changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("Type:");
            ui.label(&attr.attribute_type);
            ui.end_row();

            ui.label("A Rate (%):");
            let a_edit = ui.add(egui::DragValue::new(&mut attr.a_rate).range(0..=1000));
            if a_edit.changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("B Rate (%):");
            let b_edit = ui.add(egui::DragValue::new(&mut attr.b_rate).range(0..=1000));
            if b_edit.changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("C Rate (%):");
            let c_edit = ui.add(egui::DragValue::new(&mut attr.c_rate).range(0..=1000));
            if c_edit.changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("D Rate (%):");
            let d_edit = ui.add(egui::DragValue::new(&mut attr.d_rate).range(0..=1000));
            if d_edit.changed() {
                *dirty = true;
            }
            ui.end_row();

            ui.label("E Rate (%):");
            let e_edit = ui.add(egui::DragValue::new(&mut attr.e_rate).range(0..=1000));
            if e_edit.changed() {
                *dirty = true;
            }
            ui.end_row();
        });
}
