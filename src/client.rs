use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        player::{local_user_id, player, user_id},
        prefab::prefab_from_url,
        transform::{translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    entity::{AnimationAction, AnimationController},
    prelude::*,
};

#[main]
fn main() {
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, (_, user)) in players {
            // First, we check if this player is the "local" player, and only then do we attach a camera
            if user == entity::get_component(entity::resources(), local_user_id()).unwrap() {
                let camera = Entity::new()
                    .with_merge(make_perspective_infinite_reverse_camera())
                    .with(aspect_ratio_from_window(), EntityId::resources())
                    .with_default(main_scene())
                    .with(user_id(), user)
                    .with(translation(), Vec3::ONE * 3.)
                    .with(lookat_target(), vec3(0., 0., 0.))
                    .spawn();
            }

            // I suspect this makes the parent-child relationships reset from server when UI is updated
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
        }
    });

    Innocent::el().spawn_interactive();
}

#[element_component]
fn Innocent(hooks: &mut Hooks) -> Element {
    let (innocent_value, set_culprit) = hooks.use_state(0.0f32);

    FocusRoot::el([FlowRow::el([
        Text::el("Completely Innocent UI"),
        Slider {
            value: innocent_value,
            on_change: Some(cb(move |value| {
                set_culprit(value);
            })),
            min: 0.,
            max: 1.,
            width: 100.,
            logarithmic: false,
            round: Some(2),
            suffix: None,
        }
        .el(),
    ])
    .with(space_between_items(), 4.0)
    .with_background(vec4(0., 0., 0., 0.9))
    .with_padding_even(10.)])
}
