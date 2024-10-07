use std::mem::MaybeUninit;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use std::fs::File;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use zerocopy::IntoBytes;

const MAX_FILE_NAME_LEN: usize = 4096;
const GB: u64 = 1073741824;

const SECRET: &str = "secret message";
const MAX_SEND_SIZE: usize = SECRET.len() + 1073741824 + MAX_FILE_NAME_LEN;
const OK_MES_SIZE: usize = SECRET.len() + "OK".len();

#[warn(dead_code)]
pub struct Config {
    file_name: String,
    addr: IpAddr,
    port: u16,
}

pub fn get_config(args: Vec<String>) -> Result<Config, String> {
    if args.len() < 4 {
        return Err("Failed 'get_config': not enough arguments provided".to_string());
    } else {
        Ok(Config {
            file_name: args[1].clone(),
            addr: IpAddr::from_str(&args[2])
                .expect("Failed parsing IP address"),
            port: args[3].clone().parse().unwrap(),
        })
    }
}

pub fn check_config(cfg: &Config) -> Result<(), String> {
    if cfg.file_name.len() >= MAX_FILE_NAME_LEN {
        return Err("File is too long".to_string());
    } 

    let file = std::fs::File::open(cfg.file_name.clone());
    if let Err(_) = file {
        return Err("Failed opening file".to_string());
    }
    let file = file.unwrap();

    if file.metadata().unwrap().len() > GB {
        return Err("Cannot use file larger than 1Gb".to_string());
    }

    Ok(())
}

pub fn send_file(cfg: Config) {
    let mut domain;

    let socket_addr: SocketAddr = match cfg.addr {
        IpAddr::V4(addr) => { domain = Domain::IPV4;
            SocketAddr::new(IpAddr::V4(addr), cfg.port)
        },
        IpAddr::V6(addr) => { domain = Domain::IPV6;
            SocketAddr::new(IpAddr::V6(addr), cfg.port)
        },
    };

    let sock_addr = SockAddr::from(socket_addr);
    
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))
        .expect("Failed creating socket");
    socket.bind(&sock_addr)
        .expect("Failed binding");


    let file_fd = std::fs::File::open(cfg.file_name.clone())
        .expect("Failed ipening file");

    let file_size = file_fd.metadata().unwrap().len();

    let mut vec: Vec<u8> = Vec::from(SECRET.as_bytes());
    file_size.as_bytes().iter().for_each(|&b| vec.push(b));
    cfg.file_name.as_bytes().iter().for_each(|&b| vec.push(b));

    let buf: [u8; MAX_SEND_SIZE] = vec.try_into().unwrap();

    socket.send(&buf)
        .expect("Failed sending header");

    let mut ans: [MaybeUninit<u8>; OK_MES_SIZE] = [const { MaybeUninit::uninit() }; OK_MES_SIZE];
    let recv_len = socket.recv(&mut ans).unwrap();

    socket.sendfile(
        &file_fd,
        0, 
        None,
    ).expect("Failed file sending");
}

