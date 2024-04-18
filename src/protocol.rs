use std::{collections::HashMap, ops::Mul};
use bevy::prelude::*;
use derive_more::{Add, Mul};
use serde::{Deserialize, Serialize};

use lightyear::prelude::*;

// Input
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Direction {
    pub(crate) up: bool,
    pub(crate) down: bool,
    pub(crate) left: bool,
    pub(crate) right: bool,
}

impl Direction {
    pub(crate) fn is_none(&self) -> bool {
        !self.up && !self.down && !self.left && !self.right
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Inputs {
    Direction(Direction),
    Delete,
    Spawn,
    // NOTE: we NEED to provide a None input so that the server can distinguish between lost input packets and 'None' inputs
    None,
}
impl UserAction for Inputs {}

// Message
#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub HashMap<PlayerId, u32>);

#[message_protocol(protocol = "MyProtocol")]
pub enum Messages {
    Message1(Message1),
}

// Components
#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub ClientId);

#[derive(
    Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut, Add, Mul,
)]
pub struct PlayerPosition(Vec2);

impl Mul<f32> for &PlayerPosition {
    type Output = PlayerPosition;

    fn mul(self, rhs: f32) -> Self::Output {
        PlayerPosition(self.0 * rhs)
    }
}

#[derive(Component, Message, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

#[derive(
    Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut, Add, Mul,
)]
pub struct CoinPosition(Vec2);

impl Mul<f32> for &CoinPosition {
    type Output = CoinPosition;

    fn mul(self, rhs: f32) -> Self::Output {
        CoinPosition(self.0 * rhs)
    }
}

#[component_protocol(protocol = "MyProtocol")]
pub enum Components {
    #[sync(once)]
    PlayerId(PlayerId),
    #[sync(full)]
    PlayerPosition(PlayerPosition),
    #[sync(once)]
    PlayerColor(PlayerColor),
    #[sync(full)]
    CoinPosition(CoinPosition),
}


#[derive(Channel)]
pub struct Channel1;


protocolize! {
    Self = MyProtocol,
    Message = Messages,
    Component = Components,
    Input = Inputs,
}

pub(crate) fn protocol() -> MyProtocol {
    let mut protocol = MyProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        direction: ChannelDirection::Bidirectional,
    ..Default::default()
    });
    protocol
}

#[derive(Bundle)]
pub(crate) struct PLayerBundle {
    pub id: PlayerId,
    pub position: PlayerPosition,
    pub color: PlayerColor,
}

impl PLayerBundle {
    pub(crate) fn new(id: ClientId, position: Vec2) -> Self {
        let color = Color::RED;
        Self { 
            id: PlayerId(id), 
            position: PlayerPosition(position), 
            color: PlayerColor(color)
        }
    }
}

#[derive(Bundle)]
pub(crate) struct CoinBundle {
    pub position: CoinPosition,
    pub color: PlayerColor,
}

impl CoinBundle {
    pub(crate) fn new(position: Vec2) -> Self {
        let color = Color::GREEN;
        Self {
            position: CoinPosition(position), 
            color: PlayerColor(color) 
        }
    }
}