mod wallpaper_render_plugin;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use wallpaper_render_plugin::WallpaperRenderPlugin;

fn main() {
    App::new()
        // .insert_resource(bevy::log::LogSettings { level: bevy::log::Level::DEBUG, ..Default::default()})
        .add_plugins(MinimalPlugins)
        .add_plugin(bevy::log::LogPlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(bevy::hierarchy::HierarchyPlugin)
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::asset::AssetPlugin)
        .add_plugin(bevy::window::WindowPlugin { add_primary_window: true, close_when_requested: false, exit_on_all_closed: false })
        .add_plugin(bevy::input::InputPlugin)
        .add_plugin(bevy::render::RenderPlugin)
        .add_plugin(bevy::core_pipeline::CorePipelinePlugin)
        .add_plugin(bevy::pbr::PbrPlugin)
        .add_plugin(bevy::sprite::SpritePlugin)
        .add_plugin(WallpaperRenderPlugin)
        .add_startup_system(setup_camera)
        .add_system(draw_square)
        .run()
        ;
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn draw_square(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(1280.)),
        material: materials.add(ColorMaterial::from(Color::BLUE)),
        ..default()
    });
}

