use cimvr_engine_interface::{make_app_state, prelude::*, pkg_namespace};

use cimvr_common::{
    render::{
        Mesh,
        MeshHandle,
        Primitive,
        Render,
        UploadMesh,
        Vertex,
        CameraComponent,
    },
    Transform,
    nalgebra::{
        Matrix4,
        Point3,
    },
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

impl UserState for ClientState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {


        let camera_ent = _io.create_entity();

        // screen units for the border of the camera
        let projection = [Matrix4::new_orthographic(-10., 10., -20., 20., -1., 1.);2];

        _io.add_component(camera_ent, &Transform::identity());
        _io.add_component(
            camera_ent,
            &CameraComponent {
                clear_color: [0.; 3],
                projection,
            },
        );

        

        _io.send(&UploadMesh {
            id: PLAYER_HANDLE,
            mesh: obj::obj_lines_to_mesh(include_str!("assets/circle.obj")),
        });
        _io.send(&UploadMesh {
            id: ENEMY_HANDLE,
            mesh: obj::obj_lines_to_mesh(include_str!("assets/circle.obj")),
        });

        Self
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        
        // Render the player while adding behavior logic
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

        // Render the enemy while adding behavior logic
        let enemy_render = Render {
            id: ENEMY_HANDLE,
            primitive: Primitive::Triangles,
            limit: None,
            shader: None,
        };
        let enemy_entity = _io.create_entity();
        _io.add_component(enemy_entity, &Transform::default().with_position(Point3::new(0., 10., 0.)));
        _io.add_component(enemy_entity, &enemy_render);
        _io.add_component(enemy_entity, &Synchronized);

        Self
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);
