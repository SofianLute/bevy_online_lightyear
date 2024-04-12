use std::{net::{Ipv4Addr, SocketAddr}, time::Duration};
use crate::{protocol::*, shared::*};
use bevy::{log::LogPlugin, prelude::*};
use bevy::log::Level;
use lightyear::{prelude::{client::{self, *}, LinkConditionerConfig}, shared::config::Mode, transport::io::{IoConfig, TransportConfig}};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub fn build_client_app() -> App{
    let mut app = App::new();
    app.add_plugins((DefaultPlugins.build().set(LogPlugin{
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        update_subscriber: None
    }), WorldInspectorPlugin::new()));
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
    let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 5001);
    let auth = Authentication::Manual {
        server_addr,
        client_id: 0,
        private_key: KEY,
        protocol_id: PROTOCOL_ID,
        };

    // You can add a link conditioner to simulate network conditions
    let link_conditioner = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(100),
    incoming_jitter: Duration::from_millis(0),
    incoming_loss: 0.00,
    };
    let client_config = ClientConfig {
    shared: shared_config(Mode::Separate).clone(),
    net: client::NetConfig::Netcode {
        auth,
        config: client::NetcodeConfig::default(),
        io: IoConfig::from_transport(TransportConfig::UdpSocket(client_addr))
        .with_conditioner(link_conditioner),
    },
        ..Default::default()
    };

    let plugin_config = PluginConfig::new(client_config, protocol());
    app.add_plugins(client::ClientPlugin::new(plugin_config));
    app.add_systems(Startup, init);
    app.add_systems(Update, handle_connections);
    app
}

fn init(mut client: ResMut<ClientConnection>) {
    client.connect().expect("failed to connect to server");
}

// Create a player entity whenever a client connects
pub(crate) fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    client: Res<ClientConnection>
) {
    for connection in connections.read() {
        println!("{:?}", client.is_connected());
        println!("client {:?} connected", connection.client_id());
    }
}