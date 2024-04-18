use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, time::{Duration, SystemTime}};
use crate::{protocol::{Direction, *}, shared::*};
use bevy::{log::LogPlugin, prelude::*};
use bevy::log::Level;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use local_ip_address::local_ip;

pub fn build_client_app() -> App{
    let mut app = App::new();
    app.add_plugins((DefaultPlugins.build().set(LogPlugin{
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        update_subscriber: None
    }), WorldInspectorPlugin::new()));
    
    let my_local_ip = local_ip().unwrap();
    println!("This is my local IP address: {:?}", my_local_ip);
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_addr = SocketAddr::new(my_local_ip, 0);
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 21)), 5001);
    let auth = Authentication::Manual {
        server_addr,
        client_id: current_time.as_millis() as u64,
        private_key: KEY,
        protocol_id: PROTOCOL_ID,
        };

    // You can add a link conditioner to simulate network conditions
    let link_conditioner = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(0),
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
    app.add_plugins(SharedPlugin);
    app.add_systems(Startup, (init, scorebord));
    app.add_systems(FixedPreUpdate, buffer_input.in_set(InputSystemSet::BufferInputs));
    app.add_systems(Update, (handle_connections, handle_predicted_spawn, handle_interpolated_spawn, receive_message1));
    app.add_systems(FixedUpdate, player_movement);
    app
}

fn init(
    mut client: ResMut<ClientConnection>,
) {
    client.connect().expect("failed to connect to server");
}

#[derive(Component)]
pub struct PlayerMesh;


pub(crate) fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    for connection in connections.read() {
        let client_id = connection.client_id();
        commands.spawn(TextBundle::from_section(
            format!("Client {}", client_id),
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    }
}

pub(crate) fn buffer_input(
    tick_manager: Res<TickManager>,
    mut input_manager: ResMut<InputManager<Inputs>>,
    keypress: Res<ButtonInput<KeyCode>>,
) {
    let tick = tick_manager.tick();
    let mut input = Inputs::None;
    let mut direction = Direction {
        up: false,
        down: false,
        left: false,
        right: false,
    };
    if keypress.pressed(KeyCode::KeyW) || keypress.pressed(KeyCode::ArrowUp) {
        direction.up = true;
    }
    if keypress.pressed(KeyCode::KeyS) || keypress.pressed(KeyCode::ArrowDown) {
        direction.down = true;
    }
    if keypress.pressed(KeyCode::KeyA) || keypress.pressed(KeyCode::ArrowLeft) {
        direction.left = true;
    }
    if keypress.pressed(KeyCode::KeyD) || keypress.pressed(KeyCode::ArrowRight) {
        direction.right = true;
    }
    if !direction.is_none() {
        input = Inputs::Direction(direction);
    }
    input_manager.add_input(input, tick)
}

fn player_movement(
    mut position_query: Query<&mut PlayerPosition, With<Predicted>>,
    mut input_reader: EventReader<InputEvent<Inputs>>,
) {
    for input in input_reader.read() {
        if let Some(input) = input.input() {
            for position in position_query.iter_mut() {
                shared_movement_behaviour(position, input);
            }
        }
    }
}

pub(crate) fn handle_predicted_spawn(
    mut predicted: Query<&mut PlayerColor, Added<Predicted>>
){
    for mut color in predicted.iter_mut() {
        color.0.set_b(1.0);
    }
}

pub(crate) fn handle_interpolated_spawn(
    mut interpolated: Query<&mut PlayerColor, Added<Interpolated>>
){
    for mut color in interpolated.iter_mut(){
        color.0.set_g(1.0);
    }
}

#[derive(Component)]
pub struct Scorebord;

pub fn scorebord (
    mut commands: Commands,
    asset_server: Res<AssetServer>
){
    commands.spawn((NodeBundle {
        style: Style{
            width: Val::Percent(20.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Percent(5.0),
            ..default()
        },
        //background_color: Color::RED.into(),
        ..default()
    },
    Name::new("HudNode"),
)).with_children(|parent| {
    parent.spawn(NodeBundle{
        style: Style{
            width: Val::Percent(100.0),
            height: Val::Percent(5.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Percent(5.0),
            ..default()
        },
        //background_color: Color::BLACK.into(),
        ..default()
    });
    
    parent.spawn((TextBundle{
        text: Text::from_section(
                "Scorebord: ",
                 TextStyle { 
                    font_size: 32.0, 
                    color: Color::WHITE, 
                    ..default()
                },
             ),
        ..default()},
        Scorebord,
        Name::new("scorebord")
        ));
    });
}

pub(crate) fn receive_message1(
    mut reader: EventReader<MessageEvent<Message1>>,
    mut scorebord_query: Query<&mut Text, With<Scorebord>>
){
    for event in reader.read(){
        for mut scorebord in &mut scorebord_query{
            info!("Player scores are: {:?}", event.message());
            let scorebord_text = &event.message().0;
            scorebord.sections[0].value = format!("Scorebord: {:#?}", scorebord_text);
        }
    }
}