use anyhow::Result;
use regex::Regex;

pub fn arange(end: i32) -> Vec<f64> {
    (0..end).map(|x| x as f64).collect()
}

pub fn linspace(start: f64, end: f64, num: usize, endpoint: Option<bool>) -> Vec<f64> {
    let endpoint = endpoint.unwrap_or(true);
    let mut result = Vec::with_capacity(num);
    for i in 0..num {
        let ratio = if endpoint {
            i as f64 / (num as f64 - 1.)
        } else {
            i as f64 / num as f64
        };
        result.push(start + (end - start) * ratio);
    }
    result
}

pub fn tempo_parser(arg: &str) -> Result<f64> {
    let tempo: f64 = arg[1..].parse()?;
    Ok(tempo)
}

pub fn pitch_parser(arg: &str) -> Result<i32> {
    let note_regex = Regex::new(r"([A-G]#?)(-?\d+)")?;
    let captures = note_regex.captures(arg).unwrap();

    let note = match captures.get(1).unwrap().as_str() {
        "C" => 0,
        "C#" => 1,
        "D" => 2,
        "D#" => 3,
        "E" => 4,
        "F" => 5,
        "F#" => 6,
        "G" => 7,
        "G#" => 8,
        "A" => 9,
        "A#" => 10,
        "B" => 11,
        _ => 0,
    };
    let octave = captures.get(2).unwrap().as_str().parse::<i32>()? + 1;
    Ok(octave * 12 + note)
}

#[cfg(test)]
mod tests {
    use crate::util::{pitch_parser, tempo_parser};

    #[test]
    fn test_tempo() {
        let tempo = tempo_parser("!120").unwrap();
        assert_eq!(tempo, 120.);
    }

    #[test]
    fn test_pitch() {
        let pitch = pitch_parser("C4").unwrap();
        assert_eq!(pitch, 60);
        let pitch = pitch_parser("C5").unwrap();
        assert_eq!(pitch, 72);
        let pitch = pitch_parser("A4").unwrap();
        assert_eq!(pitch, 69);
    }
}
