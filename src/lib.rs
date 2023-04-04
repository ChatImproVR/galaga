use cimvr_engine_interface::{make_app_state, prelude::*, pkg_namespace};

use cimvr_common::{
    render::{
        Mesh,
        MeshHandle,
        Primitive,
        Render,
        UploadMesh,
        Vertex,
    },
    Transform,
};
mod obj;


// All state associated with client-side behaviour
struct ClientState;

// Create ID based on each object's name
const PLAYER_HANDLE : MeshHandle = MeshHandle::new(pkg_namespace!("Player"));
const ENEMY_HANDLE : MeshHandle = MeshHandle::new(pkg_namespace!("Enemy"));
const PLAYER_BULLET_HANDLE : MeshHandle = MeshHandle::new(pkg_namespace!("Player Bullet"));
const ENEMY_BULLET_HANDLE : MeshHandle = MeshHandle::new(pkg_namespace!("Enemy Bullet"));

// Create Meshes for each object
// Create the Player Mesh
fn player() -> Mesh {
    let size: f32 = 0.5;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [0.0, 0.0, 1.0]), // Vertex 0
        Vertex::new([size, -size, 0.0], [0.0, 0.0, 1.0]), // Vertex 1
        Vertex::new([size, size, 0.0], [0.0, 0.0, 1.0]), // Vertex 2
        Vertex::new([-size, size, 0.0], [0.0, 0.0, 1.0]), // Vertex 3
    ];

    let indices: Vec<u32> = vec![3,0,2,1,2,0];

    Mesh {vertices, indices}

}

// Create the Enemy Mesh
// fn enemy() -> Mesh {
//     let size: f32 = 0.5;

//     let vertices = vec![
//         Vertex::new([-size, -size, 0.0], [1.0, 0.0, 0.0]), // Vertex 0
//         Vertex::new([size, -size, 0.0], [1.0, 0.0, 0.0]), // Vertex 1
//         Vertex::new([size, size, 0.0], [1.0, 0.0, 0.0]), // Vertex 2
//         Vertex::new([-size, size, 0.0], [1.0, 0.0, 0.0]), // Vertex 3
//     ];

//     let indices: Vec<u32> = vec![3,0,2,1,2,0];
    
//     Mesh {vertices, indices}

// }

fn enemy_bullet() -> Mesh {
    let size: f32 = 0.1;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([size, -size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([size, size, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([-size, size, 0.0], [1.0, 0.0, 0.0]),
    ];

    let indices: Vec<u32> = vec![3,0,2,1,2,0];

    Mesh {vertices, indices}

}

fn player_bullet() -> Mesh {
    let size: f32 = 0.1;

    let vertices = vec![
        Vertex::new([-size, -size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([size, -size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([size, size, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([-size, size, 0.0], [0.0, 1.0, 0.0]),
    ];

    let indices: Vec<u32> = vec![3,0,2,1,2,0];

    Mesh {vertices, indices}

}

impl UserState for ClientState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {

        

        _io.send(&UploadMesh {
            id: PLAYER_HANDLE,
            mesh: player(),
        });
        
        _io.send(&UploadMesh {
            id: ENEMY_HANDLE,
            mesh: obj::obj_lines_to_mesh(include_str!("assets/circle.obj")),
        });

        _io.send(&UploadMesh {
            id: PLAYER_BULLET_HANDLE,
            mesh: player_bullet(),
        });

        _io.send(&UploadMesh {
            id: ENEMY_BULLET_HANDLE,
            mesh: enemy_bullet(),
        });

        Self
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        
        
        let player_entity = _io
            .create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(PLAYER_HANDLE).primitive(Primitive::Triangles))
            .add_component(Synchronized)
            .build();

        let enemy_entity = _io
            .create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(ENEMY_HANDLE).primitive(Primitive::Triangles))
            .add_component(Synchronized)
            .build();

        Self
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);
