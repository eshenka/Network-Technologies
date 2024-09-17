use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddrV4};
use rand::Rng;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};

fn send_package(socket: &Socket, addr: &SockAddr, id: &u64) {
    let mut buf: Vec<u8> = Vec::from("secret message".as_bytes());

    buf.extend(id);

    let send_result = socket.send_to("secret message".as_bytes(), addr);

    let size = match send_result {
        Ok(size) => size,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => 0,
            _ => panic!("Failed sending message"),
        }
    };
}

fn _send_package(socket: &Socket, addr: &SockAddr, id: &u64) {
    let mut buf: Vec<u8> = Vec::from("secret message".as_bytes());


    let send_result = socket.send_to("secret message".as_bytes(), addr);

    let size = match send_result {
        Ok(size) => size,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => 0,
            _ => panic!("Failed sending message"),
        }
    };

}
fn recv_package(socket: &Socket, id: &u64) {
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


    if bytes != 0 {
        let message = &buf[0..14];
        let recv_id = &buf[14..22];
        
        if message == "secret message".as_bytes() && recv_id != id.as_bytes() {
            println!("Got message from: {:?}", addr);
        }
    }

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

    loop {
        send_package(&socket, &addr, &id);
        recv_package(&socket);
    }
}
