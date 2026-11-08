use crate::definitions::Definitions;
use crate::persistence::Session;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

const MAGIC: &[u8; 8] = b"CELLLAB\0";
/// Version du format : un en-tête (magie, version, empreinte du catalogue, longueur) puis le corps de
/// la session, qui contient tout l'état (organismes et leurs génomes, monde, eau, historique).
const VERSION: u32 = 1;
const MAX_BYTES: usize = 256 * 1024 * 1024;

pub(crate) fn save(session: &Session, path: &Path) -> Result<(), String> {
    session.check(session.simulation().definitions())?;
    let configuration = bincode_next::config::standard().with_limit::<MAX_BYTES>();
    let payload = bincode_next::serde::encode_to_vec(session, configuration)
        .map_err(|e| e.to_string())?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(".{}.tmp", Uuid::now_v7()));
    let result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        file.write_all(MAGIC)
            .and_then(|_| file.write_all(&VERSION.to_le_bytes()))
            .and_then(|_| {
                file.write_all(
                    &session
                        .simulation()
                        .definitions()
                        .fingerprint()
                        .map_err(std::io::Error::other)?
                        .to_le_bytes(),
                )
            })
            .and_then(|_| file.write_all(&(payload.len() as u64).to_le_bytes()))
            .and_then(|_| file.write_all(&payload))
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&temp, path).map_err(|e| e.to_string())?;
        File::open(parent)
            .and_then(|dir| dir.sync_all())
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

pub(crate) fn load(path: &Path, definitions: Arc<Definitions>) -> Result<Session, String> {
    definitions.validate()?;
    let mut file = File::open(path).map_err(|e| format!("{} : {e}", path.display()))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES as u64 + 64 {
        return Err("Sauvegarde trop grande.".into());
    }
    let mut header = [0u8; 28];
    file.read_exact(&mut header)
        .map_err(|e| format!("En-tête incomplet : {e}"))?;
    if &header[..8] != MAGIC || u32::from_le_bytes(header[8..12].try_into().unwrap()) != VERSION {
        return Err("Format BIN ou version non pris en charge.".into());
    }
    if u64::from_le_bytes(header[12..20].try_into().unwrap()) != definitions.fingerprint()? {
        return Err("Le catalogue RON diffère de celui de cette sauvegarde. Rétablissez ses définitions avant de charger.".into());
    }
    let length = u64::from_le_bytes(header[20..28].try_into().unwrap());
    if length > MAX_BYTES as u64 {
        return Err("Longueur BIN invalide.".into());
    }
    let mut payload = Vec::new();
    Read::by_ref(&mut file)
        .take(length)
        .read_to_end(&mut payload)
        .map_err(|e| e.to_string())?;
    if payload.len() as u64 != length {
        return Err("Sauvegarde tronquée ou données supplémentaires.".into());
    }
    let mut trailing = [0u8; 1];
    if file.read(&mut trailing).map_err(|e| e.to_string())? != 0 {
        return Err("Sauvegarde tronquée ou données supplémentaires.".into());
    }
    let configuration = bincode_next::config::standard().with_limit::<MAX_BYTES>();
    let (mut session, read): (Session, usize) =
        bincode_next::serde::decode_from_slice(&payload, configuration)
            .map_err(|e| format!("Données BIN invalides : {e}"))?;
    if read != payload.len() {
        return Err("Données BIN supplémentaires.".into());
    }
    session.validate(definitions)?;
    Ok(session)
}
