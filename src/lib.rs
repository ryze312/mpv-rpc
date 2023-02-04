use std::rc::Rc;

use mpv_client::mpv_handle;

mod mpv_event_handler;
mod logging;
mod basic_listener; // For testing purposes

#[no_mangle]
fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    let logger = Rc::new(logging::Logger::from_env());

    let listener = Box::new(basic_listener::MpvListener::new(Rc::clone(&logger)));
    let result = mpv_event_handler::MpvEventHandler::from_ptr(handle, listener, Rc::clone(&logger));
    let client = match result {
        Ok(v) => v,
        Err(e) => {
            logging::error!(logger, "Error initializing event_handling: {e}");
            return -1;
        }
    };

    client.run();
    return 0;
}