use std::io::{self, Write};
use std::iter;
use std::time::Duration;

pub struct Printer {
    line_description: String,
    last_print_length: usize,
}

impl Printer {
    pub fn new() -> Self {
        Printer {
            line_description: String::new(),
            last_print_length: 0,
        }
    }

    pub fn print_title(&mut self, title: &str) {
        self.println(&format!("== {} ==", title)).unwrap();
    }

    pub fn print_line_description(&mut self, desc: &str) {
        self.line_description = desc.to_owned();
    }

    pub fn print_statistics(&mut self, pkgs: usize, bytes: usize, duration: Duration) {
        let text = self.build_statistics_string(pkgs, bytes, duration);
        self.print(&text).unwrap();
    }

    pub fn end_line(&mut self) {
        println!("");
        self.last_print_length = 0;
    }

    fn build_statistics_string(&self,
                               mut pkgs: usize,
                               mut bytes: usize,
                               duration: Duration)
                               -> String {
        let passed_secs = duration.as_secs() as f64 +
                          (duration.subsec_nanos() as f64) / 1_000_000_000f64;
        pkgs = (pkgs as f64 / passed_secs) as usize;
        bytes = (bytes as f64 / passed_secs) as usize;
        let (scaled_bytes, bytes_suffix) = bytes_to_human(bytes);
        format!("{}: {} {}B/s - {} pps",
                self.line_description,
                scaled_bytes,
                bytes_suffix,
                pkgs)
    }

    fn println(&mut self, line: &str) -> io::Result<()> {
        self.print(line)?;
        self.end_line();
        Ok(())
    }

    fn print(&mut self, line: &str) -> io::Result<()> {
        let mut stream = io::stdout();
        stream.write_all(line.as_bytes())?;

        let length = line.chars().count();
        if self.last_print_length > length {
            let space_length = self.last_print_length - length;
            let space_string = iter::repeat(' ').take(space_length).collect::<String>();
            stream.write_all(space_string.as_bytes())?;
        }
        self.last_print_length = length;

        write!(stream, "\r")?;
        stream.flush().unwrap();
        Ok(())
    }
}

fn bytes_to_human(mut bytes: usize) -> (usize, &'static str) {
    static SIZE_SUFFIXES: [&'static str; 6] = ["", "ki", "Mi", "Gi", "Ti", "Pi"];
    for i in 0..SIZE_SUFFIXES.len() {
        if bytes >= 1024 {
            bytes /= 1024;
        } else {
            return (bytes, SIZE_SUFFIXES[i]);
        }
    }
    return (bytes, SIZE_SUFFIXES[SIZE_SUFFIXES.len() - 1]);
}
