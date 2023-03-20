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
~/code/rsam main ❯ time seq 200000 |./target/release/rsam -s 100000  -o /dev/null -r
2023-03-18 00:19:12 INFO input size: "100000"
2023-03-18 00:19:12 INFO parsed size: Absolute(100000)
2023-03-18 00:19:12 INFO input from stdin
2023-03-18 00:19:12 INFO output to: "/dev/null"
2023-03-18 00:19:12 WARN file /dev/null exist, will rewrite it
2023-03-18 00:19:12 INFO comment char: None
2023-03-18 00:19:12 INFO total line count: 200000
2023-03-18 00:19:12 INFO true size: 100000

________________________________________________________
Executed in  356.50 millis    fish           external
   usr time  269.78 millis    0.37 millis  269.40 millis
   sys time  123.92 millis    1.45 millis  122.47 millis
```
