#![feature(box_syntax)]

use std::mem::take;

type It<T> = Box<dyn Iterator<Item=T>>;

struct Primes<T> (Option<It<T>>);

impl Iterator for Primes<u64> {
    type Item = u64;
    
    fn next(&mut self) -> Option<u64> {
        let mut it = take(&mut self.0)?;
        let p = it.next()?;
        self.0 = Some(box it.filter(move |x| x % p != 0));
        Some(p)
    }
}

fn primes<I>(it: I) -> It<u64>
where I: 'static + Iterator<Item=u64> {
    box Primes(Some(box it))
}

fn read_stdin<T: std::str::FromStr>() -> Option<T> {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read from stdin");
	
	input.trim().parse().ok()
}

fn main() {
	loop {
		println!("Enter count of prime numbers:");
		let x: usize = match read_stdin() {
			Some(val) => val,
			None => {println!("You have to enter valid number!"); continue},
		};
		
		println!("First {} prime numbers:", x);
		println!("{:?}", primes(2..).take(x).collect::<Vec<u64>>());
	}
}