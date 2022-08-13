use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use rand::Rng;

use crate::constants::*;

#[derive(Component)]
pub struct Enemy {
    speed: f32,
}

#[derive(Component)]
pub struct Target;

impl Default for Enemy {
    fn default() -> Self {
        Self { speed: 40.0 }
    }
}

pub struct Timers {
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

pub fn move_enemies(
    target: Query<&Transform, With<Target>>,
    mut query: Query<(&mut Velocity, &Transform, &Enemy)>,
) {
    for (mut velocity, transform, enemy) in query.iter_mut() {
        let vec = target.single().translation - transform.translation;
        velocity.linvel = vec.truncate().normalize() * enemy.speed;
    }
}

pub fn spawn_enemies(mut commands: Commands, time: Res<Time>, mut timers: ResMut<Timers>) {
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
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert_bundle(GeometryBuilder::build_as(
                &shape,
                DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(Color::RED)),
                transform,
            ))
            .insert(Collider::ball(radius));
    }
}
