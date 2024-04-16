use std::{net::{Ipv4Addr, SocketAddr}, time::{Duration, SystemTime}};
use crate::{protocol::{Direction, *}, shared::*};
use bevy::{log::LogPlugin, prelude::*};
use bevy::log::Level;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub fn build_client_app() -> App{
    let mut app = App::new();
    app.add_plugins((DefaultPlugins.build().set(LogPlugin{
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        update_subscriber: None
    }), WorldInspectorPlugin::new()));
    
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
    let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 5001);
    let auth = Authentication::Manual {
        server_addr,
        client_id: current_time.as_millis() as u64,
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
    app.add_plugins(SharedPlugin);
    app.init_resource::<MyPlayerID>();
    app.add_systems(Startup, init);
    app.add_systems(FixedPreUpdate, buffer_input.in_set(InputSystemSet::BufferInputs));
    app.add_systems(Update, (handle_connections, handle_predicted_spawn, handle_interpolated_spawn, update_player_mesh_position));
    app.add_systems(FixedUpdate, player_movement);
    app
}

fn init(
    mut client: ResMut<ClientConnection>,
    mut commands: Commands,
) {
    client.connect().expect("failed to connect to server");

    commands.spawn(Camera3dBundle{
        transform: Transform::from_xyz(0.0, 20.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Resource)]
pub struct MyPlayerID{
    pub my_player_id: PlayerId,
}

impl Default for MyPlayerID {
    fn default() -> Self {
        Self{
            my_player_id: PlayerId(ClientId::Netcode(0)),
        }
    }
}

#[derive(Component)]
pub struct PlayerMesh;


pub(crate) fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut my_player_id: ResMut<MyPlayerID>,
) {
    for connection in connections.read() {
        let client_id = connection.client_id();
        my_player_id.my_player_id = PlayerId(client_id);
        commands.spawn(TextBundle::from_section(
            format!("Client {}", client_id),
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        commands.spawn((PbrBundle{
            mesh: meshes.add(Cuboid::new(2.0, 2.0, 2.0)),
            material: materials.add(Color::BLUE),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        }, PlayerMesh));
    }
}

pub(crate) fn update_player_mesh_position(
    player: Query<(&PlayerPosition, &PlayerId), With<Confirmed>>,
    mut player_mesh: Query<&mut Transform, With<PlayerMesh>>,
    my_player_id: Res<MyPlayerID>,
){
    if let Ok(mut player_mesh_transform) = player_mesh.get_single_mut(){
        for (player_position, player_id) in &player {
            if my_player_id.my_player_id == *player_id{
                player_mesh_transform.translation = Vec3::new(player_position.x, player_position.y, player_position.z);
            }
        }
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