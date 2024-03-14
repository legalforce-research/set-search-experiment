use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

use clap::Parser;
use serde::Serialize;
use set_search_experiment::text::FeatureExtractor;
use set_search_experiment::OrderedSet;

#[derive(Serialize)]
struct Output {
    metadata: Metadata,
    lengths: Vec<usize>,
    elem_freqs: Vec<usize>,
}

#[derive(Serialize)]
struct Metadata {
    input_txt: String,
    max_n: usize,
    n_input: usize,
    n_elems: usize,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long)]
    input_txt: String,

    #[arg(short = 'o', long)]
    output_json: String,

    #[arg(short = 'n', long, default_value_t = 1)]
    max_n: usize,

    #[arg(short = 'u', long, default_value_t = 1 << 20)]
    universe: u32,

    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let input_texts = load_lines(&args.input_txt)?;
    eprintln!("n_input: {}", input_texts.len());

    let max_n = args.max_n;
    let extractor = FeatureExtractor::new(1..=max_n, args.universe, args.seed)?;
    let mut sets = Vec::with_capacity(input_texts.len());

    for text in &input_texts {
        let tokens = text.split_whitespace().collect::<Vec<_>>();
        sets.push(extractor.extract(&tokens));
    }

    let lengths = lengths(&sets);
    let elem_freqs = elem_freqs(&sets);
    eprintln!("n_elems: {}", elem_freqs.len());

    let output = Output {
        metadata: Metadata {
            input_txt: args.input_txt,
            max_n,
            n_input: input_texts.len(),
            n_elems: elem_freqs.len(),
        },
        lengths,
        elem_freqs,
    };

    let mut writer = File::create(&args.output_json)?;
    writer.write_all(serde_json::to_string(&output)?.as_bytes())?;

    Ok(())
}

fn load_lines(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let reader = BufReader::new(File::open(path)?);
    let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
    Ok(lines)
}

fn lengths(sets: &[OrderedSet<u32>]) -> Vec<usize> {
    sets.iter().map(|set| set.len()).collect::<Vec<_>>()
}

fn elem_freqs(sets: &[OrderedSet<u32>]) -> Vec<usize> {
    let mut elem_freqs = HashMap::new();
    for set in sets {
        for elem in set.iter().cloned() {
            *elem_freqs.entry(elem).or_insert(0) += 1;
        }
    }
    let mut elem_freqs = elem_freqs.into_iter().map(|(_, c)| c).collect::<Vec<_>>();
    elem_freqs.sort_unstable_by(|a, b| b.cmp(a));
    elem_freqs
}
