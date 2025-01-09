use rustyline::DefaultEditor;

use edna::*;

fn main() {
    let mut rl = DefaultEditor::new().expect("Failed to init editor");
    while let Ok(ref line) = rl.readline(">> ") {
        let mut chars = line.chars();
        if let Some(c) = chars.next() {
            if chars.next().is_none() {
                println!("Mapping:of({c}) == {:?}", Mapping::of(c));
            }
        }
        println!("to_ascii({line}) == {:?}", to_ascii(line));
        println!("punycode::encode({line}) == {:?}", punycode::encode(line));
        println!("punycode::decode({line}) == {:?}", punycode::decode(line));
        println!("idna::punycode::encode({line}) == {:?}", idna::punycode::encode_str(line));
        rl.add_history_entry(line).expect("Failed to save history");
    }
}
