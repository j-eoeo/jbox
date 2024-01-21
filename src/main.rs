use std::{
    fs,
    f32::consts::PI
};

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::{RenderLayers, screenshot::ScreenshotManager}
    }, window::{WindowTheme, PrimaryWindow}
};

fn main(){
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "JBox".into(),
                    resolution: (512., 512.).into(),
                    window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }), FrameTimeDiagnosticsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, cube_rotator_system)
        .add_systems(Update, text_update_system)
        .add_systems(Update, screenshot_on_spacebar)
        .run();
}

#[derive(Component)]
struct FirstPassCube;

#[derive(Component)]
struct MainPassCube;

#[derive(Component)]
struct FpsText;

fn screenshot_on_spacebar(
    input: Res<Input<KeyCode>>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    if input.just_pressed(KeyCode::Space) {
        let date = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
        let path = format!("./ss/screenshot-{}.png", date);
        screenshot_manager
            .save_screenshot_to_disk(main_window.single(), path)
            .unwrap();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: ResMut<AssetServer>,
) {
    fs::create_dir_all("./ss").unwrap();

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font_size: 24.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 24.0,
                    color: Color::GOLD,
                    ..default()
            }),
        ]),
        FpsText,
    ));

    let texture_handle: Handle<Image> = asset_server.load("textures/j_dark.png");

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let image_handle = images.add(image);

    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    let first_pass_layer = RenderLayers::layer(1);

    commands.spawn((
        PbrBundle {
            mesh: cube_handle,
            material: cube_material_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        },
        FirstPassCube,
        first_pass_layer,
    ));

    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::WHITE),
                ..default()
            },
            camera: Camera {
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        first_pass_layer,
    ));

    let cube_size = 4.0;
    let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: cube_handle,
            material: material_handle,
            transform: Transform::from_xyz(0.0, 0.0, 1.5)
                .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
            ..default()
        },
        MainPassCube,
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(3.4 * time.delta_seconds());
        transform.rotate_y(4.0 *  time.delta_seconds());
        transform.rotate_z(1.0 * time.delta_seconds());
    }
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}
