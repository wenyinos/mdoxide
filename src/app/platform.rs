//! Platform-specific initialization and workarounds.

/// On Windows, Alt+Space opens the system window menu (Restore/Move/Size/Close)
/// even for borderless windows. This happens because Windows processes the key
/// combo in the message loop *before* egui can see it.
///
/// We install a thread-level keyboard hook (`WH_KEYBOARD`) that intercepts
/// Alt+Space at the earliest point in the message pipeline and blocks it.
/// When blocked, we set an atomic flag so the app can toggle the command
/// palette from its `update()` method.
///
/// This approach is more reliable than WndProc subclassing because:
/// - It runs before `TranslateMessage`/`DispatchMessage`
/// - It doesn't depend on getting the correct HWND
/// - It isn't affected by winit resetting the WndProc
#[cfg(target_os = "windows")]
mod win32 {
    use std::sync::atomic::{AtomicBool, Ordering};

    type WPARAM = usize;
    type LPARAM = isize;
    type LRESULT = isize;

    const WH_KEYBOARD: i32 = 2;
    const HC_ACTION: i32 = 0;
    const VK_SPACE: usize = 0x20;
    const KF_ALTDOWN: u32 = 0x2000;

    extern "system" {
        fn SetWindowsHookExW(
            id_hook: i32,
            lpfn: unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT,
            hmod: isize,
            thread_id: u32,
        ) -> isize;
        fn CallNextHookEx(hhk: isize, code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
        fn GetCurrentThreadId() -> u32;
    }

    /// Set by the keyboard hook when Alt+Space is intercepted.
    static PALETTE_TOGGLED: AtomicBool = AtomicBool::new(false);

    unsafe extern "system" fn keyboard_hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code == HC_ACTION {
            let vk = wparam;
            let flags = ((lparam as u32) >> 16) & 0xFFFF;
            let is_key_down = (lparam >> 31) & 1 == 0;

            if vk == VK_SPACE && (flags & KF_ALTDOWN) != 0 && is_key_down {
                PALETTE_TOGGLED.store(true, Ordering::Release);
                return 1; // Block this message — prevents WM_SYSCOMMAND / system menu
            }
        }
        unsafe { CallNextHookEx(0, code, wparam, lparam) }
    }

    pub(super) fn install_keyboard_hook() {
        unsafe {
            let tid = GetCurrentThreadId();
            let hook = SetWindowsHookExW(WH_KEYBOARD, keyboard_hook_proc, 0, tid);
            if hook == 0 {
                log::warn!("Failed to install Alt+Space keyboard hook (tid={})", tid);
            } else {
                log::info!("Alt+Space keyboard hook installed (tid={})", tid);
            }
        }
    }

    pub(super) fn take_palette_toggle() -> bool {
        PALETTE_TOGGLED.swap(false, Ordering::AcqRel)
    }
}

/// Install the platform keyboard hook (call once at startup).
pub(crate) fn install_platform_hooks() {
    #[cfg(target_os = "windows")]
    win32::install_keyboard_hook();
}

/// Returns `true` (and clears the flag) if the platform hook intercepted
/// the palette toggle shortcut this frame.
pub(crate) fn take_palette_toggle_from_hook() -> bool {
    #[cfg(target_os = "windows")]
    {
        return win32::take_palette_toggle();
    }
    #[cfg(not(target_os = "windows"))]
    false
}
