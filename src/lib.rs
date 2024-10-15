use bevy::prelude::*;

#[derive(Debug, Component)]
struct GlbComponent;

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;

pub fn setup_glb(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    glb_path: &str
) {
    commands.spawn((
        SceneBundle {
            scene: asset_server.load(glb_path), // Load the scene from GLB file
            ..default()
        },
        GlbComponent,  // Tag it for raycasting detection
    ));
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

