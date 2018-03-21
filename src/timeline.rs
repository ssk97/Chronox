//use library::*;
use simulation::*;
use std::collections::{VecDeque, BTreeSet};
use std::ops::Index;
type ChronoEnergy = u16;
pub const MAX_CHRONOENERGY: ChronoEnergy = 500;
struct TimePoint{
    world: Simulation,
    commands: Vec<ChronalCommand>,
    metadata: SimMetadata,
}
#[derive(Copy, Clone)]
pub struct Timewave{
    pub time: ChronalTime,
    pub speed: u8,
}
pub struct Timeline{
    multiverse: VecDeque<TimePoint>,
    pub left_edge: ChronalTime,
    pub right_edge: ChronalTime,
    pub present: ChronalTime,
    pub timewaves: VecDeque<Timewave>,
    pub player_timewaves: PlayerArr<Timewave>,
    pub chrono_energy: PlayerArr<ChronoEnergy>,
    next_wave: i64,//current time of the next timewave to spawn (should be < left_edge)
}
impl Timeline{
    pub fn new(starting: Simulation) -> Timeline{
        let mut multiverse = VecDeque::new();
        let timepoint = TimePoint{commands: Vec::new(), world: starting, metadata: SimMetadata::new()};
        multiverse.push_front(timepoint);
        let mut timewaves = VecDeque::new();
        timewaves.push_front(Timewave{time:0, speed: 2});//initial right-edge timewave
        let player_timewaves = PlayerArr::new(Timewave{time: 0, speed: 1});
        let chrono_energy = PlayerArr::new(450);
        Timeline{multiverse, left_edge: 0, right_edge: 1, present: 0, timewaves, player_timewaves, chrono_energy, next_wave: -1}
    }
    fn exists(&self, time: ChronalTime) -> bool{
        let index = (time - self.left_edge) as usize;
        index < self.multiverse.len()
    }
    fn get_time(&self, time: ChronalTime) -> &TimePoint {
        &self.multiverse[(time - self.left_edge) as usize]
    }
    fn get_time_mut( & mut self, time: ChronalTime) -> & mut TimePoint {
        &mut self.multiverse[(time - self.left_edge) as usize]
    }

    pub fn get_metadata(&self, time: ChronalTime) -> &SimMetadata{
        if self.exists(time) {
            &self.multiverse[(time - self.left_edge) as usize].metadata
        } else {
            self.get_metadata(time - 1)
        }
    }
    pub fn chrono_cost(&self, time: ChronalTime) -> ChronoEnergy{
        if time < self.present {
            (self.present - time) as ChronoEnergy
        } else {
            0
        }
    }
    pub fn chrono_energy_limit(&self, energy: ChronoEnergy) -> ChronalTime{
        if self.present > (energy as ChronalTime) {
            self.present - (energy as ChronalTime)
        } else {
            0
        }
    }
    //returns if successful. On failure, destroy this timewave (or reset its speed to 1 if player)
    //adds the times needed to the BTreeSet to be evaluated afterwards
    fn timecheck(&mut self, wave: Timewave, times_wanted: &mut BTreeSet<ChronalTime> ) -> bool{
        for offset in 0..wave.speed{
            let prev_time = wave.time+(offset as ChronalTime);
            let time = prev_time+1;
            if time < self.right_edge{
                times_wanted.insert(prev_time);
            } else {
                return false;
            }
        }
        return true;
    }
    pub fn evaluate_timestep(&mut self, commands: Vec<AchronalCommand>){
        //regenerate chronoenergy
        for player in Player::values(){
            if self.chrono_energy[player] < MAX_CHRONOENERGY{
                self.chrono_energy[player] += 1;
            }
        }
        //first evaluate events/orders that have been through the buffer
        for order in commands{
            let player = order.player;
            match order.event{
                AchronalCommandTypes::Chronal(data) => {
                    let cost = self.chrono_cost(data.time);
                    if cost < self.chrono_energy[player] {
                        self.chrono_energy[player] -= cost;
                        let timepoint = self.get_time_mut(data.time);
                        timepoint.commands.push(data);
                    }
                },
                AchronalCommandTypes::Timejump(data) => {
                    self.player_timewaves[player].time = data;
                },
                AchronalCommandTypes::ClearCommands(data) => {
                    let cost = self.chrono_cost(data.time);
                    if cost < self.chrono_energy[player] {
                        self.chrono_energy[player] -= cost;
                        let left_edge = self.left_edge;
                        let from_time = (data.time - left_edge) as usize;
                        for i in from_time..self.multiverse.len() {
                            let m = &mut self.multiverse[i];
                            m.commands.retain(|command_data: &ChronalCommand| {
                                debug_assert_eq!((i as ChronalTime) + left_edge, command_data.time);
                                (player != command_data.player || command_data.target != Some(data.target))
                            });
                        }
                    }
                },
            }
        }
        //move timeline forward
        if self.present < 1000 {
            self.right_edge += 2;
        } else {
            self.right_edge += 1;
            self.left_edge += 1;
            self.multiverse.pop_front();
        }
        self.present += 1;

        self.next_wave += 3;
        if (self.left_edge as i64) < self.next_wave{
            self.next_wave -= 40;
            self.timewaves.push_front(Timewave{time:self.left_edge, speed: 3});
        }

        //move timewaves forward
        let mut times_to_update = BTreeSet::new();
        //player timewaves: Player::PASSIVE is stuck at present
        for player in Player::values(){
            let timewave = self.player_timewaves[player];
            let normal = self.timecheck(timewave, &mut times_to_update);
            if !normal{
                self.player_timewaves[player].speed = 1;
            }
            self.player_timewaves[player].time += self.player_timewaves[player].speed as ChronalTime;
        }

        let mut i = 0;
        while i < self.timewaves.len(){
            let timewave = self.timewaves[i];
            let normal = self.timecheck(timewave, &mut times_to_update);
            if normal{
                self.timewaves[i].time += self.timewaves[i].speed as ChronalTime;
                i += 1;
            } else {
                self.timewaves.remove(i);
            }
        }
        //actually do the world updates
        for prev_time in times_to_update {
            let time = prev_time+1;
            let (new_world, metadata) = {
                let prev = self.get_time(prev_time);
                prev.world.update(&prev.commands)
            };
            assert_eq!(new_world.timestep, time);
            if self.exists(time) {
                let t = self.get_time_mut(time);
                t.world = new_world;
                t.metadata = metadata;
            } else {
                let timepoint = TimePoint { commands: Vec::new(), world: new_world, metadata };
                self.multiverse.push_back(timepoint);
            }
        }
    }
}


impl Index<Player> for Timeline {
    type Output = Simulation;

    fn index(&self, player: Player) -> &Simulation {
        &self.multiverse[(self.player_timewaves[player].time - self.left_edge) as usize].world
    }
}