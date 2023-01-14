mod wallpaper_render_plugin;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use heron::prelude::*;

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
        .add_plugin(bevy::window::WindowPlugin {
            add_primary_window: true,
            exit_on_close: false,
        })
        .add_plugin(bevy::input::InputPlugin)
        .add_plugin(bevy::render::RenderPlugin)
        .add_plugin(bevy::core_pipeline::CorePipelinePlugin)
        .add_plugin(bevy::pbr::PbrPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(bevy::sprite::SpritePlugin)
        .add_plugin(WallpaperRenderPlugin)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0_f32, 0_f32, 0_f32)))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(200.0),
        ..shapes::RegularPolygon::default()
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::CYAN),
                outline_mode: StrokeMode::new(Color::BLACK, 10.0),
            },
            Transform::default(),
        ))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::from_linear(Vec3::X * -20.0));
}
