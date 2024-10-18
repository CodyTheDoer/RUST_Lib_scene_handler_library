use bevy::input::mouse::MouseMotion;
use bevy::pbr::{CascadeShadowConfigBuilder};
use bevy::prelude::*;
use bevy::render::mesh::Mesh;

use std::f32::consts::*;

use wrti_library::watertight_ray_triangle_intersection;

use glam::Vec3;

#[derive(Debug, Component, Resource)]
pub struct GlbComponent;

#[derive(Debug, Resource)]
pub struct DataState{
    pub data_loaded: bool,
    pub data_parsed: bool,
}

impl DataState {
    pub fn new() -> Self {
        DataState {
            data_loaded: false,
            data_parsed: false,
        }
    }
    
    pub fn set_loaded_true(&mut self) {
        self.data_loaded = true;
    }
    
    pub fn set_parsed_true(&mut self) {
        self.data_parsed = true;
    }

    pub fn set_false(&mut self) {
        self.data_loaded = false;
        self.data_parsed = false;
    }

    pub fn loaded(&self) -> bool {
        self.data_loaded
    }

    pub fn parsed(&self) -> bool {
        self.data_parsed
    }
}

#[derive(Debug, Resource)]
pub struct GlbComponents{
    pub glb_entities: Vec<Entity>,
}

impl GlbComponents {
    pub fn new() -> Self {
        GlbComponents {
            glb_entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.glb_entities.push(entity);
    }

    pub fn get_entities(&self) -> &Vec<Entity> {
        &self.glb_entities
    }
}

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Resource)]
pub struct Triangle{
    point_a: Vec3,
    point_b: Vec3,
    point_c: Vec3,
}

impl Triangle {
    pub fn new(point_a: Vec3, point_b: Vec3, point_c: Vec3) -> Self {
        Triangle {
            point_a,
            point_b,
            point_c,
        }
    }

    pub fn as_tuple(&self) -> (Vec3, Vec3, Vec3) {
        (self.point_a, self.point_b, self.point_c)
    }
    
    // Getter methods for each field
    pub fn a(&self) -> Vec3 {
        self.point_a
    }

    pub fn b(&self) -> Vec3 {
        self.point_b
    }

    pub fn c(&self) -> Vec3 {
        self.point_c
    }
}

#[derive(Debug, Resource)]
pub struct Triangles{
    pub triangles: Vec<Triangle>,
}

impl Triangles {    
    pub fn new() -> Self {
        Triangles {
            triangles: Vec::new(),
        }
    }

    pub fn add_triangle(&mut self, triangle: Triangle) {
        self.triangles.push(triangle);
    }

    pub fn get_triangles(&self) -> &Vec<Triangle> {
        &self.triangles
    }
}

#[derive(Debug, Component)]
pub struct WorldModelCamera;

pub fn wrti_test(
    origin: Vec3,
    direction: Vec3,
    triangles: ResMut<Triangles>,
    backface_culling: bool,
) {
    for triangle in triangles.get_triangles() {
        if let Some(hit) = watertight_ray_triangle_intersection(origin, direction, triangle.as_tuple(), backface_culling) {
            // call results for testing as needed
            println!("Intersection found at t = {}", hit.t());
            
            // call all the results
            let (t, u, v, w) = hit.as_tuple();
            println!("Hit Breakdown: t: {}, u: {}, v: {}, w: {}", t, u, v, w)
        } else {
            println!("No intersection found");
        }
    }
}

pub fn setup_glb(
    mut commands: Commands, 
    mut data_state: ResMut<DataState>, 
    mut glb_components: ResMut<GlbComponents>, 
    asset_server: Res<AssetServer>, 
    glb_path: String,
) {
    let entity = commands.spawn((
        SceneBundle {
            // Load the scene from GLB file
            scene: asset_server.load(glb_path),
            ..default()
        },
        GlbComponent,  // Tag it for raycasting detection
    ))
    .id();

    glb_components.add_entity(entity); // Tag it for raycasting detection
    data_state.set_loaded_true();
}

pub fn unpack_glb_data(
    glb_components: Res<GlbComponents>, 
    mut data_state: ResMut<DataState>,
    triangles: &mut ResMut<Triangles>,
    meshes: Res<Assets<Mesh>>,
) {
    println!("GlbComponents: {:?}", glb_components.get_entities());
    println!("Triangles: {:?}", triangles.get_triangles());
    for (handle, mesh) in meshes.iter() {
        // Get the vertex positions
        if let Some(vertex_attribute) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            let positions: &[[f32; 3]] = vertex_attribute.as_float3().unwrap();
    
            // Get the indices, if they exist
            if let Some(indices) = mesh.indices() {
                match indices {
                    bevy::render::mesh::Indices::U16(indices) => {
                        // Indices as u16
                        for triangle_indices in indices.chunks(3) {
                            if triangle_indices.len() == 3 {
                                let v0 = Vec3::from(positions[triangle_indices[0] as usize]);
                                let v1 = Vec3::from(positions[triangle_indices[1] as usize]);
                                let v2 = Vec3::from(positions[triangle_indices[2] as usize]);
                                
                                let triangle: Triangle = Triangle::new(v0, v1, v2);
                                triangles.add_triangle(triangle)
                            }
                        }
                    }
                    bevy::render::mesh::Indices::U32(indices) => {
                        // Indices as u32
                        for triangle_indices in indices.chunks(3) {
                            if triangle_indices.len() == 3 {
                                let v0 = Vec3::from(positions[triangle_indices[0] as usize]);
                                let v1 = Vec3::from(positions[triangle_indices[1] as usize]);
                                let v2 = Vec3::from(positions[triangle_indices[2] as usize]);
    
                                let triangle: Triangle = Triangle::new(v0, v1, v2);
                                triangles.add_triangle(triangle)
                            }
                        }
                    }
                }
            } else {
                println!("Mesh {:?} has no indices, treating as a triangle list.", handle);
            }
        } else {
            println!("Mesh {:?} has no vertex positions.", handle);
        }
    }
    // println!("");

    // for triangle in triangles.get_triangles() {
    //     println!("{:?}", triangle);
    // }

    data_state.set_parsed_true();
}

pub fn fire_ray(
    triangles: ResMut<Triangles>,
    data_state: ResMut<DataState>,
    glb_components: Res<GlbComponents>,
    meshes: Res<Assets<Mesh>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    if data_state.loaded() && !data_state.parsed() {
        unpack_glb_data(glb_components, data_state, &mut triangles, meshes);
    }

    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };
    
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    println!("Ray: {:?}:", ray); //Ray data: Ray3d { origin: Vec3(0.15266868, 6.976395, 4.955549), direction: Dir3(Vec3(0.9497226, -0.14684285, -0.27652156)) }:
    println!("Ray origin: {:?}:", ray.origin);
    println!("Ray direction: {:?}:", ray.direction);

    let backface_culling = true;
    
    wrti_test(glam::Vec3::from(ray.origin.to_array()), glam::Vec3::from((ray.direction).to_array()), triangles.into(), backface_culling);
}

pub fn spawn_view_model(
    mut commands: Commands,
) {
    commands
        .spawn((
            Player,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 7.0, 5.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldModelCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
            ));
        });
}

pub fn spawn_lights(
    mut commands: Commands
) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            cascade_shadow_config: CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 1.6,
                ..default()
            }
        .into(),
        ..default()
        },
    ));
}

pub fn adjust_player_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let mut transform = player.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);
        transform.rotate_local_x(pitch);
    }
}

pub fn change_fov(
    input: Res<ButtonInput<KeyCode>>,
    mut world_model_projection: Query<&mut Projection, With<WorldModelCamera>>
) {
    let mut projection = world_model_projection.single_mut();
    let Projection::Perspective(ref mut perspective) = projection.as_mut() else {
        unreachable!(
            "The `Projection` component was explicitly built with `Projection::Perspective`"
        );
    };

    if input.pressed(KeyCode::ArrowUp) {
        perspective.fov -= 1.0_f32.to_radians();
        perspective.fov = perspective.fov.max(20.0_f32.to_radians());
    }
    if input.pressed(KeyCode::ArrowDown) {
        perspective.fov += 1.0_f32.to_radians();
        perspective.fov = perspective.fov.min(160.0_f32.to_radians());
    }
}

pub fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    glb_component_query: Query<&GlobalTransform, With<GlbComponent>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let glb_component = glb_component_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the glb_component.
    let Some(distance) =
        ray.intersect_plane(glb_component.translation(), InfinitePlane3d::new(glb_component.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the glb_component at that position.
    gizmos.circle(point + glb_component.up() * 0.01, glb_component.up(), 0.2, Color::WHITE);
}

pub fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

pub fn spawn_text(
    mut commands: Commands,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                concat!(
                    "Move the camera with your mouse holding right click to enable movement.\n",
                    "Press arrow up to decrease the FOV of the world model.\n",
                    "Press arrow down to increase the FOV of the world model."
                ),
                TextStyle {
                    font_size: 25.0,
                    ..default()
                },
            ));
        });
}