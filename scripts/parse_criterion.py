#!/usr/bin/env python3

"""
# 使い方

$ python parse_criterion.py target/criterion
"""

import os
import sys
import json


def get_target_dirs(path):
    files = os.listdir(path)
    dirs = [f for f in files if os.path.isdir(os.path.join(path, f)) and f != "report"]
    return dirs


def show_mean_point(title, est_json):
    jdic = json.load(open(est_json, "rt"))
    mean_ns = float(jdic["mean"]["point_estimate"])
    mean_ms = mean_ns / (1000 * 1000)
    mean_sc = mean_ms / 1000
    serr_ns = float(jdic["mean"]["standard_error"])
    serr_ms = serr_ns / (1000 * 1000)
    serr_sc = serr_ms / 1000
    print(
        f"{title}",
        f"{mean_ms:.2f}",
        f"{serr_ms:.2f}",
        f"{mean_sc:.2f}",
        f"{serr_sc:.2f}",
        sep="\t",
    )


def main():
    bench_dir = sys.argv[1]
    operations = get_target_dirs(f"{bench_dir}/")
    operations.sort()
    for operation in operations:
        methods = get_target_dirs(f"{bench_dir}/{operation}")
        methods.sort()
        print(f"# {operation}")
        print("title", "mean_ms", "error_ms", "mean_sec", "error_sec", sep="\t")
        for method in methods:
            est_json = f"{bench_dir}/{operation}/{method}/new/estimates.json"
            show_mean_point(method, est_json)
        print()


if __name__ == "__main__":
    main()
