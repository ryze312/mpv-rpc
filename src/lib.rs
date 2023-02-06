use mpv_client::mpv_handle;

mod config;
mod logging;
mod mpv_event_queue;
mod discord_client;
mod plugin;
mod utils;

use plugin::RPCPlugin;

const DISCORD_APPID: &str = "1071519995588264016";

#[no_mangle]
fn mpv_open_cplugin(handle: *mut mpv_handle) -> std::os::raw::c_int {
    let plugin = match RPCPlugin::new(handle, DISCORD_APPID) {
        Ok(plugin) => plugin,
        Err(e) => {
            println!("Error creating RPC plugin: {e}");
            return -1;
        }
    };

    plugin.run();
    return 0;
}