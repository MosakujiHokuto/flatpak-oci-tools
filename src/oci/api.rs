use reqwest::blocking::Client;
use serde::Deserialize;
use std::fmt::Debug;

use crate::Result;

pub struct OciApi {
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

impl OciApi {
    pub fn new(base: &str) -> Result<OciApi> {
	let base = base.to_string();
        let client = Client::builder().build()?;

        Ok(OciApi { base, client })
    }

    pub fn get_manifest(&self, name: &str, tag: &str) -> Result<Manifest> {
        let res = self.client
            .get(format!("{base}/v2/{name}/manifests/{tag}", base = self.base))
            .send()?;

	Ok(serde_json::from_reader(res)?)
    }
}
