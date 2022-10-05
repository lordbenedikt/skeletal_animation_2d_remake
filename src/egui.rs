use crate::{
    animation::{Animations, ShowKeyframeEvent},
    skin::{AddSkinEvent, AVAILABLE_IMAGES},
    *,
};
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

pub struct PlotState {
    pub name: String,
    pub selected_keyframe_index: usize,
}
impl Default for PlotState {
    fn default() -> Self {
        Self {
            name: String::from("anim_0"),
            selected_keyframe_index: 0,
        }
    }
}

pub struct State {
    pub interpolation_function: Function,
    pub keyframe_length: i32,
    pub skin_filename: String,
    pub skin_cols: u16,
    pub skin_rows: u16,
    pub step: i32,
    pub ccd_depth: u8,
    pub skin_is_bound: bool,
    pub skin_bound_status_is_valid: bool,
    pub edit_plot: usize,
    pub plots: Vec<PlotState>,
    pub ui_hover: bool,
    pub ui_drag: bool,
    pub new_animation_name: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            interpolation_function: Function::EaseInOut,
            keyframe_length: 400,
            edit_plot: 0,
            skin_filename: String::from("filename"),
            step: 0,
            skin_cols: 10,
            skin_rows: 10,
            ccd_depth: 2,
            skin_is_bound: false,
            skin_bound_status_is_valid: false,
            plots: vec![PlotState::default()],
            ui_hover: false,
            ui_drag: false,
            new_animation_name: String::from(""),
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(
            first_system
                .before(skin_menu)
                .before(animation_menu)
                .before(get_selection_stats),
        )
        .with_system(skin_menu)
        .with_system(animation_menu)
        .with_system(get_selection_stats)
}

fn skin_settings(ui: &mut Ui, state: &mut State, skin_state: &mut skin::State) {
    ui.horizontal(|ui| {
        ui.label(if state.skin_bound_status_is_valid {
            if state.skin_is_bound {
                String::from("skin is bound")
            } else {
                String::from("skin is loose")
            }
        } else {
            String::from("-")
        });
    });
    ui.horizontal(|ui| {
        let widget = egui::ComboBox::from_id_source("skin")
            .selected_text(&state.skin_filename)
            .show_ui(ui, |ui| {
                let filenames: Vec<String>;

                // Webassembly
                #[cfg(target_arch = "wasm32")]
                {
                    filenames = AVAILABLE_IMAGES
                        .iter()
                        .map(|&str| String::from(str))
                        .collect();
                }

                // All other platforms
                #[cfg(not(target_arch = "wasm32"))]
                {
                    filenames = fs::read_dir("./assets/img/")
                        .unwrap()
                        .map(|read_dir| {
                            read_dir
                                .unwrap()
                                .path()
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string()
                        })
                        .collect();
                }

                for filename in filenames {
                    let option =
                        ui.selectable_value(&mut state.skin_filename, filename.clone(), &filename);
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
                skin_state.queued_skins.push(skin::AddSkinEvent {
                    path: format!("img/{}", state.skin_filename),
                    cols: state.skin_cols,
                    rows: state.skin_rows,
                    as_cloth: false,
                    cut_out: false,
                });
            }
        };
        if ui.button("add skin cut out").clicked() {
            if state.skin_filename != "filename" {
                skin_state.queued_skins.push(skin::AddSkinEvent {
                    path: format!("img/{}", state.skin_filename),
                    cols: state.skin_cols,
                    rows: state.skin_rows,
                    as_cloth: false,
                    cut_out: true,
                });
            }
        };
        if ui.button("add as cloth").clicked() {
            if state.skin_filename != "filename" {
                skin_state.queued_skins.push(skin::AddSkinEvent {
                    path: format!("img/{}", state.skin_filename),
                    cols: state.skin_cols,
                    rows: state.skin_rows,
                    as_cloth: true,
                    cut_out: false,
                });
            }
        };
    });
    ui.end_row();
}

fn animation_settings_grid(ui: &mut Ui, state: &mut State, anim_state: &mut animation::State) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            egui::Grid::new("ccd_and_keyframe_length").show(ui, |ui| {
                ui.label("default ccd Depth");
                ui.add(
                    egui::DragValue::new(&mut state.ccd_depth)
                        .speed(1)
                        .clamp_range(1..=10),
                );
                ui.end_row();

                ui.label("default keyframe length");
                ui.add(
                    egui::DragValue::new(&mut state.keyframe_length)
                        .speed(1)
                        .clamp_range(1..=10000)
                        .suffix("ms"),
                );
                ui.end_row();
            });
        });
        ui.vertical(|ui| {
            egui::Grid::new("is_playing_and_blending_style").show(ui, |ui| {
                ui.label("blending style:");
                if ui.button(anim_state.blending_style.to_string()).clicked() {
                    if anim_state.blending_style == animation::BlendingStyle::FourWayAdditive {
                        anim_state.blending_style = animation::BlendingStyle::Layering;
                    } else if anim_state.blending_style == animation::BlendingStyle::Layering {
                        anim_state.blending_style = animation::BlendingStyle::FourWayAdditive;
                    }
                };
                ui.end_row();

                ui.label("animation:");
                ui.label(if anim_state.running {
                    "is playing"
                } else {
                    "is paused"
                });
                ui.end_row();
            });
        });
    });
}

pub fn get_selection_stats(
    mut state: ResMut<State>,
    transform_state: Res<transform::State>,
    skeleton: Res<skeleton::Skeleton>,
    q: Query<(Option<&skin::Skin>, Option<&bone::Bone>, Entity), With<Transformable>>,
) {
    state.skin_bound_status_is_valid = false;

    // If at least one entity is selected
    if let Some(e) = transform_state.selected_entities.iter().next() {
        // If entity exists
        if let Ok((opt_skin, _, _)) = q.get(*e) {
            // If entity has Skin component
            if opt_skin.is_some() {
                state.skin_bound_status_is_valid = true;
                state.skin_is_bound = false;
                for mapping in skeleton.skin_mappings.iter() {
                    if mapping.skin.is_none() {
                        continue;
                    }
                    if mapping.skin.unwrap() == *e {
                        state.skin_is_bound = true;
                        break;
                    }
                }
            }
        }
    }
}

fn layer_label(ui: &mut Ui, dir: usize, anim_state: &animation::State) {
    if anim_state.blending_style == animation::BlendingStyle::Layering {
        ui.label(format!("{}", dir + 1));
    } else if anim_state.blending_style == animation::BlendingStyle::FourWayAdditive && dir < 4 {
        ui.label(if dir == 0 {
            "up   "
        } else if dir == 1 {
            "down"
        } else if dir == 2 {
            "left "
        } else {
            "right"
        });
    }
}

fn animation_single(
    ui: &mut Ui,
    state: &mut State,
    layer_index: usize,
    anim_state: &animation::State,
    animations: &mut Animations,
    mouse: &Input<MouseButton>,
    keys: &Input<KeyCode>,
    show_keyframe_evw: &mut EventWriter<animation::ShowKeyframeEvent>,
    q: &Query<&mut Transform>,
) {
    if layer_index >= state.plots.len() {
        return;
    }
    ui.horizontal(|ui| {
        // Choose Animation
        egui::ComboBox::from_id_source(format!("current_animation_{}", layer_index))
            .selected_text(&state.plots[layer_index].name)
            .show_ui(ui, |ui| {
                for animation_name in animations.map.keys() {
                    ui.selectable_value(
                        &mut state.plots[layer_index].name,
                        String::from(animation_name),
                        animation_name,
                    );
                }
            });
        // Choose Easing Function
        let function_combo_box =
            egui::ComboBox::from_id_source(format!("easing_function_{}", layer_index))
                .selected_text(state.interpolation_function.to_string())
                .show_ui(ui, |ui| {
                    for function in Function::all() {
                        if ui
                            .selectable_value(
                                &mut state.interpolation_function,
                                function,
                                function.to_string(),
                            )
                            .changed()
                        {
                            // Easing Function was changed
                            for (_, anim) in animations.map.iter_mut() {
                                for (_, comp_anim) in anim.comp_animations.iter_mut() {
                                    for i in 0..comp_anim.transforms.len() {
                                        if i == state.plots[layer_index].selected_keyframe_index {
                                            comp_anim.interpolation_functions[i] =
                                                state.interpolation_function;
                                        }
                                    }
                                }
                            }
                        };
                    }
                });
        // Remove Keyframe
        if ui.button("remove keyframe").clicked() {
            let opt_animation = animations.map.get_mut(&state.plots[layer_index].name);
            if let Some(animation) = opt_animation {
                if state.plots[layer_index].selected_keyframe_index == 0 {
                    for i in (1..animation.keyframes.len()).rev() {
                        animation.keyframes[i] -= animation.keyframes[1];
                    }
                }
                animation.remove_keyframe(state.plots[layer_index].selected_keyframe_index);
            }
        };
        // Remove Plot
        if state.plots.len() > 1 {
            if ui.button("remove plot").clicked() {
                state.plots.remove(layer_index);
                if state.edit_plot == state.plots.len() {
                    state.edit_plot = core::cmp::max(0, state.plots.len() - 1);
                }
                return;
            };
        }
        // Show edit label, if currently editing this animation
        if state.edit_plot == layer_index {
            ui.label("Edit");
        }
    });
    animation_plot(
        ui,
        state,
        layer_index,
        mouse,
        keys,
        animations,
        show_keyframe_evw,
    );
}

fn animations_all(
    ui: &mut egui::Ui,
    state: &mut State,
    animations: &mut animation::Animations,
    anim_state: &mut animation::State,
    mouse: &Input<MouseButton>,
    keys: &Input<KeyCode>,
    show_keyframe_evw: &mut EventWriter<animation::ShowKeyframeEvent>,
    q: &Query<&mut Transform>,
    q_bones: &Query<(Entity, &Transformable), With<bone::Bone>>,
) {
    // LAYERS
    ui.label("Layers");
    let mut current_layer = 0;
    for row in 0..((anim_state.layers.len() + 2) / 2) {
        ui.horizontal(|ui| {
            for col in 0..2 {
                if current_layer < anim_state.layers.len() {
                    // Show ComboBox to choose animation for current layer
                    layer_label(ui, current_layer, anim_state);
                    egui::ComboBox::from_id_source(format!("layer_{}", current_layer))
                        .selected_text(&anim_state.layers[current_layer])
                        .show_ui(ui, |ui| {
                            for animation_name in animations.map.keys() {
                                ui.selectable_value(
                                    &mut anim_state.layers[current_layer],
                                    String::from(animation_name),
                                    animation_name,
                                );
                            }
                        });
                    if ui
                        .add(egui::Button::new("✖").fill(Color32::from_black_alpha(0)))
                        .clicked()
                    {
                        anim_state.layers.remove(current_layer);
                    }
                } else if current_layer == anim_state.layers.len()
                    && !(anim_state.blending_style == animation::BlendingStyle::FourWayAdditive
                        && current_layer >= 4)
                {
                    // Show ComboBox to choose animation for a new layer
                    layer_label(ui, current_layer, anim_state);
                    egui::ComboBox::from_id_source(format!("layer_{}", anim_state.layers.len()))
                        .selected_text("add layer")
                        .show_ui(ui, |ui| {
                            for animation_name in animations.map.keys() {
                                let mut new_layer = String::new();
                                if ui
                                    .selectable_value(
                                        &mut new_layer,
                                        String::from(animation_name),
                                        animation_name,
                                    )
                                    .clicked()
                                {
                                    anim_state.layers.push(new_layer);
                                };
                            }
                        });
                }
                current_layer += 1;
            }
        });
    }

    // General Animation Settings
    ui.label("Animations                        ");
    animation_settings_grid(ui, state, anim_state);
    // Free selected bones removing them from the current animation layer
    if ui.button("Free Bones").clicked() {
        let anim = animations
            .map
            .get_mut(&state.plots[state.edit_plot].name)
            .unwrap();
        for (entity, transformable) in q_bones.iter() {
            if transformable.is_selected {
                anim.comp_animations.remove(&entity);
            }
        }
    }

    ui.separator();

    // PLOTS
    for i in 0..state.plots.len() {
        animation_single(
            ui,
            state,
            i,
            anim_state,
            animations,
            mouse,
            keys,
            show_keyframe_evw,
            q,
        );
    }

    // Add Animation
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.new_animation_name);
        if ui.button("Add Animation").clicked() && !state.new_animation_name.is_empty() {
            animations.map.insert(
                state.new_animation_name.clone(),
                animation::Animation::default(),
            );
        };
        if ui.button("Add Plot").clicked() {
            state.plots.push(PlotState {
                name: String::new(),
                selected_keyframe_index: 0,
            });
        };
    });
}

fn animation_plot(
    ui: &mut egui::Ui,
    state: &mut State,
    layer_index: usize,
    mouse: &Input<MouseButton>,
    keys: &Input<KeyCode>,
    animations: &mut animation::Animations,
    show_keyframe_evw: &mut EventWriter<animation::ShowKeyframeEvent>,
) {
    // if layer doesn't exist, return
    if layer_index >= state.plots.len() {
        return;
    }
    let response = egui::plot::Plot::new(format!("example_plot_{}", layer_index))
        .height(50.0)
        .center_y_axis(true)
        .allow_drag(!keys.pressed(KeyCode::LControl))
        .show_y(false)
        .data_aspect(1.0)
        .show(ui, |plot_ui| {
            if let Some(anim) = animations.map.get_mut(&state.plots[layer_index].name) {
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
                    if i == state.plots[layer_index].selected_keyframe_index {
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
                        state.plots[layer_index].selected_keyframe_index = hovered_keyframe;
                        // Show keyframe
                        show_keyframe_evw.send(ShowKeyframeEvent {
                            animation_name: state.plots[layer_index].name.clone(),
                            keyframe_index: state.plots[layer_index].selected_keyframe_index,
                        });
                        // Show interpolation function of current keyframe in ui
                        for (_, comp_anim) in anim.comp_animations.iter() {
                            let mut stop = false;
                            for i in 0..comp_anim.transforms.len() {
                                if i == state.plots[layer_index].selected_keyframe_index {
                                    state.interpolation_function =
                                        comp_anim.interpolation_functions[i];
                                    stop = true;
                                    break;
                                }
                            }
                            if stop {
                                break;
                            }
                        }
                    }
                    // Move keyframe
                    if keys.pressed(KeyCode::LControl)
                        && plot_ui.pointer_coordinate_drag_delta().x != 0.0
                    {
                        // Determine amount of disposition
                        let move_amount = if state.plots[layer_index].selected_keyframe_index == 0 {
                            0.0
                        } else {
                            let current_x =
                                anim.keyframes[state.plots[layer_index].selected_keyframe_index];
                            let min_x = anim.keyframes
                                [state.plots[layer_index].selected_keyframe_index - 1];
                            f64::max(
                                plot_ui.pointer_coordinate_drag_delta().x as f64,
                                min_x - current_x,
                            )
                        };

                        // Move keyframe and all following keyframes by move_amount
                        for i in
                            state.plots[layer_index].selected_keyframe_index..anim.keyframes.len()
                        {
                            anim.keyframes[i] += move_amount;
                        }
                    }
                }
            }
        })
        .response;
    if response.clicked() {
        state.edit_plot = layer_index;
    };
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

pub fn animation_menu(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    mut transform_state: ResMut<transform::State>,
    mut skin_state: ResMut<skin::State>,
    mut show_keyframe_evw: EventWriter<animation::ShowKeyframeEvent>,
    mut animations: ResMut<animation::Animations>,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut anim_state: ResMut<animation::State>,
    mut q: Query<&mut Transform>,
    mut q_bones: Query<(Entity, &transform::Transformable), With<bone::Bone>>,
) {
    // Hide window when transforming
    if transform_state.action != transform::Action::None
        && transform_state.action != transform::Action::Done
    {
        return;
    }

    // Show Window
    let response = egui::Window::new("Animations")
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            animations_all(
                ui,
                &mut state,
                &mut animations,
                &mut anim_state,
                &mouse,
                &keys,
                &mut show_keyframe_evw,
                &q,
                &q_bones,
            );
        })
        .unwrap()
        .response;

    check_mouse_interaction(&mut egui_context, response, &mut state, &mouse);
}

pub fn skin_menu(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    transform_state: ResMut<transform::State>,
    mut skin_state: ResMut<skin::State>,
    mouse: Res<Input<MouseButton>>,
) {
    // Hide window when transforming
    if transform_state.action != transform::Action::None
        && transform_state.action != transform::Action::Done
    {
        return;
    }

    // Show Window
    let response = egui::Window::new("Skins")
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            skin_settings(ui, &mut state, &mut skin_state);
        })
        .unwrap()
        .response;

    check_mouse_interaction(&mut egui_context, response, &mut state, &mouse);
}

fn check_mouse_interaction(
    egui_context: &mut EguiContext,
    response: egui::Response,
    state: &mut State,
    mouse: &Input<MouseButton>,
) {
    // Check whether mouse is hovering window
    if let Some(hover_pos) = egui_context.ctx_mut().pointer_hover_pos() {
        if response.rect.contains(hover_pos) {
            state.ui_hover = true;
            if mouse.just_pressed(MouseButton::Left) {
                state.ui_drag = true;
            }
        } else {
            state.ui_hover |= false;
            if mouse.just_pressed(MouseButton::Left) {
                state.ui_drag |= false;
            }
        }
    }
}

fn first_system(mut state: ResMut<State>) {
    state.ui_hover = false;
    state.ui_drag = false;
}
