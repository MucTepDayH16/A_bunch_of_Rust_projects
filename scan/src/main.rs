macro_rules! scan {
    ($($t:ty),+) => {{
        use std::io::BufRead;
        let mut buf = String::new();
        std::io::stdin().lock().read_line(&mut buf).unwrap();
        let mut input = buf.split(' ');
		($(
			input.next()
				.and_then(|w| w.trim().parse::<$t>().ok())
		),+)
    }}
}

fn main() {
	let exit_cmd = String::from("exit");
	
	loop {
		let x = scan!(String, i32);
		println!("{:?}", x);
		
		match x {
			(Some(s), Some(n)) if s == exit_cmd =>
				std::process::exit(n as i32),
			_ => continue,
		}
	}
}