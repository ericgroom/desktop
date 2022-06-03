mod mwinit;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use raw_window_handle::{RawWindowHandle, HasRawWindowHandle};
use bevy::window::WindowId;

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
        .add_plugin(crate::mwinit::MWinitPlugin)
        // .add_plugin(WinitPlugin)
        // .add_startup_system(create_main_window)
        // .add_plugin(wallpaper_plugin)
        .add_startup_system(setup_camera)
        .add_system(draw_circle)
        // .add_plugin(LogWindowsPlugin)
        .run()
        ;
}

struct SendableHandle(RawWindowHandle);
unsafe impl Send for SendableHandle {}
unsafe impl Sync for SendableHandle {}
impl From<RawWindowHandle> for SendableHandle {
    fn from(handle: RawWindowHandle) -> Self {
       SendableHandle(handle)
    }
}

#[derive(Default)]
struct LogWindowsPlugin;

impl Plugin for LogWindowsPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.get_sub_app_mut(bevy::render::RenderApp).expect("app should exist already");
        render_app.add_system_to_stage(bevy::render::RenderStage::Prepare, log_windows);
    }
}

fn log_windows(windows: Res<bevy::render::view::ExtractedWindows>) {
    for window in windows.values() {
        println!("{:?}", window.id);
    }
}

struct WallpaperWindowPlugin {
    handle: SendableHandle
}

impl Plugin for WallpaperWindowPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_non_send_resource::<bevy::winit::WinitWindows>()
        .init_resource::<bevy::winit::WinitSettings>()
        .set_runner(runner)
        .insert_non_send_resource(winit::event_loop::EventLoop::new())
        ;

        // let window_id = WindowId::primary();

        // let mut create_window_events = app.world.resource_mut::<bevy::ecs::event::Events<bevy::window::CreateWindow>>();
        // create_window_events.send(bevy::window::CreateWindow {
        //     id: window_id,
        //     descriptor: bevy::window::WindowDescriptor {
        //         width: 800.0,
        //         height: 600.0,
        //         present_mode: bevy::window::PresentMode::Immediate,
        //         title: "test".into(),
        //         ..default()
        //     }
        // });

        // let mut windows = app.world.get_resource_mut::<bevy::window::Windows>().expect("this should run after windows is created");
        // let descriptor = WindowDescriptor::default();
        // let id =  bevy::window::WindowId::primary();
        // let window = bevy::window::Window::new(
        //     id,
        //     &descriptor,
        //     1280, 
        //     720, 
        //     1.0, 
        //     Some(IVec2::new(0, 0)), 
        //     self.handle.0
        // );
        // windows.add(window);
        // let mut window_created_events = app.world.resource_mut::<bevy::ecs::event::Events<bevy::window::WindowCreated>>();
        // window_created_events.send(bevy::window::WindowCreated { 
        //     id: id
        // });
    }
}

fn runner(mut app: App) {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).expect("can make a window");
    let mut windows = app.world.get_resource_mut::<bevy::window::Windows>().expect("this should run after windows is created");
    let descriptor = WindowDescriptor::default();
    let id = WindowId::new();
    let mut bwindow = bevy::window::Window::new(
        id,
        &descriptor,
        1280, 
        720, 
        1.0, 
        Some(IVec2::new(0, 0)), 
        window.raw_window_handle()
    );
    bwindow.set_present_mode(bevy::window::PresentMode::Immediate);
    windows.add(bwindow);
    let mut window_created_events = app.world.resource_mut::<bevy::ecs::event::Events<bevy::window::WindowCreated>>();
    window_created_events.send(bevy::window::WindowCreated { 
        id: id
    });
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        if event == winit::event::Event::MainEventsCleared {
            app.update();
            window.request_redraw();
        }
    }
    );
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn draw_circle(
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

