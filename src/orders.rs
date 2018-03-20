use simulation::*;
use std::collections::VecDeque;
pub type ChronalTime = u32;

//chronal events
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct TransportCommand{
    pub player: Player,
    pub from: NodeInd,
    pub to: NodeInd,
    pub percent: u8,
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct SendAllCommand{
    pub player: Player,
    pub from: NodeInd,
    pub to: Option<NodeInd>,
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub enum ChronalCommand{
    Transport(TransportCommand),
    SendAll(SendAllCommand),
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct ChronalEvent{
    pub time: ChronalTime,
    pub command: ChronalCommand,
}

//achronal events
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub enum AchronalEvent{
    Chronal(ChronalEvent),
    Timejump(ChronalTime), //gives a time directly, no backing struct
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct AchronalCommand{
    pub player: Player,
    pub event: AchronalEvent,
}
pub type CommandBuffer = VecDeque<Vec<AchronalCommand>>;