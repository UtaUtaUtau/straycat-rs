use anyhow::{anyhow, Result};

fn to_uint6(b64: u8) -> Result<u8> {
    // u8 character to Base64 value equivalent
    let c = b64;
    if c >= 97 {
        Ok(c - 71)
    } else if c >= 65 {
        Ok(c - 65)
    } else if c >= 48 {
        Ok(c + 4)
    } else if c == 43 {
        Ok(62)
    } else if c == 47 {
        Ok(63)
    } else {
        Err(anyhow!("Out of range"))
    }
}

fn to_int12(a: u8, b: u8) -> Result<i16> {
    // two Base64 chars to a weird signed 12-bit integer
    let uint12 = (to_uint6(a)? as u16) << 6 | (to_uint6(b)? as u16);
    if uint12 >> 11 & 1 == 1 {
        Ok(uint12 as i16 - 4096)
    } else {
        Ok(uint12 as i16)
    }
}

fn to_int12_stream<S: AsRef<str>>(b64: S) -> Result<Vec<i16>> {
    // String of Base64 pairs to a Vec of the weird signed 12-bit integer
    let b64 = b64.as_ref();
    if b64.len() % 2 != 0 {
        return Err(anyhow!("Cannot parse stream."));
    }

    let stream_len = b64.len() / 2;
    let mut stream = Vec::with_capacity(stream_len);
    let b64_bytes = b64.as_bytes();

    for i in 0..stream_len {
        stream.push(to_int12(b64_bytes[i * 2], b64_bytes[i * 2 + 1])?);
    }

    Ok(stream)
}

pub fn pitch_string_to_midi<S: AsRef<str>>(pitch_string: S) -> Result<Vec<f64>> {
    // UTAU pitchbend argument to MIDI
    let pitch_string = pitch_string.as_ref();
    let mut pitchbend: Vec<i16> = Vec::new();

    let pitch_rle: Vec<&str> = pitch_string.split("#").collect();
    for i in 0..pitch_rle.len() / 2 {
        let pair = &pitch_rle[i * 2..i * 2 + 2];
        let mut stream = to_int12_stream(pair[0])?;
        let last_point = stream[stream.len() - 1];
        let rle: usize = pair[1].parse()?;
        pitchbend.append(&mut stream);
        for _ in 1..rle {
            pitchbend.push(last_point);
        }
    }

    if pitch_rle.len() % 2 == 1 {
        let mut stream = to_int12_stream(pitch_rle[pitch_rle.len() - 1])?;
        pitchbend.append(&mut stream);
    }

    let ref_pitch = pitchbend[0];
    let flat_pitch = pitchbend.iter().all(|x| *x == ref_pitch);

    let pitchbend: Vec<f64> = pitchbend
        .into_iter()
        .map(|x| if flat_pitch { 0. } else { x as f64 / 100. })
        .collect();
    Ok(pitchbend)
}

#[cfg(test)]
mod tests {
    use super::pitch_string_to_midi;

    #[test]
    fn test_pitch_string() {
        let test = "B7CPCVCVCTCQCNCICDB+B5B0BvBrBnBlBk#14#BjBF/++Y8k615d4p4f4l4y5G5f596e7B7l8H8n9D9Z9q9092919y9t9n9f9Y9Q9I9C898584858/9L9b9v+G+f+4/Q/m/5AIATAY#2#AWAUARAOALAHAFACABAA";
        let pitchbend = pitch_string_to_midi(&test).expect("Failed to parse");
        for p in &pitchbend {
            println!("{}", p);
        }
    }
}
