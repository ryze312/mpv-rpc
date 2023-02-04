use mpv_client::mpv_handle;

mod mpv_event_handler;
mod basic_listener; // For testing purposes

#[no_mangle]
fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    let listener = Box::new(basic_listener::MpvListener);
    let client = mpv_event_handler::MpvEventHandler::from_ptr(handle, listener);

    if let Err(e) = client.initialize() {
        println!("[RPC] Error initializing mpv client: {e}")
    }

    while client.poll_events() {

    }
    return 0;
}