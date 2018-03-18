use simulation::*;
use library::*;
//use ggez::*;
use ggez::event::*;

use plain_enum::*;
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
    pub scroll_speed: f32
}
pub struct GameInterface{
    pub selected: Option<NodeInd>,
    pub center_loc: Point2,
    keyboard: KeyboardStates,
}

impl GameInterface {
    pub fn new() -> GameInterface {
        GameInterface { selected: None, center_loc: pt(0., 0.), keyboard: KeyboardStates::new(false) }
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
    pub fn mouse_up(&mut self, button: MouseButton, pt: Ipt, player: Player, sim: &Simulation, orders: &mut OrdersType) {
        if let Some(selected) = self.selected {
            match button {
                MouseButton::Left => {
                    let next_o = sim.check_planets(pt, 96);
                    if let Some(next) = next_o {
                        if next != selected {
                            if sim.world.contains_edge(selected, next) {
                                let command = TransportCommand { from: selected, to: next, percent: 50 };
                                let order = Order { player, command: CommandEnum::Transport(command) };
                                orders.back_mut().unwrap().push(order);
                            }
                        }
                    }
                }
                MouseButton::Right => {
                    let next_o = sim.check_planets(pt, 96);
                    let mut command = SendAllCommand { from: selected, to: None };
                    if let Some(next) = next_o {
                        if next != selected {
                            if sim.world.contains_edge(selected, next) {
                                command = SendAllCommand { from: selected, to: Some(next) };
                            }
                        }
                    }
                    let order = Order { player, command: CommandEnum::SendAll(command) };
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