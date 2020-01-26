use std::fmt;

pub struct Note {
    octave: u8, // 0..2
    tone: u8,   // 0..11
    exact: bool,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", TONE_NAMES[self.tone as usize], self.octave + 1)
    }
}

pub fn get_note(finetune: i8, period: u16) -> Note {
    let ft_idx = (8 + finetune) as usize;
    match NOTES[ft_idx].binary_search_by(|probe| probe.cmp(&period).reverse()) {
        Ok(idx) => {
            let octave = idx as u8 / 12;
            let tone = idx as u8 % 12;
            Note {
                octave,
                tone,
                exact: true,
            }
        }
        Err(idx) => {
            let idx = if idx > 0 { idx - 1 } else { 0 };
            let octave = idx as u8 / 12;
            let tone = idx as u8 % 12;
            Note {
                octave,
                tone,
                exact: false,
            }
        }
    }
}

static TONE_NAMES: [&str; 12] = [
    "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
];

static NOTES: [[u16; 36]; 16] = [
    [
        // Finetune -8
        907, 856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, // Octave 1
        453, 428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, // Octave 2
        226, 214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, // Octave 3
    ],
    [
        // Finetune -7
        900, 850, 802, 757, 715, 675, 636, 601, 567, 535, 505, 477, // Octave 1
        450, 425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 238, // Octave 2
        225, 212, 200, 189, 179, 169, 159, 150, 142, 134, 126, 119, // Octave 3
    ],
    [
        // Finetune -6
        894, 844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474, // Octave 1
        447, 422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237, // Octave 2
        223, 211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118, // Octave 3
    ],
    [
        // Finetune -5
        887, 838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470, // Octave 1
        444, 419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235, // Octave 2
        222, 209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118, // Octave 3
    ],
    [
        // Finetune -4
        881, 832, 785, 741, 699, 660, 623, 588, 555, 524, 494, 467, // Octave 1
        441, 416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233, // Octave 2
        220, 208, 196, 185, 175, 165, 156, 147, 139, 131, 123, 117, // Octave 3
    ],
    [
        // Finetune -3
        875, 826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463, // Octave 1
        437, 413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232, // Octave 2
        219, 206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116, // Octave 3
    ],
    [
        // Finetune -2
        868, 820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460, // Octave 1
        434, 410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230, // Octave 2
        217, 205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115, // Octave 3
    ],
    [
        // Finetune -1
        862, 814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457, // Octave 1
        431, 407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228, // Octave 2
        216, 203, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114, // Octave 3
    ],
    [
        // Finetune 0
        856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453, // Octave 1
        428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226, // Octave 2
        214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113, // Octave 3
    ],
    [
        // Finetune 1
        850, 802, 757, 715, 674, 637, 601, 567, 535, 505, 477, 450, // Octave 1
        425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 239, 225, // Octave 2
        213, 201, 189, 179, 169, 159, 150, 142, 134, 126, 119, 113, // Octave 3
    ],
    [
        // Finetune 2
        844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474, 447, // Octave 1
        422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237, 224, // Octave 2
        211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118, 112, // Octave 3
    ],
    [
        // Finetune 3
        838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470, 444, // Octave 1
        419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235, 222, // Octave 2
        209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118, 111, // Octave 3
    ],
    [
        // Finetune 4
        832, 785, 741, 699, 660, 623, 588, 555, 524, 495, 467, 441, // Octave 1
        416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233, 220, // Octave 2
        208, 196, 185, 175, 165, 156, 147, 139, 131, 124, 117, 110, // Octave 3
    ],
    [
        // Finetune 5
        826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463, 437, // Octave 1
        413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232, 219, // Octave 2
        206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116, 109, // Octave 3
    ],
    [
        // Finetune 6
        820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460, 434, // Octave 1
        410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230, 217, // Octave 2
        205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115, 109, // Octave 3
    ],
    [
        // Finetune 7
        814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457, 431, // Octave 1
        407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228, 216, // Octave 2
        204, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114, 108, // Octave 3
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_note() {
        assert_eq!("C-1".to_owned(), format!("{}", get_note(0, 856).unwrap()));
        assert_eq!("G-2".to_owned(), format!("{}", get_note(3, 280).unwrap()));
        assert_eq!("D#3".to_owned(), format!("{}", get_note(-7, 189).unwrap()));
    }
}
