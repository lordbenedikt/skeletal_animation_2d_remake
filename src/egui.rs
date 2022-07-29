use crate::{skeleton::AddSkinEvent, *};
use bevy_egui::{
    egui::{self, Pos2, TextBuffer, Ui},
    EguiContext,
};
use interpolate::Function;
use std::{fs, ops::RangeInclusive};

pub struct State {
    pub interpolation_function: Function,
    pub skin_filename: String,
    pub skin_cols: u16,
    pub skin_rows: u16,
    pub animation: String,
    pub keyframe_length: i32,
    pub step: i32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            interpolation_function: Function::EaseInOut,
            skin_filename: String::from("filename"),
            animation: String::from("anim_0"),
            keyframe_length: 60,
            step: 0,
            skin_cols: 10,
            skin_rows: 10,
        }
    }
}
fn skin_menu(ui: &mut Ui, state: &mut State, mut add_skin_evw: EventWriter<AddSkinEvent>) {
    ui.label("Skin");
    ui.horizontal(|ui| {
        let widget = egui::ComboBox::from_id_source("skin")
            .selected_text(&state.skin_filename)
            .show_ui(ui, |ui| {
                let paths = fs::read_dir("./assets/img/").unwrap();
                for path in paths {
                    let filename = path
                        .unwrap()
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    let option =
                        ui.selectable_value(&mut state.skin_filename, filename.clone(), filename);
                    // if option.clicked() {
                    //     update_skin_evw.send(UpdateSkinEvent);
                    // }
                }
            });
        ui.label("cols");
        ui.add(
            egui::DragValue::new(&mut state.skin_cols)
                .speed(1)
                .clamp_range(1..=100),
        );
        ui.label("rows");
        ui.add(
            egui::DragValue::new(&mut state.skin_rows)
                .speed(1)
                .clamp_range(1..=100),
        );
    });
    ui.horizontal(|ui| {
        if ui.button("add skin").clicked() {
            if state.skin_filename != "filename" {
                add_skin_evw.send(skeleton::AddSkinEvent {
                    filename: format!("img/{}", state.skin_filename),
                    cols: state.skin_cols,
                    rows: state.skin_rows,
                    as_cloth: false,
                });
            }
        };
        if ui.button("add as cloth").clicked() {
            if state.skin_filename != "filename" {
                add_skin_evw.send(skeleton::AddSkinEvent {
                    filename: format!("img/{}", state.skin_filename),
                    cols: state.skin_cols,
                    rows: state.skin_rows,
                    as_cloth: true,
                });
            }
        };
    });
    ui.end_row();
}

fn animation_settings(ui: &mut Ui, state: &mut State) {
    ui.label("Animation Settings");
    ui.horizontal(|ui| {
        let widget = egui::ComboBox::from_id_source("easing_function")
            .selected_text(state.interpolation_function.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.interpolation_function,
                    Function::Linear,
                    "linear",
                );
                ui.selectable_value(
                    &mut state.interpolation_function,
                    Function::EaseIn,
                    Function::EaseIn.to_string(),
                );

                ui.selectable_value(
                    &mut state.interpolation_function,
                    Function::EaseOut,
                    Function::EaseOut.to_string(),
                );
                ui.selectable_value(
                    &mut state.interpolation_function,
                    Function::EaseInOut,
                    Function::EaseInOut.to_string(),
                );
                ui.selectable_value(
                    &mut state.interpolation_function,
                    Function::EaseOutElastic,
                    Function::EaseOutElastic.to_string(),
                );
            });
        ui.add(
            egui::DragValue::new(&mut state.keyframe_length)
                .speed(1)
                .clamp_range(1..=10000)
                .suffix("ms"),
        );
        ui.label("Length");
    });
}

pub fn ui_action(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    mut transform_state: ResMut<transform::State>,
    mut add_skin_evw: EventWriter<AddSkinEvent>,
) {
    let response = egui::Window::new("Menu")
        .show(egui_context.ctx_mut(), |ui| {
            animation_settings(ui, &mut state);
            skin_menu(ui, &mut state, add_skin_evw);
        })
        .unwrap()
        .response;

    if let Some(hover_pos) = egui_context.ctx_mut().pointer_hover_pos() {
        if response.rect.contains(hover_pos) {
            transform_state.action = transform::Action::Done;
        }
    }
}
