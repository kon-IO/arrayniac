use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::ExitCode,
};

use arrayniac::structure::parse_root;
use sonic_rs::Value;

fn confirm(message: &str) -> bool {
    loop {
        print!("{} [y/n]: ", message);
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let mut char_iter = line.chars().skip_while(|c| c.is_whitespace());
        let choice;
        match char_iter.next() {
            None => continue,
            Some(c) => {
                if c == 'y' || c == 'Y' {
                    choice = true;
                } else if c == 'n' || c == 'N' {
                    choice = false;
                } else {
                    continue;
                }
            }
        };
        let mut char_iter = char_iter.skip_while(|c| c.is_whitespace());
        if char_iter.next().is_none() {
            return choice;
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} [input_filename] [output_filename] [index_output_filename]", &args[0]);
        return ExitCode::FAILURE;
    }

    if Path::new(&args[2]).exists() {
        if !confirm(&format!(
            "Output file {} already exists. Do you want to override?",
            args[2]
        )) {
            return ExitCode::FAILURE;
        }
    }

    if Path::new(&args[3]).exists() {
        if !confirm(&format!(
            "Output file {} already exists. Do you want to override?",
            args[3]
        )) {
            return ExitCode::FAILURE;
        }
    }

    let Ok(mut f) = File::open(&args[1]) else {
        eprintln!("Could not open input file {}", args[1]);
        return ExitCode::FAILURE;
    };

    let mut buf = String::new();
    if f.read_to_string(&mut buf).is_err() {
        eprintln!("Failed to read from input file {}", args[1]);
        return ExitCode::FAILURE;
    };

    let root: Value = sonic_rs::from_str(&buf).unwrap();

    let parsed_json = parse_root(&root);
    dbg!(&parsed_json.get_variants());

    let (out_json, index) = parsed_json.to_string();

    let Ok(mut out_file) = File::create(&args[2]) else {
        eprintln!("Could not open output file {}", args[2]);
        return ExitCode::FAILURE;
    };

    if let Err(_) = out_file.write_all(out_json.as_bytes()) {
        eprintln!("Failed to write to file {}", args[2]);
    };

    let Ok(mut index_file) = File::create(&args[3]) else {
        eprintln!("Could not open output file {}", args[3]);
        return ExitCode::FAILURE;
    };

    if let Err(_) = index_file.write_all(index.as_bytes()) {
        eprintln!("Failed to write to file {}", args[3]);
    }

    ExitCode::SUCCESS
}
