use std::thread;
use std::time::Duration;

use uiohook_rs::hook::event::Key;
use uiohook_rs::hook::global::{hook_stop, register_hook, reserve_events};
use uiohook_rs::{hook_start, keyboard, EventType};

fn main() {
    register_hook(|e| println!("{:?}", e));

    // We must hold the hook because if it falls out of scope it will be unregistered.
    let _h = keyboard!(Key::Escape, |_, _| {
        hook_stop().expect("couldn't stop the hook.");
    });

    // We reserve all mouse events, you will see the events printed
    // through the hook, but they wont be propogated, meaning you wont be
    // able to move your mouse while this example is running.
    unsafe {
        reserve_events(|event| match event.get_type() {
            EventType::Mouse => true,
            _ => false,
        })
    };

    println!("starting hook");
    let handle = hook_start().expect("opps already running");
    println!("wating a second");
    thread::sleep(Duration::from_secs(60));
    println!("stopping hookd");
    handle.stop().unwrap();
}
