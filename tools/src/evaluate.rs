use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use serde::Serialize;
use set_search_experiment::metric::Evaluation;
use set_search_experiment::text::FeatureExtractor;
use set_search_experiment::FilterConfig;
use set_search_experiment::LinearScan;
use set_search_experiment::OrderedSet;
use set_search_experiment::Record;

#[derive(Serialize)]
struct Output {
    metadata: Metadata,
    no_filter: Vec<Counter>,
    length_filter: Vec<Counter>,
    position_filter: Vec<Counter>,
    all_filters: Vec<Counter>,
}

#[derive(Serialize)]
struct Metadata {
    database_file: String,
    query_file: String,
    n_database: usize,
    n_queries: usize,
    max_n: usize,
    radius: f32,
    seed: Option<u64>,
}

#[derive(Default, Debug, Serialize)]
struct Counter {
    length_filtered: usize,
    position_filtered: usize,
    verified: usize,
    undefined: usize,
    accepted: usize,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'd', long)]
    database_file: PathBuf,

    #[arg(short = 'q', long)]
    query_file: PathBuf,

    #[arg(short = 'o', long)]
    output_json: PathBuf,

    #[arg(short = 'n', long, default_value_t = 1)]
    max_n: usize,

    #[arg(short = 'u', long, default_value_t = 1 << 20)]
    universe: u32,

    #[arg(short = 'r', long)]
    radius: f32,

    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let database_texts = load_lines(&args.database_file)?;
    let query_texts = load_lines(&args.query_file)?;

    let extractor = FeatureExtractor::new(1..=args.max_n, args.universe, args.seed)?;
    eprintln!("n_database: {}", database_texts.len());
    eprintln!("n_queries: {}", query_texts.len());

    eprintln!("Indexing...");
    let start_tp = Instant::now();
    let index = {
        let mut records = Vec::with_capacity(database_texts.len());
        for (id, text) in database_texts.iter().enumerate() {
            let tokens = text.split_whitespace().collect::<Vec<_>>();
            let set = extractor.extract(&tokens);
            let record = Record { id: id as u32, set };
            records.push(record);
        }
        LinearScan::from_records(&records, extractor.universe())?
    };
    let duration = start_tp.elapsed();
    eprintln!("Elapsed: {:.3} sec", duration.as_millis() as f64 / 1000.);

    eprintln!("Generating queries...");
    let queries = query_texts
        .iter()
        .map(|text| text.split_whitespace().collect::<Vec<_>>())
        .map(|tokens| extractor.extract(&tokens))
        .collect::<Vec<_>>();

    eprintln!("Evaluating no filter...");
    let index = index.filter_config(FilterConfig {
        length: false,
        position: false,
    });
    let no_filter = evaluate_range_search(&index, &queries, args.radius);

    eprintln!("Evaluating length filter...");
    let index = index.filter_config(FilterConfig {
        length: true,
        position: false,
    });
    let length_filter = evaluate_range_search(&index, &queries, args.radius);

    eprintln!("Evaluating position filter...");
    let index = index.filter_config(FilterConfig {
        length: false,
        position: true,
    });
    let position_filter = evaluate_range_search(&index, &queries, args.radius);

    eprintln!("Evaluating all filters...");
    let index = index.filter_config(FilterConfig {
        length: true,
        position: true,
    });
    let all_filters = evaluate_range_search(&index, &queries, args.radius);

    let output = Output {
        metadata: Metadata {
            database_file: args.database_file.to_string_lossy().to_string(),
            query_file: args.query_file.to_string_lossy().to_string(),
            n_database: database_texts.len(),
            n_queries: query_texts.len(),
            max_n: args.max_n,
            radius: args.radius,
            seed: args.seed,
        },
        no_filter,
        length_filter,
        position_filter,
        all_filters,
    };
    let j = serde_json::to_string_pretty(&output).unwrap();

    let mut file = File::create(args.output_json).unwrap();
    file.write_all(j.as_bytes()).unwrap();

    Ok(())
}

fn load_lines<P>(path: P) -> Result<Vec<String>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let reader = BufReader::new(File::open(path)?);
    let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
    Ok(lines)
}

fn evaluate_range_search(
    index: &LinearScan,
    queries: &[OrderedSet<u32>],
    radius: f32,
) -> Vec<Counter> {
    let mut counters = Vec::with_capacity(queries.len());
    for query in queries {
        let evals = index.evaluate(query, radius);
        let mut counter = Counter::default();
        for eval in evals {
            match eval {
                Evaluation::LengthFiltered => counter.length_filtered += 1,
                Evaluation::PositionFiltered => counter.position_filtered += 1,
                Evaluation::Verified => counter.verified += 1,
                Evaluation::Undefined => counter.undefined += 1,
                Evaluation::Accepted(_) => counter.accepted += 1,
            }
        }
        counters.push(counter);
    }
    counters
}
