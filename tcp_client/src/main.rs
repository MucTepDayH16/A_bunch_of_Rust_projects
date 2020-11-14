use
std::{
    io::Read,
    net::TcpStream,
    thread
};

static RUST_PORT: u16 = 7878;

struct IPv4(u8,u8,u8,u8);

//  static SERVER: IPv4 = IPv4( 0xc0, 0xa8, 0x01, 0x23 );
static SERVER: IPv4 = IPv4( 0x7f, 0x00, 0x00, 0x01 );

fn main() {
    let ip = format!(
        "{}.{}.{}.{}:{}", 
        SERVER.0,
        SERVER.1,
        SERVER.2,
        SERVER.3,
        RUST_PORT
    );

    println!("Connecting {}...", ip);
    let mut strm = match TcpStream::connect(&ip) {
        Ok(c)   => c,
        Err(_)  => {
            println!("Cannot fetch {}", &ip);
            return;
        }
    };

    let th = thread::spawn(move || {
        let mut buff = Vec::<u8>::new();
        'peer: loop {
            match strm.read(&mut buff) {
                Ok(_)   => println!("1 package"),
                Err(_)  => break 'peer,
            };
        }
    });

    th.join().unwrap();
}
