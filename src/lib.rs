use bevy::input::mouse::MouseMotion;
use bevy::pbr::{CascadeShadowConfigBuilder};
use bevy::prelude::*;
use std::f32::consts::*;

#[derive(Debug, Component, Resource)]
pub struct GlbComponent;

#[derive(Debug, Resource)]
pub struct GlbComponents{
    pub glb_components: Vec<GlbComponent>,
}

impl GlbComponents {
    pub fn new() -> Self {
        GlbComponents {
            glb_components: Vec::new(),
        }
    }

    pub fn add_glb_component(&mut self, glb_component: GlbComponent) {
        self.glb_components.push(glb_component);
    }

    pub fn get_glb_components(&self) -> &Vec<GlbComponent> {
        &self.glb_components
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

pub fn setup_glb(
    mut commands: Commands, 
    mut glb_components: ResMut<GlbComponents>, 
    asset_server: Res<AssetServer>, 
    glb_path: String,
) {
    commands.spawn((
        SceneBundle {
            // Load the scene from GLB file
            scene: asset_server.load(glb_path),
            ..default()
        },
        GlbComponent,  // Tag it for raycasting detection
        glb_components.add_glb_component(GlbComponent), // Tag it for raycasting detection
    ));
}

pub fn unpack_glb_data(
    glb_components: Res<GlbComponents>, 
    triangles: Res<Triangles>,
) {
    println!("{:?}", glb_components.get_glb_components());
    println!("{:?}", triangles.get_triangles());
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


/*
Results: 
Res(AssetServer { info: AssetInfos { path_to_id: {cube.glb#Scene0: {TypeId(0x9836914029f44d2f40c171506adef220): Index { type_id: TypeId(0x9836914029f44d2f40c171506adef220), index: AssetIndex { generation: 0, index: 0 } }}}, infos: {Index { type_id: TypeId(0x9836914029f44d2f40c171506adef220), index: AssetIndex { generation: 0, index: 0 } }: AssetInfo { weak_handle: (Weak), path: Some(cube.glb#Scene0), load_state: Loading, dep_load_state: Loading, rec_dep_load_state: Loading, loading_dependencies: {}, failed_dependencies: {}, loading_rec_dependencies: {}, failed_rec_dependencies: {}, dependants_waiting_on_load: {}, dependants_waiting_on_recursive_dep_load: {}, loader_dependencies: {}, handle_drops_to_skip: 0 }} } })
[]

Looks like I need to figure out the asset server but big progress!
*/

// pub fn fire_ray(
//     camera_query: Query<(&Camera, &GlobalTransform)>,
//     windows: Query<&Window>,
// ) {
//     let (camera, camera_transform) = camera_query.single();

//     let Some(cursor_position) = windows.single().cursor_position() else {
//         return;
//     };
    
//     let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
//         return;
//     };
// }