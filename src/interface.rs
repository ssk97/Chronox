use simulation::*;
use library::*;
//use ggez::*;
use ggez::event::*;

pub struct GameInterface{
    pub selected: Option<NodeInd>,
}

impl GameInterface{
    pub fn new() -> GameInterface{
        GameInterface{selected: None}
    }

    pub fn mouse_up(&mut self, button: MouseButton, pt: Ipt, player: Player, sim: &Simulation, orders: &mut OrdersType) {
        if let Some(selected) = self.selected {
            let next_o = sim.check_planets(pt, 96);
            if let Some(next) = next_o {
                if next != selected {
                    if sim.world.contains_edge(selected, next){
                        let check_command = match button{
                            MouseButton::Left => {
                                let c = TransportCommand{from: selected, to: next, percent: 50};
                                Some(CommandEnum::Transport(c))
                            },
                            _ => None
                        };
                        if let Some(command) = check_command {
                            let order = Order { player, command};
                            orders.back_mut().unwrap().push(order);
                        }
                    }
                }
            }
        }
        self.selected = None;
    }

    pub fn mouse_down(&mut self, button: MouseButton, pt: Ipt, _player: Player, sim: &Simulation, _orders: &mut OrdersType) {
        if button == MouseButton::Left {
            self.selected = sim.check_planets(pt, 96);
        }
    }
}