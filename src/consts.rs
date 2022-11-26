use bevy::{prelude::*, math::vec2};

pub const DBG_MODE: bool = false;

pub const BLOCK_SIZE: f32 = 20.;// The size of a block in pixels

pub const LIMITS: Vec2 = vec2(10., 20.);

pub const GAME_SPEED: f32 = 1.; // Game speed in seconds

pub const DOWN_KEY_MULTIPLIER: f32 = 25.;

pub const RELATIVE_SIDES_MOVING_SPEED: f32 = 25.; // The time it takes to be able to move to the sides in relation to GAME_SPEED: GAME_SPEED / RELATIVE_SIDES_MOVING _SPEED

// Blocks should appear as coordinates in this format
// 
// |  1  |
// -1-0--1
// | -1  |

// Use Vec2::ZERO to fill the coordinates that aren't used
pub const BLOCK_TYPES: [[Vec2; 5]; 5] = [
    [Vec2::ZERO, vec2(0., 1.), vec2(0., -1.), vec2(-1., 1.), Vec2::ZERO], 
    [Vec2::ZERO, vec2(-1., 0.), vec2(1., 0.), Vec2::ZERO, Vec2::ZERO],
    [Vec2::ZERO, vec2(-1., 0.), vec2(1., 0.), vec2(0., -1.), Vec2::ZERO],
    [Vec2::ZERO, vec2(-1., 0.), vec2(-1., 1.), vec2(0., -1.), Vec2::ZERO],
    [Vec2::ZERO, vec2(1., 0.), vec2(1., -1.), vec2(0., -1.), Vec2::ZERO],
    ];

pub const SCORE_INCREMENT: usize = 100;