//use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::*;
use std::str::FromStr;
use std::cmp::*;
use simulation::*;
use std::collections::VecDeque;
use bincode::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemConfig{
    pub tick_time: u32,
    pub command_delay: usize,
    pub port_from: Option<u16>,
    pub port_to: Option<u16>,
}

pub struct NetworkManager{
    sock: UdpSocket,
    target: SocketAddrV4,
    received: VecDeque<bool>, //TODO: handle >2 players
}

const PACKET_CONNECT: u8 = 0;
const PACKET_ORDER: u8 = 1;

impl NetworkManager{
    pub fn new(ip: &str, conf: &SystemConfig) -> NetworkManager{
        let (port_to, port_from) = (conf.port_from.unwrap(), conf.port_to.unwrap());
        let addr = format!("{}:{}","0.0.0.0",port_from);
        let target = SocketAddrV4::from_str(&format!("{}:{}",ip,port_to)).expect("Error parsing Socket Addr V4");
        let sock = UdpSocket::bind(&addr).expect(&format!("Error binding socket to {}",&addr));
        sock.set_nonblocking(true).expect("socket nonblocking failed");
        let mut received = VecDeque::new();
        for _ in 0..conf.command_delay{
            received.push_front(true);
        }
        NetworkManager{sock, target, received}
    }
    pub fn attempt_connect(&mut self, _player: Player) -> bool{
        //send connection request
        let mut buf = [0; 512];
        let request= [0;1];
        self.sock.send_to(&request, self.target).expect("Sending failed");
        //and see if someone is connecting to us
        match self.sock.recv_from(&mut buf) {
            Ok(n) => {
                let (number_of_bytes, src_addr) = n;
                if buf[0] == PACKET_CONNECT && number_of_bytes == 1{
                    match src_addr{
                        SocketAddr::V4(src_addr_v4) =>{
                            if src_addr_v4 != self.target {
                                println!("Detected incoming-- gave up on {}, connecting to {}", self.target, &src_addr_v4);
                                self.target = src_addr_v4;
                                //and send them a request. Note that if this is dropped, things are bad.
                                //But this should never happen anyway, so w/e
                                let request = [0;1];
                                self.sock.send_to(&request, self.target).expect("Sending failed");
                            }
                        },
                        SocketAddr::V6(_) => println!("Packet received from ipv6, discarding"),
                    }
                    println!("Connected!");
                    return true;
                } else {
                    println!("Unknown packet received from {} (first byte {})", &src_addr, buf[0]);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                //wait_for_fd();
                //println!("no socket data");
            }
            Err(e) => println!("encountered IO error: {}", e),
        }
        return false;
    }
    fn process_commands(&mut self, orders: &mut OrdersType, buf: &[u8], turn_t: u64, conf: &SystemConfig){
        let rec_turn_t: u64 =  deserialize_from(&buf[0..8]).unwrap();
        let rec_turn = rec_turn_t as usize;
        let turn = turn_t as usize;
        let mut rec_orders: OrdersType = deserialize_from(&buf[8..]).unwrap();
        let from = max(turn, rec_turn);
        let to = min(turn+conf.command_delay, rec_turn+rec_orders.len());
        println!("from:{},to:{},rec_orders:{},orders:{},recieved:{:?}",from, to, rec_orders.len(), orders.len(), self.received);
        for i in from..to{
            let mine = i-turn;
            let rec = i-rec_turn;
            for o in rec_orders[rec].drain(0..){
                if !orders[mine].contains(&o){
                    orders[mine].push(o);
                }
            }
            if !self.received[mine] {
                self.received[mine] = true;
            }
        }
    }
    pub fn receive_commands(&mut self, orders: &mut VecDeque<Vec<Order>>, turn: u64, conf: &SystemConfig){
        let mut buf = [0; 512];
        //first check if someone is connecting to us
        match self.sock.recv_from(&mut buf) {
            Ok(n) => {
                let (number_of_bytes, src_addr) = n;
                if buf[0] == PACKET_ORDER{
                    match src_addr{
                        SocketAddr::V4(src_addr_v4) =>{
                            if src_addr_v4 != self.target {
                                println!("Packet from {}, but connected to {}", self.target, &src_addr_v4);
                            }
                            let reader = &buf[1..number_of_bytes];
                            self.process_commands(orders, reader, turn, conf);
                        },
                        SocketAddr::V6(_) => println!("Packet received from ipv6, discarding"),
                    }
                    println!("Connected!");
                } else {
                    println!("Unknown packet received from {} (first byte {})", &src_addr, buf[0]);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                //wait_for_fd();
                //println!("no socket data");
            }
            Err(e) => println!("encountered IO error: {}", e),
        }
    }
    pub fn send_commands(&mut self, orders: &OrdersType, turn: u64){
        let mut buf = Vec::new();
        buf.push(PACKET_ORDER);
        serialize_into(&mut buf, &turn).unwrap();
        serialize_into(&mut buf, orders).unwrap();
        self.sock.send_to(&buf, self.target).expect("Sending failed");
        println!("Sending Packet of size {}", buf.len());
    }
    pub fn can_advance(&self) -> bool {
        if self.received[0]{
            print!(".");
            true
        } else {
            println!("connection, skipped turn");
            false
        }
    }
    pub fn advance(&mut self) {
        self.received.push_back(false);
        let x = self.received.pop_front().unwrap();
        debug_assert!(x == true);
    }
}