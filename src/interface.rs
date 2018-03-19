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
    pub width: i32,
    pub height: i32,
}
pub struct GameInterface{
    pub selected: Option<NodeInd>,
    pub center_loc: Vector2,
    keyboard: KeyboardStates,
}

fn chronal_event(command: ChronalCommand, timestep: ChronalTime, orders: &mut CommandBuffer){
    let event = ChronalEvent{time: timestep+(orders.len() as ChronalTime), command};
    let order = AchronalEvent::Chronal(event);
    orders.back_mut().unwrap().push(order);
}
impl GameInterface {
    pub fn new() -> GameInterface {
        GameInterface { selected: None, center_loc: Vector2::new(0., 0.), keyboard: KeyboardStates::new(false) }
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
            let event = TimejumpCommand{time_to, player};
            let order = AchronalEvent::Timejump(event);
            orders.back_mut().unwrap().push(order);

        } else {
            let world_pt = pt + na::Vector2::new(self.center_loc.x.round() as i32, self.center_loc.y.round() as i32);
            if let Some(selected) = self.selected {
                match button {
                    MouseButton::Left => {
                        let next_o = sim.check_planets(world_pt, 96);
                        if let Some(next) = next_o {
                            if next != selected {
                                if sim.world.contains_edge(selected, next) {
                                    let transport = TransportCommand { player, from: selected, to: next, percent: 50 };
                                    let command = ChronalCommand::Transport(transport);
                                    chronal_event(command, sim.timestep, orders);
                                }
                            }
                        }
                    }
                    MouseButton::Right => {
                        let next_o = sim.check_planets(world_pt, 96);
                        let mut send_all = SendAllCommand { player, from: selected, to: None };
                        if let Some(next) = next_o {
                            if next != selected {
                                if sim.world.contains_edge(selected, next) {
                                    send_all.to = Some(next);
                                }
                            }
                        }
                        let command = ChronalCommand::SendAll(send_all);
                        chronal_event(command, sim.timestep, orders);
                    }
                    _ => {},
                }
            }
            self.selected = None;
        }
    }
    pub fn mouse_down(&mut self, button: MouseButton, pt: Ipt, player: Player, timeline: &Timeline, _orders: &mut CommandBuffer, conf: &InterfaceConfig) {
        let sim = &timeline[player];
        if pt.y > (conf.height - conf.ui_height) {
            self.selected = None;
        } else {
            let world_pt = pt + na::Vector2::new(self.center_loc.x.round() as i32, self.center_loc.y.round() as i32);
            if button == MouseButton::Left || button == MouseButton::Right {
                self.selected = sim.check_planets(world_pt, 96);
            }
        }
    }
    pub fn mouse_move(&mut self, state: MouseState, rel: Vector2){
        if state.middle() {
            self.center_loc -= rel;
        }
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