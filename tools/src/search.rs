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
use set_search_experiment::text::FeatureExtractor;
use set_search_experiment::FilterConfig;
use set_search_experiment::LinearScan;
use set_search_experiment::Record;

#[derive(Serialize)]
struct Output {
    metadata: Metadata,
    answers: Vec<Answer>,
}

#[derive(Serialize)]
struct Metadata {
    database_file: String,
    query_file: String,
    n_database: usize,
    n_queries: usize,
    max_n: usize,
    radius: Option<f32>,
    topk: Option<usize>,
    length: bool,
    position: bool,
}

#[derive(Serialize)]
struct Answer {
    query: String,
    founds: Vec<Found>,
}

#[derive(Serialize)]
struct Found {
    id: u32,
    dist: f32,
    text: String,
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
    radius: Option<f32>,

    #[arg(short = 'k', long)]
    topk: Option<usize>,

    #[arg(short = 'L', long)]
    length: bool,

    #[arg(short = 'P', long)]
    position: bool,

    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !(args.radius.is_some() ^ args.topk.is_some()) {
        eprintln!("Either -r or -k must be specified.");
        return Ok(());
    }

    let database_texts = load_lines(&args.database_file)?;
    let query_texts = load_lines(&args.query_file)?;
    eprintln!("n_database: {}", database_texts.len());
    eprintln!("n_queries: {}", query_texts.len());

    let extractor = FeatureExtractor::new(1..=args.max_n, args.universe, args.seed)?;

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
        LinearScan::from_records(&records, extractor.universe())?.filter_config(FilterConfig {
            length: args.length,
            position: args.position,
        })
    };
    let duration = start_tp.elapsed();
    eprintln!("Elapsed: {:.3} sec", duration.as_millis() as f64 / 1000.);

    eprintln!("Querying...");
    let start_tp = Instant::now();
    let mut answers = Vec::with_capacity(query_texts.len());
    for (i, query_text) in query_texts.iter().enumerate() {
        if i % 100 == 0 {
            eprintln!("{} / {}", i, query_texts.len());
        }
        let tokens = query_text.split_whitespace().collect::<Vec<_>>();
        let query = extractor.extract(&tokens);
        let searched = if let Some(radius) = args.radius {
            index.range_query(&query, radius)
        } else if let Some(topk) = args.topk {
            index.topk_query(&query, topk)
        } else {
            unreachable!()
        };
        let mut founds = Vec::with_capacity(searched.len());
        for ans in searched {
            founds.push(Found {
                id: ans.id,
                dist: ans.dist,
                text: database_texts[ans.id as usize].clone(),
            });
        }
        answers.push(Answer {
            query: query_text.clone(),
            founds,
        });
    }
    let duration = start_tp.elapsed();
    eprintln!(
        "Elapsed: {:.3} ms per query",
        duration.as_millis() as f64 / query_texts.len() as f64
    );

    let avg_founds =
        answers.iter().map(|ans| ans.founds.len()).sum::<usize>() as f64 / answers.len() as f64;
    eprintln!("Average # of founds: {:.3}", avg_founds);

    let output = Output {
        metadata: Metadata {
            database_file: args.database_file.to_string_lossy().to_string(),
            query_file: args.query_file.to_string_lossy().to_string(),
            n_database: database_texts.len(),
            n_queries: query_texts.len(),
            max_n: args.max_n,
            radius: args.radius,
            topk: args.topk,
            length: args.length,
            position: args.position,
        },
        answers,
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
