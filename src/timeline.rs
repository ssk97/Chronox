//use library::*;
use simulation::*;
use std::collections::{VecDeque, BTreeSet};
use std::ops::Index;
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
    pub player_timewaves: PlayerArr<Timewave>,
    pub timewaves: VecDeque<Timewave>,
}
impl Timeline{
    pub fn new(starting: Simulation) -> Timeline{
        let mut multiverse = VecDeque::new();
        let timepoint = TimePoint{commands: Vec::new(), world: starting, metadata: SimMetadata::new()};
        multiverse.push_front(timepoint);
        let player_timewaves = PlayerArr::new(Timewave{time: 0, speed: 1});
        let mut timewaves = VecDeque::new();
        timewaves.push_front(Timewave{time:0, speed: 2});//initial right-edge timewave
        Timeline{multiverse, left_edge: 0, right_edge: 1, present: 0, player_timewaves, timewaves}
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
    pub fn evaluate_timestep(&mut self, commands: Vec<AchronalEvent>){
        //first evaluate events
        for event in commands{
            match event{
                AchronalEvent::Chronal(data) => {
                    let timepoint = self.get_time_mut(data.time);
                    timepoint.commands.push(data.command);
                },
                AchronalEvent::Timejump(data) => {
                    self.player_timewaves[data.player].time = data.time_to;
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

        if (self.present+self.left_edge)%40 == 0{//Current increments by 2 so won't skip... FOR NOW
            self.timewaves.push_front(Timewave{time:self.left_edge+1, speed: 3});//TODO: make not skip ever
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