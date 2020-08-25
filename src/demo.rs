use bevy::prelude::*;

use alga::linear::AffineTransformation;
use na::{Isometry3, Point3, Vector3};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

use ncollide3d::narrow_phase::*;
use ncollide3d::pipeline::object::*;
use ncollide3d::query::*;
use ncollide3d::shape::*;
use ncollide3d::world::*;
use std::collections::HashMap;

#[derive(Debug)]
struct Velocity(Vec3);

#[derive(Debug)]
struct Wall;

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let world = ncollide_setup();

        app.add_resource(world)
            .add_resource(HashMap::<CollisionObjectSlabHandle, Entity>::new())
            .add_startup_system(setup.system())
            .add_system(ncollide_sync.system())
            .add_system(move_ball.system())
            .add_system(ncollide_step.thread_local_system());
    }
}

fn move_ball(time: Res<Time>, mut query: Query<(&Velocity, &mut Translation)>) {
    for (vel, mut tx) in &mut query.iter() {
        let dt = time.delta_seconds * vel.0;
        let pos = tx.0 + dt;
        tx.0 = pos;
    }
}

fn ncollide_step(
    mut ecs_world: &mut World,
    // collision_handles: Res<HashMap<CollisionObjectSlabHandle, Entity>>,
    // mut world: ResMut<CollisionWorld<f64, CollisionObjectData>>,
    resources: &mut Resources,
) {
    let collision_handles = resources
        .get::<HashMap<CollisionObjectSlabHandle, Entity>>()
        .unwrap();

    let mut world = resources
        .get_mut::<CollisionWorld<f64, CollisionObjectData>>()
        .unwrap();

    // let dt = time.delta_seconds_f64;
    // let ball_handle = *ball_handle_res;
    // Poll and handle events.
    // for event in world.proximity_events() {
    //     handle_proximity_event(&world, event);
    // }

    for event in world.contact_events() {
        handle_contact_event(&collision_handles, &mut ecs_world, &world, event);
    }

    // // Integrate velocities and positions.
    // let ball_pos;
    // {
    //     // Integrate the velocities.
    //     let ball_object = world.collision_object(ball_handle).unwrap();
    //     let ball_velocity = ball_object.data().velocity.as_ref().unwrap();

    //     // Integrate the positions.
    //     ball_pos = ball_object.position().append_translation(&na::Translation {
    //         vector: dt * *ball_velocity.lock().unwrap(),
    //     });
    // }

    // let ball_object = world.get_mut(ball_handle).unwrap();
    // ball_object.set_position(ball_pos);

    // Submit the position update to the world.
    // world.set_position(ball_handle, ball_pos);
    world.update();
}

// fn ncollide_setup_walls(
//     query: Query<(
//         &Wall,
//         &Translation,
//         &Rotation,
//         Without<CollisionObjectSlabHandle, &Entity>,
//     )>,
// ) {
//     for (wall, tx, rotation) in &mut query.iter() {}
// }

fn ncollide_sync(
    mut world: ResMut<CollisionWorld<f64, CollisionObjectData>>,
    tx: &Translation,
    handle: &CollisionObjectSlabHandle,
) {
    let obj = world.get_mut(*handle).unwrap();
    let pos = tx.0;
    obj.set_position(Isometry3::new(
        Vector3::new(pos.x() as f64, pos.y() as f64, pos.z() as f64),
        na::zero(),
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world: ResMut<CollisionWorld<f64, CollisionObjectData>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut collision_handles: ResMut<HashMap<CollisionObjectSlabHandle, Entity>>,
) {
    // The ball is part of group 1 and can interact with everything.
    let mut ball_groups = CollisionGroups::new();
    ball_groups.set_membership(&[1]);
    let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);

    let ball = ShapeHandle::new(Ball::new(0.5));
    let ball_pos = Isometry3::new(Vector3::new(1.0, 1.0, 1.0), na::zero());
    let ball_data = CollisionObjectData::new("ball");
    let (ball_handle, _) = world.add(ball_pos, ball, ball_groups, contacts_query, ball_data);

    let ball_entity = Entity::new();

    collision_handles.insert(ball_handle, ball_entity);

    // add entities to the world
    commands
        // bottom
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            translation: Translation::new(0.0, -5.0, 0.0),
            ..Default::default()
        })
        .with(Wall)
        // top
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            translation: Translation::new(0.0, 5.0, 0.0),
            rotation: Rotation::from_rotation_z(3.14),
            ..Default::default()
        })
        // back
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            translation: Translation::new(0.0, 0.0, -5.0),
            rotation: Rotation::from_rotation_xyz(1.57, 0.0, 0.0),
            ..Default::default()
        })
        // left
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            translation: Translation::new(-5.0, 0.0, 0.0),
            rotation: Rotation::from_rotation_xyz(0.0, 0.0, -1.57),
            ..Default::default()
        })
        // right
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            translation: Translation::new(5.0, 0.0, 0.0),
            rotation: Rotation::from_rotation_xyz(0.0, 0.0, 1.57),
            ..Default::default()
        })
        // sphere
        .spawn_as_entity(
            ball_entity,
            PbrComponents {
                mesh: meshes.add(Mesh::from(bevy::prelude::shape::Icosphere {
                    subdivisions: 4,
                    radius: 0.5,
                })),
                material: materials.add(Color::rgb(0.1, 0.4, 0.8).into()),
                translation: Translation::new(0.0, -1.5, 1.5),
                ..Default::default()
            },
        )
        .with(Velocity(Vec3::new(-2.0, 1.0, -0.5)))
        .with(ball_handle)
        // light
        .spawn(LightComponents {
            translation: Translation::new(4.0, 8.0, 4.0),
            ..Default::default()
        })
        // camera
        .spawn(Camera3dComponents {
            transform: Transform::new_sync_disabled(Mat4::face_toward(
                Vec3::new(0.0, 1.0, 20.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        });
}

#[derive(Clone)]
struct CollisionObjectData {
    pub name: &'static str,
}

impl CollisionObjectData {
    pub fn new(name: &'static str) -> CollisionObjectData {
        CollisionObjectData { name: name }
    }
}

fn handle_contact_event(
    collision_handles: &HashMap<CollisionObjectSlabHandle, Entity>,
    ecs_world: &mut World,
    world: &CollisionWorld<f64, CollisionObjectData>,
    event: &ContactEvent<CollisionObjectSlabHandle>,
) {
    if let &ContactEvent::Started(collider1, collider2) = event {
        if collision_handles.contains_key(&collider1) {
            let entity = collision_handles.get(&collider1).unwrap();
            println!("collision with collider 1 is ball: {:?}", entity);
        }

        if collision_handles.contains_key(&collider2) {
            let entity = collision_handles.get(&collider2).unwrap();
            println!("collision with collider 2 is ball: {:?}", entity);
            if let Ok(vel) = ecs_world.get::<Velocity>(*entity) {
                println!("collider 2 has velocity: {:?}", *vel);
            }
        }
        // NOTE: real-life applications would avoid this systematic allocation.
        let pair = world.contact_pair(collider1, collider2, true).unwrap();
        // let mut collector = Vec::new();
        let manifold = pair.3;
        let deepest_contact = manifold.deepest_contact();

        println!("Collision with wall started...");

        let mut entity = None;
        if collision_handles.contains_key(&collider1) {
            entity = Some(collision_handles.get(&collider1).unwrap());
        }

        if collision_handles.contains_key(&collider2) {
            entity = Some(collision_handles.get(&collider2).unwrap());
        }

        if let Some(entity) = entity {
            if let Ok(mut vel) = ecs_world.get_mut::<Velocity>(*entity) {
                // println!("collider 1 has velocity: {:?}", *vel);

                let normal = deepest_contact.unwrap().contact.normal;

                let nx = normal.get((0, 0)).unwrap();
                let ny = normal.get((1, 0)).unwrap();
                let nz = normal.get((2, 0)).unwrap();

                let normal_vec3 = Vec3::new(*nx as f32, *ny as f32, *nz as f32);

                vel.0 = vel.0 - 2.0 * vel.0.dot(normal_vec3) * normal_vec3;
            }
        }
    }
}

fn ncollide_setup() -> CollisionWorld<f64, CollisionObjectData> {
    // let plane_left = ShapeHandle::<f64>::new(Plane::new(Vector3::x_axis()));
    let plane_left_tri_a = ShapeHandle::<f64>::new(Triangle::new(
        Point3::new(-5.0, -5.0, -5.0),
        Point3::new(-5.0, -5.0, 5.0),
        Point3::new(-5.0, 5.0, 5.0),
    ));
    let plane_left_tri_b = ShapeHandle::<f64>::new(Triangle::new(
        Point3::new(-5.0, -5.0, -5.0),
        Point3::new(-5.0, 5.0, 5.0),
        Point3::new(-5.0, 5.0, -5.0),
    ));
    let plane_right = ShapeHandle::<f64>::new(Plane::new(-Vector3::x_axis()));
    let plane_bottom = ShapeHandle::<f64>::new(Plane::new(Vector3::y_axis()));
    let plane_top = ShapeHandle::<f64>::new(Plane::new(-Vector3::y_axis()));
    let plane_front = ShapeHandle::<f64>::new(Plane::new(-Vector3::z_axis()));
    let plane_back = ShapeHandle::<f64>::new(Plane::new(Vector3::z_axis()));

    let planes_pos = [
        Isometry3::new(Vector3::new(0.0, 0.0, 0.0), na::zero()),
        Isometry3::new(Vector3::new(5.0, 0.0, 0.0), na::zero()),
        Isometry3::new(Vector3::new(0.0, -5.0, 0.0), na::zero()),
        Isometry3::new(Vector3::new(0.0, 5.0, 0.0), na::zero()),
        Isometry3::new(Vector3::new(0.0, 0.0, 5.0), na::zero()),
        Isometry3::new(Vector3::new(0.0, 0.0, -5.0), na::zero()),
        Isometry3::new(Vector3::new(0.0, 0.0, 0.0), na::zero()),
    ];

    // All the other objects are part of the group 2 and interact only with the ball (but not with
    // each other).
    let mut others_groups = CollisionGroups::new();
    others_groups.set_membership(&[2]);
    others_groups.set_whitelist(&[1]);

    let plane_data = CollisionObjectData::new("ground");

    let mut world = CollisionWorld::<f64, CollisionObjectData>::new(0.01);

    let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);

    world.add(
        planes_pos[0],
        plane_left_tri_a,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[1],
        plane_right,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[2],
        plane_bottom,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[3],
        plane_top,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[4],
        plane_front,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[5],
        plane_back,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );
    world.add(
        planes_pos[6],
        plane_left_tri_b,
        others_groups,
        contacts_query,
        plane_data.clone(),
    );

    return world;
}
