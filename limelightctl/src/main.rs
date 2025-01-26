use anyhow::Result;
use clap::Parser;
use url::Url;

mod download;
mod status;
mod upload;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(long, default_value = "http://limelight.local:5807")]
    limelight: Url,
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, Parser)]
enum Subcommand {
    Download(download::Download),
    Status(status::Status),
    Upload(upload::Upload),
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
        Subcommand::Download(c) => c.run(&args.limelight).await,
        Subcommand::Status(c) => c.run(&args.limelight).await,
        Subcommand::Upload(c) => c.run(&args.limelight).await,
    }
}
