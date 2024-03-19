# set-search-experiment

This is an experimental project for simple similarity search on sets.

## Repository structure

- `src` contains the source code for the similarity search.
- `tools` contains command line tools to perform and evaluate the similarity search.
- `bench` contains tools to measure the time performance of the similarity search.
- `scripts` contains scripts to analyze the results of the similarity search.

## Dataset preparation

The tools in this project handles similarity search on documents.
This project assumes the following text file as input:

- Each document is separated by a newline.
- Each word in a document is separated by a space.

Example files are available in the `data/` directory.

```shell
$ ls -1 data
gutenberg.db.txt.zst
gutenberg.query.txt.zst
```

These files are compressed by zstd and must be decompressed to use.
Install `zstd` and run the `unpack.sh` script:

```shell
$ sudo apt install zstd
$ ./unpack.sh
```

## Usage

TBW

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

The datasets under the `data/` directory are from [Project Gutenberg](https://gutenberg.org/),
which follows the public domain license.
