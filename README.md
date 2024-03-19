# set-search-experiment

This is an experimental project for simple similarity search on sets.

## Repository structure

- `src` contains the source code for the similarity search.
- `tools` contains command line tools to perform and evaluate the similarity search.
- `bench` contains tools to measure the time performance of the similarity search.

## Dataset preparation

The tools in this project handles similarity search on documents.
This project assumes the following text file as input:

- Each document is separated by a newline.
- Each word in a document is separated by a space.

Example files are available in `data/` directory.

```shell
$ ls -1 data
gutenberg.db.txt.zst
gutenberg.query.txt.zst
```

These files are compressed by zstd and must be decompressed
before they can be used as input for programs under the `tools/` directory.

```shell
$ unzstd data/gutenberg.db.txt.zst
data/gutenberg.db.txt.zst: 1228394 bytes
$ unzstd data/gutenberg.query.txt.zst
data/gutenberg.query.txt.zst: 10089 bytes
```

## Stats

```shell
$ cargo run --release -p tools --bin stats -- -i data/gutenberg.db.txt -o gutenberg.db.json
$ python scripts/plot_stats.py gutenberg.db.json figs
```

## Search tools

```shell
$ cargo run --release -p tools --bin linear_search -- -d data/gutenberg.db.txt -q data/gutenberg.query.txt -o range-search-result.json -r 0.5 -L -P
```

```shell
$ cargo run --release -p tools --bin linear_search -- -d data/gutenberg.db.txt -q data/gutenberg.query.txt -o topk-search-result.json -k 3 -L -P
```

## Evaluate

```shell
$ cargo run --release -p tools --bin evaluate -- -d data/gutenberg.db.txt -q data/gutenberg.query.txt -o eval.json -r 0.5
$ python scripts/parse_eval.py eval.json
```

## Benchmark

```shell
$ cd bench
$ cargo bench
```

## Disclaimer

This software is developed by LegalOn Technologies, Inc.,
but not an officially supported LegalOn Technologies product.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
