use std::{
    fmt,
    fs::File,
    io::{self, BufRead},
    str::FromStr,
};

use rstest::*;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Uts46 {
    input: String,
    to_unicode: String,
    to_ascii: String,
    line: usize,
}

fn parse_vec<T: FromStr>(s: &str) -> Vec<T>
where
    <T as FromStr>::Err: fmt::Debug,
{
    s.strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or(s)
        .split(',')
        .map(|x| x.trim().parse().expect("parse error"))
        .collect()
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
enum ReadLineError {
    #[error("Bidi: {0}")]
    Bidi(String),
    #[error("Context: {0}")]
    Context(String),
    #[error("invalid char: {0:X}")]
    InvalidChar(u32),
    #[error("not yet implemented: {0}")]
    NYI(String),
}

impl From<u32> for ReadLineError {
    fn from(n: u32) -> Self {
        Self::InvalidChar(n)
    }
}

fn unescape(s: &mut String) -> Result<(), u32> {
    while let Some((_, b)) = s.split_once("\\u") {
        let n = u32::from_str_radix(&b[0..4], 16).unwrap();
        let c = char::from_u32(n).ok_or(n)?;
        let i = s.find("\\u").unwrap();
        s.replace_range(i..i + 4, &c.to_string());
    }

    Ok(())
}

fn read_line(n: usize, line: &str) -> Result<Uts46, ReadLineError> {
    let (line, _) = line.rsplit_once('#').unwrap();
    let mut iter = line.split(';').map(str::trim);

    let mut input = iter
        .next()
        .map(|x| {
            if x == "\"\"" {
                return "";
            }
            x
        })
        .unwrap()
        .to_string();
    unescape(&mut input)?;

    let mut to_unicode = iter.next().unwrap().to_string();
    unescape(&mut to_unicode)?;
    if to_unicode == "" {
        to_unicode = input.clone();
    }

    let to_unicode_status = iter.next().unwrap();
    let stati: Vec<String> = parse_vec(to_unicode_status);
    // skip Bidi and Context tests
    for x in stati {
        match x.chars().next() {
            // ToASCII step n
            Some('A') => (),
            Some('B') => return Err(ReadLineError::Bidi(input)),
            Some('C') => return Err(ReadLineError::Context(input)),
            // Processing step n
            Some('P') => (),
            Some('U') => return Err(ReadLineError::NYI("Use STD3ASCIIRules".into())),
            //Some('V') => return Err(ReadLineError::NYI("Validity".into())),
            Some('V') => (),
            // toUnicode issues
            Some('X') => (),
            None => (),
            Some(x) => {
                unimplemented!("status {x} is not implemented");
            }
        };
    }

    let mut to_ascii = iter.next().unwrap().to_string();
    unescape(&mut to_ascii)?;
    if to_ascii == "" {
        to_ascii = input.clone();
    }

    Ok(Uts46 {
        input,
        to_unicode,
        to_ascii,
        line: n,
    })
}

#[fixture]
#[once]
fn uts46() -> Vec<Uts46> {
    let file = File::open("IdnaTestV2.txt").expect("failed to open IdnaTestV2.txt");
    let reader = io::BufReader::new(file);
    reader
        .lines()
        .enumerate()
        .map(|(n, x)| (n, x.unwrap().trim().to_string()))
        .filter(|(_, x)| !x.is_empty() && !x.starts_with('#'))
        .filter_map(|(n, x)| {
            read_line(n + 1, &x)
                .map_err(|e| eprintln!("skipping line {n}: {e}"))
                .ok()
        })
        .collect()
}

#[rstest]
#[ignore]
fn test_to_ascii(uts46: &Vec<Uts46>) {
    for x in uts46.iter() {
        let input = &x.input;
        let expected = &x.to_ascii;
        let got = edna::to_ascii(input).unwrap();
        let n = &x.line;
        assert_eq!(
            &got,
            expected,
            "failed line {n} ({})",
            input
                .chars()
                .map(|x| format!("\\u{:0>4X}", x as u32))
                .collect::<String>()
        );
    }
}
