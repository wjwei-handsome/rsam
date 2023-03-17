# rsam

A random sampler for text-based data.

## install

```bash
git clone https://github.com/wjwei-handsome/rsam.git
cd rsam
cargo build --release
cp ./target/release/rsam /your/bin/path
```

## usage

```bash
## sample 1000 lines from a file
rsam -s 1000 -o output.txt input.txt

## sample 1000 lines for multi files
rsam -s 1000 -o output.txt input1.txt input2.txt

## sample 1000 lines from a file and output to stdout
rsam -s 1000 input.txt 1>output.txt 2>output.log

## sample 1000 lines from a file and rewrite the exist output file
rsam -s 1000 input.txt -o output.txt -r

## sample 1% lines from a file
rsam -s 0.1 -o output.txt input.txt
rsam -s .1 -o output.txt input.txt

## keep the comment lines
rsam -s 0.1 -o output.txt -c "#" input.txt # keep the comment lines start with "#"

## read from stdin
zcat input.txt.gz | rsam -s 0.1 -o output.txt
```

## benchmark

```bash

```
