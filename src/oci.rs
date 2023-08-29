use crate::workdir;
use flate2::read::GzDecoder;
use log::info;
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::path::Path;
use tar::Archive;

pub struct Image {
    ar: Archive<File>,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(rename = "Layers")]
    layers: Vec<String>,
}

impl Image {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Image> {
        let file = File::open(path)?;
        let ar = Archive::new(file);
        Ok(Image { ar })
    }

    /**
     * Unpack FS Layers into specified location
     */
    pub fn unpack_fs_layers<P: AsRef<Path>>(
        &mut self,
        extract_path: P,
        tmp_path: P,
    ) -> io::Result<()> {
        let _pushd = workdir::pushd(tmp_path);

        self.ar.unpack(".")?;

        // XXX Assuming only single element
        let mut vm: Vec<Manifest> = serde_json::from_reader(File::open("manifest.json")?)?;
        let m = vm
            .pop()
            .ok_or(io::Error::new(io::ErrorKind::Other, "Empty manifest"))?;

        for l in m.layers {
            info!("[Image] Unpacking layer {}", l);
            let lar_tgz = File::open(l)?;
            let lar_tar = GzDecoder::new(lar_tgz);
            let mut lar = Archive::new(lar_tar);
            lar.unpack(extract_path.as_ref())?;
        }

        Ok(())
    }
}
