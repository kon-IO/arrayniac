# arrayniac

> Minimize large JSON files by transforming objects into arrays

This is currently only a command-line tool.

## Contents

- [Usage](#usage)
- [Building](#building)

## Usage

```bash
arrayniac [input_file] [output_file] [output_index_file]
```

The tool accepts a JSON file to minimize, and provides two minified output files: the minimized JSON and an index for reconstructing the original JSON structure.

## Building

This project is written in [Rust](https://www.rust-lang.org/) and uses its package manager, Cargo. To build, [install the Rust tools on your system](https://www.rust-lang.org/tools/install), clone the repository, and run `cargo build` for a debug build, or `cargo build --release` for a release build *(The executable will be in target/debug or target/release respectively)*. You can also build and run the tool directly with `cargo run -- [input_file] [output_file] [output_index_file]` or `cargo run --release -- [input_file] [output_file] [output_index_file]`.
