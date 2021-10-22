use uiohook_rs::hook::event::Key;
use uiohook_rs::hook::global::{hook_stop, register_hook};
use uiohook_rs::{hook_start_blocking, keyboard};

fn main() {
    register_hook(|e| println!("{:?}", e));
    // We must hold the hook because if it falls out of scope it will be unregistered.
    let _h = keyboard!(Key::Escape, |_, _| {
        hook_stop().expect("couldn't stop the hook.");
    });

    hook_start_blocking().expect("oops hook already running ?");
}
