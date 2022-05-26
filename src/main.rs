use windows::{
    Win32::UI::WindowsAndMessaging::{FindWindowW, SendMessageTimeoutW, SMTO_NORMAL, EnumWindows, FindWindowExW},
    core::PCWSTR,
    Win32::Foundation::{WPARAM, LPARAM, BOOL, HWND},
};

use bevy::prelude::*;

fn main() {
    unsafe {
        let window = get_workerw();
        println!("{:?}", window);
    }
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(bevy::log::LogPlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(bevy::hierarchy::HierarchyPlugin)
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::asset::AssetPlugin)
        .add_plugin(bevy::window::WindowPlugin { add_primary_window: false, exit_on_close: true })
        .add_plugin(bevy::render::RenderPlugin)
        .add_plugin(bevy::core_pipeline::CorePipelinePlugin)
        .add_plugin(bevy::pbr::PbrPlugin)
        ;
}

static mut WORKER_W: Option<HWND> = None;

unsafe fn get_workerw() -> HWND {
    let progman = FindWindowW("Progman", PCWSTR::default());

    SendMessageTimeoutW(progman, 0x052C, WPARAM::default(), LPARAM::default(), SMTO_NORMAL, 1000, std::ptr::null_mut());

    unsafe extern "system" fn enum_callback(top_handle: HWND, _top_param: LPARAM) -> BOOL {
        let pointer = FindWindowExW(top_handle, HWND::default(), "SHELLDLL_DefView", PCWSTR::default());
        if pointer != HWND::default() {
            // can't seem to pass this as a closure so it seems I have to use a global var?
            WORKER_W = Some(FindWindowExW(HWND::default(), top_handle, "WorkerW", PCWSTR::default()));
            return BOOL::from(false);
        }
        return BOOL::from(true);
    }
    EnumWindows(Some(enum_callback), LPARAM::default());

    return WORKER_W.expect("what is error handling?");
}

