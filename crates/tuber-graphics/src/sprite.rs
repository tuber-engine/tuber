use crate::texture::{TextureRegion, TextureSource};
use std::time::Instant;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::W;

pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub texture: TextureSource,
}

pub struct AnimatedSprite {
    pub width: f32,
    pub height: f32,
    pub texture: TextureSource,
    pub animation_state: AnimationState,
}

pub struct AnimationState {
    pub keyframes: Vec<TextureRegion>,
    pub current_keyframe: usize,
    pub start_instant: Instant,
    pub frame_duration: u32,
    pub flip_x: bool,
}

pub fn sprite_animation_step_system(ecs: &mut Ecs) {
    for (_, (mut animated_sprite,)) in ecs.query::<(W<AnimatedSprite>,)>() {
        let mut animation_state = &mut animated_sprite.animation_state;
        animation_state.current_keyframe = ((animation_state.start_instant.elapsed().as_millis()
            / animation_state.frame_duration as u128)
            % animation_state.keyframes.len() as u128)
            as usize
    }
}
