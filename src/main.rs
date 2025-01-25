use anyhow::Result;
use clap::Parser;
use url::Url;

mod download;
mod status;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(long, default_value = "http://limelight.local:5807")]
    limelight: Url,
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
enum Subcommand {
    Status(status::Status),
    Download(download::Download),
}

#[tokio::main]
async fn main() {
    if let Err(e) = do_main().await {
        eprintln!("{e:#?}");
        std::process::exit(1);
    }
}

async fn do_main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    match args.subcommand {
        Subcommand::Status(c) => c.run(&args.limelight).await,
        Subcommand::Download(c) => c.run(&args.limelight).await,
    }
}
