// Add libraries from the cimvr_engine_interface crate
use cimvr_engine_interface::{dbg, make_app_state, pcg::Pcg, pkg_namespace, prelude::*, FrameTime};

// Add libraries from the cimvr_common crate
use cimvr_common::{
    desktop::{InputEvent, KeyCode},
    gamepad::{Axis, Button, GamepadState},
    glam::{EulerRot, Quat, Vec3},
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::input_helper::InputHelper,
    Transform,
};

// Add libraries from the obj_reader crate
use obj_reader::obj::obj_lines_to_mesh;

// Add libraries from the serde crate
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
const ENEMY_SIZE: f32 = 3.; // Because of the obj file, this value is not used (update this value after changing the obj size)

// Create some constant values for Player
const PLAYER_SPAWN_TIME: f32 = 3.0;
const PLAYER_BULLET_SPEED: f32 = 100.;
const PLAYER_SPEED: f32 = 100.;
const PLAYER_SIZE: f32 = 3.; // Because of the obj file, this value is not used (update this value after changing the obj size)

// Create some constant values for Bullet
const BULLET_SIZE: f32 = 0.5;

// All state associated with client-side behaviour
#[derive(Default)]
struct ClientState {
    input: InputHelper,
}

// Add movement command as message from client to server
#[derive(Message, Serialize, Deserialize, Clone, Copy)]
#[locality("Remote")]
struct MoveCommand(Vec3);

// Add fire command as a message from client to server
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
#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Score {
    pub score: u32,
    pub second_digit: u32,
    pub first_digit: u32,
    pub second_digit_entity: EntityId,
    pub first_digit_entity: EntityId,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            score: 0,
            second_digit: 10,
            first_digit: 10,
            second_digit_entity: EntityId(0),
            first_digit_entity: EntityId(0),
        }
    }
}

// Create mesh handleer based on each object's name
const PLAYER_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player"));
const ENEMY_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy"));
const PLAYER_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player Bullet"));
const ENEMY_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy Bullet"));
const WINDOW_SIZE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Window Size"));

const ZERO_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Zero Text"));
const ONE_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("One Text"));
const TWO_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Two Text"));
const THREE_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Three Text"));
const FOUR_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Four Text"));
const FIVE_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Five Text"));
const SIX_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Six Text"));
const SEVEN_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Seven Text"));
const EIGHT_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Eight Text"));
const NINE_TEXT_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Nine Text"));

// Create Meshes for each object

// Create the Player Mesh --> This is commented out because we are using obj file
// fn player() -> Mesh {
//     let size: f32 = PLAYER_SIZE;

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
//     let size: f32 = ENEMY_SIZE;

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
    let size: f32 = BULLET_SIZE;

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
    let size: f32 = BULLET_SIZE;

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

        // Declare the enemy color as faded gray
        let text_color = [0.37, 0.37, 0.37];

        let mut zero_text = obj_lines_to_mesh(&include_str!("assets/zero.obj"));

        zero_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: ZERO_TEXT_HANDLE,
            mesh: zero_text,
        });

        let mut one_text = obj_lines_to_mesh(&include_str!("assets/one.obj"));

        one_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: ONE_TEXT_HANDLE,
            mesh: one_text,
        });

        let mut two_text = obj_lines_to_mesh(&include_str!("assets/two.obj"));

        two_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: TWO_TEXT_HANDLE,
            mesh: two_text,
        });

        let mut three_text = obj_lines_to_mesh(&include_str!("assets/three.obj"));

        three_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: THREE_TEXT_HANDLE,
            mesh: three_text,
        });

        let mut four_text = obj_lines_to_mesh(&include_str!("assets/four.obj"));

        four_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: FOUR_TEXT_HANDLE,
            mesh: four_text,
        });

        let mut five_text = obj_lines_to_mesh(&include_str!("assets/five.obj"));

        five_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: FIVE_TEXT_HANDLE,
            mesh: five_text,
        });

        let mut six_text = obj_lines_to_mesh(&include_str!("assets/six.obj"));

        six_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: SIX_TEXT_HANDLE,
            mesh: six_text,
        });

        let mut seven_text = obj_lines_to_mesh(&include_str!("assets/seven.obj"));

        seven_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: SEVEN_TEXT_HANDLE,
            mesh: seven_text,
        });

        let mut eight_text = obj_lines_to_mesh(&include_str!("assets/eight.obj"));

        eight_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: EIGHT_TEXT_HANDLE,
            mesh: eight_text,
        });

        let mut nine_text = obj_lines_to_mesh(&include_str!("assets/nine.obj"));

        nine_text
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = text_color);

        io.send(&UploadMesh {
            id: NINE_TEXT_HANDLE,
            mesh: nine_text,
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
            .add_component(Score::default())
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

        // Attach Spawn Player Function to the Engine schedule
        sched
            // Add the spawn player system
            .add_system(Self::spawn_player)
            // Subscribe to the FrameTime event
            .subscribe::<FrameTime>()
            // Add the query to the system
            .query(
                // The query name is "Player"
                "Player",
                // The query is fetch all the entities that have the PlayerStatus component with a permission to modify the component
                Query::new().intersect::<PlayerStatus>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Spawn Enemy Function to the Engine schedule
        sched
            // Add the spawn enemy system
            .add_system(Self::spawn_enemy)
            // Subscribe to the FrameTime event
            .subscribe::<FrameTime>()
            // Add the query to the system
            .query(
                // The query name is "Enemy_Count"
                "Enemy_Count",
                // The query is fetch all the entities that have the Enemy component with a permission to modify the component
                Query::new().intersect::<Enemy>(Access::Write),
            )
            // Add another query to the system
            .query(
                // The query name is "Enemy_Status"
                "Enemy_Status",
                // The query is fetch all the entities that have the EnemyStatus component with a permission to modify the component
                Query::new().intersect::<EnemyStatus>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Player Movement Function to the Engine schedule
        sched
            // Add the player movement system
            .add_system(Self::player_movement_update)
            // Subscribe to the MoveCommand event/message
            .subscribe::<MoveCommand>()
            // Add the query to the system
            .query(
                // The query name is "Player_Movement"
                "Player_Movement",
                // The query is fetch all the entities that have the Transform and Player component with a permission to modify the component
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Player>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Enemy Movement Function to the Engine schedule
        sched
            // Add the enemy movement system
            .add_system(Self::enemy_movement_update)
            // Subscribe to the FrameTime event
            .subscribe::<FrameTime>()
            // Add the query to the system
            .query(
                // The query name is "Enemy_Movement"
                "Enemy_Movement",
                // The query is fetch all the entities that have the Transform and Enemy component with a permission to modify the component
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Enemy>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Player Fire Function to the Engine schedule
        sched
            // Add the player fire system
            .add_system(Self::player_fire_update)
            // Subscribe to the FireCommand event/message
            .subscribe::<FireCommand>()
            // Add the query to the system
            .query(
                // The query name is "Player_Fire_Input"
                "Player_Fire_Input",
                // The query is fetch all the entities that have the Player component with a permission to only read the component
                Query::new().intersect::<Player>(Access::Read),
            )
            // Build that system
            .build();

        // Attach Player Bullet Movement Function to the Engine schedule
        sched
            // Add the player bullet movement system
            .add_system(Self::player_bullet_movement_update)
            // Subscribe to the FrameTime event
            .subscribe::<FrameTime>()
            // Add the query to the system
            .query(
                // The query name is "Player_Bullet_Movement"
                "Player_Bullet_Movement",
                // The query is fetch all the entities that have the Transform and Bullet component with a permission to modify the component
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Bullet>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Enemy Fire Function to the Engine schedule
        sched
            // Add the enemy fire system
            .add_system(Self::enemy_fire_update)
            // Add the query to the system
            .query(
                // The query name is "Enemy_Fire_Input"
                "Enemy_Fire_Input",
                // The query is fetch all the entities that have the Enemy component with a permission to write the component
                Query::new().intersect::<Enemy>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Enemy Bullet Movement Function to the Engine schedule
        sched
            // Add the enemy bullet movement system
            .add_system(Self::enemy_bullet_movement_update)
            // Subscribe to the FrameTime event
            .subscribe::<FrameTime>()
            // Add the query to the system
            .query(
                // The query name is "Enemy_Bullet_Movement"
                "Enemy_Bullet_Movement",
                // The query is fetch all the entities that have the Transform and Bullet component with a permission to modify the component
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<Bullet>(Access::Write),
            )
            // Add another query to the system
            .query(
                // The query name is "Enemy_Bullet_Count_Update"
                "Enemy_Bullet_Count_Update",
                // The query is fetch all the entities that have the Enemy component with a permission to write the component
                Query::new().intersect::<Enemy>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Player Bullet to Enemy Collision Function to the Engine schedule
        sched
            // Add the player bullet to enemy collision system
            .add_system(Self::player_bullet_to_enemy_collision)
            // Add the query to the system
            .query(
                // The query name is "Player_Bullet"
                "Player_Bullet",
                // The query is fetch all the entities that have the Transform and Bullet component
                // The Transform will only have the permission to read whereas the Bullet will have the permission to write
                Query::new()
                    .intersect::<Transform>(Access::Read)
                    .intersect::<Bullet>(Access::Write),
            )
            // Add another query to the system
            .query(
                // The query name is "Enemy"
                "Enemy",
                // The query is fetch all the entities that have the Enemy and Transfrom component
                // The Enemy will have the permission to write whereas the Transform will only have the permission to read
                Query::new()
                    .intersect::<Enemy>(Access::Write)
                    .intersect::<Transform>(Access::Read),
            )
            // Add another query to the system
            .query(
                // The query name is "Score_Update"
                "Score_Update",
                // The query is fetch all the entities that have the Score component with a permission to write the component
                Query::new().intersect::<Score>(Access::Write),
            )
            // Build that system
            .build();

        // Attach Enemy Bullet to Player Collision Function to the Engine schedule
        sched
            // Add the enemy bullet to player collision system
            .add_system(Self::enemy_bullet_to_player_collision)
            // Add the query to the system
            .query(
                // The query name is "Enemy_Bullet"
                "Enemy_Bullet",
                // The query is fetch all the entities that have the Transform and Bullet component
                // The Transform will only have the permission to read whereas the Bullet will have the permission to write
                Query::new()
                    .intersect::<Transform>(Access::Read)
                    .intersect::<Bullet>(Access::Write),
            )
            // Add another query to the system
            .query(
                // The query name is "Player"
                "Player",
                // The query is fetch all the entities that have the Player and Transfrom component
                // The Player will have the permission to write whereas the Transform will only have the permission to read
                Query::new()
                    .intersect::<Player>(Access::Write)
                    .intersect::<Transform>(Access::Read),
            )
            // Add another query to the system
            .query(
                // The query name is "Player_Status_Update"
                "Player_Status_Update",
                // The query is fetch all the entities that have the PlayerStatus component with a permission to write the component
                Query::new().intersect::<PlayerStatus>(Access::Write),
            )
            // Add another query to the system
            .query(
                // The query name is "Score_Update"
                "Score_Update",
                // The query is fetch all the entities that have the Score component with a permission to write the component
                Query::new().intersect::<Score>(Access::Write),
            )
            // Build that system
            .build();

        sched
            .add_system(Self::score_display)
            .query("Score", Query::new().intersect::<Score>(Access::Read))
            .build();

        Self
    }
}

// Implement the function systems for the server
impl ServerState {
    // The function that will spawn the player
    fn spawn_player(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Get the FrameTime event
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };
        // For every entity that qualify from the query "Player" will be processed
        for entity in query.iter("Player") {
            // If the player is dead
            if !(query.read::<PlayerStatus>(entity).status) {
                // Read the time when the PlayerStatus componenet
                let mut dead_time = query.read::<PlayerStatus>(entity).dead_time;
                // If the player just died
                if dead_time == 0.0 {
                    // Record the dead time to the current time
                    dead_time = frame_time.time;
                }
                // If the player has been dead for a certain amount of time (PLAYER_SPAWN_TIME)
                if dead_time + PLAYER_SPAWN_TIME < frame_time.time {
                    // Recreate the player entity
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
                    // Throw away the timer entity (PlayerStatus)
                    io.remove_entity(entity);
                    // Recreate the PlayerStatus entity with the default value
                    io.create_entity()
                        .add_component(PlayerStatus::default())
                        .build();
                }
                // Otherwise, update the dead time on the PlayerStatus entity
                else {
                    query.modify::<PlayerStatus>(entity, |value| {
                        value.dead_time = dead_time;
                    })
                }
            }
        }
    }
    // The function that will spawn the enemy
    fn spawn_enemy(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Get the FrameTime event
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        // If there are less enemy entities on the screen than the max enemy count from the query "Enemy_Count"
        if (query.iter("Enemy_Count").count() as u32) < ENEMY_COUNT {
            // For every entity that qualify from the query "Enemy_Status" will be processed
            for entity in query.iter("Enemy_Status") {
                // Read the dead time from the EnemyStatus component
                let mut dead_time = query.read::<EnemyStatus>(entity).0;

                // If the enemy just died
                if dead_time == 0.0 {
                    // Record the dead time of the enemy to the current time
                    dead_time = frame_time.time;
                }

                // If the enemy has been dead for a certain amount of time (ENEMY_SPAWN_TIME)
                if dead_time + ENEMY_SPAWN_TIME < frame_time.time {
                    // Recreate the enemy entity
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
                    // Throw away the timer entity (EnemyStatus)
                    io.remove_entity(entity);
                    // Recreate the EnemyStatus entity with the default value
                    io.create_entity().add_component(EnemyStatus(0.0)).build();
                }
                // Otherwise, update the dead time on the EnemyStatus entity
                else {
                    query.modify::<EnemyStatus>(entity, |value| {
                        value.0 = dead_time;
                    })
                }
            }
        }
    }
    // The function that will handle the player movement
    fn player_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // When a MoveCommand event is received from the client
        for player_movement in io.inbox::<MoveCommand>() {
            // For every entity that qualify from the query "Player_Movement" will be processed
            for entity in query.iter("Player_Movement") {
                // Set the limit of the player movement
                let x_limit = WITDH / 2.0;
                // If the player is about to go out of bound
                if query.read::<Player>(entity).current_position.x + player_movement.0.x - 3.
                    < -x_limit
                    || query.read::<Player>(entity).current_position.x + player_movement.0.x + 3.
                        > x_limit
                {
                    // Do not move the player and conclude the function
                    return;
                }

                // Otherwise, move the player
                query.modify::<Transform>(entity, |transform| {
                    transform.pos += player_movement.0;
                });
                // Update the new player position
                query.modify::<Player>(entity, |player| {
                    player.current_position += player_movement.0;
                });
            }
        }
    }

    // The function that will handle the enemy movement
    fn enemy_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // For every entity that qualify from the query "Enemy_Movement" will be processed
        for entity in query.iter("Enemy_Movement") {
            // Get the FrameTime event
            let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };
            // Set pcg for random movement and direction (random generator)
            let mut pcg_random_move = Pcg::new();
            let mut pcg_random_direction = Pcg::new();

            // Based on the random value, the enemty will move in a random x direction
            let x = if pcg_random_direction.gen_bool() {
                pcg_random_move.gen_f32() * 1.
            } else {
                pcg_random_move.gen_f32() * -1.
            };

            // Based on the random value, the enemty will move in a random y direction
            let y = if pcg_random_direction.gen_bool() {
                pcg_random_move.gen_f32() * 1.
            } else {
                pcg_random_move.gen_f32() * -1.
            };

            // Declare the enemy direction that will be used for the next frame
            let speed = Vec3::new(x, y, 0.);

            // Update the enemy speed based on the frame_time delta value
            let direction = speed.normalize() * frame_time.delta * ENEMY_SPEED;

            // Declare the out of bound limits
            let x_limit = WITDH / 2.0;
            let y_upper_limit = HEIGHT / 2.;
            let y_limit = HEIGHT / 5.;

            // Read the current enemy position
            let current_position = query.read::<Enemy>(entity).current_position;

            // If the enemy is about to go out of bound
            if (current_position.x + direction.x - 3. < -x_limit)
                || (current_position.x + direction.x + 3. > x_limit)
                || (current_position.y + direction.y >= y_upper_limit)
                || (current_position.y + direction.y < y_limit)
            {
                // Do not move the enemy and conclude the function
                return;
            }
            // Otherwise, move the enemy
            query.modify::<Transform>(entity, |transform| {
                transform.pos += direction;
            });
            // Update the new enemy position
            query.modify::<Enemy>(entity, |enemy| {
                enemy.current_position += direction;
            });
        }
    }

    // The function that will handle the player fire
    fn player_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // If the FireCommand event is received from the client
        if let Some(FireCommand(value)) = io.inbox_first() {
            // For every entity that qualify from the query "Player_Fire_Input" will be processed
            for entity in query.iter("Player_Fire_Input") {
                // Create the bullet entity from the plauyer position (the left bullet)
                io.create_entity()
                    // Add the render component as triangle
                    .add_component(
                        Render::new(PLAYER_BULLET_HANDLE).primitive(Primitive::Triangles),
                    )
                    // Add the synchronized component
                    .add_component(Synchronized)
                    // Add the bullet component that is from the player and from which entity is from (player entity in this case)
                    .add_component(Bullet {
                        from_enemy: false,
                        from_player: true,
                        entity_id: entity,
                    })
                    // Add the transform component with the position based on the player current position + top left
                    .add_component(Transform::default().with_position(
                        query.read::<Player>(entity).current_position
                            + Vec3::new(-PLAYER_SIZE / 2., PLAYER_SIZE / 2., 0.0),
                    ))
                    // Build the entity
                    .build();

                // Create the bullet entity from the plauyer position (the right bullet)
                io.create_entity()
                    // Add the render component as triangle
                    .add_component(
                        Render::new(PLAYER_BULLET_HANDLE).primitive(Primitive::Triangles),
                    )
                    // Add the synchronized component
                    .add_component(Synchronized)
                    // Add the bullet component that is from the player and from which entity is from (player entity in this case)
                    .add_component(Bullet {
                        from_enemy: false,
                        from_player: true,
                        entity_id: entity,
                    })
                    // Add the transform component with the position based on the player current position + top right
                    .add_component(Transform::default().with_position(
                        query.read::<Player>(entity).current_position
                            + Vec3::new(PLAYER_SIZE / 2., PLAYER_SIZE / 2., 0.0),
                    ))
                    // Build the entity
                    .build();
            }
        }
    }

    // The function that will handle the player bullet movement
    fn player_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Get the FrameTime event
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        // For every entity that qualify from the query "Player_Bullet_Movement" will be processed
        for entity in query.iter("Player_Bullet_Movement") {
            // If the bullet is from the player
            if query.read::<Bullet>(entity).from_player {
                // If the bullet is out of bound
                if query.read::<Transform>(entity).pos.y > HEIGHT / 2. - 2.5 {
                    // Remove the bullet entity
                    io.remove_entity(entity);
                }
                // Otherwise, move the bullet
                query.modify::<Transform>(entity, |transform| {
                    transform.pos +=
                        Vec3::new(0.0, 1.0, 0.0) * frame_time.delta * PLAYER_BULLET_SPEED;
                });
            }
        }
    }

    // The function that will handle the enemy fire update
    fn enemy_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Set the random generator for the enemy fire
        let mut pcg_fire = Pcg::new();

        // For every entity that qualify from the query "Enemy_Fire_Input" will be processed
        for entity in query.iter("Enemy_Fire_Input") {
            // If the random generator return true to fire
            if pcg_fire.gen_bool() {
                // If the enemy bullet count is less than the max bullet count on screen from each enemy
                if query.read::<Enemy>(entity).bullet_count < ENEMY_MAX_BULLET {
                    // Increase the bullet count that are on screen from that enemy by 1
                    query.modify::<Enemy>(entity, |value| {
                        value.bullet_count += 1;
                    });
                    // Create the bullet entity from the enemy position
                    io.create_entity()
                        // Add the render component as triangle
                        .add_component(
                            Render::new(ENEMY_BULLET_HANDLE).primitive(Primitive::Triangles),
                        )
                        // Add the synchronized component
                        .add_component(Synchronized)
                        // Add the bullet component that is from the enemy and from which entity is from (enemy entity in this case)
                        .add_component(Bullet {
                            from_enemy: true,
                            from_player: false,
                            entity_id: entity,
                        })
                        // Add the transform component with the position based on the enemy current position + top (bottom based on player persepective)
                        .add_component(Transform::default().with_position(
                            query.read::<Enemy>(entity).current_position
                                + Vec3::new(0., -ENEMY_SIZE / 2., 0.),
                        ))
                        // Build the entity
                        .build();
                }
            }
        }
    }

    // The function that will handle the enemy bullet movement
    fn enemy_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Get the FrameTime event
        if let Some(frame_time) = io.inbox_first::<FrameTime>() {
            // For every entity that qualify from the query "Enemy_Bullet_Movement" will be processed
            for entity in query.iter("Enemy_Bullet_Movement") {
                // If the bullet is from the enemy
                if query.read::<Bullet>(entity).from_enemy {
                    // If the bullet is out of bound
                    if query.read::<Transform>(entity).pos.y < -HEIGHT / 2. + 2.5 {
                        // If that enemy entity exists on the screen from the query "Enemy_Bullet_Count_Update"
                        if query
                            .iter("Enemy_Bullet_Count_Update")
                            .any(|id| id == query.read::<Bullet>(entity).entity_id)
                        {
                            // Decrease the bullet count that are on screen from that enemy by 1
                            query.modify::<Enemy>(
                                query.read::<Bullet>(entity).entity_id,
                                |value| {
                                    value.bullet_count -= 1;
                                },
                            );
                        }
                        // Remove the bullet entity
                        io.remove_entity(entity);
                    }
                    // Otherwise, move the bullet
                    query.modify::<Transform>(entity, |transform| {
                        transform.pos +=
                            Vec3::new(0.0, -1.0, 0.0) * frame_time.delta * ENEMY_BULLET_SPEED;
                    });
                }
            }
        }
    }

    // The function that will handle the collision from player bullet to enemy
    fn player_bullet_to_enemy_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // For every entity that qualify from the query "Player_Bullet" will be processed
        for entity1 in query.iter("Player_Bullet") {
            // If the bullet is from the player
            if query.read::<Bullet>(entity1).from_player {
                // For every entity that qualify from the query "Enemy" will be processed
                for entity2 in query.iter("Enemy") {
                    // Get the current position of the bullet and the enemy
                    let current_player_bullet = query.read::<Transform>(entity1).pos;
                    let current_enemy = query.read::<Transform>(entity2).pos;

                    // If the bullet hit the enemy
                    if collision_detection(
                        current_player_bullet.x,
                        current_player_bullet.y,
                        BULLET_SIZE,
                        current_enemy.x,
                        current_enemy.y,
                        ENEMY_SIZE,
                    ) {
                        // Remove the bullet entity
                        io.remove_entity(entity1);
                        // Remove the enemy entity
                        io.remove_entity(entity2);
                        // For every entity that qualify from the query "Score_Update" will be processed
                        for entity3 in query.iter("Score_Update") {
                            // Increase the score by 1
                            query.modify::<Score>(entity3, |value| {
                                value.score += 1;
                            });
                        }
                    }
                }
            }
        }
    }

    // The function that will handle the collision from enemy bullet to player
    fn enemy_bullet_to_player_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // For every entity that qualify from the query "Enemy_Bullet" will be processed
        for entity1 in query.iter("Enemy_Bullet") {
            // If the bullet is from the enemy
            if query.read::<Bullet>(entity1).from_enemy {
                // For every entity that qualify from the query "Player" will be processed
                for entity2 in query.iter("Player") {
                    // Get the current position of the bullet and the player
                    let current_enemy_bullet = query.read::<Transform>(entity1).pos;
                    let current_player = query.read::<Transform>(entity2).pos;

                    // If the bullet hit the player
                    if collision_detection(
                        current_enemy_bullet.x,
                        current_enemy_bullet.y,
                        BULLET_SIZE,
                        current_player.x,
                        current_player.y,
                        PLAYER_SIZE,
                    ) {
                        // Remove the bullet entity
                        io.remove_entity(entity1);
                        // Remove the player entity
                        io.remove_entity(entity2);
                        // For every entity that qualify from the query "Player_Status_Update" will be processed
                        for entity3 in query.iter("Player_Status_Update") {
                            // Set the player status as dead
                            query.modify::<PlayerStatus>(entity3, |value| {
                                value.status = false;
                            });
                        }
                        // For every entity that qualify from the query "Score_Update" will be processed
                        for entity4 in query.iter("Score_Update") {
                            // Reset the score to 0
                            query.modify::<Score>(entity4, |value| {
                                value.score = 0;
                            });
                        }
                    }
                }
            }
        }
    }

    fn score_display(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for entity in query.iter("Score") {
            let digit_list = [
                ZERO_TEXT_HANDLE,
                ONE_TEXT_HANDLE,
                TWO_TEXT_HANDLE,
                THREE_TEXT_HANDLE,
                FOUR_TEXT_HANDLE,
                FIVE_TEXT_HANDLE,
                SIX_TEXT_HANDLE,
                SEVEN_TEXT_HANDLE,
                EIGHT_TEXT_HANDLE,
                NINE_TEXT_HANDLE,
            ];

            // Fetch the current score based on the digit placements
            let first_digit = (query.read::<Score>(entity).score % 10) as usize;
            let second_digit = (query.read::<Score>(entity).score / 10) as usize;

            if (query.read::<Score>(entity).first_digit != first_digit as u32)
                || (query.read::<Score>(entity).second_digit != second_digit as u32)
            {
                io.remove_entity(query.read::<Score>(entity).first_digit_entity);
                io.remove_entity(query.read::<Score>(entity).second_digit_entity);
                
                // Second Digit Entity
                let second_entity_id = io
                    .create_entity()
                    // Add the render component as triangle
                    .add_component(
                        Render::new(digit_list[second_digit]).primitive(Primitive::Lines),
                    )
                    // Add the synchronized component
                    .add_component(Synchronized)
                    // Add the transform component with the position based on the enemy current position + top (bottom based on player persepective)
                    .add_component(
                        Transform::default()
                            .with_position(Vec3::new(-2.5, 0., 0.))
                            .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
                    )
                    // Build the entity
                    .build();

                // First Digit Entity
                let first_entity_id = io
                    .create_entity()
                    // Add the render component as triangle
                    .add_component(
                        Render::new(digit_list[first_digit]).primitive(Primitive::Lines),
                    )
                    // Add the synchronized component
                    .add_component(Synchronized)
                    // Add the transform component with the position based on the enemy current position + top (bottom based on player persepective)
                    .add_component(
                        Transform::default()
                            .with_position(Vec3::new(2.5, 0., 0.))
                            .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
                    )
                    // Build the entity
                    .build();

                query.modify::<Score>(entity, |value| {
                    value.first_digit_entity = first_entity_id;
                    value.second_digit_entity = second_entity_id;
                });
            }
        }
    }
}

// The function that will handle the collision detection
fn collision_detection(
    obj1_x_position: f32,
    obj1_y_position: f32,
    obj1_size: f32,
    obj2_x_position: f32,
    obj2_y_position: f32,
    obj2_size: f32,
) -> bool {
    // If the object 1 is within the object 2 based on the sqaure hitbox intersection
    if obj1_x_position - (obj1_size / 2.) <= obj2_x_position + (obj2_size / 2.)
        && obj1_x_position + (obj1_size / 2.) >= obj2_x_position - (obj2_size / 2.)
        && (obj1_y_position - (obj1_size / 2.) <= obj2_y_position + (obj2_size / 2.))
        && (obj1_y_position + (obj1_size / 2.) >= obj2_y_position - (obj2_size / 2.))
    {
        // Return true if the object 1 is within the object 2, or vice versa
        return true;
    }
    // Otherwise, return false
    return false;
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);

// Score (if possible) --> just need to display on the client side
// Update README file as well
