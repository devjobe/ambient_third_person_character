use ambient_api::{
    components::core::{
        physics::{
            box_collider, character_controller_height, character_controller_radius,
            physics_controlled, plane_collider,
        },
        player::player,
        primitives::{cube, quad},
        rendering::color,
        transform::{local_to_parent, local_to_world, rotation, scale, translation},
    },
    concepts::make_transformable,
    prelude::*,
};

use components::{
    player_grounded, player_local_velocity, player_movement_direction, player_target_speed,
};

mod anim;
use anim::*;

#[main]
pub fn main() {
    make_transformable()
        .with_default(quad())
        .with(scale(), Vec3::ONE * 1000.)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with_default(plane_collider())
        .spawn();

    for i in 0..10 {
        let h = i as f32 + 1.0;
        make_transformable()
            .with_default(cube())
            .with_default(cast_shadows())
            .with(scale(), vec3(1., 1., h * 0.25))
            .with(box_collider(), Vec3::ONE)
            .with(translation(), vec3(h, -10.0, h * 0.25 * 0.5))
            .with(color(), Vec4::ONE)
            .spawn();
    }

    for i in 0..10 {
        let h = i as f32 + 11.0;
        make_transformable()
            .with_default(cube())
            .with_default(cast_shadows())
            .with(scale(), vec3(1., 1., h * 0.25))
            .with(box_collider(), Vec3::ONE)
            .with(translation(), vec3(10.0, h - 20.0, h * 0.25 * 0.5))
            .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
            .spawn();
    }

    make_transformable()
        .with_default(cube())
        .with_default(cast_shadows())
        .with(scale(), vec3(10., 5., 1.0))
        .with(box_collider(), Vec3::ONE)
        .with(translation(), vec3(10.0, 3.0, 5.0))
        .with(color(), vec4(0.0, 1.0, 0.0, 1.0))
        .spawn();

    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_transformable())
                    .with(color(), Vec4::ONE)
                    .with(character_controller_height(), 2.)
                    .with(character_controller_radius(), 0.5)
                    .with_default(player_local_velocity())
                    .with_default(player_grounded())
                    .with_default(physics_controlled())
                    .with_default(local_to_parent())
                    .with_default(local_to_world()),
            );
        }
    });

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        if msg.jump && entity::get_component(player_id, player_grounded()) == Some(true) {
            entity::mutate_component(player_id, player_local_velocity(), |velocity| {
                if velocity.z < 0.0 {
                    let height = if msg.run {
                        JUMP_HEIGHT * 2.0
                    } else {
                        JUMP_HEIGHT
                    };
                    velocity.z = (GRAVITY * height).sqrt();
                }
            });
        }

        entity::add_component(player_id, player_movement_direction(), msg.direction);
        let speed = if msg.run { MAX_SPEED } else { WALK_SPEED };

        entity::add_component(player_id, player_target_speed(), speed);
    });

    query((
        player(),
        player_movement_direction(),
        player_target_speed(),
        rotation(),
    ))
    .each_frame(move |players| {
        for (player_id, (_, direction, speed, rot)) in players {
            update_player_movement(player_id, rot, direction, speed);
        }
    });

    fn update_player_movement(
        player_id: EntityId,
        mut player_rotation: Quat,
        input_direction: Vec2,
        mut target_speed: f32,
    ) {
        fn update_velocity(player_id: EntityId, target_speed: f32, dt: f32) -> Vec3 {
            let previous_velocity =
                entity::get_component(player_id, player_local_velocity()).unwrap_or_default();
            let t = dt * INV_ACCELERATION_TIME;
            let speed = (target_speed - previous_velocity.x) * t;

            let delta_forward = LOCAL_FORWARD * speed;
            let mut current_velocity = previous_velocity + delta_forward;

            if current_velocity.x.abs() < 0.001 {
                current_velocity.x = 0.0;
            }

            let delta_gravity = LOCAL_UP * (-GRAVITY * dt);
            current_velocity += delta_gravity;
            current_velocity = current_velocity.clamp_length(0.0, TERMINAL_SPEED);

            current_velocity
        }

        let input_rotation = input_direction
            .try_normalize()
            .map(|v| Quat::from_rotation_z(v.y.atan2(v.x)));
        let dt = frametime();
        if let Some(intended_rotation) = input_rotation {
            player_rotation = player_rotation.slerp(intended_rotation, dt * 10.0);
            entity::set_component(player_id, rotation(), player_rotation);
        } else {
            target_speed = 0.0;
        };

        let velocity = update_velocity(player_id, target_speed, dt);

        let displace = player_rotation * velocity * dt;
        let collision = physics::move_character(player_id, displace, 0.01, dt);

        let grounded = collision.down && velocity.z < 0.0;
        entity::set_component(player_id, player_local_velocity(), velocity);
        entity::set_component(player_id, player_grounded(), grounded);
    }
}
