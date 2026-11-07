use crate::definitions::Definitions;
use crate::ecosystem::PopulationHistory;
use crate::hydrology::Hydrology;
use crate::persistence::Session;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

const MAGIC: &[u8; 8] = b"CELLLAB\0";
/// Version 5 : une quatrième section porte l'état du cycle de l'eau (relief, eau, sédiments, air).
/// Version 4 : une troisième section porte les positions fines des organismes dans leur site.
/// Version 3 : le corps de la session est suivi d'une section « historique de population ».
/// La version 2 limitait l'empreinte à la biologie ; la version 1 couvrait le catalogue entier.
/// Les versions 1 et 2 restent lisibles : leur corps est identique et l'historique est alors
/// reconstruit depuis la filiation.
const VERSION: u32 = 5;
const SECTIONED_VERSION: u32 = 3;
const POSITIONED_VERSION: u32 = 4;
const HYDROLOGICAL_VERSION: u32 = 5;
const BIOLOGICAL_FINGERPRINT_VERSION: u32 = 2;
const MAX_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn save(session: &Session, path: &Path) -> Result<(), String> {
    session.check(session.simulation().definitions())?;
    let configuration = bincode_next::config::standard().with_limit::<MAX_BYTES>();
    let payload = bincode_next::serde::encode_to_vec(session, configuration)
        .map_err(|e| e.to_string())?;
    let history = bincode_next::serde::encode_to_vec(session.history(), configuration)
        .map_err(|e| e.to_string())?;
    let positions = bincode_next::serde::encode_to_vec(session.positions(), configuration)
        .map_err(|e| e.to_string())?;
    let hydrology = bincode_next::serde::encode_to_vec(session.simulation().grid().hydrology(), configuration)
        .map_err(|e| e.to_string())?;
    if payload.len() > MAX_BYTES
        || history.len() > MAX_BYTES
        || positions.len() > MAX_BYTES
        || hydrology.len() > MAX_BYTES
    {
        return Err("La sauvegarde dépasse 64 Mio.".into());
    }
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
            .and_then(|_| file.write_all(&(history.len() as u64).to_le_bytes()))
            .and_then(|_| file.write_all(&history))
            .and_then(|_| file.write_all(&(positions.len() as u64).to_le_bytes()))
            .and_then(|_| file.write_all(&positions))
            .and_then(|_| file.write_all(&(hydrology.len() as u64).to_le_bytes()))
            .and_then(|_| file.write_all(&hydrology))
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
    if file.metadata().map_err(|e| e.to_string())?.len() > 4 * MAX_BYTES as u64 + 128 {
        return Err("Sauvegarde trop grande.".into());
    }
    let mut header = [0u8; 28];
    file.read_exact(&mut header)
        .map_err(|e| format!("En-tête incomplet : {e}"))?;
    let version = u32::from_le_bytes(header[8..12].try_into().unwrap());
    if &header[..8] != MAGIC || !(1..=VERSION).contains(&version) {
        return Err("Format BIN ou version non pris en charge.".into());
    }
    let expected = if version >= BIOLOGICAL_FINGERPRINT_VERSION {
        definitions.fingerprint()?
    } else {
        definitions.legacy_fingerprint()?
    };
    if u64::from_le_bytes(header[12..20].try_into().unwrap()) != expected {
        return Err("Le catalogue RON diffère de celui de cette sauvegarde. Rétablissez ses définitions avant de charger.".into());
    }
    let length = u64::from_le_bytes(header[20..28].try_into().unwrap());
    let payload = read_block(&mut file, length)?;
    let history = if version >= SECTIONED_VERSION {
        let mut size = [0u8; 8];
        file.read_exact(&mut size)
            .map_err(|e| format!("Section d'historique incomplète : {e}"))?;
        Some(read_block(&mut file, u64::from_le_bytes(size))?)
    } else {
        None
    };
    let positions = if version >= POSITIONED_VERSION {
        let mut size = [0u8; 8];
        file.read_exact(&mut size)
            .map_err(|e| format!("Section des positions incomplète : {e}"))?;
        Some(read_block(&mut file, u64::from_le_bytes(size))?)
    } else {
        None
    };
    let water = if version >= HYDROLOGICAL_VERSION {
        let mut size = [0u8; 8];
        file.read_exact(&mut size)
            .map_err(|e| format!("Section de l'eau incomplète : {e}"))?;
        Some(read_block(&mut file, u64::from_le_bytes(size))?)
    } else {
        None
    };
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
    match history {
        Some(bytes) => {
            let (history, read): (PopulationHistory, usize) =
                bincode_next::serde::decode_from_slice(&bytes, configuration)
                    .map_err(|e| format!("Historique BIN invalide : {e}"))?;
            if read != bytes.len() {
                return Err("Historique BIN : données supplémentaires.".into());
            }
            session.restore_history(history)?;
        }
        None => session.rebuild_history(),
    }
    match positions {
        Some(bytes) => {
            let (positions, read): (Vec<(Uuid, f32, f32)>, usize) =
                bincode_next::serde::decode_from_slice(&bytes, configuration)
                    .map_err(|e| format!("Positions BIN invalides : {e}"))?;
            if read != bytes.len() {
                return Err("Positions BIN : données supplémentaires.".into());
            }
            session.restore_positions(&positions)?;
        }
        None => session.settle_legacy_positions(),
    }
    match water {
        Some(bytes) => {
            let (hydrology, read): (Hydrology, usize) =
                bincode_next::serde::decode_from_slice(&bytes, configuration)
                    .map_err(|e| format!("Hydrologie BIN invalide : {e}"))?;
            if read != bytes.len() {
                return Err("Hydrologie BIN : données supplémentaires.".into());
            }
            session.restore_hydrology(hydrology)?;
        }
        None => session.settle_legacy_hydrology(),
    }
    Ok(session)
}

/// Lit exactement `length` octets (au plus `MAX_BYTES`) ; un fichier plus court est tronqué.
fn read_block(file: &mut File, length: u64) -> Result<Vec<u8>, String> {
    if length > MAX_BYTES as u64 {
        return Err("Longueur BIN invalide.".into());
    }
    let mut block = Vec::new();
    Read::by_ref(file)
        .take(length)
        .read_to_end(&mut block)
        .map_err(|e| e.to_string())?;
    if block.len() as u64 != length {
        return Err("Sauvegarde tronquée ou données supplémentaires.".into());
    }
    Ok(block)
}
