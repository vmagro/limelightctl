use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use clap::Parser;
use url::Url;

#[derive(Debug, Clone, Parser)]
pub(crate) struct Download {
    directory: PathBuf,
    pipelines: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum ResourceId {
    Pipeline(usize),
}

impl ResourceId {
    fn disk_relpath(&self) -> PathBuf {
        match self {
            Self::Pipeline(idx) => Path::new("pipelines").join(format!("{idx}.toml")),
        }
    }

    fn download_url(&self, limelight: &Url) -> Url {
        match self {
            Self::Pipeline(idx) => {
                let mut url = limelight.join("/pipeline-atindex").expect("this is valid");
                url.query_pairs_mut().append_pair("index", &idx.to_string());
                url
            }
        }
    }
}

impl Download {
    pub async fn run(&self, limelight: &Url) -> Result<()> {
        let client = reqwest::Client::new();

        Dir::create_ambient_dir_all(&self.directory, ambient_authority())
            .with_context(|| format!("while creating {}", self.directory.display()))?;
        let dir = Dir::open_ambient_dir(&self.directory, ambient_authority())
            .with_context(|| format!("while opening {}", self.directory.display()))?;

        for id in self.pipelines.iter().map(|idx| ResourceId::Pipeline(*idx)) {
            let url = id.download_url(limelight);
            let contents: serde_json::Value = client
                .get(url.clone())
                .send()
                .await
                .with_context(|| format!("while downloading {url}"))?
                .json()
                .await
                .context("while parsing JSON")?;

            let out_path = id.disk_relpath();
            if let Some(parent) = out_path.parent() {
                dir.create_dir_all(parent)
                    .with_context(|| format!("while creating output dir {}", parent.display()))?;
            }

            let text =
                toml::to_string_pretty(&contents).context("while re-serializing to pretty json")?;
            dir.write(&out_path, &text)
                .with_context(|| format!("while writing output file {}", out_path.display()))?;
        }
        Ok(())
    }
}
