pub mod protracker;

pub struct OutputFormat {
    pub sample_rate: u32,
    pub sample_format: SampleFormat,
    pub channel_count: u16,
}

pub enum SampleFormat {
    I16,
    U16,
    F32,
}

pub enum SampleOutput<'a> {
    I16(&'a [i16]),
    U16(&'a [u16]),
    F32(&'a [f32]),
}

pub enum SampleBuffer {
    I16(Vec<i16>),
    U16(Vec<u16>),
    F32(Vec<f32>),
}

#[derive(Debug)]
pub enum InitError {
    ChannelCountError,
}

#[derive(Debug)]
pub enum PlayError {
    Other,
}
