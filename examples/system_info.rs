use uiohook_rs::system_properties;

fn main() {
    println!(
        "auto_repeat_delay: {}",
        system_properties::auto_repeat_delay().unwrap()
    );
    println!(
        "auto_repeat_rate: {}",
        system_properties::auto_repeat_rate().unwrap()
    );
    println!(
        "multi_click_time: {}",
        system_properties::multi_click_time().unwrap()
    );
    println!(
        "pointer_acceleration_multiplier: {}",
        system_properties::pointer_acceleration_multiplier().unwrap()
    );
    println!(
        "pointer_acceleration_threshold: {}",
        system_properties::pointer_acceleration_threshold().unwrap()
    );
    println!(
        "pointer_sensitivity: {}",
        system_properties::pointer_sensitivity().unwrap()
    );
    println!("screen_info: {:?}", system_properties::screen_info());
}
