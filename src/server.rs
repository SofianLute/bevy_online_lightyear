use std::net::{Ipv4Addr, SocketAddr};
use std::collections::HashMap;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::utils::Duration;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
pub use lightyear::prelude::server::*;
use lightyear::prelude::*;

use crate::{protocol::*, shared::*};

pub fn build_server_app() -> App {
let mut app = App::new();
app.add_plugins((DefaultPlugins.build().set(LogPlugin {
    level: Level::INFO,
    filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
    update_subscriber: None,
}), 
    WorldInspectorPlugin::new()
));

let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5001);
let netcode_config = NetcodeConfig::default()
    .with_protocol_id(PROTOCOL_ID)
    .with_key(KEY);
let link_conditioner = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(100),
    incoming_jitter: Duration::from_millis(0),
    incoming_loss: 0.00,
};
let io_config = IoConfig::from_transport(TransportConfig::UdpSocket(server_addr))
    .with_conditioner(link_conditioner);
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
app.add_plugins(SharedPlugin);
app.insert_resource(Global{client_id_to_entity_id: Default::default()});
app.add_systems(Startup, init);
app.add_systems(Update, handle_server_connections);
app.add_systems(FixedUpdate, movement);
app
}

fn init(
    mut connections: ResMut<ServerConnections>, 
    mut commands: Commands,
){
    for connection in &mut connections.servers {
        let _ = connection.start().inspect_err(|e| {
            error!("Failed to start server: {:?}", e);
        });
    }

    commands.spawn(
        TextBundle::from_section(
            "Server",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::End,
            ..default()
        }),
    );
}

#[derive(Resource)]
pub(crate) struct Global {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}

pub(crate) fn handle_server_connections(
    mut connections: EventReader<ConnectEvent>,
    mut global: ResMut<Global>,
    mut commands: Commands,
){
    for connection in connections.read() {
        let client_id = *connection.context();

        let replicate = Replicate{
            prediction_target: NetworkTarget::Single(client_id),
            interpolation_target: NetworkTarget::AllExceptSingle(client_id),
            ..default()
        }; 
        let entity = commands.spawn((PLayerBundle::new(client_id, Vec3::ZERO), replicate));
        
        // Add a mapping from client id to entity id
         global.client_id_to_entity_id.insert(client_id, entity.id());
    }
}

fn movement(
    mut position_query: Query<&mut PlayerPosition>,
    mut input_reader: EventReader<InputEvent<Inputs>>,
    global: Res<Global>,
) {
    for input in input_reader.read() {
        let client_id = input.context();
        if let Some(input) = input.input() {
            if let Some(player_entity) = global.client_id_to_entity_id.get(client_id) {
                if let Ok(position) = position_query.get_mut(*player_entity) {
                    shared_movement_behaviour(position, input);
                }
            }
        }
    }
}
