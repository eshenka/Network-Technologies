use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr};
use std::time::{SystemTime, Duration};
use rand::Rng;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use zerocopy::{AsBytes, byteorder::U64};

fn send_package(socket: &Socket, addr: &SockAddr, id: u64) {
    let mut vec: Vec<u8> = Vec::from("secret message".as_bytes());
    id.as_bytes().iter().for_each(|&b| vec.push(b));
    let buf: [u8; 22] = vec.try_into().unwrap();

    let send_result = socket.send_to(&buf, addr);

    let _size = match send_result {
        Ok(size) => size,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => 0,
            _ => panic!("Failed sending message"),
        }
    };
}

fn print_new_client(client: Client) {
    println!("New client with id {:?} joined", client.id);
}

fn recv_package(socket: &Socket, id: u64, clients: &mut Vec<Client>) {
    let mut buf: [MaybeUninit<u8>; 256] = [const { MaybeUninit::uninit() }; 256];
    let recv_result = socket.recv_from(&mut buf);

    let size: usize = 0;
    let addr_def_init = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    let addr_def = SockAddr::from(addr_def_init);

    let buf: &[u8; 256] = unsafe { std::mem::transmute(&buf) };

    let (bytes, addr) = match recv_result {
        Ok(res) => res,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => (size, addr_def),
            _ => panic!("Failed recieving message"),
        }
    };

    if bytes == 22 {
        let message = &buf[0..14];
        let recv_id = &buf[14..22];

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
    println!("Client with id {:?} has left", client.id);
}

fn remove_inactive_clients(clients: &mut Vec<Client>) {
    let time = SystemTime::now();
    let wait_time = Duration::new(5, 0);

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

fn main() {
    let addr_init = SocketAddrV4::new(Ipv4Addr::new(224, 0, 0, 1), 8880);
    let addr = SockAddr::from(addr_init);

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .expect("Failed socket creation");

    socket.set_reuse_port(true)
        .expect("Failed setting port reusable");
    socket.set_nonblocking(true)
        .expect("Failed to make socket nonblocking");
    socket.bind(&addr)
        .expect("Failed bind");

    let mut rng = rand::thread_rng();
    let shift = 0x0101010101010101;
    let id: u64 = rng.gen_range(0..u64::MAX - shift) + shift;

    let mut clients: Vec<Client> = Vec::new();

    loop {
        send_package(&socket, &addr, id.clone());
        recv_package(&socket, id.clone(), &mut clients);

        remove_inactive_clients(&mut clients); 
    }
}
