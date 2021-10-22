#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[test]
fn auto_repeat_rate() {
    let rr = unsafe { hook_get_auto_repeat_rate() };
    assert!(rr >= 0, "could not determine auto repeat rate.");
}

#[test]
fn auto_repeat_delay() {
    let rd = unsafe { hook_get_auto_repeat_delay() };
    assert!(rd >= 0, "could not determine auto repeat delay.");
}

#[test]
fn pointer_acceleration_multiplier() {
    let am = unsafe { hook_get_pointer_acceleration_multiplier() };
    assert!(am >= 0, "could not determine pointer acceleration multiplier.");
}

#[test]
fn pointer_acceleration_threshold() {
    let at = unsafe { hook_get_pointer_acceleration_threshold() };
    assert!(am >= 0, "could not determine pointer acceleration threshold.");
}

#[test]
fn pointer_sensitivity() {
    let ps = unsafe { hook_get_pointer_sensitivity() };
    assert!(at >= 0, "could not determine pointer sensitivity.");
}

#[test]
fn multi_click_time() {
    let mct = unsafe { hook_get_multi_click_time() };
    assert!(at >= 0, "could not determine multi click time.");
}

