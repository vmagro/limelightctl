use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use clap::Parser;
use url::Url;

#[derive(Debug, Clone, Parser)]
pub(crate) struct Upload {
    directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
enum ResourceId {
    Pipeline(usize),
    DetectorModel(usize),
    DetectorLabels(usize),
}

impl ResourceId {
    fn disk_relpath(&self) -> PathBuf {
        match self {
            Self::Pipeline(idx) => Path::new("pipelines").join(format!("{idx}.toml")),
            Self::DetectorModel(idx) => Path::new("detectors").join(format!("{idx}.model.tflite")),
            Self::DetectorLabels(idx) => Path::new("detectors").join(format!("{idx}.labels.txt")),
        }
    }

    fn upload_url(&self, limelight: &Url) -> Url {
        match self {
            Self::Pipeline(idx) => {
                let mut url = limelight.join("/upload-pipeline").expect("this is valid");
                url.query_pairs_mut().append_pair("index", &idx.to_string());
                url
            }
            Self::DetectorModel(idx) => {
                let mut url = limelight.join("/upload-nn").expect("this is valid");
                url.query_pairs_mut().append_pair("type", "detector");
                url.query_pairs_mut().append_pair("index", &idx.to_string());
                url
            }
            Self::DetectorLabels(idx) => {
                let mut url = limelight.join("/upload-nnlabels").expect("this is valid");
                url.query_pairs_mut().append_pair("type", "detector");
                url.query_pairs_mut().append_pair("index", &idx.to_string());
                url
            }
        }
    }
}

impl Upload {
    pub async fn run(&self, limelight: &Url) -> Result<()> {
        let client = reqwest::Client::new();

        Dir::create_ambient_dir_all(&self.directory, ambient_authority())
            .with_context(|| format!("while creating {}", self.directory.display()))?;
        let dir = Dir::open_ambient_dir(&self.directory, ambient_authority())
            .with_context(|| format!("while opening {}", self.directory.display()))?;

        for id in (0..=9).flat_map(|idx| {
            [
                ResourceId::Pipeline(idx),
                ResourceId::DetectorModel(idx),
                ResourceId::DetectorLabels(idx),
            ]
        }) {
            let path = id.disk_relpath();
            if !dir.exists(&path) {
                eprintln!("{} does not exist, skipping", path.display());
                continue;
            }
            let contents = dir
                .read(&path)
                .with_context(|| format!("while reading file {}", path.display()))?;
            let contents = match id {
                ResourceId::Pipeline(_) => {
                    let value: serde_json::Value = toml::from_str(
                        std::str::from_utf8(&contents)
                            .with_context(|| format!("while parsing file {}", path.display()))?,
                    )
                    .with_context(|| format!("while parsing file {}", path.display()))?;
                    serde_json::to_vec_pretty(&value).context("while re-serializing json")?
                }
                _ => contents,
            };

            let url = id.upload_url(limelight);
            eprintln!("{url}");
            client
                .post(url.clone())
                .body(contents)
                .send()
                .await
                .with_context(|| format!("while POSTing {url}"))?;
        }
        Ok(())
    }
}
