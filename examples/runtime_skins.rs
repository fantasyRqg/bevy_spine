use bevy::prelude::*;
use rusty_spine::Skin;
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBundle, SpinePlugin, SpineReadyEvent, SpineSet,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpinePlugin))
        .add_systems(Startup, (setup, ui_setup))
        .add_systems(Update, on_spawn.in_set(SpineSet::OnReady))
        .add_systems(Update, (button_system, change_skin))
        .insert_resource(SelectedSkin {
            visor: None,
            gun: None,
        })
        .run();
}


fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
) {
    commands.spawn(Camera2dBundle::default());

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    commands.spawn((
        SpineBundle {
            skeleton: skeleton_handle.clone(),
            transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.5),
            ..Default::default()
        },
    ));
}

fn on_spawn(
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
) {
    for event in spine_ready_event.iter() {
        if let Ok(mut spine) = spine_query.get_mut(event.entity) {
            let _ = spine.skeleton.set_skin_by_name("template");

            let Spine(SkeletonController {
                          animation_state, ..
                      }) = spine.as_mut();
            let _ = animation_state.set_animation_by_name(0, "run", true);
        }
    }
}

#[derive(Component, Debug)]
struct CustomSkin {
    slot_name: String,
    holder_name: String,
    custom_img_path: String,
}

#[derive(Resource, Debug)]
struct SelectedSkin {
    visor: Option<Entity>,
    gun: Option<Entity>,
}

fn ui_setup(mut commands: Commands, asset_svr: Res<AssetServer>) {
    let spawn_img_button = |parent: &mut ChildBuilder, img_path: &str, slot_name: &str, holder_name: &str| {
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(90.),
                    height: Val::Px(90.),
                    border: UiRect::all(Val::Px(2.)),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::NONE),
                border_color: BorderColor(Color::WHITE),
                ..Default::default()
            },
            CustomSkin {
                slot_name: slot_name.to_string(),
                holder_name: holder_name.to_string(),
                custom_img_path: img_path.to_string(),
            },
        ))
            .with_children(|bp| {
                bp.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(80.),
                        height: Val::Px(80.),
                        ..Default::default()
                    },
                    image: UiImage {
                        texture: asset_svr.load(img_path),
                        ..Default::default()
                    },
                    // background_color: BackgroundColor(Color::NONE),
                    ..Default::default()
                });
            })
        ;
    };

    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(300.),
            height: Val::Px(100.),
            position_type: PositionType::Absolute,
            left: Val::Px(50.),
            top: Val::Px(50.),
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        },
        background_color: BackgroundColor(Color::NONE),
        ..Default::default()
    }).with_children(|parent| {
        spawn_img_button(parent, "spineboy/equips/goggles-normal.png", "goggles", "goggles");
        spawn_img_button(parent, "spineboy/equips/goggles-tactical.png", "goggles", "goggles");
    });


    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(300.),
            height: Val::Px(100.),
            position_type: PositionType::Absolute,
            left: Val::Px(50.),
            top: Val::Px(180.),
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        },
        background_color: BackgroundColor(Color::NONE),
        ..Default::default()
    }).with_children(|parent| {
        spawn_img_button(parent, "spineboy/equips/gun-normal.png", "gun", "gun");
        spawn_img_button(parent, "spineboy/equips/gun-freeze.png", "gun", "gun");
    });
}

fn button_system(
    mut interaction_query: Query<(Entity, &Interaction, &Parent, &CustomSkin), (Changed<Interaction>, With<Button>)>,
    mut selected_skin: ResMut<SelectedSkin>,
    mut button_query: Query<&mut BorderColor>,
    children_query: Query<&Children>,
) {
    for (entity, interaction, parent, custom_skin) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                if custom_skin.slot_name == "goggles" {
                    selected_skin.visor = Some(entity);
                } else if custom_skin.slot_name == "gun" {
                    selected_skin.gun = Some(entity);
                }
                children_query.get(parent.get()).unwrap().iter().for_each(|child| {
                    if let Ok(mut border_color) = button_query.get_mut(*child) {
                        border_color.0 = if child == &entity {
                            Color::GREEN
                        } else {
                            Color::WHITE
                        };
                    }
                });
            }
            _ => {}
        }
    }
}

fn change_skin(selected_skin: Res<SelectedSkin>,
               custom_skin_query: Query<(Entity, &CustomSkin)>,
               mut spine_query: Query<&mut Spine>,
) {
    if !selected_skin.is_changed() {
        return;
    }


    let mut skin_cfg = vec![];

    if let Some(visor) = selected_skin.visor {
        if let Ok(custom_skin) = custom_skin_query.get(visor) {
            skin_cfg.push(custom_skin);
        }
    }

    if let Some(gun) = selected_skin.gun {
        if let Ok(custom_skin) = custom_skin_query.get(gun) {
            skin_cfg.push(custom_skin);
        }
    }

    for mut spine in spine_query.iter_mut() {
        let skin = Skin::new("custom");
        let template_skin = spine.skeleton.skin().unwrap().clone();
        let slot = spine.skeleton.find_slot(custom_skin.slot_name.as_str()).unwrap();
        spine.skeleton.slot_at_index()

        // let Spine(SkeletonController {
        //               skeleton, ..
        //           }) = spine.as_mut();
        //
        // for custom_skin in skin_cfg.iter() {
        //     let _ = skeleton.set_attachment_by_name(&custom_skin.slot_name, &custom_skin.holder_name, &custom_skin.custom_img_path);
        // }
    }
    // for mut spine in spine_query.iter_mut() {};
}