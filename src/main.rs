/*
TODO:
Figure out how to spawn text on the screen component of the calculator
Build iterator to extract triangles from GLB files
Setup wrti logic in scene_handler_library and integrate the ray tracer
Build left click ray interchange test and see if I can just hit the triangles
Figure out how to load multiple gbl files into the scene_library_handler
*/

use scene_handler_library::{
    setup_glb, spawn_view_model, spawn_lights, animate_light_direction,
    draw_cursor, spawn_text, change_fov, adjust_player_camera, unpack_glb
}; 

use scene_handler_library::Triangles;

use wrti_library::watertight_ray_triangle_intersection;

use glam::Vec3;

use bevy::input::common_conditions::*;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(DirectionalLightShadowMap { size: 4096 })
    .insert_resource(Triangles::new())
    .add_systems(
        Startup,
        (   
            wrti_test,
            spawn_view_model,
            spawn_lights,
            spawn_text,
            |commands: Commands, asset_server: Res<AssetServer>| setup_glb(commands, asset_server, "cube.glb#Scene0".to_string()),
            |asset_server: Res<AssetServer>, triangles: Res<Triangles>|unpack_glb(asset_server, triangles),
        ),
    )
    .add_systems(
        Update, 
        (
            adjust_player_camera.run_if(input_pressed(MouseButton::Right)),
            // fire_ray.run_if(input_pressed(MouseButton::Left)),
            draw_cursor,
            change_fov,
            animate_light_direction,
        ),
    )
    .run();
}

fn wrti_test() {
    let origin = Vec3::new(0.0, 0.0, 0.0);
    let direction = Vec3::new(0.0, 0.0, 1.0);
    let triangle = (
        Vec3::new(1.0, 0.0, 5.0),
        Vec3::new(-1.0, 1.0, 5.0),
        Vec3::new(-1.0, -1.0, 5.0),
    );
    let backface_culling = false;

    if let Some(hit) = watertight_ray_triangle_intersection(origin, direction, triangle, backface_culling) {
        // call results for testing as needed
        println!("Intersection found at t = {}", hit.t());
        
        // call all the results
        let (t, u, v, w) = hit.as_tuple();
        println!("Hit Breakdown: t: {}, u: {}, v: {}, w: {}", t, u, v, w)
    } else {
        println!("No intersection found");
    }
}