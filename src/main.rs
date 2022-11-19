use bevy::{prelude::*, sprite::collide_aabb::collide};
use itertools::izip;
use rand::{self, Rng};
use std::{fmt::Display, time::Duration};

mod consts;
use consts::*;

// --- Events ---
struct SpawnBlockEvent;
struct GameOverEvent;

struct MoveDownEvent;
struct MoveSidesEvent;
// --- Resources ---
#[derive(Resource)]
struct Score(usize);

#[derive(Resource)]
struct MainGameTimer(Timer);
impl Default for MainGameTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(GAME_SPEED, TimerMode::Repeating))
    }
}

// --- Components ---
#[derive(Component)]
struct BlockParent {
    index: usize,                  // The Index of BLOCK_TYPES
    despawned_children: Vec<Vec2>, // The vectors of the children who have been despawned
    moving: bool,
}

#[derive(Component)]
struct NormalBlock {
    parent: Entity,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup_system)
        .add_system(should_move_block_system)
        .add_system(game_time_system)
        .add_system(spawn_block_system)
        .add_system(move_sideways_system)
        .add_event::<GameOverEvent>()
        .add_event::<MoveDownEvent>()
        .add_event::<SpawnBlockEvent>()
        .add_event::<MoveSidesEvent>()
        .insert_resource(Score(0))
        .run();
}

// --- Startup systems ---
fn startup_system(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
}

// --- Normal Systems ---
fn spawn_single_block_system(commands: &mut Commands, translation: Vec2, color: Color, parent: Entity) -> Entity {
    if DBG_MODE {
        println!("=> Spawned block: {translation}");
    }
    commands
        .spawn(SpriteBundle {
            transform: Transform {
                translation: Vec3::from((
                    translation.x * BLOCK_SIZE,
                    translation.y * BLOCK_SIZE,
                    0.,
                )),
                scale: Vec3::new(BLOCK_SIZE, BLOCK_SIZE, 0.),
                ..Default::default()
            },
            sprite: Sprite {
                color: color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(NormalBlock { parent })
        .id()
}

/// Tests whether the blocks should move and if they should moves them.
/// If no blocks move it sends the event
/// SpawnBlockEvent
/// It also tests whether the game should end, case which it will send the message
/// GameOverEvent
// Note: Why is this so different from  move_sideways_system?
// I thought of a better workaround than the one I had used,
// But saw almost no benefit in rewriting this.
fn should_move_block_system(
    mut move_event: EventReader<MoveDownEvent>,
    mut spawn_block_event: EventWriter<SpawnBlockEvent>,
    mut blocks_query: Query<(&mut Transform, &mut BlockParent), With<BlockParent>>,
    mut game_over_event: EventWriter<GameOverEvent>,
) {
    for _ in move_event.iter() {
        if DBG_MODE {
            println!("@should_move_block_system: checking whether blocks should move...\n");
        }
        let mut block_moved = false;

        let mut block_index = 0;

        // This is a workaround since we have nested query loops
        // This is ONLY used for the nested loop, not the main one
        let mut ptvec = Vec::new();
        let mut bpvec = Vec::new();
        let mut dcvec = Vec::new();
        for (parent_transform, block_parent) in blocks_query.iter() {
            ptvec.push(parent_transform.translation.clone());
            bpvec.push(block_parent.index);
            dcvec.push(block_parent.despawned_children.clone());
        }
        for (mut parent_transform, mut block_parent) in blocks_query.iter_mut() {
            // -- Check whether the block parent should move --
            // Conputes the hitboxes of all the blocks that make
            // Up the block parent and sees if they:
            // - Hit the floor
            // - Hit another block below them
            // - Hit another block while on the roof (And then the game should end)

            let block_parent_vec = BLOCK_TYPES[block_parent.index];
            let mut blocks_translations = Vec::new();

            for translation in block_parent_vec.iter() {
                if !block_parent.despawned_children.contains(translation) {
                    blocks_translations
                        .push(parent_transform.translation + (translation.extend(0.) * BLOCK_SIZE));
                }
            }

            if DBG_MODE {
                println!(
                    "=> translation of block {}: {:?}",
                    block_index, &blocks_translations
                );
            }

            let mut should_move = true;
            for translation in blocks_translations.iter() {
                // Hit the floor
                if translation.y <= -LIMITS.y * BLOCK_SIZE {
                    if DBG_MODE {
                        println!("==> Hit the floor!");
                    }
                    should_move = false;
                    break;
                }

                // Check for collisions
                //  NOTE: This is a workaround. What I initially meant to do was use the children's positions
                //  To calculate the collisions, but that revealed impossible as ***APARENTLY***  bevy calculates
                //  The children's translations by using the parent's. This means that without getting the parent
                //  Of the block I will not be able to use the old code, and I figured this was less work and less
                //  Messy too.

                for (
                    other_parents_translation,
                    other_parents_block,
                    other_parents_despawned_children,
                ) in izip!(&ptvec, &bpvec, &dcvec)
                {
                    let other_parents_block_vec = BLOCK_TYPES[*other_parents_block];

                    // If it is the same block
                    if *other_parents_translation == parent_transform.translation {
                        continue;
                    }

                    for other_translation in other_parents_block_vec.iter() {
                        if other_parents_despawned_children.contains(other_translation) {
                            continue;
                        }

                        let other_translation = *other_parents_translation
                            + (other_translation.extend(0.) * BLOCK_SIZE);

                        if other_translation.y == translation.y - BLOCK_SIZE
                            && other_translation.x == translation.x
                        {
                            if DBG_MODE {
                                println!(
                                    "==> Block  {:?} collided with block {:?}",
                                    other_translation, translation
                                );
                            }

                            should_move = false;
                            break;
                        }
                    }
                }
                if !should_move {
                    break;
                }
            }

            if should_move {
                block_moved = true;
                parent_transform.translation.y -= BLOCK_SIZE;
            }

            if !should_move {
                block_parent.moving = false;
            }

            // Test game over
            if !should_move && parent_transform.translation.y == 0. {
                game_over_event.send(GameOverEvent);
                // PlaceHolder
                panic!("Game over!");
            }

            if DBG_MODE {
                block_index += 1;
            }
        }

        if !block_moved {
            if DBG_MODE {
                println!("No blocks moved! Spawning new blocks...");
            }
            spawn_block_event.send(SpawnBlockEvent);
        }
    }
}

fn spawn_block_system(mut read_event: EventReader<SpawnBlockEvent>, mut commands: Commands) {
    for _ in read_event.iter() {
        let index = rand::thread_rng().gen_range(0..BLOCK_TYPES.len());
        let blocks = BLOCK_TYPES[index];

        let red = rand::thread_rng().gen_range(0..100) as f32 / 100.;
        let blue = rand::thread_rng().gen_range(0..100) as f32 / 100.;
        let green = rand::thread_rng().gen_range(0..100) as f32 / 100.;
        let color = Color::rgb(red, green, blue);

        let mut children = Vec::new();

        // The parent is invisible
        let parent = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0., 0., 0., 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BlockParent {
                index,
                despawned_children: Vec::new(),
                moving: true,
            })
            .id();

        for translation in blocks {
            let id = spawn_single_block_system(&mut commands, translation, color, parent);
            children.push(id);
        }

        children.iter().for_each(|child| {
            commands.entity(parent).add_child(*child);
        });
    }
}

fn game_time_system(
    mut timer_down: Local<MainGameTimer>,
    mut timer_sides: Local<MainGameTimer>,
    kb: Res<Input<KeyCode>>,
    mut event_down: EventWriter<MoveDownEvent>,
    mut event_sides: EventWriter<MoveSidesEvent>,
    time: Res<Time>,
) {
    // Move downwards
    if timer_down.0.finished() {
        event_down.send(MoveDownEvent);
        timer_down.0.reset();
        if DBG_MODE {
            println!("@game_time_system: Refreshed game!");
        }
    } else if kb.pressed(KeyCode::Down) {
        timer_down.0.tick(Duration::from_millis(
            ((time.delta_seconds() * DOWN_KEY_MULTIPLIER) * 1000.) as u64,
        ));
    } else {
        timer_down.0.tick(time.delta());
    }

    // Move sideways
    if timer_sides.0.finished() {
        event_sides.send(MoveSidesEvent);
        timer_sides.0.reset();
    } else {
        timer_sides.0.tick(Duration::from_millis(
            ((time.delta_seconds() * RELATIVE_SIDES_MOVING_SPEED) * 1000.) as u64,
        ));
    }
}

fn move_sideways_system (
    mut event: EventReader<MoveSidesEvent>, 
    kb: Res<Input<KeyCode>>, 
    mut parents_query: Query<(Entity, &BlockParent, &mut Transform), Without<NormalBlock>>,
    children_query: Query<(&NormalBlock, &GlobalTransform), Without<BlockParent>>
) {
    for _ in event.iter() {
        if !(kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::Right)) {
            return;
        }

        let mut can_move_right = true;
        let mut can_move_left = true;


        for (parent_entity, block_parent, mut parent_transform) in parents_query.iter_mut() {
            
            // Move only the block that's moving
            if !block_parent.moving {
                continue;
            }

            let block_parent_vec = BLOCK_TYPES[block_parent.index];
            let mut blocks_translations = Vec::new();

            for translation in block_parent_vec.iter() {
                if !block_parent.despawned_children.contains(translation) {
                    blocks_translations
                        .push(parent_transform.translation + (translation.extend(0.) * BLOCK_SIZE));
                }
            }

            // Get the children's positions
            for (child_block, child_transform) in children_query.iter() {
                // Don't get the children of the parent we are currently checking
                if parent_entity == child_block.parent {
                    // Check whether it hit the walls
                    // let translation = child_transform.translation();
                    // if translation.x >= LIMITS.x * BLOCK_SIZE {
                    //     can_move_right = false;
                    // }
                    // if translation.x <= -LIMITS.x * BLOCK_SIZE {
                    //     can_move_right = false;
                    // }
                    continue;
                }


                let child_translation = child_transform.translation();
                for translation in blocks_translations.iter() {
                    if translation.x - BLOCK_SIZE == child_translation.x && translation.y == child_translation.y{
                        can_move_left = false;
                    }
                    if translation.x + BLOCK_SIZE == child_translation.x && translation.y == child_translation.y{
                        can_move_right = false;
                    }
                }
            }
            if can_move_right && kb.pressed(KeyCode::Right) {
                parent_transform.translation.x += BLOCK_SIZE;
            }
            if can_move_left && kb.pressed(KeyCode::Left) {
                parent_transform.translation.x -= BLOCK_SIZE;
            }
        }
    }
}
