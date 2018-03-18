use simulation::*;
use library::*;
//use ggez::*;
use ggez::event::*;



pub struct GameInterface{
    pub selected: Option<NodeInd>,
    pub center_loc: Point2,
}

impl GameInterface{
    pub fn new() -> GameInterface{
        GameInterface{selected: None, center_loc: pt(0.,0.)}
    }

    pub fn mouse_up(&mut self, button: MouseButton, pt: Ipt, player: Player, sim: &Simulation, orders: &mut OrdersType) {
        if let Some(selected) = self.selected {
            match button{
                MouseButton::Left => {
                    let next_o = sim.check_planets(pt, 96);
                    if let Some(next) = next_o {
                        if next != selected {
                            if sim.world.contains_edge(selected, next){
                                let command = TransportCommand{from: selected, to: next, percent: 50};
                                let order = Order{player, command: CommandEnum::Transport(command)};
                                orders.back_mut().unwrap().push(order);
                            }
                        }
                    }
                }
                MouseButton::Right => {
                    let next_o = sim.check_planets(pt, 96);
                    let mut command = SendAllCommand{from: selected, to: None};
                    if let Some(next) = next_o {
                        if next != selected {
                            if sim.world.contains_edge(selected, next){
                                command = SendAllCommand{from: selected, to: Some(next)};
                            }
                        }
                    }
                    let order = Order{player, command: CommandEnum::SendAll(command)};
                    orders.back_mut().unwrap().push(order);
                }
                _ => {},
            }
        }
        self.selected = None;
    }

    pub fn mouse_down(&mut self, button: MouseButton, pt: Ipt, _player: Player, sim: &Simulation, _orders: &mut OrdersType) {
        if button == MouseButton::Left || button == MouseButton::Right {
            self.selected = sim.check_planets(pt, 96);
        }
    }
}