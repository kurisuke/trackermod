use hound;
use std::env;
use std::fs::File;
use trackermod::format::protracker::ProtrackerMod;
use trackermod::player::protracker::{ClockFreq, ProtrackerPlayer};
use trackermod::player::{OutputFormat, SampleFormat, SampleOutput};

fn main() {
    let mut x = env::args();
    x.next();
    let filename = x.next().unwrap();
    let filename_out = x.next().unwrap();

    let mut f = File::open(filename).unwrap();
    match ProtrackerMod::deserialize(&mut f) {
        Ok(mod_data) => {
            let output_format = OutputFormat {
                sample_rate: 48000,
                sample_format: SampleFormat::I16,
                channel_count: 2,
            };
            let spec = hound::WavSpec {
                channels: 2,
                sample_rate: 48000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };

            let mut player =
                ProtrackerPlayer::new(mod_data, ClockFreq::Pal, output_format).unwrap();
            let mut writer = hound::WavWriter::create(filename_out, spec).unwrap();

            loop {
                let samples = player.get_samples().unwrap();
                match samples {
                    SampleOutput::I16(buf) => {
                        if buf.len() == 0 {
                            break;
                        }
                        for b in buf {
                            writer.write_sample(*b).unwrap();
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            print!("Error: {}\n", e);
        }
    }
}
