use windows::{
    Win32::UI::WindowsAndMessaging::{FindWindowW, SendMessageTimeoutW, SMTO_NORMAL, EnumWindows, FindWindowExW},
    core::PCWSTR,
    Win32::Foundation::{WPARAM, LPARAM, BOOL, HWND, RECT},
    Win32::Graphics::Gdi::{GetDCEx, HRGN, GET_DCX_FLAGS, FillRect, CreateSolidBrush}
};

static mut workerw: Option<HWND> = None;
fn main() {
    unsafe {
        let progman = FindWindowW("Progman", PCWSTR::default());
        println!("progman: {:?}", progman);

        SendMessageTimeoutW(progman, 0x052C, WPARAM::default(), LPARAM::default(), SMTO_NORMAL, 1000, std::ptr::null_mut());

        unsafe extern "system" fn enum_callback(top_handle: HWND, _top_param: LPARAM) -> BOOL {
            let pointer = FindWindowExW(top_handle, HWND::default(), "SHELLDLL_DefView", PCWSTR::default());
            if pointer != HWND::default() {
                workerw = Some(FindWindowExW(HWND::default(), top_handle, "WorkerW", PCWSTR::default()));
                return BOOL::from(false);
            }
            return BOOL::from(true);
        }
        EnumWindows(Some(enum_callback), LPARAM::default());
        let dc = GetDCEx(workerw, HRGN::default(), GET_DCX_FLAGS(0x403));
        let rect = RECT { left: 0, top: 0, right: 500, bottom: 500};
        FillRect(dc, &rect, CreateSolidBrush(100));
        
        println!("{:?}", workerw);
    }
}

