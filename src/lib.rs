use mpv_client::mpv_handle;

mod mpv_event_handler;

#[no_mangle]
fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    let client = mpv_event_handler::MpvHandler::from_ptr(handle);
    while client.poll_events() {
        
    }
    return 0;
}