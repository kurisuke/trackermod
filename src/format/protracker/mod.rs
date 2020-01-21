use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::{From, TryFrom};
use std::io::{Read, Seek, SeekFrom};
use std::str::from_utf8;

pub struct ProtrackerMod {
    pub title: String,
    pub samples: Vec<Sample>,
    pub pattern_table: Vec<u8>,
    pub patterns: Vec<Pattern>,
}

pub struct Sample {
    pub name: String,
    pub finetune: i8,
    pub length: u32,
    pub volume: u8,
    pub repeat_offset: u32,
    pub repeat_length: u32,
    pub data: Vec<i8>,
}

pub struct Pattern {
    pub divisions: Vec<Division>,
}

pub struct Division {
    pub channel_data: Vec<ChannelData>,
}

pub struct ChannelData {
    pub sample: u8,
    pub period: u16,
    pub effect: Effect,
}

pub enum Effect {
    Normal {
        effect_type: EffectType,
        param1: u8,
        param2: u8,
    },
    Extended {
        effect_type: EffectTypeExtended,
        param: u8,
    },
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum EffectType {
    Arpeggio = 0x0,
    SlideUp = 0x1,
    SlideDown = 0x2,
    SlideToNote = 0x3,
    Vibrato = 0x4,
    SlideToNoteVolumeSlide = 0x5,
    VibratoVolumeSlide = 0x6,
    Tremolo = 0x7,
    SetPanningPosition = 0x8,
    SetSampleOffset = 0x9,
    VolumeSlide = 0xa,
    PositionJump = 0xb,
    SetVolume = 0xc,
    PatternBreak = 0xd,
    Extended = 0xe,
    SetSpeed = 0xf,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum EffectTypeExtended {
    SetFilterOnOff = 0x0,
    FineslideUp = 0x1,
    FineslideDown = 0x2,
    Glissando = 0x3,
    SetVibratoWaveform = 0x4,
    SetFinetuneValue = 0x5,
    LoopPattern = 0x6,
    SetTremoloWaveform = 0x7,
    Unused = 0x8,
    RetriggerSample = 0x9,
    FineVolumeSlideUp = 0xa,
    FineVolumeSlideDown = 0xb,
    CutSample = 0xc,
    DelaySample = 0xd,
    DelayPattern = 0xe,
    InvertLoop = 0xf,
}

pub fn deserialize<R>(mut r: &mut R) -> std::io::Result<ProtrackerMod>
where
    R: Read + Seek,
{
    // check format
    // seek to 1080 = 20 + 31 * (22 + 2 + 1 + 1 + 2 + 2) + 1 + 1 + 128
    r.seek(SeekFrom::Start(1080))?;

    // check for file type (num samples / channels)
    let tag = parse_str(&mut r, 4)?;
    let (num_samples, num_channels, max_pattern) = match tag.as_ref() {
        "M.K." | "FLT4" | "4CHN" => (31u8, 4u8, 63u8),
        "M!K!" => (31u8, 4u8, 255u8),
        "6CHN" => (31u8, 4u8, 63u8),
        "FLT8" | "8CHN" => (31u8, 4u8, 63u8),
        _ => (15u8, 4u8, 63u8),
    };

    // go back to start of file
    r.seek(SeekFrom::Start(0))?;

    // module title
    let title = parse_str(&mut r, 20)?;

    // samples
    let mut samples = vec![];
    for _i in 0..num_samples {
        samples.push(parse_sample_param(&mut r)?);
    }

    // parse pattern table
    let song_length = r.read_u8()?;
    r.read_u8()?; // ignore, legacy restart
    let mut pattern_table = vec![];
    for _i in 0..song_length {
        pattern_table.push(r.read_u8()?);
    }

    let padding = 128 - song_length as usize + 4;
    for _i in 0..padding {
        r.read_u8()?;
    }

    // determine number of patterns to read: max index from pattern table
    let num_patterns = (*pattern_table.iter().max().unwrap() + 1).min(max_pattern);

    // read patterns
    let mut patterns = vec![];
    for _i in 0..num_patterns {
        patterns.push(parse_pattern(&mut r, num_channels)?);
    }

    // read samples
    parse_sample_data(&mut r, &mut samples)?;

    Ok(ProtrackerMod {
        title,
        samples,
        pattern_table,
        patterns,
    })
}

fn parse_str(r: &mut dyn Read, length: usize) -> std::io::Result<String> {
    // read fixed number of bytes
    let mut buf = vec![];
    buf.resize(length, 0);
    r.read_exact(&mut buf)?;

    // trim trailing 0
    match buf.split(|&n| n == 0).next() {
        Some(x) => Ok(String::from(from_utf8(x).unwrap_or(""))),
        None => Ok(String::new()),
    }
}

fn parse_sample_param(mut r: &mut dyn Read) -> std::io::Result<Sample> {
    let name = parse_str(&mut r, 22)?;
    let length = r.read_u16::<BigEndian>()? as u32 * 2;
    let finetune = r.read_i8()?;
    let volume = r.read_u8()?;
    let repeat_offset = r.read_u16::<BigEndian>()? as u32 * 2;
    let repeat_length = r.read_u16::<BigEndian>()? as u32 * 2;

    Ok(Sample {
        name,
        length,
        finetune,
        volume,
        repeat_offset,
        repeat_length,
        data: vec![],
    })
}

fn parse_pattern(mut r: &mut dyn Read, num_channels: u8) -> std::io::Result<Pattern> {
    let mut divisions = vec![];
    for _i in 0..64 {
        let mut channel_data = vec![];
        for _j in 0..num_channels {
            channel_data.push(parse_channel_data(&mut r)?);
        }
        divisions.push(Division { channel_data });
    }
    Ok(Pattern { divisions })
}

fn parse_channel_data(r: &mut dyn Read) -> std::io::Result<ChannelData> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;

    let nibbles = split_nibbles(&buf);
    let sample = (nibbles[0].0 << 4) | nibbles[2].0;
    let period =
        ((nibbles[0].1 as u16) << 8) | ((nibbles[1].0 as u16) << 4) | (nibbles[1].1 as u16);
    let effect = parse_effect(nibbles[2].1, nibbles[3].0, nibbles[3].1)?;

    Ok(ChannelData {
        sample,
        period,
        effect,
    })
}

fn parse_effect(t: u8, x: u8, y: u8) -> std::io::Result<Effect> {
    let effect_type = match EffectType::try_from(t) {
        Ok(effect_type) => effect_type,
        Err(e) => {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
    };

    if effect_type == EffectType::Extended {
        let effect_type_extended = match EffectTypeExtended::try_from(t) {
            Ok(effect_type_extended) => effect_type_extended,
            Err(e) => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
            }
        };
        Ok(Effect::Extended {
            effect_type: effect_type_extended,
            param: y,
        })
    } else {
        Ok(Effect::Normal {
            effect_type,
            param1: x,
            param2: y,
        })
    }
}

fn split_nibbles(v: &[u8]) -> Vec<(u8, u8)> {
    v.iter().map(|x| ((x & 0xf0) >> 4, x & 0x0f)).collect()
}

fn parse_sample_data(r: &mut dyn Read, samples: &mut Vec<Sample>) -> std::io::Result<()> {
    for sample in samples {
        if sample.length > 0 {
            let mut data = vec![];
            data.resize(sample.length as usize, 0);
            r.read_exact(&mut data)?;

            // reinterpret as i8
            let data = data
                .into_iter()
                .map(|b| i8::from_be_bytes([b]))
                .collect::<Vec<_>>();
            sample.data = data;
        }
    }
    Ok(())
}
