use crate::{skin::AddSkinEvent, *};
use bevy_egui::{
    egui::{
        self,
        plot::{MarkerShape, Points, Value, Values},
        Color32, Pos2, TextBuffer, Ui,
    },
    EguiContext,
};
use interpolate::Function;
use std::{fs, ops::RangeInclusive};

pub struct AnimationState {
    pub name: String,
    pub new_name: String,
    pub selected_keyframe_index: usize,
    pub keyframe_length: i32,
}
impl Default for AnimationState {
    fn default() -> Self {
        Self {
            name: String::from("anim_0"),
            new_name: String::from(""),
            selected_keyframe_index: 0,
            keyframe_length: 400,
        }
    }
}

pub struct State {
    pub interpolation_function: Function,
    pub skin_filename: String,
    pub skin_cols: u16,
    pub skin_rows: u16,
    pub step: i32,
    pub ccd_depth: u8,
    pub skin_is_bound: bool,
    pub skin_bound_status_is_valid: bool,
    pub animation: AnimationState,
}
impl Default for State {
    fn default() -> Self {
        Self {
            interpolation_function: Function::EaseInOut,
            skin_filename: String::from("filename"),
            step: 0,
            skin_cols: 10,
            skin_rows: 10,
            ccd_depth: 2,
            skin_is_bound: false,
            skin_bound_status_is_valid: false,
            animation: AnimationState::default(),
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

fn animation_settings(ui: &mut Ui, state: &mut State, anim_state: &animation::State) {
    ui.label("Animation Settings");
    ui.horizontal(|ui| {
        let widget = egui::ComboBox::from_id_source("easing_function")
            .selected_text(state.interpolation_function.to_string())
            .show_ui(ui, |ui| {
                for function in Function::all() {
                    ui.selectable_value(
                        &mut state.interpolation_function,
                        function,
                        function.to_string(),
                    );
                }
            });
        ui.add(
            egui::DragValue::new(&mut state.animation.keyframe_length)
                .speed(1)
                .clamp_range(1..=10000)
                .suffix("ms"),
        );
        ui.label("Length");
        ui.label(if anim_state.running {
            "    Playing"
        } else {
            "    Paused"
        });
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

fn animation_menu(
    ui: &mut egui::Ui,
    state: &mut State,
    mouse: &Input<MouseButton>,
    animations: &mut animation::Animations,
    anim_state: &animation::State,
) {
    animation_settings(ui, state, anim_state);

    let widget = egui::ComboBox::from_id_source("current_animation")
        .selected_text(&state.animation.name)
        .show_ui(ui, |ui| {
            for animation_name in animations.map.keys() {
                ui.selectable_value(
                    &mut state.animation.name,
                    String::from(animation_name),
                    animation_name,
                );
            }
        });
    // ui.horizontal(|ui| {
    //     ui.text_edit_singleline(&mut state.animation.new_name);
    //     if ui.button("Create Animation").clicked() {
    //         animations.map.insert(state.animation.new_name.clone(), animation::Animation::default());
    //         state.animation.name = state.animation.new_name.clone();
    //     };
    // });
    if ui.button("remove keyframe").clicked() {
        let opt_animation = animations.map.get_mut(&state.animation.name);
        if let Some(animation) = opt_animation {
            if state.animation.selected_keyframe_index == 0 {
                for i in (1..animation.keyframes.len()).rev() {
                    animation.keyframes[i] -= animation.keyframes[1];
                }
            }
            animation.remove_keyframe(state.animation.selected_keyframe_index);
        }
    };

    animation_plot(ui, state, mouse, animations);
}

fn animation_plot(
    ui: &mut egui::Ui,
    state: &mut State,
    mouse: &Input<MouseButton>,
    animations: &mut animation::Animations,
) -> egui::Response {
    use egui::plot::{Line, Points, Values};

    egui::plot::Plot::new("example_plot")
        .height(50.0)
        .center_y_axis(true)
        .allow_drag(false)
        .show_y(false)
        .data_aspect(1.0)
        .show(ui, |plot_ui| {
            if let Some(anim) = animations.map.get_mut(&state.animation.name) {
                // Create values for keyframe markers
                let values_all: Vec<Value> = anim
                    .keyframes
                    .iter()
                    .map(|&kf| Value { x: kf, y: 0.0 })
                    .collect();
                let mut values_not_selected: Vec<Value> = vec![];
                let mut values_selected: Vec<Value> = vec![];
                for i in 0..values_all.len() {
                    let new_value = values_all[i];
                    if i == state.animation.selected_keyframe_index {
                        values_selected.push(new_value);
                    } else {
                        values_not_selected.push(new_value);
                    }
                }

                let points = Points::new(Values::from_values(values_not_selected))
                    .filled(true)
                    .radius(5.0)
                    .shape(MarkerShape::Diamond)
                    .color(Color32::LIGHT_RED);
                let points_selected = Points::new(Values::from_values(values_selected))
                    .filled(true)
                    .radius(5.0)
                    .shape(MarkerShape::Diamond)
                    .color(Color32::LIGHT_YELLOW);
                plot_ui.points(points);
                plot_ui.points(points_selected);

                // Get hovered keyframe index
                let mut opt_hovered_keyframe: Option<usize> = None;
                if plot_ui.plot_hovered() {
                    let w = plot_ui.plot_bounds().width();
                    let opt_keyframe_ind = get_closest_keyframe(
                        plot_ui.pointer_coordinate().unwrap(),
                        values_all,
                        0.0475 * w,
                    );
                    if let Some(keyframe_ind) = opt_keyframe_ind {
                        opt_hovered_keyframe = Some(keyframe_ind);
                    }
                }

                if let Some(hovered_keyframe) = opt_hovered_keyframe {
                    // Select keyframe
                    if mouse.just_pressed(MouseButton::Left) {
                        state.animation.selected_keyframe_index = hovered_keyframe;
                    }
                    // Move keyframe
                    if plot_ui.pointer_coordinate_drag_delta().x != 0.0 {
                        // Determine amount of disposition
                        let move_amount = if state.animation.selected_keyframe_index == 0 {
                            0.0
                        } else {
                            let current_x = anim.keyframes[state.animation.selected_keyframe_index];
                            let min_x = anim.keyframes[state.animation.selected_keyframe_index - 1];
                            f64::max(
                                plot_ui.pointer_coordinate_drag_delta().x as f64,
                                min_x - current_x,
                            )
                        };

                        // Move keyframe and all following keyframes by move_amount
                        for i in state.animation.selected_keyframe_index..anim.keyframes.len() {
                            anim.keyframes[i] += move_amount;
                        }
                    }
                }
            }
        })
        .response
}

fn get_closest_keyframe(pos: Value, values: Vec<Value>, max_dist: f64) -> Option<usize> {
    let mut res = None;
    let mut shortest_dist = max_dist + 1.;
    for i in (0..values.len()).rev() {
        let value = values[i];
        let distance = ((pos.x - value.x).powi(2) + (pos.y - value.y).powi(2)).sqrt();
        if distance <= max_dist && distance < shortest_dist {
            res = Some(i);
            shortest_dist = distance;
        }
    }
    res
}

pub fn ui_action(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    mut transform_state: ResMut<transform::State>,
    mut add_skin_evw: EventWriter<AddSkinEvent>,
    mut animations: ResMut<animation::Animations>,
    mouse: Res<Input<MouseButton>>,
    anim_state: Res<animation::State>,
) {
    let response = egui::Window::new("Menu")
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            skin_menu(ui, &mut state, add_skin_evw);
            ccd_settings(ui, &mut state);
            animation_menu(ui, &mut state, &mouse, &mut animations, &anim_state);
        })
        .unwrap()
        .response;

    if let Some(hover_pos) = egui_context.ctx_mut().pointer_hover_pos() {
        if response.rect.contains(hover_pos) && mouse.get_just_pressed().count() != 0 {
            transform_state.action = transform::Action::Done;
        }
    }
}
