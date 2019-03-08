use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use super::common::*;

pub fn read_csv(file_path: &str) -> HashSet<(String, FeedType)> {
    debug!("Read csv file {:?}", file_path);
    let f = File::open(file_path).unwrap();
    let file = BufReader::new(&f);

    let mut result = HashSet::new();

    for line_res in file.lines().skip(1) {
        let line = line_res.unwrap();
        let tokens = line.split(',').collect::<Vec<_>>();

        if tokens.len() != 2 {
            warn!("Found bad line (invalid column count) {:?}", line);
            continue;
        }

        let url = tokens[0];
        let kind_str = tokens[1];

        match FeedType::from_str(kind_str) {
            Ok(feed_type) => {
                result.insert((url.to_string(), feed_type));
            }
            Err(_err) => warn!("Found bad line (invalid feed kind) {:?}", line),
        }
    }

    result
}
