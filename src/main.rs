extern crate rand_pcg;
use clap::Parser;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Stdin, Write};

// const KEEP_NUM: f64 = 1000000.0;

#[derive(Debug)]
enum Size {
    Absolute(usize),
    Relative(f64),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// sizes to radnom sample, it should be a string
    #[arg(short, long)]
    size: String,

    /// output file names, it could be empty for stdout if `sizes` is a single number
    #[arg(short, long, required = false, default_value = "stdout")]
    output: String,

    /// comment line character
    #[arg(short, long, default_value = "")]
    comment: String,

    /// input files
    inputs: Vec<String>,
}

fn main() {
    // parse args
    let args = Args::parse();

    // parse size
    let size = args.size;
    println!("input size: {:?}", size);
    let parsed_size: Size = parse_size(&size);
    println!("parsed size: {:?}", parsed_size);

    // parse output name and check if it is stdout
    let output_name = args.output;
    println!("output name: {:?}", output_name);
    let mut output_stdout = true;
    if output_name != "stdout" {
        output_stdout = false;
    }

    let comment = if args.comment.is_empty() {
        None
    } else {
        Some(args.comment)
    };

    let (total_data_size, fix_lines, store_lines) = get_inputs_info(args.inputs, comment);
    let mut rng: Pcg64 = Pcg64::from_rng(thread_rng()).expect("failed to init rng");
    let true_size = get_usize(&parsed_size, total_data_size);
    if true_size > total_data_size {
        println!("size is larger than total data size, use total data size instead");
        std::process::exit(1);
    } else {
        println!("true size: {}", true_size);
    }
    println!("read lines: {}", store_lines.len());
    let mut all_array: Vec<usize> = (0..total_data_size).collect();
    let mut sampled_idx = all_array.iter().choose_multiple(&mut rng, true_size);
    println!("sampled idx: {:?}", sampled_idx);
    let mut output_file = File::create(output_name).expect("failed to create output file");
    // output skipped lines

    if output_stdout {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        for line in fix_lines {
            handle
                .write_all(line.as_bytes())
                .expect("failed to write to output file");
            handle.write_all(b"\n"); // add delimiter
        }
        for idx in sampled_idx {
            println!("sampled line: {}", store_lines[*idx]);
            handle
                .write_all(store_lines[*idx].as_bytes())
                .expect("failed to write to output file");
            handle.write_all(b"\n"); // add delimiter
        }
    } else {
        for line in fix_lines {
            output_file
                .write_all(line.as_bytes())
                .expect("failed to write to output file");
            output_file.write_all(b"\n"); // add delimiter
        }
        // output sampled lines with delimiter \n
        for idx in sampled_idx {
            println!("sampled line: {}", store_lines[*idx]);
            output_file
                .write_all(store_lines[*idx].as_bytes())
                .expect("failed to write to output file");
            output_file.write_all(b"\n"); // add delimiter
        }
    }
}
fn get_inputs_info(
    inputs: Vec<String>,
    comment: Option<String>,
) -> (usize, Vec<String>, Vec<String>) {
    let mut line_nums = 0;
    let mut skipped_lines = Vec::new(); // init skipped lines
    let mut stored_lines = Vec::new(); // init stored lines
    if inputs.is_empty() {
        // no input files specified, read from stdin instead
        println!("input from stdin");

        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin);
        // count lines, but nor read whole file in memory

        // skip comment lines and stroe it in a vector
        if let Some(comment) = comment {
            // has comment char specified
            for line in reader.lines() {
                let line = line.unwrap();
                if !line.starts_with(&*comment) {
                    line_nums += 1;
                    stored_lines.push(line);
                } else {
                    skipped_lines.push(line);
                }
            }
        } else {
            // no comment char specified
            for _line in reader.lines() {
                line_nums += 1;
                stored_lines.push(_line.unwrap());
            }
        }
        return (line_nums, skipped_lines, stored_lines);
    } else {
        println!("input from files: {:?}", inputs);
        // extract input files
        for filename in inputs.iter() {
            // check if file exists
            let file =
                File::open(filename).expect(format!("Unable to open file {}", filename).as_str());
            let reader = BufReader::new(file);
            // skip comment lines and stroe it in a vector
            if let Some(ref comment) = comment {
                // has comment char specified
                for line in reader.lines() {
                    let line = line.unwrap();
                    if !line.starts_with(&*comment) {
                        line_nums += 1;
                        stored_lines.push(line);
                    } else {
                        skipped_lines.push(line);
                    }
                }
            } else {
                // no comment char specified
                for _line in reader.lines() {
                    line_nums += 1;
                    stored_lines.push(_line.unwrap());
                }
            }
        }
        return (line_nums, skipped_lines, stored_lines);
    }
}
fn get_usize(size: &Size, data_size: usize) -> usize {
    match size {
        Size::Absolute(n) => *n,
        Size::Relative(f) => (data_size as f64 * *f).floor() as usize,
    }
}

fn parse_size(size: &String) -> Size {
    if size.contains('.') {
        let size: f64 = size.parse::<f64>().expect("size should be a number");
        Size::Relative(size)
    } else {
        let size: usize = size.parse::<usize>().expect("size should be a number");
        Size::Absolute(size)
    }
}
