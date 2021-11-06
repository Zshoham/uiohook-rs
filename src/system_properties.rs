//! Utility methods for system properties that might affect how events are interpreted.

use ffi::screen_data;
use uiohook_sys as ffi;

crate::map_native! {
    /// Data describing a single monitor.
    ///
    /// This struct is returned by the [`screen_info`] function, see its documentation for
    /// more information.
    screen_data => ScreenData {
        /// The screen number assigned by the OS.
        number => number: u8,
        x => x: i16,
        y => y: i16,
        width => width: u16,
        height => height:u16
    }
}

pub fn auto_repeat_rate() -> Option<u64> {
    let rr: i64 = unsafe { ffi::hook_get_auto_repeat_rate() as i64 };
    if rr < 0 {
        None
    } else {
        Some(rr as u64)
    }
}

pub fn auto_repeat_delay() -> Option<u64> {
    let rd: i64 = unsafe { ffi::hook_get_auto_repeat_delay() as i64 };
    if rd < 0 {
        None
    } else {
        Some(rd as u64)
    }
}

pub fn pointer_acceleration_multiplier() -> Option<u64> {
    let am: i64 = unsafe { ffi::hook_get_pointer_acceleration_multiplier() as i64 };
    if am < 0 {
        None
    } else {
        Some(am as u64)
    }
}

pub fn pointer_acceleration_threshold() -> Option<u64> {
    let at: i64 = unsafe { ffi::hook_get_pointer_acceleration_threshold() as i64 };
    if at < 0 {
        None
    } else {
        Some(at as u64)
    }
}

pub fn pointer_sensitivity() -> Option<u64> {
    let ps: i64 = unsafe { ffi::hook_get_pointer_sensitivity() as i64 };
    if ps < 0 {
        None
    } else {
        Some(ps as u64)
    }
}

pub fn multi_click_time() -> Option<u64> {
    let mct: i64 = unsafe { ffi::hook_get_multi_click_time() as i64 };
    if mct < 0 {
        None
    } else {
        Some(mct as u64)
    }
}

pub fn screen_info() -> Vec<ScreenData> {
    let mut native_vec = unsafe {
        let mut count = 0u8;
        let screens = ffi::hook_create_screen_info(&mut count);
        Vec::from_raw_parts(screens, count as usize, count as usize)
    };
    native_vec.iter_mut().map(ScreenData::from).collect()
}
