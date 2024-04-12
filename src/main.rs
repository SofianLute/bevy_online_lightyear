mod protocol;
mod client;
mod server;
mod shared;

use client::*;
use server::*;

use clap::*;

#[derive(Parser, PartialEq, Debug)]
enum Cli {
    Server,
    Client,
}

fn main(){
let cli = Cli::parse();
run(cli);
}

fn run(cli: Cli){
    match cli{
        Cli::Client => {
            let mut client_app = build_client_app();
            client_app.run();
        }
        Cli::Server => {
            let mut server_app = build_server_app();
            server_app.run();
        }
    }
}