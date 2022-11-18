use bevy::utils::HashMap;

use crate::*;

#[derive(Resource, Default)]
pub struct UiElements(HashMap<String, Entity>);

pub fn system_set() -> SystemSet {
    SystemSet::new()
        // .with_system(update_debug_mouse_position_text)
        .with_system(manual_button)
}

pub fn spawn_ui_elements(
    mut commands: Commands,
    mut ui_elements: ResMut<UiElements>,
    asset_server: Res<AssetServer>,
) {
    // // Spawn Debug Mouse Position TextBundle
    // ui_elements.0.insert(
    //     String::from("debug_mouse_position"),
    //     commands
    //         .spawn(TextBundle {
    //             text: Text::from_section(
    //                 String::from("Position"),
    //                 TextStyle {
    //                     font: asset_server.load("fonts/SpaceMono-Regular.ttf"),
    //                     font_size: 30.0,
    //                     color: Color::BLACK,
    //                 },
    //             ),
    //             ..Default::default()
    //         })
    //         .id(),
    // );

    // Spawn Manual Button
    ui_elements.0.insert(
        String::from("manual_button"),
        commands
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(150.0), Val::Px(45.0)),
                    position: UiRect {
                        left: Val::Px(0.),
                        top: Val::Px(0.),
                        ..Default::default()
                    },
                    margin: UiRect {bottom: Val::Px(10.0), right: Val::Px(10.0), left: Val::Auto, top: Val::Auto },
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: (*COLOR_BLACK.set_a(0.3)).into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Manual",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ));
            })
            .id(),
    );
}

pub fn manual_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    ui_elements: Res<UiElements>,
) {
    if let Ok((interaction, mut color)) =
        interaction_query.get_mut(*ui_elements.0.get("manual_button").unwrap())
    {

        match *interaction {
            Interaction::Clicked => {
                // open::that("https://github.com/lordbenedikt/skeletal_animation_2d_remake");
                web_sys::window().unwrap().open_with_url("https://github.com/lordbenedikt/skeletal_animation_2d_remake");
                *color = (*COLOR_LIGHTER_GRAY.clone().set_a(0.3)).into();
            }
            Interaction::Hovered => {
                *color = (*COLOR_LIGHT_GRAY.clone().set_a(0.3)).into();
            }
            Interaction::None => {
                *color = (*COLOR_BLACK.clone().set_a(0.3)).into();
            }
        }
    }
}

pub fn update_debug_mouse_position_text(
    mut q: Query<&mut Text>,
    ui_elements: Res<UiElements>,
    cursor_pos: Res<CursorPos>,
) {
    let mut text = q
        .get_mut(*ui_elements.0.get("debug_mouse_position").unwrap())
        .unwrap();
    text.sections[0].value = format!("position: {}", cursor_pos.0);
}
