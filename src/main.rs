mod protocol;
mod client;
mod server;
mod shared;

use server::*;

fn main(){
let mut server_app = build_server_app();
server_app.run();
}
