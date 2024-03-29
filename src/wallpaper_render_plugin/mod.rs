mod windows_voodoo;
mod winit_config;
mod winit_windows;

pub use winit_config::*;
pub use winit_windows::*;

use bevy::app::{App, AppExit, CoreStage, Plugin};
use bevy::ecs::prelude::*;
use bevy::ecs::{
    event::{Events, ManualEventReader},
    system::Resource,
    world::World,
};
use bevy::math::{ivec2, DVec2, UVec2, Vec2};
use bevy::utils::{
    tracing::{info, trace, warn},
    Instant,
};
use bevy::window::{
    CreateWindow, ModifiesWindows, RequestRedraw, WindowBackendScaleFactorChanged,
    WindowCloseRequested, WindowCreated, WindowFocused, WindowMoved, WindowResized,
    WindowScaleFactorChanged, Windows,
};

use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{self, Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

#[derive(Default)]
pub struct WallpaperRenderPlugin;

impl Plugin for WallpaperRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<WinitWindows>()
            .init_resource::<WinitSettings>()
            .set_runner(winit_runner)
            .add_system_to_stage(CoreStage::PostUpdate, change_window.label(ModifiesWindows));
        let event_loop = EventLoop::new();
        let create_window_reader = WinitCreateWindowReader::default();
        app.insert_resource(create_window_reader)
            .insert_non_send_resource(event_loop);
    }
}

fn change_window(
    winit_windows: NonSend<WinitWindows>,
    mut windows: ResMut<Windows>,
    mut window_dpi_changed_events: EventWriter<WindowScaleFactorChanged>,
) {
    for bevy_window in windows.iter_mut() {
        let id = bevy_window.id();
        for command in bevy_window.drain_commands() {
            match command {
                bevy::window::WindowCommand::SetWindowMode {
                    mode,
                    resolution:
                        UVec2 {
                            x: width,
                            y: height,
                        },
                } => {
                    let window = winit_windows.get_window(id).unwrap();
                    match mode {
                        bevy::window::WindowMode::BorderlessFullscreen => {
                            window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        }
                        bevy::window::WindowMode::Fullscreen => {
                            window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                                get_best_videomode(&window.current_monitor().unwrap()),
                            )));
                        }
                        bevy::window::WindowMode::SizedFullscreen => window.set_fullscreen(Some(
                            winit::window::Fullscreen::Exclusive(get_fitting_videomode(
                                &window.current_monitor().unwrap(),
                                width,
                                height,
                            )),
                        )),
                        bevy::window::WindowMode::Windowed => window.set_fullscreen(None),
                    }
                }
                bevy::window::WindowCommand::SetTitle { title } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_title(&title);
                }
                bevy::window::WindowCommand::SetScaleFactor { scale_factor } => {
                    window_dpi_changed_events.send(WindowScaleFactorChanged { id, scale_factor });
                }
                bevy::window::WindowCommand::SetResolution {
                    logical_resolution:
                        Vec2 {
                            x: width,
                            y: height,
                        },
                    scale_factor,
                } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_inner_size(
                        winit::dpi::LogicalSize::new(width, height)
                            .to_physical::<f64>(scale_factor),
                    );
                }
                bevy::window::WindowCommand::SetPresentMode { .. } => (),
                bevy::window::WindowCommand::SetResizable { resizable } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_resizable(resizable);
                }
                bevy::window::WindowCommand::SetMaximized { maximized } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_maximized(maximized);
                }
                bevy::window::WindowCommand::SetMinimized { minimized } => {
                    let window = winit_windows.get_window(id).unwrap();
                    window.set_minimized(minimized);
                }
                bevy::window::WindowCommand::SetPosition {
                    monitor_selection,
                    position,
                } => {
                    let window = winit_windows.get_window(id).unwrap();

                    use bevy::window::MonitorSelection::*;
                    let maybe_monitor = match monitor_selection {
                        Current => window.current_monitor(),
                        Primary => window.primary_monitor(),
                        Index(i) => window.available_monitors().nth(i),
                    };
                    if let Some(monitor) = maybe_monitor {
                        let monitor_position = DVec2::from(<(_, _)>::from(monitor.position()));
                        let position = monitor_position + position.as_dvec2();

                        window.set_outer_position(LogicalPosition::new(position.x, position.y));
                    } else {
                        warn!("Couldn't get monitor selected with: {monitor_selection:?}");
                    }
                }
                bevy::window::WindowCommand::SetResizeConstraints { resize_constraints } => {
                    let window = winit_windows.get_window(id).unwrap();
                    let constraints = resize_constraints.check_constraints();
                    let min_inner_size = LogicalSize {
                        width: constraints.min_width,
                        height: constraints.min_height,
                    };
                    let max_inner_size = LogicalSize {
                        width: constraints.max_width,
                        height: constraints.max_height,
                    };

                    window.set_min_inner_size(Some(min_inner_size));
                    if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                        window.set_max_inner_size(Some(max_inner_size));
                    }
                }
                e => {
                    println!("ignored event: {:?}", e);
                }
            }
        }
    }
}

fn run<F>(event_loop: EventLoop<()>, event_handler: F) -> !
where
    F: 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    event_loop.run(event_handler)
}

// TODO: It may be worth moving this cfg into a procedural macro so that it can be referenced by
// a single name instead of being copied around.
// https://gist.github.com/jakerr/231dee4a138f7a5f25148ea8f39b382e seems to work.
#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn run_return<F>(event_loop: &mut EventLoop<()>, event_handler: F)
where
    F: FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    use winit::platform::run_return::EventLoopExtRunReturn;
    event_loop.run_return(event_handler);
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn run_return<F>(_event_loop: &mut EventLoop<()>, _event_handler: F)
where
    F: FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    panic!("Run return is not supported on this platform!")
}

pub fn winit_runner(app: App) {
    winit_runner_with(app);
}

// #[cfg(any(
//     target_os = "linux",
//     target_os = "dragonfly",
//     target_os = "freebsd",
//     target_os = "netbsd",
//     target_os = "openbsd"
// ))]
// pub fn winit_runner_any_thread(app: App) {
//     winit_runner_with(app, EventLoop::new_any_thread());
// }

/// Stores state that must persist between frames.
struct WinitPersistentState {
    /// Tracks whether or not the application is active or suspended.
    active: bool,
    /// Tracks whether or not an event has occurred this frame that would trigger an update in low
    /// power mode. Should be reset at the end of every frame.
    low_power_event: bool,
    /// Tracks whether the event loop was started this frame because of a redraw request.
    redraw_request_sent: bool,
    /// Tracks if the event loop was started this frame because of a `WaitUntil` timeout.
    timeout_reached: bool,
    last_update: Instant,
}
impl Default for WinitPersistentState {
    fn default() -> Self {
        Self {
            active: true,
            low_power_event: false,
            redraw_request_sent: false,
            timeout_reached: false,
            last_update: Instant::now(),
        }
    }
}

#[derive(Default, Resource)]
struct WinitCreateWindowReader(ManualEventReader<CreateWindow>);

pub fn winit_runner_with(mut app: App) {
    let mut event_loop = app
        .world
        .remove_non_send_resource::<EventLoop<()>>()
        .unwrap();
    let mut create_window_event_reader = app
        .world
        .remove_resource::<WinitCreateWindowReader>()
        .unwrap()
        .0;
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
    let mut redraw_event_reader = ManualEventReader::<RequestRedraw>::default();
    let mut winit_state = WinitPersistentState::default();
    app.world
        .insert_non_send_resource(event_loop.create_proxy());

    let return_from_run = app.world.resource::<WinitSettings>().return_from_run;

    trace!("Entering winit event loop");

    let event_handler = move |event: Event<()>,
                              event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        match event {
            event::Event::NewEvents(start) => {
                let winit_config = app.world.resource::<WinitSettings>();
                let windows = app.world.resource::<Windows>();
                let focused = windows.iter().any(|w| w.is_focused());
                // Check if either the `WaitUntil` timeout was triggered by winit, or that same
                // amount of time has elapsed since the last app update. This manual check is needed
                // because we don't know if the criteria for an app update were met until the end of
                // the frame.
                let auto_timeout_reached = matches!(start, StartCause::ResumeTimeReached { .. });
                let now = Instant::now();
                let manual_timeout_reached = match winit_config.update_mode(focused) {
                    UpdateMode::Continuous => false,
                    UpdateMode::Reactive { max_wait }
                    | UpdateMode::ReactiveLowPower { max_wait } => {
                        now.duration_since(winit_state.last_update) >= *max_wait
                    }
                };
                // The low_power_event state and timeout must be reset at the start of every frame.
                winit_state.low_power_event = false;
                winit_state.timeout_reached = auto_timeout_reached || manual_timeout_reached;
            }
            event::Event::WindowEvent {
                event,
                window_id: winit_window_id,
                ..
            } => {
                let world = app.world.cell();
                let winit_windows = world.get_non_send_resource_mut::<WinitWindows>().unwrap();
                let mut windows = world.get_resource_mut::<Windows>().unwrap();
                let window_id =
                    if let Some(window_id) = winit_windows.get_window_id(winit_window_id) {
                        window_id
                    } else {
                        warn!(
                            "Skipped event for unknown winit Window Id {:?}, event: {:?}",
                            winit_window_id, event
                        );
                        return;
                    };

                let window = if let Some(window) = windows.get_mut(window_id) {
                    window
                } else {
                    // If we're here, this window was previously opened
                    info!("Skipped event for closed window: {:?}", window_id);
                    return;
                };
                winit_state.low_power_event = true;

                match event {
                    WindowEvent::Resized(size) => {
                        window.update_actual_size_from_backend(size.width, size.height);
                        let mut resize_events =
                            world.get_resource_mut::<Events<WindowResized>>().unwrap();
                        resize_events.send(WindowResized {
                            id: window_id,
                            width: window.width(),
                            height: window.height(),
                        });
                    }
                    WindowEvent::CloseRequested => {
                        let mut window_close_requested_events = world
                            .get_resource_mut::<Events<WindowCloseRequested>>()
                            .unwrap();
                        window_close_requested_events.send(WindowCloseRequested { id: window_id });
                    }
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    } => {
                        let mut backend_scale_factor_change_events = world
                            .get_resource_mut::<Events<WindowBackendScaleFactorChanged>>()
                            .unwrap();
                        backend_scale_factor_change_events.send(WindowBackendScaleFactorChanged {
                            id: window_id,
                            scale_factor,
                        });
                        let prior_factor = window.scale_factor();
                        window.update_scale_factor_from_backend(scale_factor);
                        let new_factor = window.scale_factor();
                        if let Some(forced_factor) = window.scale_factor_override() {
                            // If there is a scale factor override, then force that to be used
                            // Otherwise, use the OS suggested size
                            // We have already told the OS about our resize constraints, so
                            // the new_inner_size should take those into account
                            *new_inner_size = winit::dpi::LogicalSize::new(
                                window.requested_width(),
                                window.requested_height(),
                            )
                            .to_physical::<u32>(forced_factor);
                        } else if approx::relative_ne!(new_factor, prior_factor) {
                            let mut scale_factor_change_events = world
                                .get_resource_mut::<Events<WindowScaleFactorChanged>>()
                                .unwrap();

                            scale_factor_change_events.send(WindowScaleFactorChanged {
                                id: window_id,
                                scale_factor,
                            });
                        }

                        let new_logical_width = new_inner_size.width as f64 / new_factor;
                        let new_logical_height = new_inner_size.height as f64 / new_factor;
                        if approx::relative_ne!(window.width() as f64, new_logical_width)
                            || approx::relative_ne!(window.height() as f64, new_logical_height)
                        {
                            let mut resize_events =
                                world.get_resource_mut::<Events<WindowResized>>().unwrap();
                            resize_events.send(WindowResized {
                                id: window_id,
                                width: new_logical_width as f32,
                                height: new_logical_height as f32,
                            });
                        }
                        window.update_actual_size_from_backend(
                            new_inner_size.width,
                            new_inner_size.height,
                        );
                    }
                    WindowEvent::Focused(focused) => {
                        window.update_focused_status_from_backend(focused);
                        let mut focused_events =
                            world.get_resource_mut::<Events<WindowFocused>>().unwrap();
                        focused_events.send(WindowFocused {
                            id: window_id,
                            focused,
                        });
                    }
                    WindowEvent::Moved(position) => {
                        let position = ivec2(position.x, position.y);
                        window.update_actual_position_from_backend(position);
                        let mut events = world.get_resource_mut::<Events<WindowMoved>>().unwrap();
                        events.send(WindowMoved {
                            id: window_id,
                            position,
                        });
                    }
                    _ => {}
                }
            }
            event::Event::Suspended => {
                winit_state.active = false;
            }
            event::Event::Resumed => {
                winit_state.active = true;
            }
            event::Event::MainEventsCleared => {
                handle_create_window_events(
                    &mut app.world,
                    event_loop,
                    &mut create_window_event_reader,
                );
                let winit_config = app.world.resource::<WinitSettings>();
                let update = if winit_state.active {
                    let windows = app.world.resource::<Windows>();
                    let focused = windows.iter().any(|w| w.is_focused());
                    match winit_config.update_mode(focused) {
                        UpdateMode::Continuous | UpdateMode::Reactive { .. } => true,
                        UpdateMode::ReactiveLowPower { .. } => {
                            winit_state.low_power_event
                                || winit_state.redraw_request_sent
                                || winit_state.timeout_reached
                        }
                    }
                } else {
                    false
                };
                if update {
                    winit_state.last_update = Instant::now();
                    app.update();
                }
            }
            Event::RedrawEventsCleared => {
                {
                    let winit_config = app.world.resource::<WinitSettings>();
                    let windows = app.world.resource::<Windows>();
                    let focused = windows.iter().any(|w| w.is_focused());
                    let now = Instant::now();
                    use UpdateMode::*;
                    *control_flow = match winit_config.update_mode(focused) {
                        Continuous => ControlFlow::Poll,
                        Reactive { max_wait } | ReactiveLowPower { max_wait } => {
                            ControlFlow::WaitUntil(now + *max_wait)
                        }
                    };
                }
                // This block needs to run after `app.update()` in `MainEventsCleared`. Otherwise,
                // we won't be able to see redraw requests until the next event, defeating the
                // purpose of a redraw request!
                let mut redraw = false;
                if let Some(app_redraw_events) = app.world.get_resource::<Events<RequestRedraw>>() {
                    if redraw_event_reader.iter(app_redraw_events).last().is_some() {
                        *control_flow = ControlFlow::Poll;
                        redraw = true;
                    }
                }
                if let Some(app_exit_events) = app.world.get_resource::<Events<AppExit>>() {
                    if app_exit_event_reader.iter(app_exit_events).last().is_some() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                winit_state.redraw_request_sent = redraw;
            }
            _ => (),
        }
    };

    if return_from_run {
        run_return(&mut event_loop, event_handler);
    } else {
        run(event_loop, event_handler);
    }
}

fn handle_create_window_events(
    world: &mut World,
    event_loop: &EventLoopWindowTarget<()>,
    create_window_event_reader: &mut ManualEventReader<CreateWindow>,
) {
    let world = world.cell();
    let mut winit_windows = world.get_non_send_resource_mut::<WinitWindows>().unwrap();
    let mut windows = world.get_resource_mut::<Windows>().unwrap();
    let create_window_events = world.get_resource::<Events<CreateWindow>>().unwrap();
    let mut window_created_events = world.get_resource_mut::<Events<WindowCreated>>().unwrap();
    for create_window_event in create_window_event_reader.iter(&create_window_events) {
        let window = winit_windows.create_window(
            event_loop,
            create_window_event.id,
            &create_window_event.descriptor,
        );
        windows.add(window);
        window_created_events.send(WindowCreated {
            id: create_window_event.id,
        });
    }
}
