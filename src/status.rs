use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use url::Url;

#[derive(Debug, Clone, Parser)]
pub(crate) struct Status {}

impl Status {
    pub async fn run(&self, limelight: &Url) -> Result<()> {
        let url = limelight.join("/status").expect("valid url");
        let body: serde_json::Value = reqwest::get(url.clone())
            .await
            .with_context(|| format!("while GETting {url}"))?
            .json()
            .await
            .context("while parsing JSON")?;
        println!(
            "{}",
            serde_json::to_string_pretty(&body)
                .expect("serde_json::Value can always be serialized")
        );
        Ok(())
    }
}
