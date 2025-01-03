use rustyline::DefaultEditor;

use edna::punycode::*;
use edna::*;

fn main() {
    let mut rl = DefaultEditor::new().expect("Failed to init editor");
    while let Ok(ref line) = rl.readline(">> ") {
        println!("to_ascii: {:?}", to_ascii(line));
        println!("encode: {:?}", encode(line));
        println!("decode: {:?}", decode(line));
        rl.add_history_entry(line).expect("Failed to save history");
    }
}
