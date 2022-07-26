use crate::*;
use bevy_egui::{
    egui::{self, Pos2, Ui},
    EguiContext,
};
use interpolate::Function;
use std::fs;

pub struct State {
    pub interpolation_function: Function,
    pub skin_file_name: String,
    pub animation: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            interpolation_function: Function::EaseInOut,
            skin_file_name: String::from("Skin"),
            animation: String::from("anim_0"),
        }
    }
}
fn choose_skin(ui: &mut Ui, state: &mut State) -> bool {
    let widget = egui::ComboBox::from_label("Skin")
        .selected_text(&state.skin_file_name)
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
                    ui.selectable_value(&mut state.skin_file_name, filename.clone(), filename);
                // if option.clicked() {
                //     update_skin_evw.send(UpdateSkinEvent);
                // }
                if option.hovered() {
                    // state.taken = true;
                }
            }
        });
    if widget.response.hovered() {
        return true;
    }
    false
}

fn choose_function(ui: &mut Ui, state: &mut State) -> bool {
    let widget = egui::ComboBox::from_label("Easing Function")
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
    if widget.response.hovered() {
        return true;
    }
    false
}

pub fn ui_action(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    mut transform_state: ResMut<transform::State>,
    // mut update_skin_evw: EventWriter<UpdateSkinEvent>,
) {
    let response = egui::Window::new("Bone Settings")
        .show(egui_context.ctx_mut(), |ui| {
            ui.label("Bone Settings");
            choose_function(ui, &mut state);
            choose_skin(ui, &mut state);
        })
        .unwrap()
        .response;

    if let Some(hover_pos) = egui_context.ctx_mut().pointer_hover_pos() {
        if response.rect.contains(hover_pos) {
            transform_state.action = transform::Action::Done;
        } 
    }
}
