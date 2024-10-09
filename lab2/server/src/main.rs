use socket2::{Domain, Protocol, Socket, Type, SockAddr};
use std::fs::File;
use std::io::Write;
use std::net::SocketAddrV4;
use std::path::PathBuf;
use std::thread;
use std::mem::MaybeUninit;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const BACKLOG: i32 = 50;

const MAX_FILE_NAME_LEN: usize = 4096;
const SECRET: &str = "secret message";
const MAX_SEND_SIZE: usize = SECRET.len() + 8 + MAX_FILE_NAME_LEN;

const DIR_PATH: &str = "uploads/";

const RECV_SIZE: usize = 4096;


fn client_processing(client_socket: Socket, client_addr: SockAddr) {
    println!("Starting new client process");

    let mut connection_msg: [MaybeUninit<u8>; MAX_SEND_SIZE] = [const { MaybeUninit::uninit()}; MAX_SEND_SIZE];

    let connect_msg_size = client_socket.recv(&mut connection_msg).unwrap();

    let buf: &[u8; MAX_SEND_SIZE] = unsafe { std::mem::transmute(&connection_msg) };

    if &buf[0..SECRET.len()] != SECRET.as_bytes() {
        return;
    }

    let mut ok_msg: Vec<u8> = Vec::new();
    ok_msg.extend(SECRET.as_bytes());
    ok_msg.extend("OK".as_bytes());

    client_socket.send(&ok_msg).unwrap();

    let file_name_size = connect_msg_size - SECRET.len() - 8;
    let file_name_idx = connect_msg_size - file_name_size;
    let file_name = String::from_utf8(buf[file_name_idx..connect_msg_size].to_vec()).unwrap();
    let file_name: Vec<&str> = file_name.split('/').collect();
    let file_name = file_name[file_name.len() - 1];

    
    let file_path = PathBuf::from(DIR_PATH).join(file_name);

    let file_size = &buf[SECRET.len()..file_name_idx];
    let file_size = usize::from_le_bytes(file_size.try_into().unwrap());

    let recieved = Arc::new(Mutex::new(0));
    let mut bytes_recv = 0;

    let mut file = File::options()
        .write(true)
        .create(true)
        .open(file_path).unwrap();

    let terminated = Arc::new(Mutex::new(false));

    thread::spawn({
        let mut old_bytes = 0;
        let smth = Arc::clone(&recieved);
        let terminated = Arc::clone(&terminated);
        let i = 1;
        move || loop {
            let new_bytes = smth.lock().unwrap();

            let delta = *new_bytes - old_bytes;
            old_bytes = *new_bytes;

            let full_speed = *new_bytes / (3 * i);
            let moment_speed = delta / 3;

            drop(new_bytes);

            let client = SockAddr::as_socket_ipv4(&client_addr);
            println!("Client {:?} average speed: {full_speed} bytes/second; moment speed {moment_speed} bytes/second", client);

            if *terminated.lock().unwrap() {
                break;
            }

            thread::sleep(Duration::from_secs(3));
        }
    });

    while bytes_recv < file_size {
        let mut file_data = [0u8; RECV_SIZE];

        let data_size = client_socket.recv(unsafe { std::mem::transmute(&mut file_data[..]) }).unwrap();
        if data_size == 0 {
            break;
        }

        file.write(&file_data[..data_size]).unwrap();

        let mut bytes = recieved.lock().unwrap();
        *bytes += data_size;
        bytes_recv = *bytes;
    }

    *terminated.lock().unwrap() = true;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
        .expect("Failed socket creation");

    let socket_addr = SocketAddrV4::from_str(&args[1]).unwrap();
    let sock_addr = SockAddr::from(socket_addr);

    socket.bind(&sock_addr).unwrap();

    socket.listen(BACKLOG)
        .expect("Failed listening");


    loop {
        let (snd_socket, snd_addr) = socket.accept().unwrap();
        thread::spawn(move || {
            client_processing(snd_socket, snd_addr)
        }); 
    };
    

}
