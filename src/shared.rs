use std::time::Duration;
use lightyear::prelude::*;

pub const KEY: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const PROTOCOL_ID: u64 = 0;

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        // How often the client will send packets to the server (by default it is every frame).
        // Currently, the client only works if it sends packets every frame, for proper input handling.
        client_send_interval: Duration::default(),
        // How often the server will send packets to clients? You can reduce this to save bandwidth.
        server_send_interval: Duration::from_millis(40),
        // The tick rate that will be used for the FixedUpdate schedule
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        // Here we make the `Mode` an argument so that we can run `lightyear` either in `Separate` mode (distinct client and server apps)
        // or in `HostServer` mode (the server also acts as a client).
        mode,
    }
}

