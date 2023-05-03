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

// use rand::prelude::*;

// Create some constant value for Windows
const WITDH: f32 = 80.;
const HEIGHT: f32 = 120.;

// Create some constant values for Enemy
const ENEMY_COUNT: u32 = 2;
const ENEMY_MAX_BULLET: u32 = 10;
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
    pub bullet_count: u32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            current_position: Vec3::new(0.0, 50.0, 0.0),
            bullet_count: 0,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct Bullet {
    from_player: bool,
    from_enemy: bool,
    entity_id: EntityId,
}

impl Default for Bullet {
    fn default() -> Self {
        Self {
            from_player: false,
            from_enemy: false,
            entity_id: EntityId(0),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Copy, Clone, Default)]
pub struct EnemyStatus(f32);

#[derive(Component, Serialize, Deserialize, Copy, Clone)]
pub struct PlayerStatus {
    pub status: bool,
    pub dead_time: f32,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            status: true,
            dead_time: 0.0,
        }
    }
}

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
                let distance = direction.normalize() * frame_time.delta * PLAYER_SPEED;

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
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        io.create_entity()
            .add_component(PlayerStatus::default())
            .build();

        io.create_entity().add_component(EnemyStatus(0.0)).build();

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
            .build();

        // Create the Window
        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(WINDOW_SIZE_HANDLE).primitive(Primitive::Lines))
            .add_component(Synchronized)
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
            .build();

        Self
    }
}

impl ServerState {
    fn spawn_player(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        for key in query.iter("Player") {
            if !(query.read::<PlayerStatus>(key).status) {
                let mut dead_time = query.read::<PlayerStatus>(key).dead_time;
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
                    io.remove_entity(key);
                    io.create_entity()
                        .add_component(PlayerStatus::default())
                        .build();
                } else {
                    query.modify::<PlayerStatus>(key, |value| {
                        value.dead_time = dead_time;
                    })
                }
            }
        }
    }

    fn spawn_enemy(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        if (query.iter("Enemy_Count").count() as u32) < ENEMY_COUNT {
            for key2 in query.iter("Enemy_Status") {
                let mut dead_time = query.read::<EnemyStatus>(key2).0;

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
                    io.remove_entity(key2);
                    io.create_entity().add_component(EnemyStatus(0.0)).build();
                } else {
                    query.modify::<EnemyStatus>(key2, |value| {
                        value.0 = dead_time;
                    })
                }
            }
        }
    }

    fn player_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for player_movement in io.inbox::<MoveCommand>() {
            if player_movement.from_player {
                for key in query.iter("Player_Movement") {
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
        for key in query.iter("Enemy_Movement") {
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
            let current_position = query.read::<Enemy>(key).current_position;

            if (current_position.x + direction.x - 3. < -x_limit)
                || (current_position.x + direction.x + 3. > x_limit)
                || (current_position.y + direction.y >= y_upper_limit)
                || (current_position.y + direction.y < y_limit)
            {
                return;
            }
            query.modify::<Transform>(key, |transform| {
                transform.pos += direction;
            });
            query.modify::<Enemy>(key, |enemy| {
                enemy.current_position += direction;
            });
        }
    }

    fn player_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for player_fire in io.inbox().collect::<Vec<FireCommand>>() {
            if player_fire.from_player {
                for key in query.iter("Player_Fire_Input") {
                    io.create_entity()
                        .add_component(
                            Render::new(PLAYER_BULLET_HANDLE).primitive(Primitive::Triangles),
                        )
                        .add_component(Synchronized)
                        .add_component(Bullet {
                            from_enemy: false,
                            from_player: true,
                            entity_id: key,
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
                            entity_id: key,
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
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        for key in query.iter("Player_Bullet_Movement") {
            if query.read::<Bullet>(key).from_player {
                if query.read::<Transform>(key).pos.y > HEIGHT / 2. - 2.5 {
                    io.remove_entity(key);
                }
                query.modify::<Transform>(key, |transform| {
                    transform.pos +=
                        Vec3::new(0.0, 1.0, 0.0) * frame_time.delta * PLAYER_BULLET_SPEED;
                });
            }
        }
    }

    fn enemy_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let mut pcg_fire = Pcg::new();

        for key in query.iter("Enemy_Fire_Input") {
            if pcg_fire.gen_bool() {
                if query.read::<Enemy>(key).bullet_count < ENEMY_MAX_BULLET {
                    query.modify::<Enemy>(key, |value| {
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
                            entity_id: key,
                        })
                        .add_component(Transform::default().with_position(
                            query.read::<Enemy>(key).current_position + Vec3::new(0., 1.5, 0.),
                        ))
                        .build();
                }
            }
        }
    }

    fn enemy_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(frame_time) = io.inbox_first::<FrameTime>() {
            for key in query.iter("Enemy_Bullet_Movement") {
                if query.read::<Bullet>(key).from_enemy {
                    if query.read::<Transform>(key).pos.y < -HEIGHT / 2. + 2.5 {
                        // If that enemy key exists in the game or not
                        if query
                            .iter("Enemy_Bullet_Count_Update")
                            .any(|id| id == query.read::<Bullet>(key).entity_id)
                        {
                            query.modify::<Enemy>(query.read::<Bullet>(key).entity_id, |value| {
                                value.bullet_count -= 1;
                            });
                        }
                        io.remove_entity(key);
                    }
                    query.modify::<Transform>(key, |transform| {
                        transform.pos +=
                            Vec3::new(0.0, -1.0, 0.0) * frame_time.delta * ENEMY_BULLET_SPEED;
                    });
                }
            }
        }
    }

    fn player_bullet_to_enemy_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Size is 3 pixel sqaure
        for key1 in query.iter("Player_Bullet") {
            if query.read::<Bullet>(key1).from_player {
                for key2 in query.iter("Enemy") {
                    let bullet_size = 0.5;
                    let enemy_size = 3.;
                    let current_player_bullet = query.read::<Transform>(key1).pos;
                    let current_enemy = query.read::<Transform>(key2).pos;

                    if collision_detection(
                        current_player_bullet.x,
                        current_player_bullet.y,
                        bullet_size,
                        current_enemy.x,
                        current_enemy.y,
                        enemy_size,
                    ) {
                        io.remove_entity(key1);
                        io.remove_entity(key2);
                    }
                }
            }
        }
    }

    fn enemy_bullet_to_player_collision(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for key1 in query.iter("Enemy_Bullet") {
            if query.read::<Bullet>(key1).from_enemy {
                for key2 in query.iter("Player") {
                    let bullet_size = 0.5;
                    let enemy_size = 3.;
                    let current_enemy_bullet = query.read::<Transform>(key1).pos;
                    let current_player = query.read::<Transform>(key2).pos;

                    if collision_detection(
                        current_enemy_bullet.x,
                        current_enemy_bullet.y,
                        bullet_size,
                        current_player.x,
                        current_player.y,
                        enemy_size,
                    ) {
                        io.remove_entity(key1);
                        io.remove_entity(key2);
                        for key3 in query.iter("Player_Status_Update") {
                            query.modify::<PlayerStatus>(key3, |value| {
                                value.status = false;
                            });
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

// Score (if possible)
