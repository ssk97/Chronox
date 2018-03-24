use simulation::*;
use timeline::*;
use library::*;
//use ggez::*;
use ggez::event::*;

plain_enum_mod!(keyboard_states, Key {
    Up, Left, Right, Down,
});

fn to_keystate(keycode: Keycode) -> Option<Key>{
    match keycode{
        Keycode::Up  | Keycode::W => Some(Key::Up),
        Keycode::Left| Keycode::A => Some(Key::Left),
        Keycode::Down| Keycode::S => Some(Key::Down),
        Keycode::Right|Keycode::D => Some(Key::Right),
        _ => None
    }
}

type KeyboardStates= EnumMap<Key, bool>;

#[derive(Serialize, Deserialize, Debug)]
pub struct InterfaceConfig{
    pub scroll_speed: f32,
    pub colors: Vec<u32>,
    pub ui_height: i32,
    pub energy_bar_height: i32,
    pub width: i32,
    pub height: i32,
}
pub struct GameInterface{
    pub selected: Option<NodeInd>,
    pub center_loc: Vector2,
    pub send_percent: i32,
    keyboard: KeyboardStates,
}
fn add_order(order: AchronalCommand, orders: &mut CommandBuffer){
    orders.back_mut().unwrap().push(order);

}
fn chronal_event(command: ChronalCommand, player: Player, orders: &mut CommandBuffer){
    let event = AchronalCommandTypes::Chronal(command);
    let order = AchronalCommand{player, event};
    add_order(order, orders);
}
impl GameInterface {
    pub fn new() -> GameInterface {
        GameInterface { selected: None, center_loc: Vector2::new(0., 0.), keyboard: KeyboardStates::new(false), send_percent: 50 }
    }

    pub fn update(&mut self, conf: &InterfaceConfig) {
        if self.keyboard[Key::Up] {
            self.center_loc.y -= conf.scroll_speed;
        }
        if self.keyboard[Key::Left] {
            self.center_loc.x -= conf.scroll_speed;
        }
        if self.keyboard[Key::Down] {
            self.center_loc.y += conf.scroll_speed;
        }
        if self.keyboard[Key::Right] {
            self.center_loc.x += conf.scroll_speed;
        }
    }
    pub fn mouse_up(&mut self, button: MouseButton, pt: Ipt, player: Player, timeline: &Timeline, orders: &mut CommandBuffer, conf: &InterfaceConfig) {
        let sim = &timeline[player];
        if pt.y > (conf.height-conf.ui_height) {
            let width = conf.width as f32;
            let x_pos = (pt.x as f32)/width;
            let time = x_pos*((timeline.right_edge - timeline.left_edge) as f32);
            let time_to = (time as ChronalTime)+timeline.left_edge;
            let event = AchronalCommandTypes::Timejump(time_to);
            let command = AchronalCommand{event, player};
            add_order(command, orders);

        } else {
            let world_pt = pt + na::Vector2::new(self.center_loc.x.round() as i32, self.center_loc.y.round() as i32);
            if let Some(selected) = self.selected {
                match button {
                    MouseButton::Left => {
                        let next_o = sim.check_planets(world_pt, 96);
                        if let Some(next) = next_o {
                            if next != selected {
                                if sim.world.contains_edge(selected, next) {
                                    let transport = TransportCommand { to: next, percent: (self.send_percent as u8) };
                                    let command = ChronalCommandTypes::Transport(transport);
                                    let event = ChronalCommand{time: sim.timestep+(orders.len() as ChronalTime), target: self.selected, player, command};
                                    chronal_event(event, player, orders);
                                }
                            }
                        }
                    }
                    MouseButton::Right => {
                        let next_o = sim.check_planets(world_pt, 96);
                        let mut send_all = SendAllCommand {to: None };
                        if let Some(next) = next_o {
                            if next != selected {
                                if sim.world.contains_edge(selected, next) {
                                    send_all.to = Some(next);
                                }
                            }
                        }
                        let command = ChronalCommandTypes::SendAll(send_all);
                        let event = ChronalCommand{time: sim.timestep+(orders.len() as ChronalTime), target: self.selected, player, command};
                        chronal_event(event, player, orders);
                    }
                    _ => {},
                }
            }
            self.selected = None;
        }
    }
    pub fn mouse_down(&mut self, button: MouseButton, pt: Ipt, player: Player, timeline: &Timeline, orders: &mut CommandBuffer, conf: &InterfaceConfig) {
        let sim = &timeline[player];
        if pt.y > (conf.height - conf.ui_height) {
            self.selected = None;
        } else {
            if button == MouseButton::Left || button == MouseButton::Right {
                let world_pt = pt + na::Vector2::new(self.center_loc.x.round() as i32, self.center_loc.y.round() as i32);
                let selection = sim.check_planets(world_pt, 96);
                if let Some(prev_selected) = self.selected {
                    if let Some(this_selection) = selection{
                        if this_selection == prev_selected{
                            let clear = ClearCommand{time: sim.timestep+(orders.len() as ChronalTime),target:this_selection };
                            let event = AchronalCommandTypes::ClearCommands(clear);
                            let command = AchronalCommand{event, player};
                            add_order(command, orders);
                        }
                    }
                    self.selected = None;
                } else {
                    self.selected = selection;
                }
            }
        }
    }
    pub fn mouse_move(&mut self, state: MouseState, rel: Vector2){
        if state.middle() {
            self.center_loc -= rel;
        }
    }
    pub fn mouse_wheel(&mut self, amount: i32){
        self.send_percent += amount*10;
        self.send_percent = bound(self.send_percent, 10, 100);
    }

    pub fn key_down(&mut self, keycode: Keycode) {
        if let Some(key) = to_keystate(keycode) {
            self.keyboard[key] = true;
        }
    }
    pub fn key_up(&mut self, keycode: Keycode) {
        if let Some(key) = to_keystate(keycode) {
            self.keyboard[key] = false;
        }
    }
}