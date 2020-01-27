use super::{InitError, OutputFormat, PlayError, SampleBuffer, SampleFormat, SampleOutput};
use crate::format::protracker::{Effect, EffectType, EffectTypeExtended, ProtrackerMod};

pub struct ProtrackerPlayer {
    pt_mod: ProtrackerMod,
    clock_freq: ClockFreq,
    output_format: OutputFormat,
    state: PlayerState,
    buffer: SampleBuffer,
}

pub enum ClockFreq {
    Pal,
    Ntsc,
}

static CLOCK_FREQ_PAL: f64 = 7_093_789.2;
static CLOCK_FREQ_NTSC: f64 = 7_159_090.5;

struct PlayerState {
    pub sequence_pos: usize,
    pub cur_pattern: usize,
    pub cur_division: usize,
    pub cur_tick: u8,
    pub ticks_per_min: u16,
    pub ticks_per_div: u8,
    pub channels: Vec<ChannelState>,
}

struct ChannelState {
    pub sample_no: Option<u8>,
    pub volume: u16,
    pub volume_diff: i16,
    pub offset: f64,
    pub period: u16,
    pub period_diff: i16,
    pub period_target: Option<u16>,
    pub in_loop: bool,
}

impl ProtrackerPlayer {
    pub fn new(
        pt_mod: ProtrackerMod,
        clock_freq: ClockFreq,
        output_format: OutputFormat,
    ) -> Result<ProtrackerPlayer, InitError> {
        if output_format.channel_count < 1 || output_format.channel_count > 2 {
            Err(InitError::ChannelCountError)
        } else {
            let buffer = match output_format.sample_format {
                SampleFormat::I16 => SampleBuffer::I16(vec![]),
                SampleFormat::U16 => SampleBuffer::U16(vec![]),
                SampleFormat::F32 => SampleBuffer::F32(vec![]),
            };
            let num_channels = pt_mod.patterns[0].divisions[0].channel_data.len();
            let start_pattern = pt_mod.sequence[0];
            let mut player = ProtrackerPlayer {
                pt_mod,
                clock_freq,
                output_format,
                state: PlayerState::default(num_channels),
                buffer,
            };
            player.state.cur_pattern = start_pattern as usize;
            Ok(player)
        }
    }

    pub fn get_samples(&mut self) -> Result<SampleOutput, PlayError> {
        if self.state.sequence_pos >= self.pt_mod.sequence.len() {
            // song finished, return empty slice
            match &self.buffer {
                SampleBuffer::I16(_) => Ok(SampleOutput::I16(&[])),
                SampleBuffer::U16(_) => Ok(SampleOutput::U16(&[])),
                SampleBuffer::F32(_) => Ok(SampleOutput::F32(&[])),
            }
        } else {
            self.next_tick();

            match &self.buffer {
                SampleBuffer::I16(buf) => Ok(SampleOutput::I16(&buf)),
                SampleBuffer::U16(buf) => Ok(SampleOutput::U16(&buf)),
                SampleBuffer::F32(buf) => Ok(SampleOutput::F32(&buf)),
            }
        }
    }

    fn next_tick(&mut self) {
        if self.state.cur_tick == 0 {
            // new division

            // parse notes & effects and set parameters
            self.update_division();

            let samples_per_tick = (self.output_format.sample_rate as f32 * 60.0
                / self.state.ticks_per_min as f32)
                .floor() as usize;
            match &mut self.buffer {
                SampleBuffer::I16(buf) => {
                    buf.resize(
                        samples_per_tick * self.output_format.channel_count as usize,
                        0,
                    );
                }
                SampleBuffer::U16(buf) => {
                    buf.resize(
                        samples_per_tick * self.output_format.channel_count as usize,
                        0,
                    );
                }
                SampleBuffer::F32(buf) => {
                    buf.resize(
                        samples_per_tick * self.output_format.channel_count as usize,
                        0.0,
                    );
                }
            };
        }
        self.update_tick();

        self.calc_output_samples();

        self.state.cur_tick += 1;
        self.state.cur_tick %= self.state.ticks_per_div;

        if self.state.cur_tick == 0 {
            // advance division
            self.state.cur_division += 1;
            self.state.cur_division %= 64;
            if self.state.cur_division == 0 {
                // advance sequence position
                self.state.sequence_pos += 1;
                if self.state.sequence_pos < self.pt_mod.sequence.len() {
                    self.state.cur_pattern = self.pt_mod.sequence[self.state.sequence_pos] as usize;
                }
            }
        }
    }

    fn calc_output_samples(&mut self) {
        let advances = self.calc_advances();

        let samples_per_tick = (self.output_format.sample_rate as f32 * 60.0
            / self.state.ticks_per_min as f32)
            .floor() as usize;
        for idx in 0..samples_per_tick {
            if self.output_format.channel_count == 1 {
                let mut val = 0.0;
                let num_input_channels = self.state.channels.len();
                for c in 0..num_input_channels {
                    val += self.next_sample(c, advances[c]);
                }
                val /= (num_input_channels / 2) as f64;

                match &mut self.buffer {
                    SampleBuffer::I16(buf) => buf[idx] = val.floor() as i16,
                    SampleBuffer::U16(buf) => buf[idx] = (val.floor() + 16384.0) as u16,
                    SampleBuffer::F32(buf) => {
                        buf[idx] = (val / 16384.0) as f32;
                    }
                };
            } else if self.output_format.channel_count == 2 {
                let mut val_l = 0.0;
                let mut val_r = 0.0;
                let num_input_channels = self.state.channels.len();
                for c in 0..num_input_channels {
                    if c % 4 == 0 || c % 4 == 3 {
                        val_l += self.next_sample(c, advances[c]);
                    } else {
                        val_r += self.next_sample(c, advances[c]);
                    }
                    val_l /= (num_input_channels / 4) as f64;
                    val_r /= (num_input_channels / 4) as f64;
                }

                match &mut self.buffer {
                    SampleBuffer::I16(buf) => buf[2 * idx] = val_l.floor() as i16,
                    SampleBuffer::U16(buf) => buf[2 * idx] = (val_l.floor() + 16384.0) as u16,
                    SampleBuffer::F32(buf) => {
                        buf[2 * idx] = (val_l / 16384.0) as f32;
                    }
                };

                match &mut self.buffer {
                    SampleBuffer::I16(buf) => buf[2 * idx + 1] = val_r.floor() as i16,
                    SampleBuffer::U16(buf) => buf[2 * idx + 1] = (val_r.floor() + 16384.0) as u16,
                    SampleBuffer::F32(buf) => {
                        buf[2 * idx + 1] = (val_r / 16384.0) as f32;
                    }
                };
            }
        }
    }

    fn next_sample(&mut self, channel_no: usize, advance: f64) -> f64 {
        let channel = &mut self.state.channels[channel_no];

        if channel.sample_no.is_none() {
            return 0.0;
        }

        let sample = &self.pt_mod.samples[channel.sample_no.unwrap() as usize - 1];

        let offset_int = channel.offset.floor() as usize;
        let val = if offset_int < sample.length as usize {
            sample.data[offset_int] as f64 * channel.volume as f64
        } else {
            0.0
        };

        channel.offset += advance;

        if sample.repeat_length > 2 {
            // can loop
            if channel.in_loop {
                let mut overflow =
                    channel.offset - (sample.repeat_offset + sample.repeat_length) as f64;
                while overflow > 0.0 {
                    channel.offset = sample.repeat_offset as f64 + overflow;
                    overflow =
                        channel.offset - (sample.repeat_offset + sample.repeat_length) as f64;
                }
            } else {
                // enter loop
                let mut overflow = channel.offset - sample.length as f64;
                channel.in_loop = true;
                while overflow > 0.0 {
                    channel.offset = sample.repeat_offset as f64 + overflow;
                    overflow =
                        channel.offset - (sample.repeat_offset + sample.repeat_length) as f64;
                }
            }
        }

        val
    }

    fn update_division(&mut self) {
        let pattern = &self.pt_mod.patterns[self.state.cur_pattern];
        let division = &pattern.divisions[self.state.cur_division];

        for (idx, cd) in division.channel_data.iter().enumerate() {
            if cd.sample > 0 {
                // new note
                let cs = &mut self.state.channels[idx];
                let vol = self.pt_mod.samples[cd.sample as usize - 1].volume as u16;

                // handle effects
                match cd.effect {
                    Effect::Normal {
                        effect_type,
                        param1,
                        param2,
                    } => {
                        match effect_type {
                            EffectType::SlideUp => {
                                cs.reset(cd.sample, vol, cd.period);
                                cs.period_diff = (param1 * 16 + param2) as i16;
                            }
                            EffectType::SlideDown => {
                                cs.reset(cd.sample, vol, cd.period);
                                cs.period_diff = 0 - (param1 * 16 + param2) as i16;
                            }
                            EffectType::SlideToNote => {
                                // do not set the new note directly, keep the last one
                                cs.period_target = Some(cd.period);
                                if cd.period < cs.period {
                                    cs.period_diff = 0 - (param1 * 16 + param2) as i16;
                                } else {
                                    cs.period_diff = (param1 * 16 + param2) as i16;
                                }
                            }
                            EffectType::SetVolume => {
                                cs.reset(cd.sample, vol, cd.period);
                                cs.volume = (param1 * 16 + param2) as u16;
                            }
                            EffectType::SetSpeed => {
                                let speed_val = param1 * 16 + param2;
                                if speed_val == 0 {
                                    // ignore
                                } else if speed_val >= 1 && speed_val <= 32 {
                                    self.state.ticks_per_div = speed_val;
                                } else {
                                    self.state.ticks_per_min = 4 * 6 * speed_val as u16;
                                }
                            }
                            _ => {
                                cs.reset(cd.sample, vol, cd.period);
                            }
                        }
                    }
                    Effect::Extended {
                        effect_type: _,
                        param: _,
                    } => {
                        cs.reset(cd.sample, vol, cd.period);
                    }
                }
            }
        }
    }

    fn update_tick(&mut self) {
        for channel in self.state.channels.iter_mut() {
            match channel.period_target {
                Some(period_target) => {
                    if channel.period_diff < 0 {
                        channel.period = ((channel.period as i16 + channel.period_diff) as u16)
                            .max(period_target);
                    } else {
                        channel.period = ((channel.period as i16 + channel.period_diff) as u16)
                            .min(period_target);
                    }
                }
                None => {
                    channel.period = (channel.period as i16 + channel.period_diff) as u16;
                }
            }
        }
    }

    fn calc_advances(&self) -> Vec<f64> {
        let mut advances = vec![];
        for channel in self.state.channels.iter() {
            let cf = match self.clock_freq {
                ClockFreq::Pal => CLOCK_FREQ_PAL,
                ClockFreq::Ntsc => CLOCK_FREQ_NTSC,
            };
            let samples_per_sec = cf / channel.period as f64;
            advances.push(samples_per_sec / (self.output_format.sample_rate * 2) as f64);
        }
        advances
    }
}

impl PlayerState {
    fn default(num_channels: usize) -> PlayerState {
        let channels: Vec<_> = (0..num_channels).map(|_| ChannelState::default()).collect();

        PlayerState {
            sequence_pos: 0,
            cur_pattern: 0,
            cur_division: 0,
            cur_tick: 0,
            ticks_per_min: 4 * 6 * 125,
            ticks_per_div: 6,
            channels,
        }
    }
}

impl ChannelState {
    fn default() -> ChannelState {
        ChannelState {
            sample_no: None,
            volume: 0,
            volume_diff: 0,
            offset: 0.0,
            period: 0,
            period_diff: 0,
            period_target: None,
            in_loop: false,
        }
    }

    fn reset(&mut self, sample_no: u8, volume: u16, period: u16) {
        self.sample_no = Some(sample_no);
        self.volume = volume;
        self.volume_diff = 0;
        self.offset = 0.0;
        self.period = period;
        self.period_diff = 0;
        self.period_target = None;
        self.in_loop = false;
    }
}
