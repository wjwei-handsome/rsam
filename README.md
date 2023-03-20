# rsam

A random sampler for text-based data using reservoir sampling algorithm.

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
rsam -s 1000 -o output.txt -i input.txt

## sample 1000 lines from a file and output to stdout
rsam -s 1000 -i input.txt 1>output.txt 2>output.log

## sample 1000 lines from a file and rewrite the exist output file
rsam -s 1000 -i input.txt -o output.txt -r

## sample 1% lines from a file
rsam -s 0.1 -o output.txt -i input.txt
rsam -s .1 -o output.txt -i input.txt

## keep the comment lines
rsam -s 0.1 -o output.txt -c "#" -i input.txt # keep the comment lines start with "#"

## read from stdin
zcat input.txt.gz | rsam -s 0.1 -o output.txt
```

## benchmark

environment: 1.4 GHz 4-core Intel Core i5;16 GB 2133 MHz DDR3;macOS 10.15.7;

```bash
~/code/rsam main* ❯ time seq 200000 |./target/release/rsam -s 100000 -o /dev/null -r
2023-03-20T17:43:04.722500+08:00 INFO input size: "100000"
2023-03-20T17:43:04.722699+08:00 INFO parsed size: Absolute(100000)
2023-03-20T17:43:04.722726+08:00 INFO input from stdin
2023-03-20T17:43:04.722741+08:00 INFO output to: "/dev/null"
2023-03-20T17:43:04.722776+08:00 WARN file /dev/null exist, will rewrite it
2023-03-20T17:43:04.722796+08:00 INFO comment char: None
2023-03-20T17:43:04.771903+08:00 INFO total line count: 200000
2023-03-20T17:43:04.771934+08:00 INFO true size: 100000
2023-03-20T17:43:04.771947+08:00 INFO Start sample
2023-03-20T17:43:04.803320+08:00 INFO sample done
2023-03-20T17:43:04.803407+08:00 INFO start output

________________________________________________________
Executed in  116.49 millis    fish           external
   usr time  126.31 millis    0.35 millis  125.96 millis
   sys time   14.19 millis    1.24 millis   12.95 millis
```
