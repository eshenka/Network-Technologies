use core::panic;
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::time::{SystemTime, Duration};
use rand::Rng;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use zerocopy::AsBytes;

const BUFFER_SIZE: usize = 256;
const MESSAGE_SIZE: usize = 22;
const SECRET_SIZE: usize = 14;
const ID_SIZE: usize = 8;

const SECRET: &str = "secret message";

const WAIT_TIME: u64 = 5;

fn send_package(socket: &Socket, addr: &str, id: u64) {
    let addr = SocketAddr::from_str(addr).unwrap();
    let addr = SockAddr::from(addr);

    let mut vec: Vec<u8> = Vec::from(SECRET.as_bytes());
    id.as_bytes().iter().for_each(|&b| vec.push(b));
    let buf: [u8; 22] = vec.try_into().unwrap();

    let send_result = socket.send_to(&buf, &addr);

    let _size = match send_result {
        Ok(size) => size,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => 0,
            _ => panic!("Failed sending message"),
        }
    };
}

fn print_new_client(client: Client) {
    println!("New client [id: {:?}; IP: {:?}] joined", client.id, client.addr);
}

fn recv_package(socket: &Socket, clients: &mut Vec<Client>) {
    let mut buf: [MaybeUninit<u8>; BUFFER_SIZE] = [const { MaybeUninit::uninit() }; BUFFER_SIZE];
    let recv_result = socket.recv_from(&mut buf);

    let size_default: usize = 0;
    let addr_default = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    let addr_default = SockAddr::from(addr_default);

    let buf: &[u8; BUFFER_SIZE] = unsafe { std::mem::transmute(&buf) };

    let (bytes, addr) = match recv_result {
        Ok(res) => res,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => (size_default, addr_default),
            _ => panic!("Failed recieving message"),
        }
    };

    if bytes == MESSAGE_SIZE && &buf[0..SECRET_SIZE] == SECRET.as_bytes() {
        let recv_id = &buf[SECRET_SIZE..SECRET_SIZE + ID_SIZE];

        let addr = addr.as_socket().unwrap();

        let index = clients.iter().position(|&client| client.id.as_bytes() == recv_id);
        match index {
            Some(ind) => {
                clients[ind].time = SystemTime::now();
            },
            None => {
                let new_client = Client {
                    id: u64::from_le_bytes(recv_id.try_into().unwrap()),
                    addr,
                    time: SystemTime::now(),
                };

                print_new_client(new_client.clone());
                clients.push(new_client);
            }
        }
    }

}

fn print_inactive_client(client: &Client) {
    println!("Client [id: {:?}; IP: {:?}] has left", client.id, client.addr);
}

fn remove_inactive_clients(clients: &mut Vec<Client>) {
    let time = SystemTime::now();
    let wait_time = Duration::new(WAIT_TIME, 0);

    for client in &mut *clients {
        if time.duration_since(client.time).unwrap() >= wait_time {
            print_inactive_client(client);
        }
    }

    clients.retain(|&client| time.duration_since(client.time).unwrap() < wait_time);
}

#[derive(Clone, Copy)]
struct Client {
    id: u64,
    addr: SocketAddr,
    time: SystemTime,
}

fn generate_client_id() -> u64 {
    let mut rng = rand::thread_rng();
    let shift = 0x0101010101010101;
    
    rng.gen_range(0..u64::MAX - shift) + shift
}

fn bind_to_multicast_socket(addr: &str) -> Socket {
    let addr = SocketAddr::from_str(addr)
        .expect("Wrong addr");

    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))
        .expect("Failed socket creation");

    socket.set_reuse_port(true)
        .expect("Failed setting port reusable");
    socket.set_nonblocking(true)
        .expect("Failed to make socket nonblocking");

    let addr = SockAddr::from(addr);
    socket.bind(&addr)
        .expect("Failed bind");

    socket
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("No address provided");
    }
    let addr = &args[1];

    let socket = bind_to_multicast_socket(&addr);

    let id = generate_client_id();

    let mut clients: Vec<Client> = Vec::new();

    loop {
        send_package(&socket, &addr, id.clone());
        recv_package(&socket, &mut clients);

        remove_inactive_clients(&mut clients); 
    }
}
