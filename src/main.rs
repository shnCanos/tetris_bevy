use bevy::{prelude::*, sprite::collide_aabb::collide};
use std::fmt::Display;

mod consts;
use consts::*;

// --- Events ---
struct SpawnSingleBlockEvent {
    // NOTE! This Vec2's units are blocks and not pixels!
    translation: Vec2,
    color: Color,
}

impl Display for SpawnSingleBlockEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {} | y: {} | color: {:?}", self.translation.x, self.translation.y, self.color)
    }
}

struct SpawnBlockEvent;
struct GameOverEvent;

struct MoveEvent;
struct ShouldMoveEvent (Vec2); // The position of the parent of the block that should move
// --- Resources ---
#[derive(Resource)]
struct Score (usize);

#[derive(Resource)]
struct MainGameTimer (Timer);


// --- Components ---
#[derive(Component)]
struct BlockParent (usize); // The Index of BLOCK_TYPES

#[derive(Component)]
struct NormalBlock;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        
        .add_startup_system(startup_system)
        
        .add_system(spawn_single_block_system)
        .add_system(should_move_block_system)
        
        .add_event::<SpawnSingleBlockEvent>()
        .add_event::<GameOverEvent>()
        .add_event::<MoveEvent>()
        .add_event::<ShouldMoveEvent>()
        .add_event::<SpawnBlockEvent>()
        
        .insert_resource(Score (0))
        .insert_resource(MainGameTimer (Timer::from_seconds(GAME_SPEED, TimerMode::Repeating)))
        
        .run();
}

// --- Startup systems ---
fn startup_system(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
}

// --- Normal Systems ---
fn spawn_single_block_system(mut commands: Commands, mut event: EventReader<SpawnSingleBlockEvent>) {
    for block in event.iter() {
        if DBG_MODE {
            println!("Spawned block: {block}");
        }
        commands.spawn(SpriteBundle {
            transform: Transform {
                translation: Vec3::from((block.translation.x * BLOCK_SIZE, block.translation.y * BLOCK_SIZE, 0.)),
                scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.),
                ..Default::default()
            },
            sprite: Sprite {
                color: block.color,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

// Tests whether the blocks should move and if they should sends an event:
// ShouldMoveEvent (position_of_the_block_parent)
// If no blocks move it sends the event
// SpawnBlockEvent
fn should_move_block_system (
    mut move_event: EventReader<MoveEvent>,
    mut should_move_event: EventWriter<ShouldMoveEvent>,
    mut spawn_block_event: EventWriter<SpawnBlockEvent>,
    blocks_query: Query<(&Transform, &BlockParent), With<BlockParent>>,
    all_blocks_query: Query<&Transform, With<NormalBlock>>,
) {
    for _ in move_event.iter() {
        let mut block_moved = false;
        
        for (parent_transform, block_parent) in blocks_query.iter() {
            
            // -- Check whether the block parent should move --
            // Conputes the hitboxes of all the blocks that make
            // Up the block parent and sees if they:
            // - Hit the floor
            // - Hit another block below them
            
            let block_parent = BLOCK_TYPES[block_parent.0];
            let mut blocks_translations = Vec::new();

            for translation in block_parent.iter() {
                blocks_translations.push(parent_transform.translation + (translation.extend(0.) * BLOCK_SIZE));
            }

            let mut should_move = true;
            for translation in blocks_translations.iter() {
                // Hit the floor
                if translation.y >= LIMITS.y * BLOCK_SIZE {
                    should_move = false;
                    break;
                }
                
                // Check for collisions
                let hitbox = Vec2::splat(BLOCK_SIZE + 1.); // The hitbox has to be slightly bigger than the block

                for other_blocks_translation in all_blocks_query.iter() {
                    let other_blocks_translation = other_blocks_translation.translation;

                    // Check whether they don't have the same parent
                    if blocks_translations.contains(&other_blocks_translation) {
                        break;
                    }

                    match collide(*translation, hitbox, other_blocks_translation, hitbox) {
                        Some(collision) => {
                            if DBG_MODE {
                                println!("Block at {} {} | Collided!: {:?}", translation.x, translation.y, collision);
                            }
                            match collision {
                                bevy::sprite::collide_aabb::Collision::Top |
                                bevy::sprite::collide_aabb::Collision::Bottom => {
                                    should_move = false;
                                    break;
                                },
                                _ => ()
                            };
                        },
                        None => (),
                    }
                }

                if !should_move {
                    break;
                }
            }

            if should_move {
                block_moved = true;
                should_move_event.send(ShouldMoveEvent (parent_transform.translation.truncate()));
            }
        }

        if block_moved {
            spawn_block_event.send(SpawnBlockEvent);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}
