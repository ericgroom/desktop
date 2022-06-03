use std::sync::atomic::AtomicIsize;

use windows::{
    Win32::UI::WindowsAndMessaging::{FindWindowW, SendMessageTimeoutW, SMTO_NORMAL, EnumWindows, FindWindowExW},
    core::PCWSTR,
    Win32::{Foundation::{WPARAM, LPARAM, BOOL, HWND}},

};

static mut PARENT_HANDLE: AtomicIsize = AtomicIsize::new(0);

pub unsafe fn get_workerw() -> HWND {
    let progman = FindWindowW("Progman", PCWSTR::default());

    SendMessageTimeoutW(progman, 0x052C, WPARAM::default(), LPARAM::default(), SMTO_NORMAL, 1000, std::ptr::null_mut());

    unsafe extern "system" fn enum_callback(top_handle: HWND, _top_param: LPARAM) -> BOOL {
        let pointer = FindWindowExW(top_handle, HWND::default(), "SHELLDLL_DefView", PCWSTR::default());
        if pointer != HWND::default() {
            // can't seem to pass this as a closure so it seems I have to use a global var?
            let worker_w_hwnd = FindWindowExW(HWND::default(), top_handle, "WorkerW", PCWSTR::default());
            *PARENT_HANDLE.get_mut() = worker_w_hwnd.0;
            return BOOL::from(false);
        }
        return BOOL::from(true);
    }
    EnumWindows(Some(enum_callback), LPARAM::default());

    let isize = PARENT_HANDLE.get_mut().clone();
    if isize == 0 {
        panic!("couldn't find workerw")
    }
    return std::mem::transmute(isize);
}