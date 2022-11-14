use bevy::{prelude::*, math::vec2};

pub const DBG_MODE: bool = true;

pub const BLOCK_SIZE: f32 = 20.;// The size of a block in pixels

pub const LIMITS: Vec2 = vec2(5., 10.);

pub const GAME_SPEED: f32 = 1.; // Game speed in seconds

pub const DOWN_KEY_MULTIPLIER: f32 = 50.;

// Blocks should appear as coordinates in this format
// 
// |  1  |
// -1-0--1
// | -1  |

// Use Vec2::ZERO to fill the coordinates that aren't used
pub const BLOCK_TYPES: [[Vec2; 4]; 1] = [[Vec2::ZERO, vec2(0., 1.), vec2(0., -1.), vec2(-1., 1.)]];