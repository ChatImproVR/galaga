use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*, FrameTime};

use cimvr_common::{
    desktop::{InputEvent, KeyCode},
    gamepad::{Axis, Button, GamepadState},
    glam::{EulerRot, Quat, Vec3},
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::input_helper::InputHelper,
    Transform,
};
use obj::obj_lines_to_mesh;
use serde::{Deserialize, Serialize};
mod obj;

// All state associated with client-side behaviour
#[derive(Default)]
struct ClientState {
    input: InputHelper,
}

// Add movement command
#[derive(Message, Serialize, Deserialize, Clone, Copy)]
#[locality("Remote")]
struct MoveCommand {
    pub direction: Vec3,
    pub from_player: bool,
    pub from_enemy: bool,
}

// Add fire command
#[derive(Message, Serialize, Deserialize, Clone, Copy)]
#[locality("Remote")]
struct FireCommand {
    pub is_fired: bool,
    pub from_player: bool,
    pub from_enemy: bool,
}

#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Player {
    pub current_position: Vec3,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            current_position: Vec3::new(0.0, -50.0, 0.0),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Enemy {
    pub current_position: Vec3,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            current_position: Vec3::new(0.0, 50.0, 0.0),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Copy, Clone, Default)]
pub struct Bullet {
    pub from_player: bool,
    pub from_enemy: bool,
}

#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}

impl Default for WinSize {
    fn default() -> Self {
        Self { w: 80.0, h: 120.0 }
    }
}

// Create ID based on each object's name
const PLAYER_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player"));
const ENEMY_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy"));
const PLAYER_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player Bullet"));
const ENEMY_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy Bullet"));
const WINDOW_SIZE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Window Size"));

// Create some constant value for Windows
const WITDH: f32 = 80.;
const HEIGHT: f32 = 120.;

// Create Meshes for each object
// Create the Player Mesh
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

// // Create the Enemy Mesh
fn enemy() -> Mesh {
    let size: f32 = 3.;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [1.0, 0.0, 0.0]), // Vertex 0
        Vertex::new([size, -size, 0.0], [1.0, 0.0, 0.0]),  // Vertex 1
        Vertex::new([size, size, 0.0], [1.0, 0.0, 0.0]),   // Vertex 2
        Vertex::new([-size, size, 0.0], [1.0, 0.0, 0.0]),  // Vertex 3
    ];

    let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

    Mesh { vertices, indices }
}

// Create Player Bullet Mesh
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

// Create Enemy Bullet
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

impl UserState for ClientState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let color = [0., 1., 0.];
        let mut newMesh = obj_lines_to_mesh(&include_str!("assets/galagaship.obj"));

        newMesh.vertices.iter_mut().for_each(|v| v.uvw = color);

        io.send(&UploadMesh {
            id: PLAYER_HANDLE,
            mesh: newMesh,
        });

        io.send(&UploadMesh {
            id: ENEMY_HANDLE,
            mesh: enemy(),
        });

        io.send(&UploadMesh {
            id: PLAYER_BULLET_HANDLE,
            mesh: player_bullet(),
        });

        io.send(&UploadMesh {
            id: ENEMY_BULLET_HANDLE,
            mesh: enemy_bullet(),
        });

        io.send(&UploadMesh {
            id: WINDOW_SIZE_HANDLE,
            mesh: window_size(),
        });

        sched
            .add_system(Self::player_input_movement_update)
            .subscribe::<InputEvent>()
            .subscribe::<GamepadState>()
            .subscribe::<FrameTime>()
            .build();

        sched
            .add_system(Self::player_input_fire_update)
            .subscribe::<InputEvent>()
            .subscribe::<GamepadState>()
            .build();

        sched
            .add_system(Self::enemy_random_movement_update)
            .subscribe::<FrameTime>()
            .build();

        sched.add_system(Self::enemy_random_fire_update).build();

        Self::default()
    }
}

impl ClientState {
    fn player_input_movement_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.input.handle_input_events(io);

        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        let deadzone = 0.3;
        let mut direction = Vec3::ZERO;

        if let Some(GamepadState(gamepads)) = io.inbox_first() {
            if let Some(gamepad) = gamepads.into_iter().next() {
                if gamepad.axes[&Axis::LeftStickX] < -deadzone {
                    direction += Vec3::new(-1.0, 0.0, 0.0);
                }
                if gamepad.axes[&Axis::LeftStickX] > deadzone {
                    direction += Vec3::new(1.0, 0.0, 0.0);
                }
            }
            if self.input.key_held(KeyCode::A) {
                direction += Vec3::new(-1.0, 0.0, 0.0);
            }

            if self.input.key_held(KeyCode::D) {
                direction += Vec3::new(1.0, 0.0, 0.0);
            }

            if direction != Vec3::ZERO {
                let distance = direction.normalize() * frame_time.delta * 150.0;

                let command = MoveCommand {
                    direction: distance,
                    from_player: true,
                    from_enemy: false,
                };

                io.send(&command);
            }
        }
    }

    fn player_input_fire_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.input.handle_input_events(io);

        if let Some(GamepadState(gamepads)) = io.inbox_first() {
            if let Some(gamepad) = gamepads.into_iter().next() {
                if gamepad.buttons[&Button::East] {
                    let command = FireCommand {
                        is_fired: true,
                        from_player: true,
                        from_enemy: false,
                    };
                    io.send(&command);
                }
            }
        }

        if self.input.key_pressed(KeyCode::Space) {
            let command = FireCommand {
                is_fired: true,
                from_player: true,
                from_enemy: false,
            };
            io.send(&command);
        }
    }

    fn enemy_random_movement_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        // Add random movement number generator within a range
        let direction = Vec3::new(0., 0., 0.);

        if direction != Vec3::ZERO {
            let distance = direction.normalize() * frame_time.delta * 100.0;

            let command = MoveCommand {
                direction: distance,
                from_player: false,
                from_enemy: true,
            };

            io.send(&command);
        }
    }

    fn enemy_random_fire_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Need to add the randomness here of firing or not

        let command = FireCommand {
            is_fired: false,
            from_player: false,
            from_enemy: true,
        };
        io.send(&command);
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Create Player with components
        io.create_entity()
            .add_component(
                Transform::default()
                    .with_position(Vec3::new(0.0, -50.0, 0.0))
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
            )
            .add_component(Render::new(PLAYER_HANDLE).primitive(Primitive::Lines))
            .add_component(Player::default())
            .add_component(WinSize::default())
            .add_component(Synchronized)
            .build();

        // Create Enemy with components
        io.create_entity()
            .add_component(Transform::default().with_position(Vec3::new(0.0, 50.0, 0.0)))
            .add_component(Render::new(ENEMY_HANDLE).primitive(Primitive::Triangles))
            .add_component(Synchronized)
            .add_component(WinSize::default())
            .add_component(Enemy::default())
            .build();

        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(WINDOW_SIZE_HANDLE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        sched
            .add_system(Self::player_movement_update)
            .subscribe::<MoveCommand>()
            .query::<Transform>(Access::Write)
            .query::<WinSize>(Access::Read)
            .query::<Player>(Access::Write)
            .build();

        sched
            .add_system(Self::enemy_movement_update)
            .subscribe::<MoveCommand>()
            .query::<Transform>(Access::Write)
            .query::<WinSize>(Access::Read)
            .query::<Enemy>(Access::Write)
            .build();

        sched
            .add_system(Self::player_fire_update)
            .subscribe::<FireCommand>()
            .query::<Player>(Access::Read)
            .build();

        sched
            .add_system(Self::player_bullet_movement_update)
            .subscribe::<FrameTime>()
            .query::<Transform>(Access::Write)
            .query::<Bullet>(Access::Write)
            .build();

        sched
            .add_system(Self::enemy_fire_update)
            .subscribe::<FireCommand>()
            .query::<Enemy>(Access::Read)
            .build();

        sched
            .add_system(Self::enemy_bullet_movement_update)
            .subscribe::<FrameTime>()
            .query::<Transform>(Access::Write)
            .query::<Bullet>(Access::Write)
            .build();

        Self
    }
}

impl ServerState {
    fn player_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for player_movement in io.inbox::<MoveCommand>() {
            if player_movement.from_player {
                for key in query.iter() {
                    let x_limit = query.read::<WinSize>(key).w / 2.0;
                    if query.read::<Player>(key).current_position.x + player_movement.direction.x
                        - 3.
                        < -x_limit
                        || query.read::<Player>(key).current_position.x
                            + player_movement.direction.x
                            + 3.
                            > x_limit
                    {
                        return;
                    }

                    query.modify::<Transform>(key, |transform| {
                        transform.pos += player_movement.direction;
                    });
                    query.modify::<Player>(key, |player| {
                        player.current_position += player_movement.direction;
                    });
                }
            }
        }
    }

    fn enemy_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for enemy_movement in io.inbox::<MoveCommand>() {
            if enemy_movement.from_enemy {
                for key in query.iter() {
                    let x_limit = query.read::<WinSize>(key).w / 2.0;
                    let y_limit = query.read::<WinSize>(key).h / 4.;
                    if query.read::<Enemy>(key).current_position.x + enemy_movement.direction.x - 3.
                        < -x_limit
                        || query.read::<Enemy>(key).current_position.x
                            + enemy_movement.direction.x
                            + 3.
                            > x_limit
                        || query.read::<Enemy>(key).current_position.y + enemy_movement.direction.y
                            >= y_limit
                        || query.read::<Enemy>(key).current_position.y + enemy_movement.direction.y
                            < y_limit - 20.0
                    {
                        return;
                    }
                    dbg!(query.read::<Enemy>(key).current_position);
                    query.modify::<Transform>(key, |transform| {
                        transform.pos += enemy_movement.direction;
                    });
                    query.modify::<Enemy>(key, |enemy| {
                        enemy.current_position += enemy_movement.direction;
                    });
                }
            }
        }
    }

    fn player_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for player_fire in io.inbox().collect::<Vec<FireCommand>>() {
            if player_fire.from_player {
                for key in query.iter() {
                    io.create_entity()
                        .add_component(
                            Render::new(PLAYER_BULLET_HANDLE).primitive(Primitive::Triangles),
                        )
                        .add_component(Synchronized)
                        .add_component(Bullet {
                            from_enemy: false,
                            from_player: true,
                        })
                        .add_component(Transform::default().with_position(
                            query.read::<Player>(key).current_position + Vec3::new(-1.5, 1.5, 0.0),
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
                        })
                        .add_component(Transform::default().with_position(
                            query.read::<Player>(key).current_position + Vec3::new(1.5, 1.5, 0.0),
                        ))
                        .build();
                }
            }
        }
    }

    fn player_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(frame_time) = io.inbox_first::<FrameTime>() {
            for key in query.iter() {
                if query.read::<Bullet>(key).from_player {
                    if query.read::<Transform>(key).pos.y > HEIGHT / 2. - 5. {
                        io.remove_entity(key);
                    }
                    query.modify::<Transform>(key, |transform| {
                        transform.pos += Vec3::new(0.0, 1.0, 0.0) * frame_time.delta * 150.0;
                    });
                }
            }
        }
    }

    fn enemy_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for enemy_fire in io.inbox().collect::<Vec<FireCommand>>() {
            if enemy_fire.from_enemy {
                for key in query.iter() {
                    io.create_entity()
                        .add_component(
                            Render::new(ENEMY_BULLET_HANDLE).primitive(Primitive::Triangles),
                        )
                        .add_component(Synchronized)
                        .add_component(Bullet {
                            from_enemy: true,
                            from_player: false,
                        })
                        .add_component(Transform::default().with_position(
                            query.read::<Enemy>(key).current_position + Vec3::new(0., 1.5, 0.0),
                        ))
                        .build();
                }
            }
        }
    }

    fn enemy_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(frame_time) = io.inbox_first::<FrameTime>() {
            for key in query.iter() {
                if query.read::<Bullet>(key).from_enemy == true {
                    if query.read::<Transform>(key).pos.y < -HEIGHT / 2. + 5. {
                        io.remove_entity(key);
                    }
                    query.modify::<Transform>(key, |transform| {
                        transform.pos += Vec3::new(0.0, -1.0, 0.0) * frame_time.delta * 150.0;
                    });
                }
            }
        }
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);

// Enemy Random movement and firerate
// Enemy bullet collision and player bullet collision
// Enemy and Player Spawning
// Enemy Count
// Score (if possible)
// Tetris next?
