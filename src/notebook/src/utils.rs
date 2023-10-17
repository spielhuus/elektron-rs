use rand::{
    distributions::{Alphanumeric, DistString},
    prelude::Distribution,
    Rng,
};

pub fn newlines(input: String) -> String {
    input
        .lines()
        .into_iter()
        .collect::<Vec<&str>>()
        .join("<br/>")
}

pub struct Symbols;
impl Distribution<char> for Symbols {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        const RANGE: u32 = 26;
        const GEN_ASCII_STR_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        loop {
            let var = rng.next_u32() >> (32 - 6);
            if var < RANGE {
                return GEN_ASCII_STR_CHARSET[var as usize] as char;
            }
        }
    }
}

pub fn clean_svg(input: &str) -> String {
    let mut vec: Vec<String> = Vec::new();
    let rand_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    for line in input.lines() {
        if !line.starts_with("<?xml version=") {
            let res = line.replace("id=\"", format!("id=\"{}", rand_string).as_str());
            let res = res.replace(
                "xlink:href=\"#",
                format!("xlink:href=\"#{}", rand_string).as_str(),
            );
            vec.push(res);
        }
    }
    vec.join("\n")
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use super::Symbols;

    #[test]
    fn test_standard() {
        let rand_string1: String = thread_rng()
            .sample_iter(&Symbols)
            .take(30)
            .map(char::from)
            .collect();

        let rand_string2: String = thread_rng()
            .sample_iter(&Symbols)
            .take(30)
            .map(char::from)
            .collect();

        assert!(rand_string1.chars().any(|c| matches!(c, 'a'..='z')));
        assert!(rand_string2.chars().any(|c| matches!(c, 'a'..='z')));
        assert_ne!(rand_string1, rand_string2);
    }
}
