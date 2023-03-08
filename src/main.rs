use std::{f32::consts::PI, num::NonZeroU8};

mod camera_controller;
mod mipmap_generator;

use bevy::{
    core_pipeline::{bloom::BloomSettings, fxaa::Fxaa},
    prelude::*,
};
use camera_controller::{CameraController, CameraControllerPlugin};
use mipmap_generator::{generate_mipmaps, MipmapGeneratorPlugin, MipmapGeneratorSettings};

use crate::convert::{change_gltf_to_use_ktx2, convert_images_to_ktx2};

mod convert;

pub fn main() {
    let args = &mut std::env::args();
    args.next();
    if let Some(arg) = &args.next() {
        if arg == "--convert" {
            println!("This will take a few minutes");
            convert_images_to_ktx2();
            change_gltf_to_use_ktx2();
        }
    }

    let mut app = App::new();

    app.insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(1.75, 1.9, 1.99)))
        .insert_resource(AmbientLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            brightness: 0.02,
        })
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(CameraControllerPlugin)
        // Generating mipmaps takes a minute
        .insert_resource(MipmapGeneratorSettings {
            anisotropic_filtering: NonZeroU8::new(16),
            ..default()
        })
        .add_plugin(MipmapGeneratorPlugin)
        // Mipmap generation be skipped if ktx2 is used
        .add_system(generate_mipmaps::<StandardMaterial>)
        .add_startup_system(setup)
        .add_system(proc_scene);

    app.run();
}

#[derive(Component)]
pub struct PostProcScene;

#[derive(Component)]
pub struct GrifLight;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Loading models, generating mipmaps");

    // sponza
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("main_sponza/NewSponza_Main_glTF_002.gltf#Scene0"),
            ..default()
        })
        .insert(PostProcScene);

    // curtains
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("PKG_A_Curtains/NewSponza_Curtains_glTF.gltf#Scene0"),
            ..default()
        })
        .insert(PostProcScene);

    // Sun
    commands
        .spawn(DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                PI * -0.43,
                PI * -0.08,
                0.0,
            )),
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 1.0, 0.99),
                illuminance: 400000.0,
                shadows_enabled: true,
                shadow_depth_bias: 0.3,
                shadow_normal_bias: 0.7,
            },
            ..default()
        })
        .insert(GrifLight);

    // Sun Refl
    commands
        .spawn(SpotLightBundle {
            transform: Transform::from_xyz(2.0, -0.0, -2.0)
                .looking_at(Vec3::new(0.0, 999.0, 0.0), Vec3::X),
            spot_light: SpotLight {
                range: 15.0,
                intensity: 1000.0,
                color: Color::rgb(1.0, 0.97, 0.85),
                shadows_enabled: false,
                inner_angle: PI * 0.4,
                outer_angle: PI * 0.5,
                ..default()
            },
            ..default()
        })
        .insert(GrifLight);

    // Sun refl 2nd bounce / misc bounces
    commands
        .spawn(SpotLightBundle {
            transform: Transform::from_xyz(2.0, 5.5, -2.0)
                .looking_at(Vec3::new(0.0, -999.0, 0.0), Vec3::X),
            spot_light: SpotLight {
                range: 13.0,
                intensity: 800.0,
                color: Color::rgb(1.0, 0.97, 0.85),
                shadows_enabled: false,
                inner_angle: PI * 0.3,
                outer_angle: PI * 0.4,
                ..default()
            },
            ..default()
        })
        .insert(GrifLight);

    // sky
    // seems to be making blocky artifacts. Even if it's the only light.
    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(0.8, 0.9, 0.97),
                intensity: 100000.0,
                shadows_enabled: false,
                range: 24.0,
                radius: 3.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 30.0, 0.0),
            ..default()
        })
        .insert(GrifLight);

    // sky refl
    commands
        .spawn(SpotLightBundle {
            transform: Transform::from_xyz(0.0, -2.0, 0.0)
                .looking_at(Vec3::new(0.0, 999.0, 0.0), Vec3::X),
            spot_light: SpotLight {
                range: 11.0,
                intensity: 300.0,
                color: Color::rgb(0.8, 0.9, 0.97),
                shadows_enabled: false,
                inner_angle: PI * 0.46,
                outer_angle: PI * 0.49,
                ..default()
            },
            ..default()
        })
        .insert(GrifLight);

    // sky low
    commands
        .spawn(SpotLightBundle {
            transform: Transform::from_xyz(3.0, 2.0, 0.0)
                .looking_at(Vec3::new(0.0, -999.0, 0.0), Vec3::X),
            spot_light: SpotLight {
                range: 12.0,
                radius: 0.0,
                intensity: 1800.0,
                color: Color::rgb(0.8, 0.9, 0.95),
                shadows_enabled: false,
                inner_angle: PI * 0.34,
                outer_angle: PI * 0.5,
                ..default()
            },
            ..default()
        })
        .insert(GrifLight);

    // Camera
    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: Transform::from_xyz(-10.5, 1.7, -1.0)
                    .looking_at(Vec3::new(0.0, 3.5, 0.0), Vec3::Y),
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: std::f32::consts::PI / 3.0,
                    near: 0.1,
                    far: 1000.0,
                    aspect_ratio: 1.0,
                }),
                ..default()
            },
            BloomSettings::NATURAL,
        ))
        .insert(CameraController::default().print_controls())
        .insert(Fxaa::default());
}

pub fn all_children<F: FnMut(Entity)>(
    children: &Children,
    children_query: &Query<&Children>,
    closure: &mut F,
) {
    for child in children {
        if let Ok(children) = children_query.get(*child) {
            all_children(children, children_query, closure);
        }
        closure(*child);
    }
}

#[allow(clippy::type_complexity)]
pub fn proc_scene(
    mut commands: Commands,
    flip_normals_query: Query<Entity, With<PostProcScene>>,
    children_query: Query<&Children>,
    has_std_mat: Query<&Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    lights: Query<
        Entity,
        (
            Or<(With<PointLight>, With<DirectionalLight>, With<SpotLight>)>,
            Without<GrifLight>,
        ),
    >,
    cameras: Query<Entity, With<Camera>>,
) {
    for entity in flip_normals_query.iter() {
        if let Ok(children) = children_query.get(entity) {
            all_children(children, &children_query, &mut |entity| {
                // Sponza needs flipped normals
                if let Ok(mat_h) = has_std_mat.get(entity) {
                    if let Some(mat) = materials.get_mut(mat_h) {
                        mat.flip_normal_map_y = true;
                    }
                }

                // Sponza has a bunch of lights by default
                if lights.get(entity).is_ok() {
                    commands.entity(entity).despawn_recursive();
                }

                // Sponza has a bunch of cameras by default
                if cameras.get(entity).is_ok() {
                    commands.entity(entity).despawn_recursive();
                }
            });
            commands.entity(entity).remove::<PostProcScene>();
        }
    }
}
