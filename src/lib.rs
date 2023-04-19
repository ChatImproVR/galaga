use cimvr_engine_interface::{dbg, make_app_state, pcg::Pcg, pkg_namespace, prelude::*, FrameTime};

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

// Create some constant value for Windows
const WITDH: f32 = 80.;
const HEIGHT: f32 = 120.;

// Create some constant value for Others
const ENEMY_COUNT: u32 = 1;
const ENEMY_MAX_BULLET: u32 = 10;

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
    pub is_alive: bool,
}

impl Player {
    fn new() -> Self {
        Self::default()
    }

    pub fn update_status(mut self, status: bool) -> Self {
        self.is_alive = status;
        self
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            current_position: Vec3::new(0.0, -50.0, 0.0),
            is_alive: true,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Enemy {
    pub current_position: Vec3,
    pub bullet_count: u32,
}

impl Enemy {
    fn new() -> Self {
        Self::default()
    }

    pub fn update_bullet_count(mut self, count: u32) -> Self {
        self.bullet_count = count;
        self
    }

    pub fn decrease_one_bullet_count(mut self) -> Self {
        self.bullet_count -= 1;
        self
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            current_position: Vec3::new(0.0, 50.0, 0.0),
            bullet_count: 0,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Copy, Clone, Default)]
pub struct Bullet {
    from_player: bool,
    from_enemy: bool,
}

#[derive(Component, Serialize, Deserialize, Copy, Clone, Default)]
pub struct EnemyCount(u32);

#[derive(Component, Serialize, Deserialize, Copy, Clone, Default)]
pub struct UpdateEnemyBullet(bool);

// Create ID based on each object's name
const PLAYER_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player"));
const ENEMY_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy"));
const PLAYER_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player Bullet"));
const ENEMY_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy Bullet"));
const WINDOW_SIZE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Window Size"));

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
        let player_color = [0., 1., 0.];
        let mut new_player_mesh = obj_lines_to_mesh(&include_str!("assets/galagaship.obj"));

        new_player_mesh
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = player_color);

        let enemy_color = [1., 0., 0.];
        let mut new_enemy_mesh = obj_lines_to_mesh(&include_str!("assets/galaga_enemy.obj"));

        new_enemy_mesh
            .vertices
            .iter_mut()
            .for_each(|v| v.uvw = enemy_color);

        io.send(&UploadMesh {
            id: PLAYER_HANDLE,
            mesh: new_player_mesh,
        });

        io.send(&UploadMesh {
            id: ENEMY_HANDLE,
            mesh: new_enemy_mesh,
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

        let direction = Vec3::new(x, y, 0.);

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
        let mut pcg_fire = Pcg::new();

        if pcg_fire.gen_bool() {
            let command = FireCommand {
                is_fired: false,
                from_player: false,
                from_enemy: true,
            };
            io.send(&command);
        }
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
            .add_component(Synchronized)
            .build();

        // Create Enemy with components
        io.create_entity()
            .add_component(
                Transform::default()
                    .with_position(Vec3::new(0.0, 50.0, 0.0))
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
            )
            .add_component(Render::new(ENEMY_HANDLE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .add_component(Enemy::default())
            .add_component(EnemyCount(1))
            .build();

        // Create the Window
        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(WINDOW_SIZE_HANDLE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        sched
            .add_system(Self::spawn_player)
            .query::<Player>(Access::Write)
            .build();

        sched
            .add_system(Self::spawn_enemy)
            .query::<Enemy>(Access::Write)
            .query::<EnemyCount>(Access::Write)
            .build();

        sched
            .add_system(Self::player_movement_update)
            .subscribe::<MoveCommand>()
            .query::<Transform>(Access::Write)
            .query::<Player>(Access::Write)
            .build();

        sched
            .add_system(Self::enemy_movement_update)
            .subscribe::<MoveCommand>()
            .query::<Transform>(Access::Write)
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
            .query::<Enemy>(Access::Write)
            .build();

        sched
            .add_system(Self::enemy_bullet_movement_update)
            .subscribe::<FrameTime>()
            .query::<Transform>(Access::Write)
            .query::<Bullet>(Access::Write)
            .build();

        sched
            .add_system(Self::bullet_to_bullet_collision)
            .query::<Transform>(Access::Write)
            .query::<Bullet>(Access::Write)
            .build();

        Self
    }
}

impl ServerState {
    fn spawn_player(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for key in query.iter() {
            if !(query.read::<Player>(key).is_alive) {
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
            }
        }
    }

    fn spawn_enemy(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for key in query.iter() {
            if query.read::<EnemyCount>(key).0 < ENEMY_COUNT {
                io.create_entity()
                    .add_component(
                        Transform::default()
                            .with_position(Vec3::new(0.0, 50.0, 0.0))
                            .with_rotation(Quat::from_euler(EulerRot::XYZ, 90., 0., 0.)),
                    )
                    .add_component(Render::new(ENEMY_HANDLE).primitive(Primitive::Lines))
                    .add_component(Synchronized)
                    .add_component(Enemy::default())
                    .add_component(EnemyCount(query.read::<EnemyCount>(key).0 + 1))
                    .build();
                query.modify::<EnemyCount>(key, |value| {
                    value.0 += 1;
                })
            }
        }
    }

    fn player_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for player_movement in io.inbox::<MoveCommand>() {
            if player_movement.from_player {
                for key in query.iter() {
                    let x_limit = WITDH / 2.0;
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
                    let x_limit = WITDH / 2.0;
                    let y_upper_limit = HEIGHT / 2.;
                    let y_limit = HEIGHT / 5.;
                    let current_position = query.read::<Enemy>(key).current_position;

                    if (current_position.x + enemy_movement.direction.x - 3. < -x_limit)
                        || (current_position.x + enemy_movement.direction.x + 3. > x_limit)
                        || (current_position.y + enemy_movement.direction.y >= y_upper_limit)
                        || (current_position.y + enemy_movement.direction.y < y_limit)
                    {
                        return;
                    }
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
                    // if query.read::<Enemy>(key).bullet_count < ENEMY_MAX_BULLET{
                    // query.modify::<Enemy>(key, |value|{
                    //     value.bullet_count += 1;
                    // });
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
                if query.read::<Bullet>(key).from_enemy {
                    if query.read::<Transform>(key).pos.y < -HEIGHT / 2. + 5. {
                        io.remove_entity(key);
                        query.read::<Enemy>(key).decrease_one_bullet_count();
                    }
                    query.modify::<Transform>(key, |transform| {
                        transform.pos += Vec3::new(0.0, -1.0, 0.0) * frame_time.delta * 50.0;
                    });
                }
            }
        }
    }

    fn bullet_to_bullet_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // for key in query.iter() {
        //     if query.read::<Bullet>(key).from_enemy
        // }
    }

    fn player_bullet_to_enemy_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {}

    fn enemy_bullet_to_player_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {}
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);

// Enemy Random firerate (make it less true on fire)
// Enemy bullet collision and player bullet collision
// Score (if possible)
// Tetris next?
