use std::env;
use std::path::PathBuf;

fn print_item(path: &PathBuf, prefix: &str, depth: usize, lines_at: &Vec<usize>, last: bool) {
    // TODO(Zandr Martin/2018-01-28): colors? symlink resolution?
    match path.strip_prefix(prefix) {
        Ok(p) => match p.to_str() {
            Some(s) => {
                if s.starts_with(".") {
                    return;
                }
                for i in 0..depth {
                    if lines_at.contains(&i) {
                        print!("│   ");
                    } else {
                        print!("    ");
                    }
                }
                if last {
                    print!("└── ");
                } else {
                    print!("├── ");
                }
                println!("{}", s);
            }
            None => return,
        },
        Err(_) => return,
    };
}

fn process_dir(dir: PathBuf, prefix: &str, depth: usize, lines_at: &mut Vec<usize>) {
    let mut contents: Vec<_> = dir.read_dir().unwrap().filter_map(|e| e.ok()).collect();
    contents.sort_by_key(|k| k.path());

    if contents.is_empty() {
        return;
    }

    let end = contents.len() - 1;

    for (i, item) in contents.iter().enumerate() {
        let is_last = i == end;
        let path = item.path();
        print_item(&path, prefix, depth, &lines_at, is_last);
        if !is_last {
            lines_at.push(depth);
        }
        if path.is_dir() {
            let prfx = path.clone();
            if !prfx
                .strip_prefix(prefix)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(".")
            {
                let prfx = prfx.to_str().unwrap();
                process_dir(path, prfx, depth + 1, lines_at);
            }
        }
        if !is_last {
            lines_at.pop();
        }
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0); // remove the name of the binary
    let dir = if args.len() > 0 {
        args[0].as_str()
    } else {
        "."
    };

    println!("{}", dir);
    process_dir(PathBuf::from(dir), dir, 0, &mut vec![]);
}
