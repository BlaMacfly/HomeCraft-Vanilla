//! Sauvegardes : dump cohérent de `characters` et `realmd` (tables verrouillées le temps du dump),
//! réglages, manifeste avec empreintes, archive zip horodatée dans UserData/Sauvegardes.
//! Restauration : vérification des empreintes, sauvegarde de sécurité automatique, import.

use crate::db::MariaDb;
use crate::paths::Layout;
use crate::util;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackupManifest {
    pub version: u32,
    pub created: String,
    pub label: String,
    pub characters: u64,
    pub accounts: u64,
    pub files: BTreeMap<String, String>, // nom -> sha256
}

#[derive(Clone, Debug)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub manifest: BackupManifest,
    pub size: u64,
}

/// Bases sauvegardées (personnages et comptes) et nom du fichier dans l'archive.
const DUMPS: [(&str, &str); 2] = [(crate::paths::DB_CHARACTERS, "tbccharacters.sql"), (crate::paths::DB_REALMD, "tbcrealmd.sql")];

/// Crée une sauvegarde. `progress` reçoit des messages d'étape.
pub fn create(layout: &Layout, db: &MariaDb, label: &str, progress: &dyn Fn(&str)) -> Result<BackupInfo, String> {
    let dir = layout.backups();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stamp = util::datetime_compact();
    let tmp = dir.join(format!(".tmp-{stamp}"));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let result = (|| {
        let mut files = BTreeMap::new();
        for (dbname, file) in DUMPS {
            progress(&format!("export de {dbname}"));
            let out = tmp.join(file);
            db.dump_database(dbname, &out)?;
            files.insert(file.to_string(), util::sha256_file(&out)?);
        }
        let settings = layout.settings_file();
        if settings.is_file() {
            let out = tmp.join("settings.json");
            std::fs::copy(&settings, &out).map_err(|e| e.to_string())?;
            files.insert("settings.json".into(), util::sha256_file(&out)?);
        }
        let manifest = BackupManifest {
            version: 1,
            created: util::datetime_readable(),
            label: label.to_string(),
            characters: db.characters_count().unwrap_or(0),
            accounts: db.accounts_count().unwrap_or(0),
            files,
        };
        progress("compression");
        let zip_path = dir.join(format!("sauvegarde-{stamp}.zip"));
        let f = std::fs::File::create(&zip_path).map_err(|e| e.to_string())?;
        let mut w = zip::ZipWriter::new(f);
        let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).large_file(true);
        for name in manifest.files.keys() {
            w.start_file(name, opts).map_err(|e| e.to_string())?;
            let mut src = std::fs::File::open(tmp.join(name)).map_err(|e| e.to_string())?;
            std::io::copy(&mut src, &mut w).map_err(|e| e.to_string())?;
        }
        w.start_file("backup.json", opts).map_err(|e| e.to_string())?;
        w.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes()).map_err(|e| e.to_string())?;
        w.finish().map_err(|e| e.to_string())?;
        let size = std::fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0);
        Ok(BackupInfo { path: zip_path, manifest, size })
    })();
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

fn read_manifest(path: &Path) -> Result<BackupManifest, String> {
    if !path.is_file() {
        return Err(format!("archive introuvable ou invalide : {}", path.display()));
    }
    let f = std::fs::File::open(path).map_err(|e| format!("{} : {e}", path.display()))?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    let mut entry = z.by_name("backup.json").map_err(|_| "backup.json absent de l'archive".to_string())?;
    let mut s = String::new();
    entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

pub fn list(layout: &Layout) -> Vec<BackupInfo> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(layout.backups()) {
        for e in rd.filter_map(Result::ok) {
            let p = e.path();
            if p.extension().map(|x| x == "zip").unwrap_or(false) {
                if let Ok(manifest) = read_manifest(&p) {
                    let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                    out.push(BackupInfo { path: p, manifest, size });
                }
            }
        }
    }
    out.sort_by(|a, b| b.manifest.created.cmp(&a.manifest.created));
    out
}

/// Extrait l'archive dans `dest` en vérifiant chaque empreinte. Renvoie le manifeste.
fn extract_verified(path: &Path, dest: &Path) -> Result<BackupManifest, String> {
    let manifest = read_manifest(path)?;
    let f = std::fs::File::open(path).map_err(|e| format!("{} : {e}", path.display()))?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dest).map_err(|e| format!("{} : {e}", dest.display()))?;
    for (name, expected) in &manifest.files {
        let mut entry = z.by_name(name).map_err(|_| format!("{name} absent de l'archive"))?;
        let out = dest.join(name);
        let mut o = std::fs::File::create(&out).map_err(|e| format!("{} : {e}", out.display()))?;
        std::io::copy(&mut entry, &mut o).map_err(|e| e.to_string())?;
        drop(o);
        let actual = util::sha256_file(&out)?;
        if &actual != expected {
            return Err(format!("empreinte incorrecte pour {name} : archive altérée"));
        }
    }
    Ok(manifest)
}

/// Vérifie l'intégrité d'une archive sans l'importer.
pub fn verify(path: &Path, layout: &Layout) -> Result<BackupManifest, String> {
    let tmp = layout.backups().join(format!(".verify-{}", util::datetime_compact()));
    let r = extract_verified(path, &tmp);
    let _ = std::fs::remove_dir_all(&tmp);
    r
}

/// Restaure une sauvegarde : empreintes vérifiées, sauvegarde de sécurité de l'état courant, puis import.
/// Le serveur de jeu doit être arrêté (appelant) ; MariaDB doit être en service.
pub fn restore(layout: &Layout, db: &MariaDb, path: &Path, progress: &dyn Fn(&str)) -> Result<BackupManifest, String> {
    progress("vérification de l'archive");
    let tmp = layout.backups().join(format!(".restore-{}", util::datetime_compact()));
    let result = (|| {
        let manifest = extract_verified(path, &tmp)?;
        progress("sauvegarde de sécurité de l'état actuel");
        create(layout, db, "automatique avant restauration", progress)?;
        for (dbname, file) in DUMPS {
            if manifest.files.contains_key(file) {
                progress(&format!("import de {dbname}"));
                db.import_file(dbname, &tmp.join(file))?;
            }
        }
        if manifest.files.contains_key("settings.json") {
            let _ = std::fs::copy(tmp.join("settings.json"), layout.settings_file());
        }
        Ok(manifest)
    })();
    let _ = std::fs::remove_dir_all(&tmp);
    result
}
