use std::io::Write;
use std::{
    env::args,
    net::TcpListener,
    thread,
};

fn get_port(options: &Vec<String>) -> Result<u16, String> {
    if let Ok(arg_num) = options.binary_search(&String::from("--port")) {
        if let Some(port_arg) = options.get(arg_num + 1) {
            if let Ok(port_val) = u16::from_str_radix(port_arg, 10) {
                Ok(port_val)
            } else {
                Err(String::from(NOT_A_NUMBER))
            }
        } else {
            Err(String::from(NO_PORT_ERROR))
        }
    } else {
        Err(String::from(NO_PORT_ERROR))
    }
}

static NOT_A_NUMBER: &str = "Port must be a number > 0 and < 65 536";
static NO_PORT_ERROR: &str = "Specify connection's port";

struct IPv4(u8, u8, u8, u8);
static LOCALHOST: IPv4 = IPv4(0x00, 0x00, 0x00, 0x00);

fn main() {
    let mut options = Vec::new();
    for arg in args() {
        for opt in arg.split('=').map(String::from) {
            options.push(opt);
        }
    }

    let port = match get_port(&options) {
        Ok(x) => x,
        Err(s) => {
            println!("{}", s);
            return;
        }
    };

    let ip = format!(
        "{}.{}.{}.{}:{}",
        LOCALHOST.0, LOCALHOST.1, LOCALHOST.2, LOCALHOST.3, port
    );

    println!("Opening {} socket...", ip);

    let ip = &ip;
    match TcpListener::bind(ip) {
        Ok(lsnr) => {
            println!("Starting listening for port {}", ip);

            let th = thread::spawn(move || {
                for strm_res in lsnr.incoming() {
                    match strm_res {
                        Ok(mut strm) => {
                            println!(
                                "Incoming connection established, peer address: {}",
                                strm.peer_addr().unwrap()
                            );
                            let mut buff = Vec::<u8>::with_capacity(256);
                            for l in b"Hello, world!" {
                                buff.push(*l);
                            }
                            strm.write(&buff[..]).unwrap();
                        }
                        Err(err) => {
                            println!("Incoming connection lost with error: {}", err);
                        }
                    }
                }
            });

            th.join().unwrap();
            return;
        }
        Err(err) => {
            println!("An error accured: {}", err);
            return;
        }
    }
}
