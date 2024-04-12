use std::{net::{Ipv4Addr, SocketAddr}, time::Duration};

use bevy::{log::LogPlugin, prelude::*};
use lightyear::{prelude::{server, LinkConditionerConfig}, shared::config::Mode, transport::io::{IoConfig, TransportConfig}};
use lightyear::prelude::server::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{protocol::*, shared::*};

pub fn build_server_app() -> App {
let mut app = App::new();
app.add_plugins((DefaultPlugins.build().disable::<LogPlugin>(), WorldInspectorPlugin::new()));

let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5001);
let netcode_config = NetcodeConfig::default()
    .with_protocol_id(PROTOCOL_ID)
    .with_key(KEY);
let link_conditioner = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(100),
    incoming_jitter: Duration::from_millis(0),
    incoming_loss: 0.00,
};
let io_config = IoConfig::from_transport(TransportConfig::UdpSocket(server_addr)).with_conditioner(link_conditioner);

let net_config = server::NetConfig::Netcode { 
    config: netcode_config, 
    io: io_config 
};

let server_config = ServerConfig {
    shared: shared_config(Mode::Separate).clone(),
    net: vec![net_config],
    ..Default::default()
};
let plugin_config = PluginConfig::new(server_config, protocol());
app.add_plugins(server::ServerPlugin::new(plugin_config));
app.add_systems(Startup, init);
app
}

fn init(mut connections: ResMut<ServerConnections>) {
    for connection in &mut connections.servers {
        let _ = connection.start().inspect_err(|e| {
            error!("Failed to start server: {:?}", e);
        });
    }
}