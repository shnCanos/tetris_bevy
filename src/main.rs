use bevy::prelude::*;
use itertools::Itertools;
use rand::{self, Rng};
use std::time::Duration;

mod consts;
use consts::*;

// --- Events ---
struct SpawnBlockEvent;
struct GameOverEvent;

struct MoveDownEvent;
struct MoveSidesEvent;
struct RotatePieceEvent;
// --- Resources ---
#[derive(Resource)]
struct Score(usize);

#[derive(Resource)]
struct MainGameTimer {
    timer: Timer,
}
impl Default for MainGameTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(GAME_SPEED, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
struct GamePaused(bool);
impl Default for GamePaused {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(Resource)]
struct DestroyedRow (bool);

// --- Components ---
#[derive(Component)]
struct BlockParent {
    moving: bool,
}

#[derive(Component)]
struct NormalBlock {
    parent: Entity,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: (BLOCK_SIZE * LIMITS.x) * 2. - BLOCK_SIZE, // For unknown reasons, soulsparks' commit added 1 block to the limits. Easy fix
                height: (BLOCK_SIZE * LIMITS.y) * 2. + BLOCK_SIZE / 2., // A block spawned in (0,0) will have its center in (0,0), thus we need to add that last part or the blocks will be cut
                title: "Tetris, YAHOOOOOO".to_string(),
                resizable: false,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_startup_system(startup_system)
        .add_system(should_move_block_system)
        .add_system(game_time_system)
        .add_system(spawn_block_system)
        .add_system(move_sideways_system)
        .add_system(rotate_pieces_system)
        .add_system(game_over_system)
        .add_event::<GameOverEvent>()
        .add_event::<MoveDownEvent>()
        .add_event::<SpawnBlockEvent>()
        .add_event::<MoveSidesEvent>()
        .add_event::<MoveSidesEvent>()
        .add_event::<RotatePieceEvent>()
        .insert_resource(Score(0))
        .insert_resource(GamePaused::default())
        .insert_resource(DestroyedRow (false))
        .run();
}

// --- Startup systems ---
fn startup_system(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
}

// --- Normal Systems ---
fn spawn_single_block_system(
    commands: &mut Commands,
    translation: Vec2,
    color: Color,
    parent: Entity,
) -> Entity {
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

////////////////   I DISCOVERED IT FELLAS! IT'S THE SAME QUERY!!! THE QUERY IS NOT UPDATED!   ////////////////

/// Tests whether the blocks should move and if they should moves them.
/// If no blocks move it sends the event
/// SpawnBlockEvent
/// It also tests whether the game should end, case which it will send the message
/// GameOverEvent
fn should_move_block_system(
    move_event: EventReader<MoveDownEvent>,
    mut spawn_block_event: EventWriter<SpawnBlockEvent>,
    mut parents_query: Query<(Entity, &Children, &mut BlockParent, &mut Transform), Without<NormalBlock>>,
    mut game_over_event: EventWriter<GameOverEvent>,
    children_query: Query<(Entity, &GlobalTransform, &Parent), (Without<BlockParent>, With<NormalBlock>)>,
    mut commands: Commands,
    mut destroyed_row: ResMut<DestroyedRow>,
    mut score: ResMut<Score>
) {
    // -- BOTH OF THESE ONLY NEED TO BE SENT ONCE, THUS THE LACK OF A FOR LOOP --

    let got_move_event = !move_event.is_empty();
    move_event.clear();

    if !(got_move_event || destroyed_row.0) {
        return;
    }

    // If a block moves and a row was not destroyed this variable wil be false
    let mut should_spawn = true;


    'main_for_loop: for (parent_entity, children, mut block_parent, mut parent_transform) in parents_query.iter_mut() {
        // Check whether the game should end
        let mut check_game_over = || {
            if parent_transform.translation == Vec3::ZERO {
                game_over_event.send (GameOverEvent);
            }
        };


        // Ignore the moving block when a row is destroyed
        // if block_parent.moving && row_destroyed { continue; }
        
        // For each child block in the block that is moving, get their translation, and collect all of them into a vector
        // No need to worry about the BLOCK_TYPES nor the despawned_children!
        let blocks_translations = children
            .iter()
            .filter_map(|&e| children_query.get(e).ok())
            .map(|e| e.1.translation()) // Get the global translation of each child of the parent
            .collect_vec();
        
        if blocks_translations.is_empty() {
            commands.entity(parent_entity).despawn();
            continue;
        }

        // The children of the parent we are currently checking
        for translation in blocks_translations.iter() {
            // Check whether it hit the floor
            if translation.y - BLOCK_SIZE <= -LIMITS.y * BLOCK_SIZE {
                block_parent.moving = false;
                check_game_over();
                continue 'main_for_loop; // We check more than one block
            }
        }

        // Loop through the positions of each of the child blocks in the world
        // And check for collisions

        for translation in blocks_translations.iter() {
            for (_, child_transform, child_parent) in children_query.iter() {
                if parent_entity == child_parent.get() {
                    continue;
                }

                let child_translation = child_transform.translation();

                if translation.y - child_translation.y == BLOCK_SIZE && translation.x == child_translation.x {
                    block_parent.moving = false;
                    check_game_over();
                    continue 'main_for_loop;
                }
            }
        }

        // If it reaches this line it means that the block should move
        should_spawn = false;
        parent_transform.translation.y -= BLOCK_SIZE;
    }

    // This is meant for the specific case of 
    // Getting a row with a piece with nothing
    // below broken. Without this, this would
    // Mean that the blocks that were supposed
    // To teleport would move down once and
    // then take their time.
    if should_spawn && destroyed_row.0 {
        destroyed_row.0 = false;
    }

    if row_completed_function(&children_query, &mut commands, &parents_query, &mut score.0) {
        destroyed_row.0 = true;
    }

    if should_spawn && !destroyed_row.0 {
        if DBG_MODE {
            dbg!(should_spawn, destroyed_row.0);
        }
        spawn_block_event.send(SpawnBlockEvent);
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
    mut event_rotate: EventWriter<RotatePieceEvent>,
    time: Res<Time>,
    mut paused: ResMut<GamePaused>,
) {
    // Check if the game is paused
    if kb.just_pressed(KeyCode::Escape) {
        paused.0 = !paused.0;
    }
    if paused.0 {
        return;
    }

    // Move downwards
    if timer_down.timer.finished() {
        event_down.send(MoveDownEvent);
        timer_down.timer.reset();
        if DBG_MODE {
            println!("@game_time_system: Refreshed game!");
        }
    } else if kb.pressed(KeyCode::Down) {
        timer_down.timer.tick(Duration::from_millis(
            ((time.delta_seconds() * DOWN_KEY_MULTIPLIER) * 1000.) as u64,
        ));
    } else {
        timer_down.timer.tick(time.delta());
    }

    // Move sideways
    if timer_sides.timer.finished() {
        event_sides.send(MoveSidesEvent);
        timer_sides.timer.reset();
    } else {
        timer_sides.timer.tick(Duration::from_millis(
            ((time.delta_seconds() * RELATIVE_SIDES_MOVING_SPEED) * 1000.) as u64,
        ));
    }

    if kb.just_pressed(KeyCode::Z) {
        event_rotate.send(RotatePieceEvent);
    }
}

fn move_sideways_system (
    mut event: EventReader<MoveSidesEvent>, 
    kb: Res<Input<KeyCode>>,
    mut parents_query: Query<(Entity, &Children, &BlockParent, &mut Transform), Without<NormalBlock>>,
    children_query: Query<(&NormalBlock, &GlobalTransform), Without<BlockParent>>
) {
    'main_loop: for _ in event.iter() {
        let move_direction: f32 = (kb.pressed(KeyCode::Right) as i32 - kb.pressed(KeyCode::Left) as i32) as f32;

        if move_direction == 0. {
            return;
        }
        
        for (parent_entity, children, block_parent, mut parent_transform) in parents_query.iter_mut() {
            // Move only the block that's moving
            if !block_parent.moving {
                continue;
            }
            
            // For each child block in the block that is moving, get their translation, and collect all of them into a vector
            // No need to worry about the BLOCK_TYPES nor the despawned_children!
            let blocks_translations = children
                .iter()
                .filter_map(|&e| children_query.get(e).ok())
                .map(|e| e.1.translation()) // Get the global translation of each child of the parent
                .collect_vec();

            // Loop through the positions of each of the child blocks in the world
            for (child_block, child_transform) in children_query.iter() {
                // Don't get the children of the parent we are currently checking
                if parent_entity == child_block.parent {
                    // Check whether it hit the walls
                    let translation = child_transform.translation();
                    if translation.x >= (LIMITS.x - move_direction) * BLOCK_SIZE || translation.x <= -(LIMITS.x + move_direction) * BLOCK_SIZE {
                        break 'main_loop;
                    }

                    continue;
                }

                let child_translation = child_transform.translation();
                for translation in blocks_translations.iter() {
                    if translation.x + BLOCK_SIZE * move_direction == child_translation.x && translation.y == child_translation.y {
                        break 'main_loop;
                    }
                }
            }
            
            parent_transform.translation.x += BLOCK_SIZE * move_direction;
        }
    }
}
// This code probably has bugs
fn row_completed_function (
    block_query: &Query<(Entity, &GlobalTransform, &Parent), (Without<BlockParent>, With<NormalBlock>)>,
    commands: &mut Commands,
    parent_query: &Query<(Entity, &Children, &mut BlockParent, &mut Transform), Without<NormalBlock>>,
    score: &mut usize,
) -> bool {
    let mut rows: Vec<Vec<Vec2>> = Vec::new();
    for (_, ctransform, cparent) in block_query.iter() {
        if parent_query.get(cparent.get()).unwrap().2.moving {
            continue;
        }
        let translation = ctransform.translation().truncate();
        
        let mut create_new = true;
        for row in rows.iter_mut() {
            // Due to the fact that we use Vec::ZERO in the coordinates we don't use, we need to check
            // Whether the coordinates are the exact same as another block already in the list, or this
            // Thing goes absolutely nuts
            if !row.iter().filter(|x| x.y == translation.y ).collect_vec().is_empty() && !row.contains(&translation) {
                row.push(translation);
                create_new = false;
            }

            if row.contains(&translation) {
                create_new = false;
            }
        }

        if create_new {
            rows.push(vec![translation]);
        }   
    }

    for row in rows.iter() {
        // Despawn row
        if row.len() >= LIMITS.x as usize * 2 - 1{
            for (centity, ctransform, cparent) in block_query.iter() {
                if parent_query.get(cparent.get()).unwrap().2.moving {
                    continue;
                }
                if ctransform.translation().y == row[0].y // All the elements in row are the same
                {
                    commands.entity(centity).despawn();
                }
            }
            *score += SCORE_INCREMENT;
            return true;
        }
    }
    return false;
}

// Formerly known as "rakuna_matata()"
fn rotate_pieces_system (
    parents_query: Query<(Entity, &BlockParent), Without<NormalBlock>>,
    mut children_query: Query<(&Parent, &mut Transform), With<NormalBlock>>,
    mut event: EventReader<RotatePieceEvent>,
) {

    for _ in event.iter() {
        for (parent_entity, block_parent) in parents_query.iter() {
            // We only care about the moving one
            if !block_parent.moving {
                continue;
            }

            // Get the children
            for (child_parent, mut child_relative_transform) in children_query.iter_mut() {
                // We only care about the moving one
                if child_parent.get() != parent_entity {
                    continue;
                }

                // Rotate the piece
                // x = y and y = -x, if you don't understand you have a skill issue
                let temp_x = child_relative_transform.translation.x;
                child_relative_transform.translation.x = child_relative_transform.translation.y;
                child_relative_transform.translation.y = -temp_x

            }
        }
    }
}

use bevy::app::AppExit;
fn game_over_system (
    mut exit: EventWriter<AppExit>,
    mut game_over: EventReader<GameOverEvent>,
    score: Res<Score>
) {
    for _ in game_over.iter() {
        println!("You lost! Your score was: {}", score.0);
        exit.send(AppExit);
    }
}