//! The transactions processing engine CLI tool

mod cents;
mod client;
mod engine;
mod err;
mod transaction;

use std::path::PathBuf;
use structopt::StructOpt;

use engine::Engine;
use transaction::Transaction;

#[derive(Debug, StructOpt)]
#[structopt(name = "transactions", about = "A toy transactions processing engine")]
struct Opt {
    #[structopt(
        parse(from_os_str),
        help = "Path to the input csv that contains all transactions in chronological order"
    )]
    input: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(opt.input)
        .unwrap();
    let mut engine = Engine::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let tx: Transaction = record.deserialize(None).unwrap();
        // TODO: error handling and logging
        let _ = engine.handle_tx(tx);
    }
    print!("{}", engine);
}
