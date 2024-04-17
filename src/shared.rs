use bevy::prelude::*;
use bevy::utils::Duration;
use lightyear::prelude::*;
use lightyear::shared::config::Mode;
use crate::protocol::*;

pub const KEY: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const PROTOCOL_ID: u64 = 0;
pub const PLAYER_RADIUS: f32 = 4.0;
pub const COIN_RADIUS: f32 = 2.0;
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, shared_init)
            .add_systems(Update, draw);
    }
}

pub fn shared_init(
    mut commands: Commands
){
    commands.spawn(Camera2dBundle::default());
}

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_millis(40),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 64.0),
        },
        mode,
    }
}

pub(crate) fn shared_movement_behaviour(mut position: Mut<PlayerPosition>, input: &Inputs) {
    const MOVE_SPEED: f32 = 1.0;
    match input {
        Inputs::Direction(direction) => {
            if direction.up {
                position.y += MOVE_SPEED;
            }
            if direction.down {
                position.y -= MOVE_SPEED;
            }
            if direction.left {
                position.x -= MOVE_SPEED;
            }
            if direction.right {
                position.x += MOVE_SPEED;
            }
        }
        _ => {}
    }
}

pub(crate) fn draw(
    mut gizmos: Gizmos,
    players: Query<(&PlayerPosition, &PlayerColor)>,
    coins: Query<(&CoinPosition, &PlayerColor)>
) {
    for (player_position, player_color) in &players {
        gizmos.circle_2d(
        Vec2::new(player_position.x, player_position.y),
        PLAYER_RADIUS,
        player_color.0
        );
    }

    for (coin_position, coin_color) in &coins {
        gizmos.circle_2d(
            Vec2::new(coin_position.x, coin_position.y),
            COIN_RADIUS,
            coin_color.0
            ); 
    }
}