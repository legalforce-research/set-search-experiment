use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion, SamplingMode,
};
use set_search_experiment::text::FeatureExtractor;
use set_search_experiment::FilterConfig;
use set_search_experiment::LinearScan;
use set_search_experiment::OrderedSet;
use set_search_experiment::Record;

const SAMPLE_SIZE: usize = 10;

// Replace these with the files you want to benchmark (in zstd format).
const DATABASE_TXT_ZST: &[u8] = include_bytes!("../../data/gutenberg.db.txt.zst");
const QUERY_TXT_ZST: &[u8] = include_bytes!("../../data/gutenberg.query.txt.zst");

const SEED: u64 = 42;
const MAX_N: usize = 3;
const UNIVERSE: u32 = 1 << 20;

const FILTER_CONFIGS: &[FilterConfig] = &[
    FilterConfig {
        length: false,
        position: false,
    },
    FilterConfig {
        length: true,
        position: false,
    },
    FilterConfig {
        length: false,
        position: true,
    },
    FilterConfig {
        length: true,
        position: true,
    },
];

fn database_txt() -> Vec<String> {
    let bytes = zstd::decode_all(DATABASE_TXT_ZST).unwrap();
    String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|s| s.to_owned())
        .collect()
}

fn query_txt() -> Vec<String> {
    let bytes = zstd::decode_all(QUERY_TXT_ZST).unwrap();
    String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|s| s.to_owned())
        .collect()
}

fn criterion_range_search_linear_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_search/linear_scan");
    group.sample_size(SAMPLE_SIZE);
    group.sampling_mode(SamplingMode::Flat);

    let database_texts = database_txt();
    let query_texts = query_txt();

    for max_n in 1..=MAX_N {
        perform_range_search_linear_scan(&mut group, &database_texts, &query_texts, max_n);
    }
}

fn perform_range_search_linear_scan(
    group: &mut BenchmarkGroup<WallTime>,
    database_texts: &[String],
    query_texts: &[String],
    max_n: usize,
) {
    let n = database_texts.len();
    let m = query_texts.len();

    let extractor = FeatureExtractor::new(1..=max_n, UNIVERSE, Some(SEED)).unwrap();
    let mut index = make_linear_scan(database_texts, &extractor);
    let queries = make_queries(query_texts, &extractor);

    for r in [0.1, 0.2, 0.5] {
        for &cfg in FILTER_CONFIGS {
            let l = usize::from(cfg.length);
            let s = usize::from(cfg.position);
            index = index.filter_config(cfg);
            let group_id = format!("N={max_n}_n={n}_m={m}_r={r}/L={l}_S={s}");
            group.bench_function(group_id, |b| {
                b.iter(|| {
                    for query in &queries {
                        index.range_query(query, r);
                    }
                });
            });
        }
    }
}

fn make_linear_scan(database_texts: &[String], extractor: &FeatureExtractor) -> LinearScan {
    let mut records = Vec::with_capacity(database_texts.len());
    for (id, text) in database_texts.iter().enumerate() {
        let tokens = text.split_whitespace().collect::<Vec<_>>();
        let set = extractor.extract(&tokens);
        let record = Record { id: id as u32, set };
        records.push(record);
    }
    LinearScan::from_records(&records, UNIVERSE).unwrap()
}

fn make_queries(query_texts: &[String], extractor: &FeatureExtractor) -> Vec<OrderedSet<u32>> {
    query_texts
        .iter()
        .map(|text| text.split_whitespace().collect::<Vec<_>>())
        .map(|tokens| extractor.extract(&tokens))
        .collect::<Vec<_>>()
}

criterion_group!(benches, criterion_range_search_linear_scan);
criterion_main!(benches);
