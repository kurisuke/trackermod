use std::env;
use std::fs::File;

fn main() {
    let mut x = env::args();
    x.next();
    let filename = x.next().unwrap();

    let mut f = File::open(filename).unwrap();
    match trackermod::format::protracker::deserialize(&mut f) {
        Ok(_mod_data) => {
            println!("Parsed successfully!");
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
