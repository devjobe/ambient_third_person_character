use std::cell::RefCell;

use crate::components::{
    anim_character, anim_grounded, anim_jump_layer, anim_velocity, player_camera_ref,
    player_grounded, player_local_velocity,
};
use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::{local_user_id, player, user_id},
        prefab::prefab_from_url,
        transform::{rotation, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

mod anim;
use anim::*;

#[main]
fn main() {
    let _sun = Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-1.))
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(light_ambient(), Vec3::ONE * 0.2)
        .with(fog_color(), vec3(1., 1., 1.))
        .with(fog_density(), 0.1)
        .with(fog_height_falloff(), 0.01)
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();

    let animations = AnimationAssets::new();
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, (_, user)) in players {
            // First, we check if this player is the "local" player, and only then do we attach a camera
            if user == entity::get_component(entity::resources(), local_user_id()).unwrap() {
                let camera = Entity::new()
                    .with_merge(make_perspective_infinite_reverse_camera())
                    .with(aspect_ratio_from_window(), EntityId::resources())
                    .with_default(main_scene())
                    .with(user_id(), user)
                    .with(translation(), Vec3::ONE * 5.)
                    .with(lookat_target(), vec3(0., 0., 0.))
                    .spawn();

                entity::add_components(id, Entity::new().with(player_camera_ref(), camera));
            }

            let character = make_transformable()
                .with(
                    rotation(),
                    Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
                )
                .with_default(local_to_parent())
                .with_default(local_to_world())
                .with(parent(), id)
                .with_default(main_scene())
                .with_default(anim_character())
                .with_default(anim_velocity())
                .with_default(anim_grounded())
                .with_default(anim_jump_layer())
                .with(
                    prefab_from_url(),
                    asset::url("assets/xbot/X Bot.fbx").unwrap(),
                )
                .spawn();

            entity::add_component(id, children(), vec![character]);

            let mut blend = Blend::default();
            blend.idle = 1.0;
            animations.set_controller(character, blend);
        }
    });

    // Since we're only attaching player_camera_ref to the local player, this system will only
    // run for the local player
    let cursor_lock = RefCell::new(input::CursorLockGuard::new(true));

    query((player(), player_camera_ref(), translation())).each_frame(move |players| {
        for (_, (_, camera_id, pos)) in players {
            let (delta, pressed) = input::get_delta();
            if !cursor_lock.borrow_mut().auto_unlock_on_escape(&pressed) {
                return;
            }

            let old_direction = (entity::get_component(camera_id, lookat_target()).unwrap()
                - entity::get_component(camera_id, translation()).unwrap())
            .try_normalize()
            .unwrap_or(vec3(0.8, 0.0, -0.2));

            let mut look_direction =
                Quat::from_rotation_z(delta.mouse_position.x * 0.01) * old_direction;
            look_direction.z += delta.mouse_position.y * 0.01;

            let camera_forward = vec3(look_direction.x, look_direction.y, 0.)
                .try_normalize()
                .unwrap_or(LOCAL_FORWARD);
            let camera_right = LOCAL_UP.cross(camera_forward);

            let mut input_direction = Vec3::ZERO;
            if pressed.keys.contains(&KeyCode::W) {
                input_direction += camera_forward;
            }

            if pressed.keys.contains(&KeyCode::S) {
                input_direction -= camera_forward;
            }

            if pressed.keys.contains(&KeyCode::A) {
                input_direction -= camera_right;
            }

            if pressed.keys.contains(&KeyCode::D) {
                input_direction += camera_right;
            }

            let jump = pressed.keys.contains(&KeyCode::Space);
            let run = pressed.keys.contains(&KeyCode::LShift);

            messages::Input::new(input_direction.truncate(), jump, run).send_server_reliable();

            let target = pos + LOCAL_UP * 1.0;
            entity::set_component(camera_id, lookat_target(), target);
            entity::set_component(
                camera_id,
                translation(),
                target - look_direction.try_normalize().unwrap_or(old_direction) * 5.0,
            );
        }
    });

    query((
        anim_character(),
        anim_velocity(),
        anim_jump_layer(),
        parent(),
    ))
    .build()
    .each_frame(move |characters| {
        for (character, (_, previous_velocity, previous_jump_layer, player)) in characters {
            let player_velocity =
                entity::get_component(player, player_local_velocity()).unwrap_or_default();
            let current_grounded =
                entity::get_component(player, player_grounded()).unwrap_or_default();

            const STEPS: f32 = 100.0;
            const HALF: Vec3 = Vec3::splat(0.5);
            let mut current_velocity = (player_velocity * STEPS - HALF).ceil() / STEPS; // quantize

            current_velocity.z = current_velocity.z.signum();

            let blend_direction = Vec2::new(
                if !current_grounded { 1.0 } else { -1.0 },
                if current_velocity.z > 0.0 { 1.0 } else { -1.0 },
            );
            let dt = frametime();
            const JUMP_LAYER_BLEND_TIME: f32 = 0.2;
            let blend_delta = dt / JUMP_LAYER_BLEND_TIME;

            let mut jump_layer =
                (previous_jump_layer + blend_delta * blend_direction).clamp(Vec2::ZERO, Vec2::ONE);

            if previous_velocity.z == 0.0 && current_velocity.z != 0.0 {
                if current_velocity.z > 0.0 {
                    jump_layer.y = 1.0;
                } else {
                    jump_layer.y = 0.0;
                }
            }

            if jump_layer.x == 0.0 {
                jump_layer.y = 0.0;
            }

            if previous_jump_layer == jump_layer && current_velocity == previous_velocity {
                continue;
            }

            entity::set_component(character, anim_velocity(), current_velocity);
            entity::set_component(character, anim_jump_layer(), jump_layer);

            let mut blend = Blend::default();
            let jumping_weight = jump_layer.x;
            if jumping_weight > 0.0 {
                let t = jumping_weight * jump_layer.y;
                blend.jump = t;
                blend.falling_idle = 1.0 - t;
            }

            fn unorm_clamped(x: f32, min: f32, max: f32) -> f32 {
                ((x - min) / (max - min)).clamp(0.0, 1.0).abs()
            }
            let locomotion_weight = 1.0 - jump_layer.x;
            if locomotion_weight > 0.0 {
                if current_velocity.x > WALK_SPEED {
                    let t = unorm_clamped(current_velocity.x, WALK_SPEED, MAX_SPEED);
                    blend.running = t;
                    blend.walking = 1.0 - t;
                } else {
                    let t = unorm_clamped(current_velocity.x, 0.0, WALK_SPEED);
                    blend.walking = t;
                    blend.idle = 1.0 - t;
                };

                blend.running *= locomotion_weight;
                blend.walking *= locomotion_weight;
                blend.idle *= locomotion_weight;
            }

            AnimationAssets::set_blend(character, blend);
        }
    });
}
