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
            &[] => {} // Empty line nothing to do
            _ => {
                // Line isn't empty and isn't a keyword, try to resolve the items
                // Its safe to get this item (eg no panic) because we didn't match the empty
                // slice pattern, so there is at least one item.
                let path_or_name = parsed_line[0];
                let args = &parsed_line[1..];

                let mut child_process = std::process::Command::new(path_or_name)
                    .args(args.iter())
                    .spawn()
                    .unwrap();
                child_process.wait().unwrap();
            }
        }

        // Clear the line as `read_line` will just continue appending to our line buffer
        line_buf.clear();
    }
}
