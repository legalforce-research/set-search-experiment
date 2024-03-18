import argparse
import json
import statistics


def counts_foreach(logs):
    keys = [
        'length_filtered',
        'prefix_filtered',
        'position_filtered',
        'verified',
        'undefined',
        'accepted',
    ]
    return {key: [log[key] for log in logs] for key in keys}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('input_json')
    args = parser.parse_args()

    with open(args.input_json) as f:
        parsed = json.load(f)

    print('[all_filters]')
    all_filters = parsed['all_filters']
    counts = counts_foreach(all_filters)

    length_filtered = sum(counts['length_filtered'])
    prefix_filtered = sum(counts['prefix_filtered'])
    position_filtered = sum(counts['position_filtered'])
    total_filtered = length_filtered + prefix_filtered + position_filtered

    print(f'length_filtered_ratio: {length_filtered / total_filtered:.3f}')
    print(f'prefix_filtered_ratio: {prefix_filtered / total_filtered:.3f}')
    print(f'position_filtered_ratio: {position_filtered / total_filtered:.3f}')

    verified = sum(counts['verified'])
    undefined = sum(counts['undefined'])
    print(f'verified: {verified}')
    print(f'undefined: {undefined}')

    accepted_avg = statistics.mean(counts['accepted'])
    accepted_stddev = statistics.stdev(counts['accepted'])
    print(f'accepted_per_query: {accepted_avg:.2f} ± {accepted_stddev:.2f}')


if __name__ == "__main__":
    main()
