use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

pub fn print_write(writer: &mut BufWriter<File>, text: &str) {
    println!("{text}");
    writeln!(writer, "{}", text).unwrap();
}