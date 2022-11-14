use bevy::{prelude::*, sprite::collide_aabb::collide};
use std::{fmt::Display, time::Duration};
use rand::{self, Rng};

mod consts;
use consts::*;

// --- Events ---
struct SpawnBlockEvent;
struct GameOverEvent;

struct MoveEvent;
// --- Resources ---
#[derive(Resource)]
struct Score (usize);

#[derive(Resource)]
struct MainGameTimer (Timer);
impl Default for MainGameTimer {
    fn default() -> Self {
        Self ( Timer::from_seconds(GAME_SPEED, TimerMode::Repeating) )
    }
}

// --- Components ---
#[derive(Component)]
struct BlockParent (usize); // The Index of BLOCK_TYPES

#[derive(Component)]
struct NormalBlock;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        
        .add_startup_system(startup_system)

        .add_system(should_move_block_system)
        .add_system(game_time_system)
        .add_system(spawn_block_system)
        
        .add_event::<GameOverEvent>()
        .add_event::<MoveEvent>()
        .add_event::<SpawnBlockEvent>()
        
        .insert_resource(Score (0))
        
        .run();
}

// --- Startup systems ---
fn startup_system(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
}

// --- Normal Systems ---
fn spawn_single_block_system(commands: &mut Commands, translation: Vec2, color: Color) -> Entity{
    if DBG_MODE {
        println!("=> Spawned block: {translation}");
    }
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::from((translation.x * BLOCK_SIZE, translation.y * BLOCK_SIZE, 0.)),
            scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.),
            ..Default::default()
        },
        sprite: Sprite {
            color: color,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(NormalBlock)
    .id()
}

// Tests whether the blocks should move and if they should moves them.
// If no blocks move it sends the event
// SpawnBlockEvent
fn should_move_block_system (
    mut move_event: EventReader<MoveEvent>,
    mut spawn_block_event: EventWriter<SpawnBlockEvent>,
    mut blocks_query: Query<(&mut Transform, &BlockParent), With<BlockParent>>,
    all_blocks_query: Query<&Transform, (With<NormalBlock>, Without<BlockParent>)>,
) {
    for _ in move_event.iter() {
        if DBG_MODE {
            println!("@should_move_block_system: checking whether blocks should move...\n");
        }
        let mut block_moved = false;
        
        for (mut parent_transform, block_parent) in blocks_query.iter_mut() {
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

            if DBG_MODE {
                println!("=> translation: {:?}", &blocks_translations);
            }

            let mut should_move = true;
            for translation in blocks_translations.iter() {
                // Hit the floor
                if translation.y <= -LIMITS.y * BLOCK_SIZE {
                    should_move = false;
                    break;
                }
                
                // Check for collisions
                // let hitbox = Vec2::splat(BLOCK_SIZE + 1.); // The hitbox has to be slightly bigger than the block

                for other_blocks_translation in all_blocks_query.iter() {
                    let other_blocks_translation = other_blocks_translation.translation;

                    // Check whether they don't have the same parent
                    if blocks_translations.contains(&other_blocks_translation) {
                        continue;
                    }

                    if other_blocks_translation.y == translation.y - BLOCK_SIZE && other_blocks_translation.x == translation.x {
                        if DBG_MODE {
                            println!("=> Block at {} {} with {} {} | Collided!", translation.x, translation.y, other_blocks_translation.x, other_blocks_translation.y);
                        }
                        should_move = false;
                        break;
                    }

                    // match collide(*translation, hitbox, other_blocks_translation, hitbox) {
                    //     Some(collision) => {
                    //         if DBG_MODE {
                    //             println!("=> Block at {} {} with {} {} | Collided!: {:?}", translation.x, translation.y, other_blocks_translation.x, other_blocks_translation.y, collision);
                    //         }
                    //         match collision {
                    //             bevy::sprite::collide_aabb::Collision::Top |
                    //             bevy::sprite::collide_aabb::Collision::Bottom => {
                    //                 should_move = false;
                    //                 break;
                    //             },
                    //             _ => ()
                    //         };
                    //     },
                    //     None => (),
                    // }
                }

                if !should_move {
                    break;
                }
            }

            if should_move {
                block_moved = true;
                parent_transform.translation.y -= BLOCK_SIZE;
            }
        }

        if !block_moved {
            if DBG_MODE {
                println!("Spawning blocks...");
            }
            spawn_block_event.send(SpawnBlockEvent);
        }
    }
}

fn spawn_block_system (
    mut read_event: EventReader<SpawnBlockEvent>,
    mut commands: Commands,
) {
    for _ in read_event.iter() {
        // TODO spawn random block here (index random)
        let index = 0;
        let blocks = BLOCK_TYPES[index];
        
        let red = rand::thread_rng().gen_range(0..100) as f32 / 100.;
        let blue = rand::thread_rng().gen_range(0..100) as f32 / 100.;
        let green = rand::thread_rng().gen_range(0..100) as f32 / 100.;
        let color = Color::rgb(red, green, blue);

        let mut children = Vec::new();
        for translation in blocks {
            let id = spawn_single_block_system(&mut commands, translation, color);
            children.push(id);

        }

        // The parent is invisible
        let parent = commands.spawn(
            SpriteBundle {
                sprite: Sprite { 
                    color: Color::rgba(0., 0., 0., 0.),
                    ..Default::default()
                },
                ..Default::default()
            }
        )
        .insert(BlockParent (index))
        .id();

        children.iter().for_each(|child| { commands.entity(parent).add_child(*child); });
    }
}

fn game_time_system (
    mut timer: Local<MainGameTimer>,
    kb: Res<Input<KeyCode>>,
    mut event: EventWriter<MoveEvent>,
    time: Res<Time>,
) {
    if timer.0.finished() {
        event.send(MoveEvent);
        timer.0.reset();
        if DBG_MODE {
            println!("@game_time_system: Refreshed game!");
        }
        return;
    }

    if kb.pressed(KeyCode::Down) {
        timer.0.tick(
            Duration::from_millis(
                ((time.delta_seconds() * DOWN_KEY_MULTIPLIER) * 1000. ) as u64)
        );
        return;
    }

    timer.0.tick(time.delta());
}

#[cfg(test)]
mod tests {
    use super::*;
}
