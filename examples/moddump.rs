use std::env;
use std::fs::File;
use trackermod::format::protracker::ProtrackerMod;

fn main() {
    let mut x = env::args();
    x.next();
    let filename = x.next().unwrap();

    let mut f = File::open(filename).unwrap();
    match ProtrackerMod::deserialize(&mut f) {
        Ok(mod_data) => {
            println!("{}", mod_data.info_str());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
