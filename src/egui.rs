use crate::{skin::AddSkinEvent, *};
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
    pub ccd_depth: u8,
    pub skin_is_bound: bool,
    pub skin_bound_status_is_valid: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            interpolation_function: Function::EaseInOut,
            skin_filename: String::from("filename"),
            animation: String::from("anim_0"),
            keyframe_length: 1500,
            step: 0,
            skin_cols: 10,
            skin_rows: 10,
            ccd_depth: 2,
            skin_is_bound: false,
            skin_bound_status_is_valid: false,
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(ui_action)
        .with_system(get_selection_values)
}

fn skin_menu(ui: &mut Ui, state: &mut State, mut add_skin_evw: EventWriter<AddSkinEvent>) {
    ui.label("Skin");
    ui.horizontal(|ui| {
        ui.label(if state.skin_bound_status_is_valid {
            state.skin_is_bound.to_string()
        } else {
            String::from("-")
        });
    });
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
                add_skin_evw.send(skin::AddSkinEvent {
                    filename: format!("img/{}", state.skin_filename),
                    cols: state.skin_cols,
                    rows: state.skin_rows,
                    as_cloth: false,
                });
            }
        };
        if ui.button("add as cloth").clicked() {
            if state.skin_filename != "filename" {
                add_skin_evw.send(skin::AddSkinEvent {
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
                    Function::Linear.to_string(),
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
                ui.selectable_value(
                    &mut state.interpolation_function,
                    Function::EaseInOutElastic,
                    Function::EaseInOutElastic.to_string(),
                );
                ui.selectable_value(
                    &mut state.interpolation_function,
                    Function::EaseInOutBack,
                    Function::EaseInOutBack.to_string(),
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

fn ccd_settings(ui: &mut Ui, state: &mut State) {
    ui.label("CCD Settings");
    ui.add(
        egui::DragValue::new(&mut state.ccd_depth)
            .speed(1)
            .clamp_range(1..=10),
    );
}

pub fn get_selection_values(
    mut state: ResMut<State>,
    transform_state: Res<transform::State>,
    skeleton: Res<skeleton::Skeleton>,
    q: Query<(Option<&skin::Skin>, Option<&bone::Bone>, Entity), With<Transformable>>,
) {
    state.skin_bound_status_is_valid = false;

    // Skip if no entities are selected
    if let Some(e) = transform_state.selected_entities.last() {
        if q.get(*e).unwrap().0.is_some() {
            state.skin_bound_status_is_valid = true;
            state.skin_is_bound = false;
            for mapping in skeleton.skin_mappings.iter() {
                if mapping.skin.unwrap() == *e {
                    state.skin_is_bound = true;
                    break;
                }
            }
        }
    }

    // let mut skin_selected = false;
    // state.skin_bound_status_is_valid = true;
    // for e in transform_state.selected_entities.clone() {
    //     // If entity is skin
    //     if q.get(e).unwrap().0.is_some() {
    //         // If selected skins have different states continue
    //         if !state.skin_bound_status_is_valid {
    //             continue;
    //         }

    // let mut is_bound = false;
    // for mapping in skeleton.skin_mappings.iter() {
    //     if mapping.skin.unwrap() == e {
    //         is_bound = true;
    //         break;
    //     }
    // }
    //         if skin_selected {
    //             if state.skin_is_bound != is_bound{
    //                 state.skin_bound_status_is_valid = false;
    //             }
    //         } else {
    //             state.skin_is_bound = is_bound;
    //             skin_selected = true;
    //         }
    //     }
    // }
}

fn animation_plot(ui: &mut egui::Ui)  {
    // use egui::plot::{Line, PlotPoints};
    // let n = 128;
    // let line_points: PlotPoints = (0..=n)
    //     .map(|i| {
    //         use std::f64::consts::TAU;
    //         let x = egui::remap(i as f64, 0.0..=n as f64, -TAU..=TAU);
    //         [x, x.sin()]
    //     })
    //     .collect();
    // let line = Line::new(line_points);
    // egui::plot::Plot::new("example_plot")
    //     .height(32.0)
    //     .data_aspect(1.0)
    //     .show(ui, |plot_ui| plot_ui.line(line))
    //     .response
}

pub fn ui_action(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    mut transform_state: ResMut<transform::State>,
    mut add_skin_evw: EventWriter<AddSkinEvent>,
    mouse: Res<Input<MouseButton>>,
) {
    let response = egui::Window::new("Menu")
        .show(egui_context.ctx_mut(), |ui| {
            animation_settings(ui, &mut state);
            skin_menu(ui, &mut state, add_skin_evw);
            ccd_settings(ui, &mut state);
            animation_plot(ui);
        })
        .unwrap()
        .response;

    if let Some(hover_pos) = egui_context.ctx_mut().pointer_hover_pos() {
        if response.rect.contains(hover_pos) && mouse.get_just_pressed().count() != 0 {
            transform_state.action = transform::Action::Done;
        }
    }
}
