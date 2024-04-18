mod protocol;
mod client;
mod server;
mod shared;

use client::*;

fn main(){
    let mut client_app = build_client_app();
    client_app.run();
}
