use std::{
    io::{stdin, stdout, Write},
    net::{Ipv4Addr, TcpStream},
};

// const SERVER: Ipv4Addr = Ipv4Addr::new(0x7f, 0x00, 0x00, 0x01);
const LOCALHOST: Ipv4Addr = Ipv4Addr::new(0x00, 0x00, 0x00, 0x00);
const PORT: u16 = 7878;

fn main() {
    println!("Connecting {}...", LOCALHOST);

    let mut buf = String::new();
    let stdin = stdin();
    let mut stdout = stdout();
    loop {
        let mut strm = match TcpStream::connect((LOCALHOST, PORT)) {
            Ok(c) => c,
            Err(_) => {
                println!("Cannot fetch {}", LOCALHOST);
                return;
            }
        };

        stdout.write(b"> ").unwrap();
        stdout.flush().unwrap();

        if let Err(e) = stdin.read_line(&mut buf) {
            eprintln!("{e:?}");
            continue;
        }

        if let Err(e) = strm.write(buf.as_bytes()) {
            eprintln!("{e:?}");
            continue;
        }
    }
}
