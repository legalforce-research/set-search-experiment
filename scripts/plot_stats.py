import argparse
import json
import os
import statistics

import matplotlib.pyplot as plt
from matplotlib.ticker import ScalarFormatter
import numpy as np

plt.style.use("seaborn-v0_8")

formatter = ScalarFormatter(useMathText=True)
formatter.set_scientific(True)


def find_top_n_percent_value(data, n, bins=100):
    hist, bin_edges = np.histogram(data, bins=bins, density=True)
    cdf = np.cumsum(hist) / np.sum(hist)
    target_index = np.searchsorted(cdf, n / 100.0)
    return bin_edges[target_index + 1]


def plot_length_distribution(lengths, out_dir, metadata):
    mean = statistics.mean(lengths)
    stddev = statistics.stdev(lengths)
    top_n = find_top_n_percent_value(lengths, 99.5)

    fig, ax = plt.subplots()
    ax.hist(lengths, bins=100, range=(0, top_n))
    ax.axvline(x=mean, color="red", label=f"Mean = {mean:.0f} ± {stddev:.1f}")
    ax.set_xlabel("Length")
    ax.set_ylabel("Frequency")
    ax.legend()
    fig.tight_layout()
    max_n = metadata["max_n"]
    fig.savefig(f"{out_dir}/length_distribution.max_n={max_n}.png")
    plt.close(fig)


def plot_elem_freq_distribution(elem_freqs, out_dir, metadata):
    fig, ax = plt.subplots()
    ax.plot(range(len(elem_freqs)), elem_freqs)
    ax.set_yscale("log", base=10)
    ax.set_xlabel("Elements")
    ax.set_ylabel("Frequency")
    ax.xaxis.set_major_formatter(formatter)
    fig.tight_layout()
    max_n = metadata["max_n"]
    fig.savefig(f"{out_dir}/elem_freq_distribution.max_n={max_n}.png")
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("stat_json")
    parser.add_argument("out_dir")
    args = parser.parse_args()

    os.makedirs(args.out_dir, exist_ok=True)

    with open(args.stat_json) as f:
        stats = json.load(f)

    metadata = stats["metadata"]
    print(metadata)

    lengths = stats["lengths"]
    plot_length_distribution(lengths, args.out_dir, metadata)

    elem_freqs = stats["elem_freqs"]
    plot_elem_freq_distribution(elem_freqs, args.out_dir, metadata)


if __name__ == "__main__":
    main()
