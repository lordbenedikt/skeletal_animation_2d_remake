use crate::{
    animation::{Animations, ShowKeyframeEvent},
    save_load::SaveEvent,
    *,
};
use bevy_egui::{
    egui::{
        self,
        plot::{MarkerShape, PlotPoint, Points},
        Color32, Ui,
    },
    EguiContext,
};
use interpolate::Function;
use inverse_kinematics::*;
use std::{f32::consts::PI, fs};

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

pub struct OpenWindows {
    pub is_open_animations: bool,
    pub is_open_skins: bool,
}
impl Default for OpenWindows {
    fn default() -> Self {
        OpenWindows {
            is_open_animations: false,
            is_open_skins: false,
        }
    }
}

pub struct State {
    pub ik_max_iterations: usize,
    pub loaded_standard_anim: String,
    pub interpolation_function: Function,
    pub keyframe_length: i32,
    pub skin_filename: String,
    pub skin_cols: u16,
    pub skin_rows: u16,
    pub step: i32,
    pub ik_depth: u8,
    pub ik_method: IKMethod,
    pub skin_is_bound: bool,
    pub skin_bound_status_is_valid: bool,
    pub edit_plot: usize,
    pub plots: Vec<PlotState>,
    pub ui_hover: bool,
    pub ui_drag: bool,
    pub new_animation_name: String,
    pub delaunay_triangle_size: f32,
    pub delaunay_borderline_width: f32,
    pub adjust_vertex_weights_mode: bool,
    pub brush_size: f32,
    pub save_filename: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            ik_max_iterations: 10,
            loaded_standard_anim: String::from("Choose..."),
            interpolation_function: Function::EaseInOut,
            keyframe_length: 400,
            edit_plot: 0,
            skin_filename: String::from("pooh.png"),
            step: 0,
            skin_cols: 10,
            skin_rows: 10,
            ik_depth: 2,
            ik_method: IKMethod::Jacobian,
            skin_is_bound: false,
            skin_bound_status_is_valid: false,
            plots: vec![PlotState::default()],
            ui_hover: false,
            ui_drag: false,
            new_animation_name: String::from(""),
            delaunay_triangle_size: 15.,
            delaunay_borderline_width: 3.,
            adjust_vertex_weights_mode: false,
            brush_size: 0.5,
            save_filename: String::from("my_animation"),
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(
            first_system
                .before(skin_menu)
                .before(animation_menu)
                .before(get_selection_stats)
                .before(panel),
        )
        .with_system(panel)
        .with_system(skin_menu)
        .with_system(animation_menu)
        .with_system(get_selection_stats)
}

fn skin_settings(ui: &mut Ui, state: &mut State, skin_state: &mut skin::State) {
    ui.horizontal(|ui| {
        if ui.button("toogle adjust weights mode").clicked() {
            state.adjust_vertex_weights_mode = !state.adjust_vertex_weights_mode;
        };
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

    ui.separator();

    ui.horizontal(|ui| {
        let widget = egui::ComboBox::from_id_source("skin")
            .selected_text(&state.skin_filename)
            .show_ui(ui, |ui| {
                let filenames: Vec<String>;

                // Webassembly
                #[cfg(target_arch = "wasm32")]
                {
                    filenames = skin::AVAILABLE_IMAGES
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
                skin_state.queued_skins.push(skin::AddSkinOrder::Grid {
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
                skin_state.queued_skins.push(skin::AddSkinOrder::Grid {
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
                skin_state.queued_skins.push(skin::AddSkinOrder::Grid {
                    path: format!("img/{}", state.skin_filename),
                    cols: state.skin_cols,
                    rows: state.skin_rows,
                    as_cloth: true,
                    cut_out: false,
                });
            }
        };
    });

    ui.separator();

    ui.label("Delaunay Triangulation");
    ui.horizontal(|ui| {
        if ui.button("add skin").clicked() {
            if state.skin_filename != "filename" {
                skin_state.queued_skins.push(skin::AddSkinOrder::Delaunay {
                    path: format!("img/{}", state.skin_filename),
                    triangle_size: state.delaunay_triangle_size,
                    borderline_width: state.delaunay_borderline_width,
                });
            }
        };
        ui.label("triangle size");
        ui.add(
            egui::DragValue::new(&mut state.delaunay_triangle_size)
                .speed(1)
                .clamp_range(1..=100),
        );
        ui.label("borderline width ");
        ui.add(
            egui::DragValue::new(&mut state.delaunay_borderline_width)
                .speed(0.5)
                .clamp_range(1..=30),
        );
    });
    ui.end_row();
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
    plot_index: usize,
    anim_state: &animation::State,
    animations: &mut Animations,
    mouse: &Input<MouseButton>,
    keys: &Input<KeyCode>,
    show_keyframe_evw: &mut EventWriter<animation::ShowKeyframeEvent>,
    q: &Query<&mut Transform>,
) {
    if plot_index >= state.plots.len() {
        return;
    }
    ui.horizontal(|ui| {
        // Choose Animation
        egui::ComboBox::from_id_source(format!("current_animation_{}", plot_index))
            .selected_text(&state.plots[plot_index].name)
            .show_ui(ui, |ui| {
                for animation_name in animations.map.keys() {
                    ui.selectable_value(
                        &mut state.plots[plot_index].name,
                        String::from(animation_name),
                        animation_name,
                    );
                }
            });
        // Remove Keyframe
        if ui.button("remove keyframe").clicked() {
            let opt_animation = animations.map.get_mut(&state.plots[plot_index].name);
            if let Some(animation) = opt_animation {
                if state.plots[plot_index].selected_keyframe_index == 0 {
                    for i in (1..animation.keyframes.len()).rev() {
                        animation.keyframes[i] -= animation.keyframes[1];
                    }
                }
                animation.remove_keyframe(state.plots[plot_index].selected_keyframe_index);
            }
        };
        // Remove Plot
        if state.plots.len() > 1 {
            if ui.button("remove plot").clicked() {
                state.plots.remove(plot_index);
                if state.edit_plot == state.plots.len() {
                    state.edit_plot = core::cmp::max(0, state.plots.len() - 1);
                }
                return;
            };
        }
        // Show edit label, if currently editing this animation
        if state.edit_plot == plot_index {
            ui.label("Edit");
        }
    });
    animation_plot(
        ui,
        state,
        plot_index,
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
    q_bones: &mut Query<(Entity, &Transformable, &mut bone::Bone)>,
    transform_state: &transform::State,
) {
    // LAYERS
    ui.label("LAYERS");
    ui.horizontal(|ui| {
        ui.label("blending style:");
        if ui.button(anim_state.blending_style.to_string()).clicked() {
            if anim_state.blending_style == animation::BlendingStyle::FourWayAdditive {
                anim_state.blending_style = animation::BlendingStyle::Layering;
            } else if anim_state.blending_style == animation::BlendingStyle::Layering {
                anim_state.blending_style = animation::BlendingStyle::FourWayAdditive;
            }
        };
    });
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

    ui.separator();

    // Inverse Kinematics
    ui.label("INVERSE KINEMATICS");
    ui.horizontal(|ui| {
        if ui.button(state.ik_method.to_string()).clicked() {
            state.ik_method = if state.ik_method == IKMethod::CCD {
                IKMethod::Jacobian
            } else {
                IKMethod::CCD
            };
        }
        ui.label("Depth: ");
        ui.add(
            egui::DragValue::new(&mut state.ik_depth)
                .speed(1)
                .clamp_range(1..=30),
        );
        ui.label("Max Iterations (global): ");
        ui.add(
            egui::DragValue::new(&mut state.ik_max_iterations)
                .speed(1)
                .clamp_range(1..=50),
        );
    });

    ui.separator();

    // Set Angle Constraints
    ui.label("ANGLE CONSTRAINTS (only CCD, saving not currently supported)");
    ui.horizontal(|ui| {
        if let Some(&first_selected_entity) = transform_state.selected_entities.iter().next() {
            if let Ok((_, _, mut bone)) = q_bones.get_mut(first_selected_entity) {
                if let Some(angle_constraint) = &mut bone.ik_angle_constraint {
                    ui.label("Start: ");
                    ui.add(
                        egui::DragValue::new(&mut angle_constraint.start)
                            .speed(0.1)
                            .custom_formatter(|n, _| format!("{:.1}°", n.to_degrees())),
                    );
                    ui.label("End: ");
                    ui.add(
                        egui::DragValue::new(&mut angle_constraint.end)
                            .speed(0.1)
                            .custom_formatter(|n, _| format!("{:.1}°", n.to_degrees())),
                    );
                    if ui.button("No Constraint").clicked() {
                        angle_constraint.start = 0.0;
                        angle_constraint.end = 0.0;
                    }
                    angle_constraint.start = (angle_constraint.start + 2. * PI) % (2. * PI);
                    angle_constraint.end = (angle_constraint.end + 2. * PI) % (2. * PI);
                    if angle_constraint.end < angle_constraint.start {
                        angle_constraint.end += 2. * PI;
                    }
                }
            }
        }
    });

    ui.separator();

    // General Animation Settings
    ui.horizontal(|ui| {
        ui.label("ANIMATION  ");
        ui.label(if anim_state.running {
            "(animation is playing)"
        } else {
            "(animation is paused)"
        });
    });
    ui.horizontal(|ui| {
        ui.label("default keyframe length");
        ui.add(
            egui::DragValue::new(&mut state.keyframe_length)
                .speed(1)
                .clamp_range(1..=10000)
                .suffix("ms"),
        );

        // Free selected bones removing them from the current animation layer
        if ui.button("Free Selected Bones").clicked() {
            let anim = animations
                .map
                .get_mut(&state.plots[state.edit_plot].name)
                .unwrap();
            for (entity, transformable, _) in q_bones.iter() {
                if transformable.is_selected {
                    anim.comp_animations.remove(&entity);
                }
            }
        }
    });

    ui.separator();

    // Choose Easing Function
    ui.horizontal(|ui| {
        ui.label("EASING FUNCTION");
        let function_combo_box =
            egui::ComboBox::from_id_source(format!("easing_function_{}", state.edit_plot))
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
                            if let Some(anim) =
                                animations.map.get_mut(&state.plots[state.edit_plot].name)
                            {
                                for (_, comp_anim) in anim.comp_animations.iter_mut() {
                                    for i in 0..comp_anim.transforms.len() {
                                        if i == state.plots[state.edit_plot].selected_keyframe_index
                                        {
                                            comp_anim.interpolation_functions[i] =
                                                state.interpolation_function;
                                        }
                                    }
                                }
                            }
                        };
                    }
                });
    });

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
    plot_index: usize,
    mouse: &Input<MouseButton>,
    keys: &Input<KeyCode>,
    animations: &mut animation::Animations,
    show_keyframe_evw: &mut EventWriter<animation::ShowKeyframeEvent>,
) {
    // if layer doesn't exist, return
    if plot_index >= state.plots.len() {
        return;
    }
    let response = egui::plot::Plot::new(format!("example_plot_{}", plot_index))
        .height(50.0)
        .center_y_axis(true)
        .allow_drag(!keys.pressed(KeyCode::LControl))
        .show_y(false)
        .data_aspect(1.0)
        .show(ui, |plot_ui| {
            if let Some(anim) = animations.map.get_mut(&state.plots[plot_index].name) {
                // Create values for keyframe markers
                let values_all: Vec<PlotPoint> = anim
                    .keyframes
                    .iter()
                    .map(|&kf| PlotPoint { x: kf, y: 0.0 })
                    .collect();
                let mut values_not_selected: Vec<PlotPoint> = vec![];
                let mut values_selected: Vec<PlotPoint> = vec![];
                for i in 0..values_all.len() {
                    let new_value = values_all[i];
                    if i == state.plots[plot_index].selected_keyframe_index {
                        values_selected.push(new_value);
                    } else {
                        values_not_selected.push(new_value);
                    }
                }

                let points = Points::new(
                    values_not_selected
                        .iter()
                        .map(|v| [v.x, v.y])
                        .collect::<Vec<[f64; 2]>>(),
                )
                .filled(true)
                .radius(5.0)
                .shape(MarkerShape::Diamond)
                .color(Color32::LIGHT_RED);
                let points_selected = Points::new(
                    values_selected
                        .iter()
                        .map(|v| [v.x, v.y])
                        .collect::<Vec<[f64; 2]>>(),
                )
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
                        state.plots[plot_index].selected_keyframe_index = hovered_keyframe;
                        // Show keyframe
                        show_keyframe_evw.send(ShowKeyframeEvent {
                            animation_name: state.plots[plot_index].name.clone(),
                            keyframe_index: state.plots[plot_index].selected_keyframe_index,
                        });
                        // Show interpolation function of current keyframe in ui
                        for (_, comp_anim) in anim.comp_animations.iter() {
                            let mut stop = false;
                            for i in 0..comp_anim.transforms.len() {
                                if i == state.plots[plot_index].selected_keyframe_index {
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
                        let move_amount = if state.plots[plot_index].selected_keyframe_index == 0 {
                            0.0
                        } else {
                            let current_x =
                                anim.keyframes[state.plots[plot_index].selected_keyframe_index];
                            let min_x =
                                anim.keyframes[state.plots[plot_index].selected_keyframe_index - 1];
                            f64::max(
                                plot_ui.pointer_coordinate_drag_delta().x as f64,
                                min_x - current_x,
                            )
                        };

                        // Move keyframe and all following keyframes by move_amount
                        for i in
                            state.plots[plot_index].selected_keyframe_index..anim.keyframes.len()
                        {
                            anim.keyframes[i] += move_amount;
                        }
                    }
                }
            }
        })
        .response;
    if response.clicked() {
        state.edit_plot = plot_index;
    };
}

fn get_closest_keyframe(pos: PlotPoint, values: Vec<PlotPoint>, max_dist: f64) -> Option<usize> {
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

pub fn panel(
    mut egui_context: ResMut<EguiContext>,
    mut open_windows: ResMut<OpenWindows>,
    mut state: ResMut<State>,
    mouse: Res<Input<MouseButton>>,
) {
    // Show Panel
    let response = egui::panel::TopBottomPanel::top("top panel")
        .show(egui_context.ctx_mut(), |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
            ui.add_space(7.);
            ui.horizontal(|ui| {
                ui.add_space(7.);
                if ui.button("Animations").clicked() {
                    open_windows.is_open_animations = !open_windows.is_open_animations;
                }
                ui.add_space(7.);
                if ui.button("Skins").clicked() {
                    open_windows.is_open_skins = !open_windows.is_open_skins;
                }
            });
            ui.add_space(7.);
        })
        .response;

    check_mouse_interaction(&mut egui_context, response, &mut state, &mouse);
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
    mut q_bones: Query<(Entity, &transform::Transformable, &mut bone::Bone)>,
    mut save_evw: EventWriter<save_load::SaveEvent>,
    mut open_windows: ResMut<OpenWindows>,
) {
    // Hide window when transforming
    if transform_state.action != transform::Action::None
        && transform_state.action != transform::Action::Done
    {
        return;
    }

    // // All other platforms
    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     filenames = fs::read_dir("./assets/img/")
    //         .unwrap()
    //         .map(|read_dir| {
    //             read_dir
    //                 .unwrap()
    //                 .path()
    //                 .file_name()
    //                 .unwrap()
    //                 .to_str()
    //                 .unwrap()
    //                 .to_string()
    //         })
    //         .collect();
    // }

    // Show Window
    let opt_response = egui::Window::new("Animations")
        .open(&mut open_windows.is_open_animations)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            #[cfg(target_arch = "wasm32")]
            {
                ui.text_edit_singleline(&mut state.save_filename);
                ui.horizontal(|ui| {
                    if ui.button("Save to local disc").clicked() {
                        save_evw.send(SaveEvent(state.save_filename.clone()));
                    }
                    if ui.button("Load locally saved file").clicked() {
                        #[link(wasm_import_module = "./load-animations.js")]
                        extern "C" {
                            fn uploadFileToLocalStorage();
                        }
                        unsafe {
                            uploadFileToLocalStorage();
                        }
                    }
                });

                ui.separator();
            }

            // WIP
            // #[cfg(not(target_arch = "wasm32"))]
            // {
            //     ui.label("Load standard animation: ");
            //     let choose_standard_animation = egui::ComboBox::from_id_source("standard_animation")
            //         .selected_text(&state.loaded_standard_anim)
            //         .show_ui(ui, |ui| {
            //             ui.selectable_value();
            //         });
            // }

            animations_all(
                ui,
                &mut state,
                &mut animations,
                &mut anim_state,
                &mouse,
                &keys,
                &mut show_keyframe_evw,
                &q,
                &mut q_bones,
                &transform_state,
            );
        });

    if let Some(inner) = opt_response {
        check_mouse_interaction(&mut egui_context, inner.response, &mut state, &mouse);
    }
}

pub fn skin_menu(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    transform_state: ResMut<transform::State>,
    mut skin_state: ResMut<skin::State>,
    mouse: Res<Input<MouseButton>>,
    mut open_windows: ResMut<OpenWindows>,
) {
    // Hide window when transforming
    if transform_state.action != transform::Action::None
        && transform_state.action != transform::Action::Done
    {
        return;
    }

    // Show Window
    let opt_response = egui::Window::new("Skins")
        .open(&mut open_windows.is_open_skins)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            skin_settings(ui, &mut state, &mut skin_state);
        });

    if let Some(inner) = opt_response {
        check_mouse_interaction(&mut egui_context, inner.response, &mut state, &mouse);
    }
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
