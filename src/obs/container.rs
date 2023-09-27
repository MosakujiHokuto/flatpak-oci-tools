use log::{debug, info};
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::path::Path;
use tar::Archive;

pub struct ContainerImage {
    ar: Archive<File>,
    manifest: Option<Manifest>,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(rename = "Layers")]
    layers: Vec<String>,
}

impl ContainerImage {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<ContainerImage> {
        let file = File::open(path)?;
        let ar = Archive::new(file);
        Ok(ContainerImage { ar, manifest: None })
    }

    /// Unpack FS Layers into specified location
    pub fn unpack<P: AsRef<Path>>(
        &mut self,
        extract_path: P,
    ) -> io::Result<()> {
        info!("Unpacking archive");
        self.ar.unpack(extract_path.as_ref())?;

        debug!("Reading manifest");
        let mut vm: Vec<Manifest> =
            serde_json::from_reader(File::open(Path::join(extract_path.as_ref(), "manifest.json"))?)?;
        if vm.len() != 1 {
            panic!("Empty manifest or multiple elements");
        }

        self.manifest = Some(vm.pop().unwrap());

        Ok(())
    }

    pub fn layers(&self) -> Option<&Vec<String>> {
	self.manifest.as_ref().map(|m| &m.layers)
    }
}
