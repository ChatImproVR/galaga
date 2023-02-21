use cimvr_engine_interface::{make_app_state, prelude::*, println, pkg_namespace};

use cimvr_common::{
    nalgebra::{Point2, UnitQuaternion, Vector2},
    render::{
        Mesh,
        MeshHandle,
        Primitive,
        Render,
        ShaderHandle,
        ShaderSource,
        UploadMesh,
        Vertex,
        DEFAULT_VERTEX_SHADER,
    },
    FrameTime,
    Transform
};

use serde::{Deserialize, Serialize};

// All state associated with client-side behaviour
struct ClientState;

// Create ID based on each object's name
const PLAYER_HANDLE : MeshHandle = MeshHandle::new(pkg_namespace!("Player"));
const ENEMY_HANDLE : MeshHandle = MeshHandle::new(pkg_namespace!("Enemy"));
const BULLET_HANDLE : MeshHandle = MeshHandle::new(pkg_namespace!("Bullet"));

// Create Meshes for each object
fn player() -> Mesh {
    let size: f32 = 0.5;

    // TODO: Make the vertex looks flat as much as possible

    let vertices = vec![
        Vertex::new([-size, -size, size], [0.0, 0.0, 0.0]),
        Vertex::new([size, -size, size], [1.0, 0.0, 0.0]),
        Vertex::new([size, size, size], [1.0, 1.0, 0.0]),
        Vertex::new([-size, size, size], [0.0, 1.0, 0.0]),
    ];

    // TODO: Make the indices more clear for the users to add color

    // let indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0];
    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    Mesh {vertices, indices}

}



impl UserState for ClientState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        println!("Hello, client!");

        _io.send(&UploadMesh {
            id: PLAYER_HANDLE,
            mesh: player(),
        });

        Self
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        let player_render = Render {
            id: PLAYER_HANDLE,
            primitive: Primitive::Triangles,
            limit: None,
            shader: None,
        };

        let player_entity = _io.create_entity();
        _io.add_component(player_entity, &Transform::default());
        _io.add_component(player_entity, &player_render);
        _io.add_component(player_entity, &Synchronized);
        Self
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);


#[cfg(test)]
mod tests {
    #[test]
    fn im_a_test() {}
}
