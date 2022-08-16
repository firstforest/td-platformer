use super::enemy::*;
use super::player::*;
use crate::constants::{WIN_HEIGHT, WIN_WIDTH};
use crate::loading::AudioAssets;
use crate::loading::FontAssets;
use crate::GameState;

use bevy::prelude::*;
use bevy_kira_audio::Audio;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

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
                    .with_system(setup_player)
                    .with_system(setup_score),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    // .with_system(print_ball_altitude)
                    .with_system(move_player_system)
                    .with_system(spawn_enemies)
                    .with_system(move_enemies)
                    .with_system(despawn_enemies)
                    .with_system(move_energy)
                    .with_system(start_collect)
                    .with_system(collect_energy)
                    .with_system(update_score),
            );

        #[cfg(debug_assertions)]
        {
            app.add_plugin(RapierDebugRenderPlugin::default());
        }
    }
}

#[derive(Component)]
struct EnergyText;

struct EnergyPoint(i32);

fn setup_score(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands
        .spawn()
        .insert_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "Energy: ",
                    TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 60.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: font_assets.fira_sans.clone(),
                    font_size: 60.0,
                    color: Color::GOLD,
                }),
            ])
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                ..default()
            }),
        )
        .insert(EnergyText);
    commands.insert_resource(EnergyPoint(0));
}

fn update_score(mut query: Query<&mut Text, With<EnergyText>>, score: Res<EnergyPoint>) {
    let s = score.0;
    for mut text in &mut query {
        text.sections[1].value = format!("{s:.2}");
    }
}

#[derive(Component)]
struct Core;
#[derive(Component)]
struct CollectArea {
    radius: f32,
}

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
        )))
        .with_children(|parent| {
            parent
                .spawn()
                .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    0., 0., 0.,
                )))
                .insert(CollectArea { radius: 200.0 })
                .insert(Collider::ball(200.0))
                .insert(Sensor)
                .insert(ColliderMassProperties::Density(0.0));
        });
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

fn despawn_enemies(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    player: Query<Entity, With<Player>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                if player.single() == *a {
                    if let Some(enemy) = enemies.iter().find(|x| x.0 == *b) {
                        despawn_enemy(&mut commands, &audio, &audio_assets, enemy);
                    }
                }
                if player.single() == *b {
                    if let Some(enemy) = enemies.iter().find(|x| x.0 == *a) {
                        despawn_enemy(&mut commands, &audio, &audio_assets, enemy);
                    }
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

fn despawn_enemy(
    commands: &mut Commands,
    audio: &Res<bevy_kira_audio::AudioChannel<bevy_kira_audio::MainTrack>>,
    audio_assets: &Res<AudioAssets>,
    enemy: (Entity, &Transform),
) {
    commands.entity(enemy.0).despawn();
    audio.play(audio_assets.attack.clone());
    let mut rand = rand::thread_rng();
    let linvel = Vec2::new(rand.gen_range(-1.0..1.0), rand.gen_range(0.0..1.0)).normalize() * 200.0;
    commands
        .spawn_bundle(EnergyBundle::default())
        .insert_bundle(TransformBundle::from(enemy.1.clone()))
        .insert(RigidBody::KinematicVelocityBased)
        .insert(Velocity {
            linvel,
            ..default()
        });
}

// Energy
#[derive(PartialEq)]
enum EnergyState {
    Created { remaining_time: f32 },
    Horming { goal_time: f32 },
    Goal,
}

#[derive(Component)]
struct Energy {
    energy: i32,
    state: EnergyState,
}

impl Default for Energy {
    fn default() -> Self {
        Self {
            energy: 1,
            state: EnergyState::Created {
                remaining_time: 0.3,
            },
        }
    }
}

#[derive(Bundle)]
struct EnergyBundle {
    energy: Energy,
    #[bundle]
    shape_bundle: ShapeBundle,
}

impl Default for EnergyBundle {
    fn default() -> Self {
        let shape = shapes::Circle {
            radius: 10.,
            ..default()
        };
        Self {
            energy: default(),
            shape_bundle: GeometryBuilder::build_as(
                &shape,
                DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(
                    Color::YELLOW_GREEN,
                )),
                Transform::default(),
            ),
        }
    }
}

fn move_energy(
    mut query: Query<(&mut Velocity, &Transform, &mut Energy)>,
    target: Query<&Transform, With<Core>>,
    time: Res<Time>,
) {
    for (mut velocity, transform, mut energy) in query.iter_mut() {
        match energy.state {
            EnergyState::Created { remaining_time } => {
                let new_time = remaining_time - time.delta_seconds();
                if new_time.is_sign_negative() {
                    velocity.linvel = Vec2::ZERO;
                    continue;
                }
                energy.state = EnergyState::Created {
                    remaining_time: new_time,
                }
            }
            EnergyState::Horming { goal_time } => {
                if goal_time - time.delta_seconds() < 0.0 {
                    energy.state = EnergyState::Goal;
                    velocity.linvel = Vec2::ZERO;
                    continue;
                }
                let diff = (target.single().translation - transform.translation).truncate();
                let acc = (diff - velocity.linvel * goal_time) * 2.0 / (goal_time * goal_time);
                velocity.linvel = velocity.linvel + acc * time.delta_seconds();
                energy.state = EnergyState::Horming {
                    goal_time: goal_time - time.delta_seconds(),
                }
            }
            _ => (),
        }
    }
}

fn collect_energy(
    mut commands: Commands,
    query: Query<(&Energy, Entity)>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    mut score: ResMut<EnergyPoint>,
) {
    for (energy, entity) in query.iter().filter(|x| x.0.state == EnergyState::Goal) {
        commands.entity(entity).despawn();
        audio.play(audio_assets.collect.clone());
        score.0 += energy.energy;
    }
}

fn start_collect(
    mut query: Query<(&mut Energy, &Transform)>,
    area_query: Query<(&CollectArea, &GlobalTransform)>,
) {
    if area_query.is_empty() {
        return;
    }
    let (area, area_transform) = area_query.single();
    for (mut energy, transform) in query.iter_mut() {
        match energy.state {
            EnergyState::Created { remaining_time: _ } => {
                if (area_transform.translation() - transform.translation).length_squared()
                    < area.radius.powi(2)
                {
                    energy.state = EnergyState::Horming { goal_time: 1.0 }
                }
            }
            _ => {}
        }
    }
}
