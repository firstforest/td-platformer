use crate::actions::Actions;
use crate::GameState;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MainGamePlugin;

impl Plugin for MainGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugin(ShapePlugin)
            .add_system_set(
                SystemSet::on_enter(GameState::Playing)
                    .with_system(setup_graphics)
                    .with_system(setup_physics),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(print_ball_altitude)
                    .with_system(move_system),
            );

        #[cfg(debug_assertions)]
        {
            app.add_plugin(RapierDebugRenderPlugin::default());
        }
    }
}

#[derive(Component)]
struct Player;

fn setup_graphics(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_physics(mut commands: Commands) {
    commands
        .spawn()
        .insert(Collider::cuboid(500.0, 50.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));

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
        .insert(ExternalImpulse::default())
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)));
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
