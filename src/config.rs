// The crtcfg.bin word for Jotego cores, 4 bytes big-endian. Ported from pupdate's
// JotegoAnalogizerSettingsService: each wizard letter sets a fixed combination of flag bits.
//
//   bit 31..20  CRT flags   a b c k l u d i j h g f
//   bit 19..17  (unused)    u u u
//   bit 16..12  SNAC flags  a f e c b
//   bit 11..0   zero

use serde::{Deserialize, Serialize};

const A: u32 = 1 << 31;
const B: u32 = 1 << 30;
const C: u32 = 1 << 29;
const K: u32 = 1 << 28;
const L: u32 = 1 << 27;
const D: u32 = 1 << 25;
const I: u32 = 1 << 24;
const J: u32 = 1 << 23;
const H: u32 = 1 << 22;
const G: u32 = 1 << 21;
const F: u32 = 1 << 20;

const SA: u32 = 1 << 16;
const SF: u32 = 1 << 15;
const SE: u32 = 1 << 14;
const SC: u32 = 1 << 13;
const SB: u32 = 1 << 12;

pub const CRT_MASK: u32 = 0xFFF0_0000;
pub const SNAC_MASK: u32 = 0x0001_F000;

// (letter the wizard used, label, flag bits)
pub const CRT: &[(char, &str, u32)] = &[
    ('a', "RGBS (SCART)", A | D),
    ('b', "RGsB", A | B | J),
    ('c', "YPbPr (Component video)", A | C | J),
    ('d', "Y/C NTSC (S-Video, Composite video)", A | K),
    ('e', "Y/C PAL (S-Video, Composite video)", A | K | L),
    ('f', "Scandoubler RGBHV (scanlines 0%)", A | F),
    ('g', "Scandoubler RGBHV (scanlines 25%)", A | F | G),
    ('h', "Scandoubler RGBHV (scanlines 50%)", A | F | H),
    ('i', "Scandoubler RGBHV (scanlines 75%)", A | F | G | H),
    ('x', "Disable analog video", 0),
];

pub const SNAC: &[(char, &str, u32)] = &[
    ('a', "None", 0),
    ('b', "DB15 Normal", SB),
    ('c', "NES", SC),
    ('d', "SNES", SC | SB),
    ('e', "PCE 2BTN/6BTN", SE | SB),
    ('f', "PCE Multitap", SE | SC),
    ('g', "PSX Digital Normal", SA),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub crt: char,
    pub snac: char,
}

fn flags(table: &[(char, &str, u32)], letter: char) -> Option<u32> {
    table.iter().find(|(l, _, _)| *l == letter).map(|(_, _, f)| *f)
}

pub fn pack(c: &Config) -> Option<u32> {
    Some(flags(CRT, c.crt)? | flags(SNAC, c.snac)?)
}

pub fn to_bytes(c: &Config) -> Option<[u8; 4]> {
    pack(c).map(u32::to_be_bytes)
}

// Only words the wizard can produce decode; anything else (hand-edited file) is None.
pub fn from_bytes(bytes: &[u8]) -> Option<Config> {
    let word = u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?);
    let crt = CRT.iter().find(|(_, _, f)| *f == word & CRT_MASK)?.0;
    let snac = SNAC.iter().find(|(_, _, f)| *f == word & SNAC_MASK)?.0;
    (word & !(CRT_MASK | SNAC_MASK) == 0).then_some(Config { crt, snac })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Golden {
        vectors: Vec<Vector>,
    }

    #[derive(Deserialize)]
    struct Vector {
        crt: String,
        snac: String,
        bytes: String,
    }

    fn golden() -> Golden {
        serde_json::from_str(include_str!("../tests/golden.json")).expect("tests/golden.json")
    }

    #[test]
    fn every_letter_pair_matches_pupdate() {
        let g = golden();
        assert_eq!(g.vectors.len(), CRT.len() * SNAC.len());
        for v in g.vectors {
            let c = Config { crt: v.crt.chars().next().unwrap(), snac: v.snac.chars().next().unwrap() };
            let hex: Vec<String> = to_bytes(&c).unwrap().iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(hex.join(" "), v.bytes, "{c:?}");
            assert_eq!(from_bytes(&to_bytes(&c).unwrap()), Some(c), "decode must invert encode for {c:?}");
        }
    }

    #[test]
    fn unknown_letters_and_words_are_rejected() {
        assert_eq!(pack(&Config { crt: 'z', snac: 'a' }), None);
        assert_eq!(from_bytes(&[0xff, 0xff, 0xff, 0xff]), None);
        assert_eq!(from_bytes(&[0, 0, 0]), None);
    }
}
