use std::net::SocketAddr;
use std::collections::HashMap;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::utils::Duration;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
pub use lightyear::prelude::server::*;
use lightyear::prelude::*;
use rand::{thread_rng, Rng};
use local_ip_address::local_ip;

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
let server_local_ip = local_ip().unwrap();
let server_addr = SocketAddr::new(server_local_ip, 5001);
let netcode_config = NetcodeConfig::default()
    .with_protocol_id(PROTOCOL_ID)
    .with_key(KEY);
let link_conditioner = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(0),
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
app.insert_resource(PlayerScore{scores: Default::default()});
app.add_systems(Startup, init);
app.add_systems(Update, (handle_server_connections, player_collision));
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

    commands.spawn((
        CoinBundle::new(Vec2::new(thread_rng().gen_range(0..200) as f32, thread_rng().gen_range(0..200) as f32)),
        Name::new("Coin"),
        Replicate::default()
    ));
}

#[derive(Resource)]
pub(crate) struct Global {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}

#[derive(Resource)]
pub(crate) struct PlayerScore {
    pub scores: HashMap<PlayerId, u32>,
}

pub(crate) fn handle_server_connections(
    mut connections: EventReader<ConnectEvent>,
    mut global: ResMut<Global>,
    mut player_score: ResMut<PlayerScore>,
    mut commands: Commands,
){
    for connection in connections.read() {
        let client_id = *connection.context();

        let replicate = Replicate{
            prediction_target: NetworkTarget::Single(client_id),
            interpolation_target: NetworkTarget::AllExceptSingle(client_id),
            ..default()
        }; 
        let player_entity = commands.spawn((
            PLayerBundle::new(client_id, Vec2::ZERO), 
            replicate,
            Name::new("Player")
        ));
        
        player_score.scores.insert(PlayerId(client_id), 0);
        // Add a mapping from client id to entity id
         global.client_id_to_entity_id.insert(client_id, player_entity.id());
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

pub(crate) fn player_collision(
    players: Query<(&PlayerPosition, &PlayerId)>,
    coins: Query<(Entity, &CoinPosition)>,
    mut commands: Commands,
    mut player_score: ResMut<PlayerScore>,
    mut server: ResMut<ServerConnectionManager>
){
    for (player_position,player_id) in &players {
        for (coin_entity, coin_position) in &coins {
            let distance = player_position.distance(**coin_position);
            if distance < COIN_RADIUS + PLAYER_RADIUS {
                commands.entity(coin_entity).despawn();
                let new_player_score = player_score.scores.get(player_id).unwrap() + 1;

                player_score.scores.insert(player_id.clone(), new_player_score);
                println!("player scores: {:#?}", player_score.scores);
                commands.spawn((
                    CoinBundle::new(Vec2::new(thread_rng().gen_range(0..400) as f32, thread_rng().gen_range(0..400) as f32)),
                    Name::new("Coin"),
                    Replicate::default()
                ));
                server
                    .send_message_to_target::<Channel1, Message1>(Message1(player_score.scores.clone()), NetworkTarget::All)
                    .unwrap_or_else(|e| {
                        error!("Failed to send message: {:?}", e);
                    });
            }
        }
    }
}
