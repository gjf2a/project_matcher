mod searcher;

use std::env::args;
use crate::searcher::search;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 5 {
        println!("Usage: project_matcher file.csv num_projects num_tries num_mutations");
    } else {
        println!("{}", search(args[1].as_str(), args[2].parse::<usize>().unwrap(), args[3].parse::<usize>().unwrap(), args[4].parse::<usize>().unwrap())?);
    }
    Ok(())
}
