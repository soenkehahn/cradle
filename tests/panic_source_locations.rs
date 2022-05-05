#[rustversion::since(1.46)]
use cradle::prelude::*;
#[rustversion::since(1.46)]
use std::{
    panic::{set_hook, take_hook},
    sync::{Arc, Mutex},
};

#[rustversion::since(1.46)]
#[test]
fn panics_contain_source_locations_of_run_and_run_output_call() {
    let f = || run!("false");
    let panic_location = get_panic_location(f);
    assert_eq!(
        Some("tests/panic_source_locations.rs:12:16".to_string()),
        panic_location
    );
    let f = || run_output!("false");
    let panic_location = get_panic_location(f);
    assert_eq!(
        Some("tests/panic_source_locations.rs:18:16".to_string()),
        panic_location
    );
}

#[rustversion::since(1.46)]
fn get_panic_location(f: fn()) -> Option<String> {
    let mutex: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let mutex_clone = mutex.clone();
    set_hook(Box::new(move |info| {
        let mut guard = mutex_clone.lock().unwrap();
        *guard = info.location().map(|x| x.clone().to_string());
    }));
    let _ = std::panic::catch_unwind(f);
    let _ = take_hook();
    let guard = mutex.lock().unwrap();
    guard.clone()
}
