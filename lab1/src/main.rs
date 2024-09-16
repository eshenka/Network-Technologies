use std::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn send_package(socket: &UdpSocket) {
    let send_result = socket.send("secret message".as_bytes());

    let size = match send_result {
        Ok(size) => size,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => 0,
            _ => panic!("Failed sending message"),
        }
    };

    if size != 0 {
        println!("sended");
    }
}

fn recieve_package(socket: &UdpSocket) {
    let mut buf = [0; 256];
    let recv_result = socket.recv_from(&mut buf);
    
    let size: usize = 0;
    let addr_def: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);


    let (bytes, _addr) = match recv_result {
        Ok(res) => res,
        Err(error) => match error.kind() {
            std::io::ErrorKind::WouldBlock => (size, addr_def),
            _ => panic!("Failed recieving message"),
        }
    };

    if bytes != 0 {
        println!("{:?}", buf);
    }

}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mulicast_address = &args[1];

    let port = "8880";

    let address = format!("{mulicast_address}:{port}");
    let socket = std::net::UdpSocket::bind(address.clone())
        .expect("Failed to bind");
    socket.connect(address)
        .expect("Failed connection");
    
    nix::sys::socket::setsockopt(
                &socket,
                nix::sys::socket::sockopt::ReusePort,
                &true)
    .expect("Failed to make port reusable");
    nix::sys::socket::setsockopt(
        &socket, 
        nix::sys::socket::sockopt::ReuseAddr, 
        &true)
    .expect("Failed to make address reusable");

    
    socket.set_nonblocking(true)
        .expect("Failed to make socket nonblocking");

    loop {
        send_package(&socket);
        recieve_package(&socket);
    }
}
