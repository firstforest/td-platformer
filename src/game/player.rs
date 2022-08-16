use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::actions::Actions;

#[derive(Component)]
pub struct Player {
    jump_power: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self { jump_power: 100. }
    }
}

pub fn setup_player(mut commands: Commands) {
    let shape = shapes::Rectangle {
        extents: Vec2::new(40., 50.),
        origin: default(),
    };
    commands
        .spawn()
        .insert(Player::default())
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
        .insert(GravityScale(10.))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)));
}

pub fn move_player_system(
    actions: Res<Actions>,
    mut players: Query<(&mut Velocity, &mut ExternalImpulse, &Player)>,
) {
    for (mut velocity, mut impulse, player) in players.iter_mut() {
        if actions.player_movement.is_none() {
            velocity.linvel = Vec2::new(0., velocity.linvel.y);
        } else {
            velocity.linvel =
                Vec2::new(actions.player_movement.unwrap().x * 100., velocity.linvel.y);
        }

        match actions.player_jump {
            Some(true) => {
                velocity.linvel = Vec2::new(velocity.linvel.x, 0.);
                impulse.impulse = Vec2::new(0., player.jump_power);
            }
            _ => {}
        }
    }
}
