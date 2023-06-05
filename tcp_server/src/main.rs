use std::{
    env::args,
    io::Read,
    net::{Ipv4Addr, TcpListener},
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

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(0x00, 0x00, 0x00, 0x00);

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

    println!("Opening {} socket...", LOCALHOST);
    match TcpListener::bind((LOCALHOST, port)) {
        Ok(lsnr) => {
            println!("Starting listening for port {}", LOCALHOST);

            for strm in lsnr.incoming() {
                match strm {
                    Ok(mut strm) => {
                        let ip = strm.peer_addr();
                        let mut buf = String::new();
                        if let Err(e) = strm.read_to_string(&mut buf) {
                            println!("{e:?}");
                        } else {
                            println!("\t{ip:?}: {buf}");
                        }
                    }
                    Err(err) => {
                        println!(
                            "Incoming connection lost with error: {}",
                            err
                        );
                    }
                }
            }
            return;
        }
        Err(err) => {
            println!("An error accured: {}", err);
            return;
        }
    }
}
