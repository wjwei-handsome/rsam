extern crate bytecount;
extern crate rand_pcg;
use clap::Parser;
use log::{error, info, warn};
use log4rs;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

// TODO: .gz files support
// TODO: add unit test
// TODO: update readme and usage
#[derive(Debug)]
enum Size {
    Absolute(usize),
    Relative(f64),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// size to radnom sample, it could be a interger (absolute) or a float (relative)
    #[arg(short, long)]
    size: String,

    /// output file name, empty for stdout
    #[arg(short, long, required = false, default_value = "stdout")]
    output: String,

    /// comment line character for fix lines (note: if multiple input files, only the last file will be used for fix lines)
    #[arg(short, long, default_value = "")]
    comment: String,

    /// quiet mode
    #[arg(short, long, default_value = "false")]
    quiet: bool,

    /// rewrite output file
    #[arg(short, long, default_value = "false")]
    rewrite: bool,

    /// input files
    inputs: Vec<String>,
}

fn main() {
    // init logger
    log4rs::init_file("log_cfg.yml", Default::default()).unwrap();

    // parse args
    let args = Args::parse();

    // parse if quiet mode
    let quiet = args.quiet;

    // parse size
    let size = args.size;
    if !quiet {
        info!("input size: {:?}", size);
    }
    let parsed_size: Size = parse_size(&size);
    // if relative size > 1, error and exit
    match parsed_size {
        Size::Relative(x) => {
            if x > 1.0 {
                error!("relative size should be less than 1.0");
                std::process::exit(1);
            }
        }
        _ => {}
    }
    if !quiet {
        info!("parsed size: {:?}", parsed_size);
    }

    // parse input files and check if it is stdin or exists
    let input_files = args.inputs;
    let mut stdin_mode = false;
    if input_files.is_empty() {
        stdin_mode = true;
    }
    if !quiet {
        if stdin_mode {
            info!("input from stdin");
        } else {
            info!("input files: {:?}", input_files);
        }
    }
    if !stdin_mode {
        // check if input files exist if not stdin mode
        input_files_exist(&input_files);
    }

    // parse output name and check if it is stdout or exists
    let output_name = args.output;
    let rewrite = args.rewrite;
    if !quiet {
        info!("output to: {:?}", output_name);
    }
    let mut output_stdout = true;
    if output_name != "stdout" {
        output_stdout = false;
        // check if output file exists
        outfile_exist(&output_name, rewrite);
    }

    // parse comment char
    let comment = if args.comment.is_empty() {
        None
    } else {
        Some(args.comment)
    };
    if !quiet {
        info!("comment char: {:?}", comment);
    }

    // start read inputs and shuffle
    let (line_count, fix_lines, stored_lines) = if stdin_mode {
        let handle = std::io::stdin();
        count_lines(handle, &comment, stdin_mode).unwrap()
    } else {
        let mut all_count = 0;
        let mut fix_lines: Vec<String> = Vec::new();
        for file in input_files.iter() {
            let handle = File::open(file).unwrap();
            let (tmp_count, tmp_fix_lines, _) = count_lines(handle, &comment, stdin_mode).unwrap();
            all_count += tmp_count;
            fix_lines = tmp_fix_lines;
            // there are no stored_lines if not stdin
        }
        (all_count, fix_lines, Vec::new())
    };
    if !quiet {
        info!("total line count: {}", line_count);
    }
    if line_count == 0 {
        error!("no data to sample");
        std::process::exit(0);
    }

    // get true size
    let true_size = get_true_size(&parsed_size, line_count);
    if !quiet {
        info!("true size: {}", true_size);
    }
    if true_size > line_count {
        error!("true size is larger than total data size! R U kidding?");
        std::process::exit(1);
    }

    // shuffle
    // info!("generate rng...");
    let mut rng: Pcg64 = Pcg64::from_rng(thread_rng()).expect("failed to init rng");
    // info!("generate all array...");
    let all_array: Vec<usize> = (0..line_count).collect();
    // info!("sampling...");
    let mut sampled_idx = all_array.iter().choose_multiple(&mut rng, true_size);
    // println!("sampled_idx: {:?}", sampled_idx);
    // info!("sorting...");
    sampled_idx.sort();
    // info!("sort done");
    // println!("sorted sampled_idx: {:?}", sampled_idx);

    // output samples
    // info!("Start output");
    output_samples(
        &output_name,
        output_stdout,
        &fix_lines,
        &stored_lines,
        &sampled_idx,
        &input_files,
        &comment,
    );
    // info!("all done");
}

fn get_true_size(size: &Size, data_size: usize) -> usize {
    match size {
        Size::Absolute(n) => *n,
        Size::Relative(f) => (data_size as f64 * *f).floor() as usize,
    }
}

fn parse_size(size: &String) -> Size {
    if size.contains('.') {
        // if contains dot, it is a relative size
        // if can't parse to float, log error and exit
        let size = size.parse::<f64>();
        match size {
            Ok(size) => Size::Relative(size),
            Err(_) => {
                error!("size should be a number");
                std::process::exit(1);
            }
        }
    } else {
        // if not, it is a abslute size
        // if can't parse to int, log error and exit
        let size = size.parse::<usize>();
        match size {
            Ok(size) => Size::Absolute(size),
            Err(_) => {
                error!("size should be a number");
                std::process::exit(1)
            }
        }
    }
}

fn input_files_exist(input_files: &Vec<String>) -> () {
    // check if input files exist
    for file in input_files.iter() {
        let path = Path::new(file);
        if !path.exists() {
            error!("file {} does not exist", file);
            std::process::exit(1);
        }
    }
}

fn outfile_exist(outputname: &String, rewrite: bool) -> () {
    // check if output file exists
    let path = Path::new(outputname);
    if path.exists() {
        if rewrite {
            // rewrite the file
            warn!("file {} exist, will rewrite it", outputname);
        } else {
            // exit
            error!("file {} exist, use -r to rewrite it", outputname);
            std::process::exit(1);
        }
    }
}

fn count_lines<R: io::Read>(
    handle: R,
    comment: &Option<String>,
    input_stdin: bool,
) -> Result<(usize, Vec<String>, Vec<String>), io::Error> {
    let mut reader = BufReader::with_capacity(1024 * 32, handle);
    let mut count = 0;
    let mut skipped_lines = Vec::new(); // init skipped lines
    let mut stored_lines = Vec::new(); // init stored lines for stdin
    if input_stdin {
        // read from stdin
        // skip comment lines and stroe it in a vector
        if let Some(comment) = comment {
            // has comment char specified
            for line in reader.lines() {
                let line = line.unwrap();
                if !line.starts_with(&*comment) {
                    count += 1;
                    stored_lines.push(line);
                } else {
                    skipped_lines.push(line);
                }
            }
        } else {
            // no comment char specified
            for _line in reader.lines() {
                count += 1;
                stored_lines.push(_line.unwrap());
            }
        }
        return Ok((count, skipped_lines, stored_lines));
    } else {
        // read from file
        // skip comment lines and stroe it in a vector
        if let Some(comment) = comment {
            // has comment char specified
            // skip comment lines, just read liners
            for line in reader.lines() {
                let line = line.unwrap();
                if !line.starts_with(&*comment) {
                    count += 1;
                } else {
                    skipped_lines.push(line);
                }
            }
        } else {
            // no comment char specified
            loop {
                let len = {
                    let buffer = reader.fill_buf()?;
                    if buffer.is_empty() {
                        break;
                    }
                    count += bytecount::count(&buffer, b'\n');
                    buffer.len()
                };
                reader.consume(len);
            }
        }
        Ok((count, skipped_lines, stored_lines))
    }
}

fn output_samples(
    output_name: &String,
    output_stdout: bool,
    fix_lines: &Vec<String>,
    stored_lines: &Vec<String>,
    sampled_idx: &Vec<&usize>,
    input_files: &Vec<String>,
    comment: &Option<String>,
) -> () {
    if stored_lines.is_empty() {
        // if stored lines is empty, it means we are reading from file
        // info!("read from files");
        output_from_files(
            input_files,
            output_name,
            output_stdout,
            fix_lines,
            sampled_idx,
            comment,
        )
    } else {
        // if stored lines is not empty, it means we are reading from stdin
        // info!("read from stdin");
        output_from_stored_lines(
            output_name,
            output_stdout,
            fix_lines,
            stored_lines,
            sampled_idx,
        )
    }
}

fn output_from_stored_lines(
    output_name: &String,
    output_stdout: bool,
    fix_lines: &Vec<String>,
    stored_lines: &Vec<String>,
    sampled_idx: &Vec<&usize>,
) -> () {
    let (std_handle, file_handle) = if output_stdout {
        let stdout = io::stdout();
        let handle = stdout.lock();
        (Some(handle), None)
    } else {
        let file = File::create(output_name).unwrap();
        (None, Some(file))
    };

    // output to file
    match (std_handle, file_handle) {
        (Some(handle), None) => {
            output_to_handle_stdin(handle, stored_lines, fix_lines, sampled_idx)
        }
        (None, Some(file)) => output_to_handle_stdin(file, stored_lines, fix_lines, sampled_idx),
        _ => unreachable!(),
    }
}

fn output_from_files(
    input_files: &Vec<String>,
    output_name: &String,
    output_stdout: bool,
    fix_lines: &Vec<String>,
    sampled_idx: &Vec<&usize>,
    comment: &Option<String>,
) -> () {
    let (std_handle, file_handle) = if output_stdout {
        let stdout = io::stdout();
        let handle = stdout.lock();
        (Some(handle), None)
    } else {
        let file = File::create(output_name).unwrap();
        (None, Some(file))
    };

    // output to file
    match (std_handle, file_handle) {
        (Some(handle), None) => {
            output_to_handle(handle, input_files, fix_lines, sampled_idx, comment)
        }
        (None, Some(file)) => output_to_handle(file, input_files, fix_lines, sampled_idx, comment),
        _ => unreachable!(),
    }
}

fn output_to_handle(
    mut handle: impl Write,
    input_files: &Vec<String>,
    fix_lines: &Vec<String>,
    sampled_idx: &Vec<&usize>,
    comment: &Option<String>,
) -> () {
    for line in fix_lines.iter() {
        handle.write_all(line.as_bytes()).unwrap();
        handle.write_all(b"\n").unwrap();
    }
    let mut tmp_hashmap = HashMap::with_capacity(sampled_idx.len());
    for item in sampled_idx {
        tmp_hashmap.insert(*item, 0);
    }
    let mut idx = 0;

    if let Some(comment) = comment {
        for file in input_files.iter() {
            let reader = BufReader::new(File::open(file).unwrap());
            for line in reader.lines() {
                let line = line.unwrap();
                if !line.starts_with(&*comment) {
                    if tmp_hashmap.contains_key(&idx) {
                        handle.write_all(line.as_bytes()).unwrap();
                        handle.write_all(b"\n").unwrap();
                    }
                    idx += 1;
                } else {
                    continue;
                }
            }
        }
    } else {
        for file in input_files.iter() {
            let reader = BufReader::new(File::open(file).unwrap());
            for line in reader.lines() {
                let line = line.unwrap();
                if tmp_hashmap.contains_key(&idx) {
                    handle.write_all(line.as_bytes()).unwrap();
                    handle.write_all(b"\n").unwrap();
                }
                idx += 1;
            }
        }
    }
}

fn output_to_handle_stdin(
    mut handle: impl Write,
    stored_lines: &Vec<String>,
    fix_lines: &Vec<String>,
    sampled_idx: &Vec<&usize>,
) -> () {
    for line in fix_lines.iter() {
        handle.write_all(line.as_bytes()).unwrap();
        handle.write_all(b"\n").unwrap();
    }
    for idx in sampled_idx.iter() {
        handle.write_all(stored_lines[**idx].as_bytes()).unwrap();
        handle.write_all(b"\n").unwrap();
    }
}
