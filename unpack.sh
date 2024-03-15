#!/bin/bash

set -eux

type unzstd

unzstd data/gutenberg.db.txt.zst
unzstd data/gutenberg.query.txt.zst
