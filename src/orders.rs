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
pub struct TimejumpCommand{
    pub player: Player,
    pub time_to: ChronalTime,
}

#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub enum AchronalEvent{
    Chronal(ChronalEvent),
    Timejump(TimejumpCommand),
}

pub type CommandBuffer = VecDeque<Vec<AchronalEvent>>;