extern crate termsize;

use std::fs;
use std::cmp::{max, min};
use std::env;
use std::os::unix::fs::PermissionsExt;

// const COLOR_BLACK_BOLD: &str = "\x1b[30;1m";
// const COLOR_RED_BOLD: &str = "\x1b[31;1m";
const COLOR_GREEN_BOLD: &str = "\x1b[32;1m";
// const COLOR_YELLOW_BOLD: &str = "\x1b[33;1m";
const COLOR_BLUE_BOLD: &str = "\x1b[34;1m";
// const COLOR_MAGENTA_BOLD: &str = "\x1b[35;1m";
const COLOR_CYAN_BOLD: &str = "\x1b[36;1m";
const COLOR_RESET: &str = "\x1b[0m";
const GUTTER_WIDTH: usize = 3;

#[derive(Debug)]
struct FormattedEntry {
    raw_output: String,
    colored_output: String,
    is_hidden: bool,
}

struct Config {
    show_hidden: bool,
    one_per_line: bool,
    show_color: bool,
}

fn is_executable(mode: u32) -> bool {
    mode & 0o111 > 0
}

fn format_entry(entry: &fs::DirEntry, prefix: &str) -> Option<FormattedEntry> {
    let ft = match entry.file_type() {
        Ok(ft) => ft,
        Err(_) => return None,
    };
    let metadata = match entry.metadata() {
        Ok(md) => md,
        Err(_) => return None,
    };
    let color = match ft {
        f if f.is_dir() => COLOR_BLUE_BOLD,
        f if f.is_file() => {
            if is_executable(metadata.permissions().mode()) {
                COLOR_GREEN_BOLD
            } else {
                COLOR_RESET
            }
        }
        f if f.is_symlink() => COLOR_CYAN_BOLD,
        _ => COLOR_RESET,
    };
    let path = entry.path();
    let path = match path.strip_prefix(prefix) {
        Ok(p) => {
            match p.to_str() {
                Some(s) => s,
                None => return None,
            }
        }
        Err(_) => return None,
    };

    Some(FormattedEntry {
        is_hidden: path.starts_with("."),
        raw_output: String::from(path),
        colored_output: format!("{}{}{}", color, path, COLOR_RESET),
    })
}

fn parse_args() -> (Vec<String>, Config) {
    let mut env_args: Vec<String> = env::args().collect();
    env_args.remove(0); // remove the name of the binary
    let mut args = vec![];
    let mut config = Config {
        show_hidden: false,
        one_per_line: false,
        show_color: false,
    };

    for arg in env_args {
        match arg.as_str() {
            "-a" => config.show_hidden = true,
            "-1" => config.one_per_line = true,
            "-G" => config.show_color = true,
            _ => args.push(arg),
        }
    }

    (args, config)
}

fn string_pad(old_str: String, length: usize, fill: char) -> String {
    let mut s = String::from(old_str);

    while s.len() < length {
        s.push(fill);
    }

    s
}

fn print_entries(entries: Vec<FormattedEntry>, config: Config) {
    let term_width = match termsize::get() {
        Some(size) => size.cols,
        None => 80,
    };
    let longest_entry = entries.iter().fold(
        0,
        |acc, e| max(acc, e.raw_output.len()),
    );
    let col_width = longest_entry + GUTTER_WIDTH;
    let mut line_length = 0;

    for entry in entries {
        let raw_text_length = entry.raw_output.len();
        let text = if config.show_color {
            entry.colored_output
        } else {
            entry.raw_output
        };
        let text_length = text.len();

        if config.one_per_line {
            println!("{}", text);
        } else {
            print!(
                "{}",
                string_pad(
                    text,
                    min(
                        term_width as usize,
                        text_length + (col_width - raw_text_length),
                    ),
                    ' ',
                )
            );
            let col_width = col_width as u16;
            line_length += col_width;
            if term_width < line_length ||
                (term_width - line_length) < col_width
            {
                println!("");
                line_length = 0;
            }
        }
    }

    if !config.one_per_line && line_length != 0 {
        println!("");
    }
}

fn main() {
    // determine width of term
    // determine longest path string
    // add 3 for gutters
    // term width / longest path + gutter == # cols

    let (args, config) = parse_args();
    let dir = if args.len() > 0 {
        args[0].as_str()
    } else {
        "."
    };

    let mut contents: Vec<_> =
        fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).collect();
    contents.sort_by_key(|k| k.path());

    let formatted_entries: Vec<_> = contents
        .iter()
        .map(|e| format_entry(e, dir))
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .filter(|e| !e.is_hidden || config.show_hidden)
        .collect();
    print_entries(formatted_entries, config);
}
