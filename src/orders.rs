use simulation::*;
use std::collections::VecDeque;
pub type ChronalTime = u32;

//chronal events
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct TransportCommand{
    pub to: NodeInd,
    pub percent: u8,
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct SendAllCommand{
    pub to: Option<NodeInd>,
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub enum ChronalCommandTypes{
    Transport(TransportCommand),
    SendAll(SendAllCommand),
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct ChronalCommand{
    pub time: ChronalTime,
    pub target: Option<NodeInd>,
    pub player: Player,
    pub command: ChronalCommandTypes,
}

//achronal events
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct ClearCommand{
    pub time: ChronalTime,
    pub target: NodeInd,
}

#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub enum AchronalCommandTypes{
    Chronal(ChronalCommand),
    Timejump(ChronalTime), //gives a time directly, no backing struct
    ClearCommands(ClearCommand)
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct AchronalCommand{
    pub player: Player,
    pub event: AchronalCommandTypes,
}
pub type CommandBuffer = VecDeque<Vec<AchronalCommand>>;