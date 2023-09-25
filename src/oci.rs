use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

use crate::{download, Result};

pub struct Api {
    base: String,
    client: reqwest::blocking::Client,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    pub media_type: String,
    pub size: u64,
    pub digest: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u64,
    pub media_type: String,
    pub config: Blob,
    pub layers: Vec<Blob>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigField {
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub author: String,
    pub architecture: String,
    pub os: String,
    pub config: ConfigField,
}

impl Api {
    pub fn new(base: &str) -> Result<Api> {
        let base = base.to_string();
        let client = Client::builder().build()?;

        Ok(Api { base, client })
    }

    pub fn get_manifest(&self, name: &str, tag: &str) -> Result<Manifest> {
        let res = self
            .client
            .get(format!(
                "{base}/v2/{name}/manifests/{tag}",
                base = self.base
            ))
            .send()?;

        Ok(serde_json::from_reader(res)?)
    }

    pub fn get_config(&self, name: &str, cfg: &Blob) -> Result<Config> {
        let res = self
            .client
            .get(format!(
                "{base}/v2/{name}/blobs/{digest}",
                base = self.base,
                digest = cfg.digest
            ))
            .send()?;
        Ok(serde_json::from_reader(res)?)
    }

    pub fn download_layer<P: AsRef<Path>>(&self, dst: P, name: &str, layer: &Blob) -> Result<()> {
        let res = self
            .client
            .get(format!(
                "{base}/v2/{name}/blobs/{digest}",
                base = self.base,
                digest = layer.digest
            ))
            .send()?;
        download::run(dst, res)
    }
}
