pub mod core;

extern crate bytecount;
extern crate rand_pcg;
use clap::Parser;

// use ftlog::appender::FileAppender;

use log::{error, info, warn, LevelFilter};
use log4rs::{
    append::console::{ConsoleAppender, Target},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
// use paris::Logger;
use rand::prelude::*;
use rand_pcg::Pcg64;
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

    /// input file
    #[arg(short, long, required = false, default_value = "stdin")]
    input: String,
}
fn main() {
    // parse args
    let args = Args::parse();

    // parse if quiet mode
    let quiet = args.quiet;

    // init logger
    // set log level if quiet mode
    let log_level = if quiet {
        LevelFilter::Warn
    } else {
        LevelFilter::Info
    };
    // Build a stderr logger.
    let log_stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(Box::new(PatternEncoder::new("{d} {h({l})} {m}{n}")))
        .build();
    let log_config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log_level)))
                .build("stderr", Box::new(log_stderr)),
        )
        .build(Root::builder().appender("stderr").build(log_level))
        .unwrap();
    // init logger using config
    log4rs::init_config(log_config).unwrap();

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
    let input_file = args.input;
    let mut stdin_mode = false;
    if input_file == "stdin" {
        stdin_mode = true;
    }
    if !quiet {
        if stdin_mode {
            info!("input from stdin");
        } else {
            info!("input file: {:?}", input_file);
        }
    }
    if !stdin_mode {
        // check if input files exist if not stdin mode
        input_files_exist(&input_file);
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

    // start read inputs
    let (line_count, fix_lines, stored_lines) = if stdin_mode {
        // if stdin mode, read from stdin
        let handle = io::stdin();
        read_inputs(handle, &comment, stdin_mode).unwrap()
    } else {
        // if not stdin mode, read from input file
        let handle = File::open(&input_file).unwrap();
        read_inputs(handle, &comment, stdin_mode).unwrap()
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

    // start sample using reservoir sampling
    if !quiet {
        info!("start sample");
    }

    let mut rng: Pcg64 = Pcg64::from_rng(thread_rng()).expect("failed to init rng");
    let mut sample_array = vec![String::new(); true_size];
    let output_string = if stdin_mode {
        let stored_lines = stored_lines.iter().map(|l| l.to_string());
        core::reservoir_sample(stored_lines, &mut sample_array, &mut rng);
        sample_array.join("\n")
    } else {
        let input_reader = BufReader::new(File::open(&input_file).unwrap());
        let input_lines = input_reader.lines().map(|l| l.unwrap());
        if let Some(comment) = comment {
            let input_lines = input_lines.filter(|l| !l.starts_with(&comment));
            core::reservoir_sample(input_lines, &mut sample_array, &mut rng);
        } else {
            core::reservoir_sample(input_lines, &mut sample_array, &mut rng);
        }
        sample_array.join("\n")
    };

    if !quiet {
        info!("sample done");
    }

    let mut out_file: Box<dyn Write> = if output_stdout {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(output_name).unwrap())
    };
    if !quiet {
        info!("start output");
    }

    if !fix_lines.is_empty() {
        let fix_strings = fix_lines.join("\n");
        out_file.write_all(fix_strings.as_bytes()).unwrap();
        out_file.write_all(b"\n").unwrap();
    }
    out_file.write_all(output_string.as_bytes()).unwrap();
    out_file.write_all(b"\n").unwrap();

    if !quiet {
        info!("ALL DONE");
    }
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

fn input_files_exist(input_file: &String) -> () {
    // check if input files exist
    let path = Path::new(input_file);
    if !path.exists() {
        error!("file {} does not exist", input_file);
        std::process::exit(1);
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

fn read_inputs<R: io::Read>(
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
