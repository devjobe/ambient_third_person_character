#![allow(unused)]

use ambient_api::{
    asset,
    entity::{self, AnimationAction, AnimationController},
    prelude::{EntityId, Vec3},
};

pub const MAX_SPEED: f32 = 6.0;
pub const WALK_SPEED: f32 = 2.0;
pub const INV_ACCELERATION_TIME: f32 = 10.0;
pub const GRAVITY: f32 = 9.8;
pub const JUMP_HEIGHT: f32 = 2.0;
pub const LOCAL_FORWARD: Vec3 = Vec3::X;
pub const LOCAL_RIGHT: Vec3 = Vec3::Y;
pub const LOCAL_UP: Vec3 = Vec3::Z;
pub const WORLD_DOWN: Vec3 = Vec3::NEG_Z;
pub const TERMINAL_SPEED: f32 = 100.0;

pub struct AnimationAssets {
    pub idle: String,
    pub walking: String,
    pub running: String,
    pub jump: String,
    pub falling_idle: String,
}

#[derive(Default, Debug)]
pub struct Blend {
    pub idle: f32,
    pub walking: f32,
    pub running: f32,
    pub jump: f32,
    pub falling_idle: f32,
}

impl AnimationAssets {
    pub fn new() -> Self {
        AnimationAssets {
            idle: asset::url("assets/xbot/Idle.fbx/animations/mixamo.com.anim").unwrap(),
            walking: asset::url("assets/xbot/Walking.fbx/animations/mixamo.com.anim").unwrap(),
            running: asset::url("assets/xbot/Running.fbx/animations/mixamo.com.anim").unwrap(),
            jump: asset::url("assets/xbot/Jumping.fbx/animations/mixamo.com.anim").unwrap(),
            falling_idle: asset::url("assets/xbot/Falling.fbx/animations/mixamo.com.anim").unwrap(),
        }
    }

    pub fn set_controller(&self, entity: EntityId, blend: Blend) {
        let actions = [
            AnimationAction {
                clip_url: &self.idle,
                looping: true,
                weight: blend.idle,
            },
            AnimationAction {
                clip_url: &self.walking,
                looping: true,
                weight: blend.walking,
            },
            AnimationAction {
                clip_url: &self.running,
                looping: true,
                weight: blend.running,
            },
            AnimationAction {
                clip_url: &self.jump,
                looping: true,
                weight: blend.jump,
            },
            AnimationAction {
                clip_url: &self.falling_idle,
                looping: true,
                weight: blend.falling_idle,
            },
        ];

        entity::set_animation_controller(
            entity,
            AnimationController {
                actions: &actions,
                apply_base_pose: false,
            },
        );
    }

    pub fn set_blend(entity: EntityId, blend: Blend) {
        entity::set_animation_blend(
            entity,
            &[
                blend.idle,
                blend.walking,
                blend.running,
                blend.jump,
                blend.falling_idle,
            ],
            &[],
            false
        );
    }
}
