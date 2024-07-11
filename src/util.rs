use anyhow::Result;
use regex::Regex;

// Rust versions of some numpy operations
pub fn arange(end: i32) -> Vec<f64> {
    (0..end).map(|x| x as f64).collect()
}

pub fn linspace(start: f64, end: f64, num: usize, endpoint: bool) -> Vec<f64> {
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

pub fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

pub fn midi_to_hz(x: f64) -> f64 {
    // Convert MIDI numbers to Hertz
    440. * ((x - 69.) / 12.).exp2()
}

pub fn tempo_parser(arg: &str) -> Result<f64> {
    // Parse tempo argument
    let tempo: f64 = arg[1..].parse()?;
    Ok(tempo)
}

pub fn pitch_parser(arg: &str) -> Result<i32> {
    // Parse pitch argument
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
