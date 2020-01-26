use super::*;

pub fn info_mod(pt_mod: &ProtrackerMod) -> String {
    let mut ret = format!("Title: {}\n", pt_mod.title);

    // samples

    ret.push_str("\nSamples:\n");
    ret.push_str(&info_samples(&pt_mod.samples));

    // sequence

    ret.push_str("\n\nSequence:\n");
    ret.push_str(&info_sequence(&pt_mod.sequence));

    // patterns

    ret.push_str("\n\n");

    for (idx, pattern) in pt_mod.patterns.iter().enumerate() {
        ret.push_str(&format!("Pattern {:>02x}:\n", idx));
        ret.push_str(&info_pattern(pattern, &pt_mod.samples));
        ret.push_str("\n");
    }

    ret
}

fn info_samples(samples: &[Sample]) -> String {
    let samples_strs: Vec<_> = samples
        .iter()
        .enumerate()
        .filter(|(_idx, sample)| sample.length > 0)
        .map(|(idx, sample)| {
            format!(
                "{:>02x} {:>22}, ft: {:>2}, len: {:>04x}, vol: {:>02x}, roff: {:>04x}, rlen:{:>04x}",
                idx + 1,
                sample.name,
                sample.finetune,
                sample.length,
                sample.volume,
                sample.repeat_offset,
                sample.repeat_length
            )
        })
        .collect();
    samples_strs.join("\n")
}

fn info_sequence(sequence: &[u8]) -> String {
    let seq_strs: Vec<_> = sequence
        .iter()
        .enumerate()
        .map(|(pos, pat)| format!("{:>02x} {:>02x}", pos, pat))
        .collect();

    let row_strs: Vec<_> = (0..8_usize)
        .map(|k| {
            let row_vec: Vec<_> = seq_strs
                .iter()
                .enumerate()
                .filter(|(pos, _)| pos % 8 == k)
                .map(|(_, pat)| String::from(pat))
                .collect();
            row_vec.join("   ")
        })
        .filter(|row_str| !row_str.is_empty())
        .collect();
    row_strs.join("\n")
}

fn info_pattern(pattern: &Pattern, samples: &[Sample]) -> String {
    let mut ret = String::new();
    for (idx, division) in pattern.divisions.iter().enumerate() {
        ret.push_str(&format!("{:>02x}      ", idx));
        let channel_strs: Vec<_> = division
            .channel_data
            .iter()
            .map(|channel| info_channel(channel, samples))
            .collect();
        ret.push_str(&channel_strs.join("      "));
        ret.push_str("\n");
    }
    ret
}

fn info_channel(channel: &ChannelData, samples: &[Sample]) -> String {
    if channel.sample > 0 {
        let note_str = format!(
            "{}",
            note::get_note(
                samples[channel.sample as usize - 1].finetune,
                channel.period,
            )
        );
        format!(
            "{:>02x}|{}|{}",
            channel.sample,
            note_str,
            info_effect(&channel.effect)
        )
    } else {
        String::from("..........")
    }
}

fn info_effect(effect: &Effect) -> String {
    match effect {
        Effect::Normal {
            effect_type,
            param1,
            param2,
        } => {
            let t: u8 = (*effect_type).into();
            format!("{:1x}{:1x}{:1x}", t, param1, param2)
        }
        Effect::Extended { effect_type, param } => {
            let t: u8 = (*effect_type).into();
            format!("e{:1x}{:1x}", t, param)
        }
    }
}
