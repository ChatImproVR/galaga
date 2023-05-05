use cimvr_engine_interface::{dbg, make_app_state, pcg::Pcg, pkg_namespace, prelude::*, FrameTime};

use cimvr_common::{
    desktop::{InputEvent, KeyCode},
    gamepad::{Axis, Button, GamepadState},
    glam::{EulerRot, Quat, Vec3},
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::input_helper::InputHelper,
    Transform,
};
use obj_reader::obj::obj_lines_to_mesh;
use serde::{Deserialize, Serialize};

// Create some constant value for Windows
const WITDH: f32 = 80.;
const HEIGHT: f32 = 120.;

// Create some constant values for Enemy
const ENEMY_COUNT: u32 = 2;
const ENEMY_MAX_BULLET: u32 = 5;
const ENEMY_SPAWN_TIME: f32 = 0.5;
const ENEMY_BULLET_SPEED: f32 = 100.;
const ENEMY_SPEED: f32 = 50.;

// Create some constant values for Player
const PLAYER_SPAWN_TIME: f32 = 3.0;
const PLAYER_BULLET_SPEED: f32 = 100.;
const PLAYER_SPEED: f32 = 100.;

// All state associated with client-side behaviour
#[derive(Default)]
struct ClientState {
    input: InputHelper,
}

// Add movement command
#[derive(Message, Serialize, Deserialize, Clone, Copy)]
#[locality("Remote")]
struct MoveCommand(Vec3);

// Add fire command
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
#[locality("Remote")]
struct FireCommand(bool);

// Add Player Component
#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Player {
    pub current_position: Vec3,
}

// Implement Default for Player Component
impl Default for Player {
    fn default() -> Self {
        Self {
            current_position: Vec3::new(0.0, -50.0, 0.0),
        }
    }
}

// Add Enemy Component
#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Enemy {
    pub current_position: Vec3,
    pub bullet_count: u32,
}

// Implement Default for Enemy Component
impl Default for Enemy {
    fn default() -> Self {
        Self {
            current_position: Vec3::new(0.0, 50.0, 0.0),
            bullet_count: 0,
        }
    }
}

// Add Bullet Component
#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Bullet {
    from_player: bool,
    from_enemy: bool,
    entity_id: EntityId,
}

// Implement Default for Bullet Component
impl Default for Bullet {
    fn default() -> Self {
        Self {
            from_player: false,
            from_enemy: false,
            entity_id: EntityId(0),
        }
    }
}

// Add Player Status Component; this is used as a spwan timer for Player
#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct PlayerStatus {
    pub status: bool,
    pub dead_time: f32,
}

// Implement Default for Player Status Component
impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            status: true,
            dead_time: 0.0,
        }
    }
}

// Add Enemy Status Component; this is used as a spwan timer for Enemy
#[derive(Component, Serialize, Deserialize, Copy, Clone, Default)]
pub struct EnemyStatus(f32);

// Add Score Component
#[derive(Component, Serialize, Deserialize, Copy, Clone, Default)]
pub struct Score(u32);

// Create mesh handleer based on each object's name
const PLAYER_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player"));
const ENEMY_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy"));
const PLAYER_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player Bullet"));
const ENEMY_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy Bullet"));
const WINDOW_SIZE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Window Size"));

// Create Meshes for each object

// Create the Player Mesh --> This is commented out because we are using obj file
// fn player() -> Mesh {
//     let size: f32 = 3.;

//     let vertices = vec![
//         Vertex::new([-size, -size, 0.0], [0.0, 0.0, 1.0]), // Vertex 0
//         Vertex::new([size, -size, 0.0], [0.0, 0.0, 1.0]),  // Vertex 1
//         Vertex::new([size, size, 0.0], [0.0, 0.0, 1.0]),   // Vertex 2
//         Vertex::new([-size, size, 0.0], [0.0, 0.0, 1.0]),  // Vertex 3
//     ];

//     let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

//     Mesh { vertices, indices }
// }

// // Create the Enemy Mesh --> This is commented out because we are using obj file
// fn enemy() -> Mesh {
//     let size: f32 = 3.;

//     let vertices = vec![
//         Vertex::new([-size, -size, 0.0], [1.0, 0.0, 0.0]), // Vertex 0
//         Vertex::new([size, -size, 0.0], [1.0, 0.0, 0.0]),  // Vertex 1
//         Vertex::new([size, size, 0.0], [1.0, 0.0, 0.0]),   // Vertex 2
//         Vertex::new([-size, size, 0.0], [1.0, 0.0, 0.0]),  // Vertex 3
//     ];

//     let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

//     Mesh { vertices, indices }
// }

// Create Player Bullet Mesh as a sqaure green
fn player_bullet() -> Mesh {
    let size: f32 = 0.5;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([size, -size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([size, size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([-size, size, 0.0], [0.0, 1.0, 0.0]),
    ];

    let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

    Mesh { vertices, indices }
}

// Create Enemy Bullet Mesh as a sqaure red
fn enemy_bullet() -> Mesh {
    let size: f32 = 0.5;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([size, -size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([size, size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([-size, size, 0.0], [1.0, 0.0, 0.0]),
    ];

    let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

    Mesh { vertices, indices }
}

// Create Window Mesh so that the users will know what is the limit of movement
fn window_size() -> Mesh {
    let vertices = vec![
        Vertex::new([-WITDH / 2., -HEIGHT / 2., 0.0], [1.; 3]),
        Vertex::new([WITDH / 2., -HEIGHT / 2., 0.0], [1.; 3]),
        Vertex::new([WITDH / 2., HEIGHT / 2., 0.0], [1.; 3]),
        Vertex::new([-WITDH / 2., HEIGHT / 2., 0.0], [1.; 3]),
    ];

    let indices: Vec<u32> = vec![3, 0, 0, 1, 1, 2, 2, 3];

    Mesh { vertices, indices }
}

// Create a struct for the Client State
impl UserState for ClientState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Declare the player color as green
        let player_color = [0., 1., 0.];

        // Read the player object file from the assets folder (that is created from blender)
        let mut new_player_mesh = obj_lines_to_mesh(&include_str!("assets/galagaship.obj"));

        // Update the player object/mesh with the player color
        new_player_mesh
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = player_color);

        // Declare the enemy color as red
        let enemy_color = [1., 0., 0.];

        // Read the enemy object file from the assets folder (that is created from blender)
        let mut new_enemy_mesh = obj_lines_to_mesh(&include_str!("assets/galaga_enemy.obj"));

        // Update the enemy object/mesh with the enemy color
        new_enemy_mesh
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = enemy_color);

        // Send the player mesh and the player mesh handler to the server side
        io.send(&UploadMesh {
            id: PLAYER_HANDLE,
            mesh: new_player_mesh,
        });

        // Send the enemy mesh and the enemy mesh handler to the server side
        io.send(&UploadMesh {
            id: ENEMY_HANDLE,
            mesh: new_enemy_mesh,
        });

        // Send the player bullet mesh and the player bullet mesh handler to the server side
        io.send(&UploadMesh {
            id: PLAYER_BULLET_HANDLE,
            mesh: player_bullet(),
        });

        // Send the enemy bullet mesh and the enemy bullet mesh handler to the server side
        io.send(&UploadMesh {
            id: ENEMY_BULLET_HANDLE,
            mesh: enemy_bullet(),
        });

        // Send the window mesh and the window mesh handler to the server side
        io.send(&UploadMesh {
            id: WINDOW_SIZE_HANDLE,
            mesh: window_size(),
        });

        // Add player movement input based on keyboard/controller input
        sched
            .add_system(Self::player_input_movement_update)
            .subscribe::<InputEvent>()
            .subscribe::<GamepadState>()
            .subscribe::<FrameTime>()
            .build();

        // Add player fire input based on keyboard/controller input
        sched
            .add_system(Self::player_input_fire_update)
            .subscribe::<InputEvent>()
            .subscribe::<GamepadState>()
            .build();

        Self::default()
    }
}

// Implement client only side functions that will send messages to the server side
impl ClientState {
    // Send the player movement input to the server side
    fn player_input_movement_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Declare the player movement direction as a vector: initially zero
        let mut direction = Vec3::ZERO;

        // Read the frame time from the engine
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        // Read the input events from the keyboard
        self.input.handle_input_events(io);

        // Add controller deadsone
        let deadzone = 0.3;

        // Read the gamepad state from the engine
        if let Some(GamepadState(gamepads)) = io.inbox_first() {
            // If gamepad input was received, check the left stick x axis
            if let Some(gamepad) = gamepads.into_iter().next() {
                // If the left stick x axis is less than the deadzone, move left by one unit
                if gamepad.axes[&Axis::LeftStickX] < -deadzone {
                    direction += Vec3::new(-1.0, 0.0, 0.0);
                }

                // If the left stick x axis is greater than the deadzone, move right by one unit
                if gamepad.axes[&Axis::LeftStickX] > deadzone {
                    direction += Vec3::new(1.0, 0.0, 0.0);
                }
            }

            // If the keyboard input was received and the key A was pressed & held, move left by one unit
            if self.input.key_held(KeyCode::A) {
                direction += Vec3::new(-1.0, 0.0, 0.0);
            }

            // If the keyboard input was received and the key D was pressed & held, move right by one unit
            if self.input.key_held(KeyCode::D) {
                direction += Vec3::new(1.0, 0.0, 0.0);
            }

            // If there was an update in the direction vector (that is no longer zero), send the movement command to the server side
            if direction != Vec3::ZERO {
                // Recalculate the direction vector based on the frame time and the player speed
                let distance = direction.normalize() * frame_time.delta * PLAYER_SPEED;

                // Create the Move command
                let command = MoveCommand(distance);

                // Send the command to the server side
                io.send(&command);
            }
        }
    }

    // Send the player fire input to the server side
    fn player_input_fire_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Read the input events from the keyboard
        self.input.handle_input_events(io);

        // Read the gamepad state from the engine
        if let Some(GamepadState(gamepads)) = io.inbox_first() {
            // If gamepad input was received
            if let Some(gamepad) = gamepads.into_iter().next() {
                // Check if the East side button on the right side of the controller was triggered
                if gamepad.buttons[&Button::East] {
                    // Create the Fire command
                    let command = FireCommand(true);
                    // Send the command to the server side
                    io.send(&command);
                }
            }
        }

        // If the keyboard input was received and the key Space was pressed, send the fire command to the server side
        if self.input.key_pressed(KeyCode::Space) {
            let command = FireCommand(true);
            io.send(&command);
        }
    }
}

// All state associated with server-side behaviour
struct ServerState;

// Implement server only side functions that will update on the server side
impl UserState for ServerState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Create a player status entity (not player entity)
        io.create_entity()
            // Add the player status component with the default values
            .add_component(PlayerStatus::default())
            // Build the entity
            .build();

        // Create an enemy status entity (not enemy entity)
        io.create_entity().add_component(EnemyStatus(0.0)).build();

        // Create a score entity
        io.create_entity()
            // Add the score component with the initial score of 0
            .add_component(Score(0))
            // Build the entity
            .build();

        // Create Player entity with components
        io.create_entity()
            // Add the transform component for movement
            .add_component(
                // Add the default transform component
                Transform::default()
                    // Set the bottom middle of the screen as the initial position
                    .with_position(Vec3::new(0.0, -50.0, 0.0))
                    // Set the initial rotation to be facing towards to the player based on the camera angle (no needed if you create the object facing a different direction)
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
            )
            // Add the render component to draw the player with lines
            .add_component(Render::new(PLAYER_HANDLE).primitive(Primitive::Lines))
            // Add the player component as default
            .add_component(Player::default())
            // Add the synchronized component to synchronize the entity with the client side
            .add_component(Synchronized)
            // Build the entity
            .build();

        // Create Enemy with components
        io.create_entity()
            // Add the transform component for movement, firing, and displaying
            .add_component(
                // Add the default transform component
                Transform::default()
                    // Set the top middle of the screen as the initial position
                    .with_position(Vec3::new(0.0, 50.0, 0.0))
                    // Set the initial rotation to be facing towards to the player based on the camera angle
                    // (no needed if you create the object facing a different direction or differen angle rotation)
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
            )
            // Add the render component to draw the enemy with lines
            .add_component(Render::new(ENEMY_HANDLE).primitive(Primitive::Lines))
            // Add the synchronized component to synchronize the entity with the client side
            .add_component(Synchronized)
            // Add the enemy component as default
            .add_component(Enemy::default())
            // Build the entity
            .build();

        // Create the Window entity with components
        io.create_entity()
            // Add the transform component for displaying the window
            .add_component(Transform::default())
            // Add the render component to draw the window with lines
            .add_component(Render::new(WINDOW_SIZE_HANDLE).primitive(Primitive::Lines))
            // Add the synchronized component to synchronize the entity with the client side
            .add_component(Synchronized)
            // Build the entity
            .build();

        sched
            .add_system(Self::spawn_player)
            .subscribe::<FrameTime>()
            .query(
                "Player",
                Query::new().intersect::<PlayerStatus>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::spawn_enemy)
            .subscribe::<FrameTime>()
            .query(
                "Enemy_Count",
                Query::new().intersect::<Enemy>(Access::Write),
            )
            .query(
                "Enemy_Status",
                Query::new().intersect::<EnemyStatus>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::player_movement_update)
            .subscribe::<MoveCommand>()
            .query(
                "Player_Movement",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Player>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::enemy_movement_update)
            .subscribe::<FrameTime>()
            .query(
                "Enemy_Movement",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Enemy>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::player_fire_update)
            .subscribe::<FireCommand>()
            .query(
                "Player_Fire_Input",
                Query::new().intersect::<Player>(Access::Read),
            )
            .build();

        sched
            .add_system(Self::player_bullet_movement_update)
            .subscribe::<FrameTime>()
            .query(
                "Player_Bullet_Movement",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Bullet>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::enemy_fire_update)
            .query(
                "Enemy_Fire_Input",
                Query::new().intersect::<Enemy>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::enemy_bullet_movement_update)
            .subscribe::<FrameTime>()
            .query(
                "Enemy_Bullet_Movement",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Bullet>(Access::Write),
            )
            .query(
                "Enemy_Bullet_Count_Update",
                Query::new().intersect::<Enemy>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::player_bullet_to_enemy_collision)
            .query(
                "Player_Bullet",
                Query::new()
                    .intersect::<Transform>(Access::Read)
                    .intersect::<Bullet>(Access::Write),
            )
            .query(
                "Enemy",
                Query::new()
                    .intersect::<Enemy>(Access::Write)
                    .intersect::<Transform>(Access::Read),
            )
            .query(
                "Score_Update",
                Query::new().intersect::<Score>(Access::Write),
            )
            .build();

        sched
            .add_system(Self::enemy_bullet_to_player_collision)
            .query(
                "Enemy_Bullet",
                Query::new()
                    .intersect::<Transform>(Access::Read)
                    .intersect::<Bullet>(Access::Write),
            )
            .query(
                "Player",
                Query::new()
                    .intersect::<Player>(Access::Write)
                    .intersect::<Transform>(Access::Read),
            )
            .query(
                "Player_Status_Update",
                Query::new().intersect::<PlayerStatus>(Access::Write),
            )
            .query(
                "Score_Update",
                Query::new().intersect::<Score>(Access::Write),
            )
            .build();

        Self
    }
}

impl ServerState {
    fn spawn_player(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        for entity in query.iter("Player") {
            if !(query.read::<PlayerStatus>(entity).status) {
                let mut dead_time = query.read::<PlayerStatus>(entity).dead_time;
                if dead_time == 0.0 {
                    dead_time = frame_time.time;
                }
                if dead_time + PLAYER_SPAWN_TIME < frame_time.time {
                    io.create_entity()
                        .add_component(
                            Transform::default()
                                .with_position(Vec3::new(0.0, -50.0, 0.0))
                                .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
                        )
                        .add_component(Render::new(PLAYER_HANDLE).primitive(Primitive::Lines))
                        .add_component(Player::default())
                        .add_component(Synchronized)
                        .build();
                    io.remove_entity(entity);
                    io.create_entity()
                        .add_component(PlayerStatus::default())
                        .build();
                } else {
                    query.modify::<PlayerStatus>(entity, |value| {
                        value.dead_time = dead_time;
                    })
                }
            }
        }
    }

    fn spawn_enemy(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        if (query.iter("Enemy_Count").count() as u32) < ENEMY_COUNT {
            for entity in query.iter("Enemy_Status") {
                let mut dead_time = query.read::<EnemyStatus>(entity).0;

                if dead_time == 0.0 {
                    dead_time = frame_time.time;
                }

                if dead_time + ENEMY_SPAWN_TIME < frame_time.time {
                    io.create_entity()
                        .add_component(
                            Transform::default()
                                .with_position(Vec3::new(0.0, 50.0, 0.0))
                                .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
                        )
                        .add_component(Render::new(ENEMY_HANDLE).primitive(Primitive::Lines))
                        .add_component(Synchronized)
                        .add_component(Enemy::default())
                        .build();
                    io.remove_entity(entity);
                    io.create_entity().add_component(EnemyStatus(0.0)).build();
                } else {
                    query.modify::<EnemyStatus>(entity, |value| {
                        value.0 = dead_time;
                    })
                }
            }
        }
    }

    fn player_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for player_movement in io.inbox::<MoveCommand>() {
            for entity in query.iter("Player_Movement") {
                let x_limit = WITDH / 2.0;
                if query.read::<Player>(entity).current_position.x + player_movement.0.x - 3.
                    < -x_limit
                    || query.read::<Player>(entity).current_position.x + player_movement.0.x + 3.
                        > x_limit
                {
                    return;
                }

                query.modify::<Transform>(entity, |transform| {
                    transform.pos += player_movement.0;
                });
                query.modify::<Player>(entity, |player| {
                    player.current_position += player_movement.0;
                });
            }
        }
    }

    fn enemy_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for entity in query.iter("Enemy_Movement") {
            let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

            let mut pcg_random_move = Pcg::new();
            let mut pcg_random_direction = Pcg::new();

            let x = if pcg_random_direction.gen_bool() {
                pcg_random_move.gen_f32() * 1.
            } else {
                pcg_random_move.gen_f32() * -1.
            };

            let y = if pcg_random_direction.gen_bool() {
                pcg_random_move.gen_f32() * 1.
            } else {
                pcg_random_move.gen_f32() * -1.
            };

            let speed = Vec3::new(x, y, 0.);

            let direction = speed.normalize() * frame_time.delta * ENEMY_SPEED;
            let x_limit = WITDH / 2.0;
            let y_upper_limit = HEIGHT / 2.;
            let y_limit = HEIGHT / 5.;
            let current_position = query.read::<Enemy>(entity).current_position;

            if (current_position.x + direction.x - 3. < -x_limit)
                || (current_position.x + direction.x + 3. > x_limit)
                || (current_position.y + direction.y >= y_upper_limit)
                || (current_position.y + direction.y < y_limit)
            {
                return;
            }
            query.modify::<Transform>(entity, |transform| {
                transform.pos += direction;
            });
            query.modify::<Enemy>(entity, |enemy| {
                enemy.current_position += direction;
            });
        }
    }

    fn player_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(FireCommand(value)) = io.inbox_first() {
            for entity in query.iter("Player_Fire_Input") {
                io.create_entity()
                    .add_component(
                        Render::new(PLAYER_BULLET_HANDLE).primitive(Primitive::Triangles),
                    )
                    .add_component(Synchronized)
                    .add_component(Bullet {
                        from_enemy: false,
                        from_player: true,
                        entity_id: entity,
                    })
                    .add_component(Transform::default().with_position(
                        query.read::<Player>(entity).current_position + Vec3::new(-1.5, 1.5, 0.0),
                    ))
                    .build();

                io.create_entity()
                    .add_component(
                        Render::new(PLAYER_BULLET_HANDLE).primitive(Primitive::Triangles),
                    )
                    .add_component(Synchronized)
                    .add_component(Bullet {
                        from_enemy: false,
                        from_player: true,
                        entity_id: entity,
                    })
                    .add_component(Transform::default().with_position(
                        query.read::<Player>(entity).current_position + Vec3::new(1.5, 1.5, 0.0),
                    ))
                    .build();
            }
        }
    }

    fn player_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        for entity in query.iter("Player_Bullet_Movement") {
            if query.read::<Bullet>(entity).from_player {
                if query.read::<Transform>(entity).pos.y > HEIGHT / 2. - 2.5 {
                    io.remove_entity(entity);
                }
                query.modify::<Transform>(entity, |transform| {
                    transform.pos +=
                        Vec3::new(0.0, 1.0, 0.0) * frame_time.delta * PLAYER_BULLET_SPEED;
                });
            }
        }
    }

    fn enemy_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let mut pcg_fire = Pcg::new();

        for entity in query.iter("Enemy_Fire_Input") {
            if pcg_fire.gen_bool() {
                if query.read::<Enemy>(entity).bullet_count < ENEMY_MAX_BULLET {
                    query.modify::<Enemy>(entity, |value| {
                        value.bullet_count += 1;
                    });
                    io.create_entity()
                        .add_component(
                            Render::new(ENEMY_BULLET_HANDLE).primitive(Primitive::Triangles),
                        )
                        .add_component(Synchronized)
                        .add_component(Bullet {
                            from_enemy: true,
                            from_player: false,
                            entity_id: entity,
                        })
                        .add_component(Transform::default().with_position(
                            query.read::<Enemy>(entity).current_position + Vec3::new(0., 1.5, 0.),
                        ))
                        .build();
                }
            }
        }
    }

    fn enemy_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(frame_time) = io.inbox_first::<FrameTime>() {
            for entity in query.iter("Enemy_Bullet_Movement") {
                if query.read::<Bullet>(entity).from_enemy {
                    if query.read::<Transform>(entity).pos.y < -HEIGHT / 2. + 2.5 {
                        // If that enemy key exists in the game or not
                        if query
                            .iter("Enemy_Bullet_Count_Update")
                            .any(|id| id == query.read::<Bullet>(entity).entity_id)
                        {
                            query.modify::<Enemy>(
                                query.read::<Bullet>(entity).entity_id,
                                |value| {
                                    value.bullet_count -= 1;
                                },
                            );
                        }
                        io.remove_entity(entity);
                    }
                    query.modify::<Transform>(entity, |transform| {
                        transform.pos +=
                            Vec3::new(0.0, -1.0, 0.0) * frame_time.delta * ENEMY_BULLET_SPEED;
                    });
                }
            }
        }
    }

    fn player_bullet_to_enemy_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Size is 3 pixel sqaure
        for entity1 in query.iter("Player_Bullet") {
            if query.read::<Bullet>(entity1).from_player {
                for entity2 in query.iter("Enemy") {
                    let bullet_size = 0.5;
                    let enemy_size = 3.;
                    let current_player_bullet = query.read::<Transform>(entity1).pos;
                    let current_enemy = query.read::<Transform>(entity2).pos;

                    if collision_detection(
                        current_player_bullet.x,
                        current_player_bullet.y,
                        bullet_size,
                        current_enemy.x,
                        current_enemy.y,
                        enemy_size,
                    ) {
                        io.remove_entity(entity1);
                        io.remove_entity(entity2);
                        for entity3 in query.iter("Score_Update") {
                            query.modify::<Score>(entity3, |value| {
                                value.0 += 1;
                            });
                            dbg!(query.read::<Score>(entity3).0);
                        }
                    }
                }
            }
        }
    }

    fn enemy_bullet_to_player_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for entity1 in query.iter("Enemy_Bullet") {
            if query.read::<Bullet>(entity1).from_enemy {
                for entity2 in query.iter("Player") {
                    let bullet_size = 0.5;
                    let enemy_size = 3.;
                    let current_enemy_bullet = query.read::<Transform>(entity1).pos;
                    let current_player = query.read::<Transform>(entity2).pos;

                    if collision_detection(
                        current_enemy_bullet.x,
                        current_enemy_bullet.y,
                        bullet_size,
                        current_player.x,
                        current_player.y,
                        enemy_size,
                    ) {
                        io.remove_entity(entity1);
                        io.remove_entity(entity2);
                        for entity3 in query.iter("Player_Status_Update") {
                            query.modify::<PlayerStatus>(entity3, |value| {
                                value.status = false;
                            });
                        }
                        for entity4 in query.iter("Score_Update") {
                            query.modify::<Score>(entity4, |value| {
                                value.0 = 0;
                            });
                            dbg!(query.read::<Score>(entity4).0);
                        }
                    }
                }
            }
        }
    }
}

fn collision_detection(
    obj1_x_position: f32,
    obj1_y_position: f32,
    obj1_size: f32,
    obj2_x_position: f32,
    obj2_y_position: f32,
    obj2_size: f32,
) -> bool {
    if obj1_x_position - (obj1_size / 2.) <= obj2_x_position + (obj2_size / 2.)
        && obj1_x_position + (obj1_size / 2.) >= obj2_x_position - (obj2_size / 2.)
        && (obj1_y_position - (obj1_size / 2.) <= obj2_y_position + (obj2_size / 2.))
        && (obj1_y_position + (obj1_size / 2.) >= obj2_y_position - (obj2_size / 2.))
    {
        return true;
    }
    return false;
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);

// Score (if possible) --> just need to display on the client side
// Update README file as well
