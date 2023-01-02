use sha2::{Digest, Sha256};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use tar::Archive;

use super::consult::{consult, Value};

fn unpack<R, P>(mut reader: R, output: P) -> Option<()>
where
    P: AsRef<Path>,
    R: Read + Seek,
{
    let mut hasher = Sha256::new();
    io::copy(&mut reader, &mut hasher).ok()?;
    let _outer_checksum = hasher.finalize();

    reader.seek(SeekFrom::Start(0)).ok()?;
    let mut archive = Archive::new(reader);

    // let required_files = &["VERSION", "CHECKSUM", "metadata.config", "contents.tar.gz"];

    for mut entry in archive.entries_with_seek().ok()?.filter_map(|e| e.ok()) {
        let path = entry.path().ok()?.as_os_str().to_str()?.to_owned();

        if path == "VERSION" {
            let mut buf = [0];
            entry.read_exact(&mut buf).ok()?;

            if buf[0] != b'3' {
                // Unsupported version
                return None;
            }
        }

        if path == "CHECKSUM" {
            // Inner checksum, deprecated, so we skip its processing
        }

        if path == "metadata.config" {
            let s = io::read_to_string(entry).ok()?;
            let _metadata = consult(s).ok()?;
        }

        if path == "contents.tar.gz" {
        }
    }

    Some(())
}
