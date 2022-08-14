use super::enemy::*;
use super::player::*;
use crate::constants::{WIN_HEIGHT, WIN_WIDTH};
use crate::loading::AudioAssets;
use crate::GameState;

use bevy::prelude::*;
use bevy_kira_audio::Audio;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MainGamePlugin;

impl Plugin for MainGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugin(ShapePlugin)
            .init_resource::<Timers>()
            .add_system_set(
                SystemSet::on_enter(GameState::Playing)
                    .with_system(setup_graphics)
                    .with_system(setup_ground)
                    .with_system(setup_core)
                    .with_system(setup_player),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    // .with_system(print_ball_altitude)
                    .with_system(move_player_system)
                    .with_system(spawn_enemies)
                    .with_system(move_enemies)
                    .with_system(despawn_enemy),
            );

        #[cfg(debug_assertions)]
        {
            app.add_plugin(RapierDebugRenderPlugin::default());
        }
    }
}

#[derive(Component)]
struct Core;

fn setup_graphics(mut commands: Commands) {
    // commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_core(mut commands: Commands) {
    let radius: f32 = 20.;
    let shape = shapes::Circle {
        radius,
        ..default()
    };
    commands
        .spawn()
        .insert(Core)
        .insert(Target)
        .insert_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::GOLD)),
            Transform::default(),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(radius))
        .insert(Damping {
            linear_damping: 0.7,
            ..Default::default()
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            0.0,
            -(WIN_HEIGHT / 2.) + 40. + radius,
            0.0,
        )));
}

fn setup_ground(mut commands: Commands) {
    let plain = shapes::Rectangle {
        extents: Vec2::new(WIN_WIDTH, 40.),
        ..default()
    };
    commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &plain,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::BLACK)),
            default(),
        ))
        .insert(Collider::cuboid(WIN_WIDTH, 20.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            0.0,
            -WIN_HEIGHT / 2. + 20.,
            0.0,
        )));
}

fn despawn_enemy(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    player: Query<Entity, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                if player.single() == *a && enemies.contains(*b) {
                    commands.entity(*b).despawn();
                    audio.play(audio_assets.attack.clone());
                }
                if player.single() == *b && enemies.contains(*a) {
                    commands.entity(*a).despawn();
                    audio.play(audio_assets.attack.clone());
                }
            }
            _ => {}
        }
        println!("Received collision event: {:?}", collision_event);
    }

    for contact_force_event in contact_force_events.iter() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}
