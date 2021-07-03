use std::io::{BufRead, Write};

const PROMPT: &'static str = "> ";

fn parse_line<'a>(line_buf: &'a str) -> Vec<&'a str> {
    //TODO: Parsing with escapes and quotes and other shell things
    line_buf.split_ascii_whitespace().collect()
}

fn main() {
    // Lock stdout because we are the only thread
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    // Lock stdin because we are the only thread
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut line_buf = String::with_capacity(256);
    loop {
        stdout.write_all(PROMPT.as_bytes()).unwrap();
        stdout.flush().unwrap();
        stdin.read_line(&mut line_buf).unwrap();

        // Parse the line
        let parsed_line = parse_line(&line_buf);

        // Try to find some keywords which the shell will interpret directly
        match parsed_line.as_slice() {
            // Leave the shell, ignore any other arguments
            ["exit", ..] => std::process::exit(0),
            _ => {
                println!("Unknown input: {}", line_buf);
            }
        }

        // Clear the line as `read_line` will just continue appending to our line buffer
        line_buf.clear();
    }
}
