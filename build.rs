use std::{
    env,
    fs::File,
    io::{self, BufRead, Write},
    str::FromStr,
};

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;

#[path = "src/mapping.rs"]
mod mapping;
use mapping::Mapping;

impl FromStr for Mapping<'_> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[expect(clippy::match_same_arms)]
        match s {
            "valid" => Ok(Self::Valid),
            "ignored" => Ok(Self::Ignored),
            "disallowed" => Ok(Self::Disallowed),
            // we don't support transitional processing
            "deviation" => Ok(Self::Valid),
            e => Err(format!(
                "no FromStr implementation for Mapping from input \'{e}\'"
            )),
        }
    }
}

impl ToTokens for Mapping<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Mapping::Mapped(s) = self {
            tokens.extend(quote!(Mapping::Mapped(#s)));
        } else {
            let variant = format!("Mapping::{self:?}");
            let token = variant.parse::<TokenStream>().unwrap();
            tokens.extend(quote!(#token));
        }
    }
}

#[derive(Debug)]
struct IdnaMap {
    start: char,
    end: char,
    status: String,
    map: String,
}

fn read_line(line: &str) -> Result<IdnaMap, (u32, u32)> {
    let (line, _) = line.rsplit_once('#').unwrap();
    let mut iter = line.split(';').map(str::trim);

    let (start, end);
    if let Some(x) = iter.next() {
        (start, end) = x.split_once("..").unwrap_or((x, x));
    } else {
        unreachable!();
    }

    let a = u32::from_str_radix(start, 16).unwrap();
    let b = u32::from_str_radix(end, 16).unwrap();

    let start = char::from_u32(a);
    let end = char::from_u32(b);

    let (start, end) = start.zip(end).ok_or((a, b))?;

    let status = iter.next().unwrap();

    let chars = iter.next().map_or(String::new(), |x| {
        x.split_whitespace()
            .map(|x| {
                char::from_u32(u32::from_str_radix(x, 16).unwrap())
                    .expect("invalid UTF-8 codepoint")
            })
            .collect::<String>()
    });

    Ok(IdnaMap {
        start,
        end,
        status: status.to_string(),
        map: chars,
    })
}

fn merge_ranges(ranges: Vec<IdnaMap>) -> Vec<IdnaMap> {
    let mut merged: Vec<IdnaMap> = Vec::new();

    for range in ranges {
        if let Some(last) = merged.last_mut() {
            if overlaps(&range, last) {
                last.end = range.end.max(last.end);
                continue;
            }
        }
        merged.push(range);
    }

    merged
}

fn overlaps(a: &IdnaMap, b: &IdnaMap) -> bool {
    let next = char::from_u32(b.end as u32 + 1);

    next.is_some_and(|next| {
        a.start <= next
            && a.status == b.status
            && if a.status == "mapped" {
                a.map == b.map
            } else {
                true
            }
    })
}

fn generate_data() -> impl Iterator<Item = IdnaMap> {
    let file = File::open("IdnaMappingTable.txt").expect("failed to open IdnaMappingTable.txt");
    let reader = io::BufReader::new(file);

    let idna_maps = reader
        .lines()
        .map(|x| x.unwrap().trim().to_string())
        .filter(|x| !x.is_empty() && !x.starts_with('#'))
        .filter_map(|x| {
            read_line(&x)
                .map_err(|(a, b)| {
                    println!("cargo:warning=skipping invalid char range: {a:X}..={b:X}");
                })
                .ok()
        })
        .filter(|x| !(x.start.is_ascii() && x.end.is_ascii()))
        .collect::<Vec<_>>();

    let merged = merge_ranges(idna_maps);

    merged.into_iter()
}

fn main() {
    println!("cargo:rustc-cfg=ugly_hack");
    let data = generate_data();
    let pairs = data.map(|x| {
        let status = match x.status.as_str() {
            "mapped" => Mapping::Mapped(x.map.as_str()),
            s => Mapping::from_str(s).expect("invalid Mapping"),
        };

        let (start, end) = (x.start, x.end);

        quote!((#start ..= #end, #status))
    });

    let tokens = parse_quote! {
        use core::ops::RangeInclusive;
        use crate::Mapping;

        #[allow(clippy::unicode_not_nfc)]
        pub const MAPPING: &[(RangeInclusive<char>, Mapping)] = &[#(#pairs),*];
    };

    let pretty = unparse(&tokens);

    let out_dir = env::var("OUT_DIR").expect("failed to get target directory");
    let out_file = out_dir + "/data.rs";
    let mut out = File::create(out_file).expect("failed to create src/idna_map.rs");

    out.write_all(pretty.as_bytes())
        .expect("failed to write pretty source");
}
