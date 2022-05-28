//! Log Analyzer
//! Copyright (C) 2022 Sebastian Müller

// std
use std::collections::HashMap;
use std::fs::{canonicalize, File};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;

// 3rd party
use clap::Parser;
use tabled::{Style, Table, Tabled};

/// Group identifier, determines the JSON key which is used for grouping the output
static GROUP_ID: &str = "type";

/// CLI tool that analyses a log file.
///
/// The log file must be a text file where each line forms a valid JSON object.
/// Each JSON object is expected to contain a "type" field of type String.
/// The tool will provide a statistic which unique "type"s are in the log file as well as the size
/// of all messages for each type, respectively.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to a log file to be analyzed
    #[clap(parse(from_os_str))]
    file: PathBuf,
}

/// Table entry structure for printing
#[derive(Tabled)]
struct LogEntryStatistic {
    #[tabled(rename = "Type")]
    t: String,
    #[tabled(rename = "Size [byte]")]
    size: usize,
}

// Implement FromIterator trait to simply collect() all LogEntryStatistics later into a HashMap
impl FromIterator<LogEntryStatistic> for HashMap<String, usize> {
    /// Creates a HashMap<String, usize> from an iterator of item type LogEntryStatistic
    fn from_iter<I: IntoIterator<Item = LogEntryStatistic>>(iter: I) -> Self {
        let mut hash_map = HashMap::new();

        iter.into_iter().for_each(|x| {
            // Accumulate sizes in HashMap
            // https://stackoverflow.com/a/30414450/6658448
            *hash_map.entry(x.t).or_insert(0) += x.size;
        });

        hash_map
    }
}

/// Error type depicting the case if parsing a JSON object fails
struct ParseError {
    /// Context/input data for which parsing failed
    context: String,
}

/// Receive one JSON object and try parse it.
///
/// # Arguments
///
/// - `raw` - Raw `String` that should be parsed
///
/// # Returns
///
/// A Result that is a LogEntryStatistic in a successful case, or a ParseError in case the
/// JSON object could not be parsed.
fn parse_json(raw: String) -> Result<LogEntryStatistic, ParseError> {
    // Error handling for JSON parse errors
    let json: serde_json::Value = if let Ok(x) = serde_json::from_str(&raw) {
        x
    } else {
        return Err(ParseError { context: raw });
    };
    // Error handling for missing GROUP_ID field
    let t: String = if let Some(x) = json.get(GROUP_ID) {
        // Error handling for GROUP_ID value is not a String
        if let Some(s) = x.as_str() {
            s.to_string()
        } else {
            return Err(ParseError { context: raw });
        }
    } else {
        return Err(ParseError { context: raw });
    };
    Ok(LogEntryStatistic { t, size: raw.len() })
}

/// Receive a valid file and read it.
///
/// Each line of the file should contain a valid JSON object. Each JSON object consists of a "type"
/// field, containing a String, and any number of arbitrary additional fields.
///
/// # Arguments
///
/// `file` - A `File` handle used for reading
///
/// # Returns
///
/// A HashMap with all unique "type"s as keys and the accumulated byte size of the JSON objects for
/// each type as value.
fn read_json_objects(file: &File) -> HashMap<String, usize> {
    //let mut log_statistics: HashMap<String, usize> = HashMap::new();
    let reader = BufReader::new(file);
    let log_statistics: HashMap<String, usize> = reader
        .lines()
        // handle line read errors
        .filter_map(|x| {
            x.map_err(|e| println!("Warning: Could not read line from file: \"{}\". Statistics might be unreliable.", e))
                .ok()
        })
        .map(parse_json)
        // handle parse errors
        .filter_map(|x| {
            x.map_err(|e| println!("Warning: Wrongly formatted object: \"{}\". Object needs to be valid JSON containing a \"{}\" field of type String. Statistics might be unreliable.", e.context, GROUP_ID))
                .ok()
        })
        // Use the `FromIterator` trait here to simply collect() all LogEntryStatistics
        .collect();

    log_statistics
}

/// Print out the log statistics provided as HashMap into a table to stdout.
///
/// The HashMap is expected to hold the "type" as key and the size in bytes as value.
///
/// # Arguments
///
/// - `statistics` - A HashMap containing a String key and an usize value
fn print_statistics(statistics: &HashMap<String, usize>) {
    println!(
        "{}",
        // Map HashMap contents into table structure
        Table::new(statistics.iter().map(|(key, value)| {
            LogEntryStatistic {
                t: key.to_string(),
                size: *value,
            }
        }))
        .with(Style::rounded())
    );
}

fn main() {
    let args = Args::parse();

    // Determine absolute path
    let absolute_path = canonicalize(&args.file).expect(&format!(
        "FILE argument was not understood: {}. Does the file exist?",
        &args.file.display()
    ));
    println!("Using logfile {}\n", &absolute_path.display());

    let file = File::open(&absolute_path).expect(&format!(
        "Could not open file: {}",
        &absolute_path.display()
    ));

    let log_statistics = read_json_objects(&file);

    print_statistics(&log_statistics);
}
