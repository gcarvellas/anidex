use compare::filter_unread_manga;
use clap::Parser;

mod config;
mod anilist;
mod mangadex;
mod compare;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    language: String,
    #[arg(short, long, default_value_t = 1)]
    jobs: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let entries = match filter_unread_manga(args.username, &args.language, args.jobs).await {
        Ok(data) => data,
        Err(e) => panic!("{}", e),
    }; 

    for entry in entries {
        println!("{}", entry);
    }

}
