use ambient_api::{
    components::core::{
        app::main_scene,
        prefab::prefab_from_url,
    },
    concepts::make_transformable,
    prelude::*,
    entity::{AnimationAction, AnimationController},
};

#[main]
pub fn main() {
    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(id, make_transformable());
        }

        /* This works            
        entity::add_component(
            id,
            prefab_from_url(),
            asset::url("assets/xbot/X Bot.fbx").unwrap(),
        );
        
        entity::set_animation_controller(
            id,
            AnimationController {
                actions: &[AnimationAction {
                    clip_url: &asset::url("assets/xbot/Idle.fbx/animations/mixamo.com.anim")
                        .unwrap(),
                    looping: true,
                    weight: 1.0,
                }],
                apply_base_pose: false,
            },
        );
        */
    });
}
