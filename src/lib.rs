use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*, FrameTime};

use cimvr_common::{
    desktop::{InputEvent, KeyCode},
    glam::Vec3,
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::input_helper::InputHelper,
    Transform,
};
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
        Self { w: 100.0, h: 200.0 }
    }
}

// Create ID based on each object's name
const PLAYER_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player"));
const ENEMY_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy"));
const PLAYER_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Player Bullet"));
const ENEMY_BULLET_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Enemy Bullet"));

// Create Meshes for each object
// Create the Player Mesh
fn player() -> Mesh {
    let size: f32 = 3.;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [0.0, 0.0, 1.0]), // Vertex 0
        Vertex::new([size, -size, 0.0], [0.0, 0.0, 1.0]),  // Vertex 1
        Vertex::new([size, size, 0.0], [0.0, 0.0, 1.0]),   // Vertex 2
        Vertex::new([-size, size, 0.0], [0.0, 0.0, 1.0]),  // Vertex 3
    ];

    let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

    Mesh { vertices, indices }
}

// Create the Enemy Mesh
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

fn enemy_bullet() -> Mesh {
    let size: f32 = 0.1;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([size, -size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([size, size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([-size, size, 0.0], [1.0, 0.0, 0.0]),
    ];

    let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

    Mesh { vertices, indices }
}

fn player_bullet() -> Mesh {
    let size: f32 = 0.1;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([size, -size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([size, size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([-size, size, 0.0], [0.0, 1.0, 0.0]),
    ];

    let indices: Vec<u32> = vec![3, 0, 2, 1, 2, 0];

    Mesh { vertices, indices }
}

impl UserState for ClientState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&UploadMesh {
            id: PLAYER_HANDLE,
            mesh: player(),
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

        sched
            .add_system(Self::player_input_movement_update)
            .subscribe::<InputEvent>()
            .subscribe::<FrameTime>()
            .build();

        sched
            .add_system(Self::player_input_fire_update)
            .subscribe::<InputEvent>()
            .subscribe::<FrameTime>()
            .build();

        sched
            .add_system(Self::enemy_random_movement_update)
            .subscribe::<FrameTime>()
            .build();

        sched
            .add_system(Self::enemy_random_fire_update)
            .subscribe::<FrameTime>()
            .build();

        Self::default()
    }
}

impl ClientState {
    fn player_input_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        self.input.handle_input_events(io);

        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        let mut direction = Vec3::ZERO;

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

    fn player_input_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        self.input.handle_input_events(io);

        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        if self.input.key_held(KeyCode::Space) {
            let command = FireCommand {
                is_fired: true,
                from_player: true,
                from_enemy: false,
            };
            io.send(&command);
        }
    }

    fn enemy_random_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        let direction = Vec3::new(-1., -1., 0.);

        if direction != Vec3::ZERO {
            let distance = direction.normalize() * frame_time.delta * 70.0;

            let command = MoveCommand {
                direction: distance,
                from_player: false,
                from_enemy: true,
            };

            io.send(&command);
        }
    }

    fn enemy_random_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        // Need to add the randomness here

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
            .add_component(Transform::default().with_position(Vec3::new(0.0, -50.0, 0.0)))
            .add_component(Render::new(PLAYER_HANDLE).primitive(Primitive::Triangles))
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
            .query::<Player>(Access::Write)
            .query::<Bullet>(Access::Write)
            .build();

        sched
            .add_system(Self::player_bullet_movement_update)
            .subscribe::<FrameTime>()
            .query::<Transform>(Access::Write)
            .query::<Bullet>(Access::Write)
            .build();

        Self
    }
}

impl ServerState {
    fn player_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(MoveCommand {
            direction,
            from_player: true,
            from_enemy: false,
        }) = io.inbox_first()
        {
            for key in query.iter() {
                let x_limit = query.read::<WinSize>(key).w / 2.0;
                if query.read::<Player>(key).current_position.x + direction.x < -x_limit
                    || query.read::<Player>(key).current_position.x + direction.x > x_limit
                {
                    return;
                }

                query.modify::<Transform>(key, |transform| {
                    transform.pos += direction;
                });
                query.modify::<Player>(key, |player| {
                    player.current_position += direction;
                });
            }
        }
    }

    fn enemy_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(MoveCommand {
            direction,
            from_player: false,
            from_enemy: true,
        }) = io.inbox_first()
        {
            for key in query.iter() {
                let x_limit = query.read::<WinSize>(key).w / 2.0;
                let y_limit = query.read::<WinSize>(key).h / 4.0;
                dbg!(y_limit);
                if query.read::<Enemy>(key).current_position.x + direction.x < -x_limit
                    || query.read::<Enemy>(key).current_position.x + direction.x > x_limit
                    || query.read::<Enemy>(key).current_position.y + direction.y < -y_limit
                    || query.read::<Enemy>(key).current_position.y + direction.y > y_limit
                {
                    return;
                }
                dbg!(query.read::<Enemy>(key).current_position);
                query.modify::<Transform>(key, |transform| {
                    transform.pos += direction;
                });
                query.modify::<Enemy>(key, |enemy| {
                    enemy.current_position += direction;
                });
            }
        }
    }

    fn player_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(FireCommand {
            is_fired: true,
            from_player: true,
            from_enemy: false,
        }) = io.inbox_first()
        {
            // let left_bullet = io
            //         .create_entity()
            //         .add_component(
            //             Render::new(PLAYER_BULLET_HANDLE).primitive(Primitive::Triangles),
            //         )
            //         .add_component(Synchronized)
            //         .add_component(Bullet {
            //             from_enemy: false,
            //             from_player: true,
            //         })
            //         .add_component(
            //             Transform::default()
            //                 .with_position(Vec3::new(-3., -23.5, 0.0)),
            //         )
            //         .build();

            for key in query.iter() {
                // let current_position = query.read::<Player>(key).current_position;
                dbg!("Hello World!");

                let left_bullet = io
                    .create_entity()
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

                let right_bullet = io
                    .create_entity()
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

    fn player_bullet_movement_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(frame_time) = io.inbox_first::<FrameTime>() {
            for key in query.iter() {
                if query.read::<Bullet>(key).from_player == true
                    && query.read::<Bullet>(key).from_enemy == false
                {
                    query.modify::<Transform>(key, |transform| {
                        transform.pos += Vec3::new(0.0, 1.0, 0.0) * frame_time.delta * 150.0;
                    });
                }
            }
        }
    }

    fn enemy_fire_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {}
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);
