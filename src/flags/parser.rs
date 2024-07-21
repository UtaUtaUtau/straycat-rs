use anyhow::Result;
use std::{mem::discriminant, str::FromStr};

use crate::consts;

#[derive(Debug)]
pub struct Flags {
    pub generate_features: Option<f64>,
    pub fry_enable: f64,
    pub fry_offset: f64,
    pub fry_transition: f64,
    pub fry_volume: f64,
    pub fry_pitch: f64,
    pub devoice_enable: f64,
    pub devoice_offset: f64,
    pub devoice_transition: f64,
    pub gender: f64,
    pub breathiness: f64,
    pub peak_compression: f64,
    pub peak_normalization: f64,
    pub tremolo: f64,
    pub pitch_offset: f64,
    pub aperiodic_mix: f64,
    pub growl: f64,
}

enum FlagToken {
    GenerateFeatures,
    FryEnable,
    FryOffset,
    FryTransition,
    FryVolume,
    FryPitch,
    DevoiceEnable,
    DevoiceOffset,
    DevoiceTransition,
    Gender,
    Breathiness,
    PeakCompression,
    PeakNormalization,
    Tremolo,
    PitchOffset,
    AperiodicMix,
    Growl,
    Number(f64),
}

impl Flags {
    pub fn new() -> Self {
        Self {
            generate_features: None,
            fry_enable: 0.,
            fry_offset: 0.,
            fry_transition: 75.,
            fry_volume: 10.,
            fry_pitch: consts::F0_FLOOR,
            devoice_enable: 0.,
            devoice_offset: 0.,
            devoice_transition: 75.,
            gender: 0.,
            breathiness: 50.,
            peak_compression: 86.,
            peak_normalization: 6.,
            tremolo: 0.,
            pitch_offset: 0.,
            aperiodic_mix: 0.,
            growl: 0.,
        }
    }
}

impl FromStr for Flags {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut flags = Self::new();
        let mut flag_tokens = Vec::new();
        let length = s.len();

        // tokenize flags
        let mut i = 0;
        while i < length {
            // current character... as a &str ik
            let mut curr = &s[i..i + 1];
            // two characters
            let two_chars = if i < s.len() - 1 {
                Some(&s[i..i + 2])
            } else {
                None
            };

            // check single character flags
            match curr {
                "g" => flag_tokens.push(FlagToken::Gender),
                "B" => flag_tokens.push(FlagToken::Breathiness),
                "P" => flag_tokens.push(FlagToken::PeakCompression),
                "p" => flag_tokens.push(FlagToken::PeakNormalization),
                "A" => flag_tokens.push(FlagToken::Tremolo),
                "t" => flag_tokens.push(FlagToken::PitchOffset),
                "S" => flag_tokens.push(FlagToken::AperiodicMix),
                "G" => flag_tokens.push(FlagToken::GenerateFeatures),
                _ => (),
            }

            // check two character flags
            if let Some(flag) = two_chars {
                let mut matched = true;
                match flag {
                    "fe" => flag_tokens.push(FlagToken::FryEnable),
                    "fo" => flag_tokens.push(FlagToken::FryOffset),
                    "fl" => flag_tokens.push(FlagToken::FryTransition),
                    "fv" => flag_tokens.push(FlagToken::FryVolume),
                    "fp" => flag_tokens.push(FlagToken::FryPitch),
                    "ve" => flag_tokens.push(FlagToken::DevoiceEnable),
                    "vo" => flag_tokens.push(FlagToken::DevoiceOffset),
                    "vl" => flag_tokens.push(FlagToken::DevoiceTransition),
                    "gw" => flag_tokens.push(FlagToken::Growl),
                    _ => matched = false,
                }
                if matched {
                    i += 1; // increment index if matched
                }
            }

            // parse numbers
            if curr.parse::<i32>().is_ok() || curr == "+" || curr == "-" {
                let st = i;
                while i + 1 < length && s[i + 1..i + 2].parse::<i32>().is_ok() {
                    i += 1;
                    curr = &s[st..i + 1];
                }
                flag_tokens.push(FlagToken::Number(curr.parse()?));
            }

            // increment
            i += 1;
        }

        i = 0; // recycle index
        let length = flag_tokens.len();
        // parse flags
        while i < length {
            let curr = &flag_tokens[i];
            // if it's not a numerical token
            if discriminant(curr) != discriminant(&FlagToken::Number(0.)) {
                if i < length - 1 {
                    // get next token
                    let next = &flag_tokens[i + 1];
                    if let FlagToken::Number(value) = next {
                        match curr {
                            FlagToken::GenerateFeatures => {
                                flags.generate_features = Some(value.clamp(0., 100.))
                            }
                            FlagToken::FryEnable => flags.fry_enable = *value,
                            FlagToken::FryOffset => flags.fry_offset = *value,
                            FlagToken::FryTransition => flags.fry_transition = value.max(1.),
                            FlagToken::FryVolume => flags.fry_volume = value.clamp(0., 100.),
                            FlagToken::FryPitch => flags.fry_pitch = value.max(0.),
                            FlagToken::DevoiceEnable => flags.devoice_enable = *value,
                            FlagToken::DevoiceOffset => flags.devoice_offset = *value,
                            FlagToken::DevoiceTransition => {
                                flags.devoice_transition = value.max(1.)
                            }
                            FlagToken::Gender => flags.gender = *value,
                            FlagToken::Breathiness => flags.breathiness = value.clamp(0., 100.),
                            FlagToken::PeakCompression => {
                                flags.peak_compression = value.clamp(0., 100.)
                            }
                            FlagToken::PeakNormalization => flags.peak_normalization = *value,
                            FlagToken::Tremolo => flags.tremolo = *value,
                            FlagToken::PitchOffset => flags.pitch_offset = *value,
                            FlagToken::AperiodicMix => flags.aperiodic_mix = value.clamp(0., 100.),
                            FlagToken::Growl => flags.growl = value.max(0.),
                            _ => (),
                        }
                        i += 1;
                    } else {
                        match curr {
                            FlagToken::GenerateFeatures => {
                                flags.generate_features = Some(consts::D4C_THRESHOLD * 100.)
                            }
                            _ => (),
                        }
                    }
                } else {
                    // Last token, only check possible option flag
                    match curr {
                        FlagToken::GenerateFeatures => {
                            flags.generate_features = Some(consts::D4C_THRESHOLD * 100.)
                        }
                        _ => (),
                    }
                }
            }
            // increment
            i += 1;
        }

        Ok(flags)
    }
}

#[cfg(test)]
mod tests {
    use super::Flags;

    #[test]
    fn test_flags() {
        let flag = "f/e1000t100";
        let flags: Flags = flag.replace("/", "").parse().expect("Cannot parse flags");
        println!("{:#?}", flags);
    }
}
