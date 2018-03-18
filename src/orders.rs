use simulation::*;
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct TransportCommand{
    pub from: NodeInd,
    pub to: NodeInd,
    pub percent: u8,
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct SendAllCommand{
    pub from: NodeInd,
    pub to: Option<NodeInd>,
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub enum CommandEnum{
    Transport(TransportCommand),
    SendAll(SendAllCommand),
}
#[derive(Serialize, Deserialize, Debug,  PartialEq)]
pub struct Order{
    pub player: Player,
    pub command: CommandEnum
}

pub type OrdersType = VecDeque<Vec<Order>>;