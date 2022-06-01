use bevy::math::IVec2;
use bevy::utils::HashMap;
use bevy::window::{Window, WindowDescriptor, WindowId, WindowMode};
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::LogicalSize;
use winit::platform::windows::WindowBuilderExtWindows;
use super::super::{WORKER_W, RAW_HANDLE};

#[derive(Debug, Default)]
pub struct WinitWindows {
    pub windows: HashMap<winit::window::WindowId, winit::window::Window>,
    pub window_id_to_winit: HashMap<WindowId, winit::window::WindowId>,
    pub winit_to_window_id: HashMap<winit::window::WindowId, WindowId>,
    // Some winit functions, such as `set_window_icon` can only be used from the main thread. If
    // they are used in another thread, the app will hang. This marker ensures `WinitWindows` is
    // only ever accessed with bevy's non-send functions and in NonSend systems.
    _not_send_sync: core::marker::PhantomData<*const ()>,
}

impl WinitWindows {
    pub fn create_window(
        &mut self,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        window_id: WindowId,
        window_descriptor: &WindowDescriptor,
    ) -> Window {
        let parent = unsafe { WORKER_W.unwrap() };
        let builder = winit::window::WindowBuilder::new();
        let winit_window = builder
            .with_parent_window(parent.0)
            .with_always_on_top(true)
            .with_maximized(true)
            .build(&event_loop)
            .expect("can create window")
            ;
        let winit_id: winit::window::WindowId = unsafe { std::mem::transmute(WORKER_W.unwrap()) };

        self.window_id_to_winit.insert(window_id, winit_id);
        self.winit_to_window_id.insert(winit_id, window_id);

        let position = winit_window
            .outer_position()
            .ok()
            .map(|position| IVec2::new(position.x, position.y));
        println!("{:?} outer position", position);
        let inner_size = winit_window.inner_size();
        println!("{:?} inner size", inner_size);
        let scale_factor = winit_window.scale_factor();
        println!("{:?} inner size", inner_size);
        let raw_window_handle = winit_window.raw_window_handle();
        self.windows.insert(winit_id, winit_window);
        Window::new(
            window_id,
            window_descriptor,
            inner_size.width,
            inner_size.height,
            scale_factor,
            position,
            raw_window_handle,
        )
    }

    pub fn get_window(&self, id: WindowId) -> Option<&winit::window::Window> {
        self.window_id_to_winit
            .get(&id)
            .and_then(|id| self.windows.get(id))
    }

    pub fn get_window_id(&self, id: winit::window::WindowId) -> Option<WindowId> {
        self.winit_to_window_id.get(&id).cloned()
    }

    pub fn remove_window(&mut self, id: WindowId) -> Option<winit::window::Window> {
        let winit_id = self.window_id_to_winit.remove(&id)?;
        // Don't remove from winit_to_window_id, to track that we used to know about this winit window
        self.windows.remove(&winit_id)
    }
}

pub fn get_fitting_videomode(
    monitor: &winit::monitor::MonitorHandle,
    width: u32,
    height: u32,
) -> winit::monitor::VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();

    fn abs_diff(a: u32, b: u32) -> u32 {
        if a > b {
            return a - b;
        }
        b - a
    }

    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match abs_diff(a.size().width, width).cmp(&abs_diff(b.size().width, width)) {
            Equal => {
                match abs_diff(a.size().height, height).cmp(&abs_diff(b.size().height, height)) {
                    Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                    default => default,
                }
            }
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

pub fn get_best_videomode(monitor: &winit::monitor::MonitorHandle) -> winit::monitor::VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();
    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match b.size().width.cmp(&a.size().width) {
            Equal => match b.size().height.cmp(&a.size().height) {
                Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                default => default,
            },
            default => default,
        }
    });

    modes.first().unwrap().clone()
}
