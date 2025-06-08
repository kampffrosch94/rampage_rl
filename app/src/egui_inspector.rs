use egui::emath;
use quicksilver::reflections::*;
use std::sync::{Mutex, OnceLock};

// we use this for grid ui ids
fn counter() -> &'static Mutex<usize> {
    static COUNTER: OnceLock<Mutex<usize>> = OnceLock::new();
    COUNTER.get_or_init(|| Mutex::new(0))
}

pub fn next_id() -> usize {
    let mut guard = counter().lock().unwrap();
    *guard += 1;
    *guard
}

pub fn reset_id() {
    let mut guard = counter().lock().unwrap();
    *guard = 0;
}

fn draw_reflection(ui: &mut egui::Ui, r: &mut StructReflection) {
    ui.heading(r.name);
    egui::Grid::new(next_id()).min_col_width(50.).num_columns(2).striped(true).show(
        ui,
        |ui| {
            for field in &mut r.fields {
                ui.label(field.name);
                draw_value(ui, &mut field.value);
                ui.end_row();
            }
        },
    );
}

fn draw_reflection_ref(ui: &mut egui::Ui, r: &StructReflection) {
    ui.heading(r.name);
    egui::Grid::new(next_id()).min_col_width(50.).num_columns(2).striped(true).show(
        ui,
        |ui| {
            for field in &r.fields {
                ui.label(field.name);
                draw_value_ref(ui, &field.value);
                ui.end_row();
            }
        },
    );
}

fn draw_numeric<Num: emath::Numeric>(ui: &mut egui::Ui, value: &mut RefOrMut<Num>) {
    match value {
        RefOrMut::Ref(_val) => {
            todo!();
        }
        RefOrMut::Mut(val) => {
            ui.add(egui::DragValue::new(*val));
        }
    }
}

pub fn draw_value(ui: &mut egui::Ui, value: &mut ValueReflection) {
    match value {
        ValueReflection::I32(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::U32(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::F32(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::I64(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::U64(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::F64(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::ISize(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::USize(it) => {
            draw_numeric(ui, it);
        }
        ValueReflection::Bool(it) => {
            ui.checkbox(it, "");
        }
        ValueReflection::String(s) => {
            ui.text_edit_singleline(&mut **s);
        }
        ValueReflection::Struct(s) => {
            ui.vertical(|ui| {
                draw_reflection(ui, s);
            });
        }
        ValueReflection::Vec(vec) => {
            ui.vertical(|ui| {
                let len = vec.len();
                for i in 0..len {
                    draw_value(ui, &mut vec.get(i));
                }
            });
        }
        ValueReflection::HashMap(hmreflection) => {
            egui::Grid::new(next_id()).min_col_width(50.).num_columns(2).striped(true).show(
                ui,
                |ui| {
                    ui.label("Key");
                    ui.label("Value");
                    ui.end_row();
                    for mut element in hmreflection.get_elements() {
                        let [key, value] = element.fields.get_disjoint_mut([0, 1]).unwrap();
                        draw_value_ref(ui, &key.value);
                        draw_value(ui, &mut value.value);
                        ui.end_row();
                    }
                },
            );
        }
        ValueReflection::CEnum(e) => {
            let name = e.variants.iter().find(|it| it.0 == *e.val).unwrap().1;
            egui::ComboBox::from_id_salt(next_id()).selected_text(format!("{name}")).show_ui(
                ui,
                |ui| {
                    for (i, name) in e.variants {
                        ui.selectable_value(&mut *e.val, *i, *name);
                    }
                },
            );
        }
    }
}

fn draw_value_ref(ui: &mut egui::Ui, value: &ValueReflection) {
    match value {
        ValueReflection::I32(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::U32(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::F32(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::I64(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::U64(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::F64(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::ISize(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::USize(it) => {
            draw_numeric_ref(ui, it);
        }
        ValueReflection::Bool(_it) => {
            todo!();
        }
        ValueReflection::String(s) => {
            ui.label(&**s);
        }
        ValueReflection::Struct(s) => {
            ui.vertical(|ui| {
                draw_reflection_ref(ui, s);
            });
        }
        ValueReflection::Vec(vec) => {
            ui.vertical(|ui| {
                let len = vec.len();
                for i in 0..len {
                    draw_value_ref(ui, &vec.get_ref(i));
                }
            });
        }
        ValueReflection::HashMap(hmreflection) => {
            egui::Grid::new(next_id()).min_col_width(50.).num_columns(2).striped(true).show(
                ui,
                |ui| {
                    for element in hmreflection.get_elements_ref() {
                        let key = &element.fields[0].value;
                        let value = &element.fields[1].value;
                        draw_value_ref(ui, key);
                        draw_value_ref(ui, value);
                        ui.end_row();
                    }
                },
            );
        }
        ValueReflection::CEnum(e) => {
            let name = e.variants.iter().find(|it| it.0 == *e.val).unwrap().1;
            ui.label(&format!("{}", name));
        }
    }
}

fn draw_numeric_ref<Num: emath::Numeric + std::fmt::Display>(
    ui: &mut egui::Ui,
    value: &RefOrMut<Num>,
) {
    match value {
        RefOrMut::Ref(val) => {
            ui.label(&format!("{}", *val));
        }
        RefOrMut::Mut(_val) => {
            todo!();
        }
    }
}
