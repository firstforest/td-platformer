use crate::actions::Actions;
use crate::constants::{WIN_HEIGHT, WIN_WIDTH};
use crate::GameState;

use bevy::prelude::*;
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
                    .with_system(setup_player),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(print_ball_altitude)
                    .with_system(move_system)
                    .with_system(spawn_enemies)
                    .with_system(move_enemies),
            );

        #[cfg(debug_assertions)]
        {
            app.add_plugin(RapierDebugRenderPlugin::default());
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Core;

#[derive(Component)]
struct Enemy {
    speed: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self { speed: 40.0 }
    }
}

struct Timers {
    enemy_spawn_timer: Timer,
}

impl Timers {
    fn new() -> Self {
        Self {
            enemy_spawn_timer: Timer::from_seconds(3.0, true),
        }
    }
}

impl Default for Timers {
    fn default() -> Self {
        Timers::new()
    }
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_player(mut commands: Commands) {
    let shape = shapes::Rectangle {
        extents: Vec2::new(40., 50.),
        origin: default(),
    };
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::CYAN)),
            Transform::default(),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule_y(5., 20.))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Velocity::default())
        .insert(Restitution {
            coefficient: 0.0,
            ..default()
        })
        .insert(ExternalImpulse::default())
        .insert(GravityScale(2.))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)));
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

fn move_system(
    actions: Res<Actions>,
    mut players: Query<(&mut Velocity, &mut ExternalImpulse), With<Player>>,
) {
    for (mut velocity, mut impulse) in players.iter_mut() {
        if actions.player_movement.is_none() {
            velocity.linvel = Vec2::new(0., velocity.linvel.y);
        } else {
            velocity.linvel =
                Vec2::new(actions.player_movement.unwrap().x * 100., velocity.linvel.y);
        }

        match actions.player_jump {
            Some(true) => {
                velocity.linvel = Vec2::new(velocity.linvel.x, 0.);
                impulse.impulse = Vec2::new(0., 20.);
            }
            _ => {}
        }
    }
}

fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        println!("Ball altitude: {}", transform.translation.y);
    }
}

fn move_enemies(
    target: Query<&Transform, With<Core>>,
    mut query: Query<(&mut Velocity, &Transform, &Enemy)>,
) {
    for (mut velocity, transform, enemy) in query.iter_mut() {
        let vec = target.single().translation - transform.translation;
        velocity.linvel = vec.truncate().normalize() * enemy.speed;
    }
}

fn spawn_enemies(mut commands: Commands, time: Res<Time>, mut timers: ResMut<Timers>) {
    timers.enemy_spawn_timer.tick(time.delta());

    if timers.enemy_spawn_timer.just_finished() {
        let radius = 10.;
        let shape = shapes::RegularPolygon {
            sides: 5,
            feature: RegularPolygonFeature::Radius(radius),
            ..default()
        };
        let mut rnd_gen = rand::thread_rng();
        let transform = Transform::from_xyz(
            rnd_gen.gen_range(0.0..WIN_WIDTH) - (WIN_WIDTH / 2.0),
            rnd_gen.gen_range(0.0..100.),
            0.0,
        );
        commands
            .spawn()
            .insert(Enemy::default())
            .insert(RigidBody::Dynamic)
            .insert(Velocity::zero())
            .insert_bundle(TransformBundle::from(transform))
            .insert_bundle(GeometryBuilder::build_as(
                &shape,
                DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::RED)),
                transform,
            ))
            .insert(Collider::ball(radius));
    }
}
