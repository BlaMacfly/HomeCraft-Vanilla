//! Construction d'une archive MPQ (format V1, celui du client 1.12.1) à partir d'un dossier : les projets de
//! patchs (ex. VanillaHD) sont distribués sous forme d'arborescences de fichiers, pas d'archives.
//! Les chemins internes utilisent le séparateur « \ » attendu par le client. Les fichiers de service des dépôts
//! ((listfile), (attributes), .patchignore, .git*, README, LICENSE) sont ignorés : la liste est régénérée.

use std::path::{Path, PathBuf};
use wow_mpq::{compression, ArchiveBuilder, FormatVersion, ListfileOption};

fn is_ignored(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with('(') && lower.ends_with(')')
        || lower.starts_with(".git")
        || lower == ".patchignore"
        || lower.starts_with("readme")
        || lower.starts_with("license")
        || lower == "thumbs.db"
        || lower == "desktop.ini"
}

fn collect(dir: &Path, base: &Path, out: &mut Vec<(PathBuf, String)>) -> Result<(), String> {
    for e in std::fs::read_dir(dir).map_err(|e| format!("{} : {e}", dir.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if name.starts_with('.') {
                continue;
            }
            collect(&p, base, out)?;
        } else if !is_ignored(&name) {
            let rel = p.strip_prefix(base).map_err(|e| e.to_string())?;
            let archive_name = rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("\\");
            out.push((p, archive_name));
        }
    }
    Ok(())
}

/// Si le dossier ne contient qu'un sous-dossier (archive GitHub « Patch-X-main »), on descend dedans.
fn effective_root(src: &Path) -> PathBuf {
    if let Ok(rd) = std::fs::read_dir(src) {
        let entries: Vec<_> = rd.filter_map(Result::ok).filter(|e| !e.file_name().to_string_lossy().starts_with('.')).collect();
        if entries.len() == 1 && entries[0].path().is_dir() {
            return entries[0].path();
        }
    }
    src.to_path_buf()
}

/// Construit `out` depuis `src`. Renvoie le nombre de fichiers archivés.
pub fn build_from_dir(src: &Path, out: &Path, progress: &dyn Fn(usize, usize)) -> Result<usize, String> {
    let root = effective_root(src);
    let mut files = Vec::new();
    collect(&root, &root, &mut files)?;
    if files.is_empty() {
        return Err("dossier vide".into());
    }
    if files.len() > 65_000 {
        return Err(format!("{} fichiers : au-delà de la capacité d'une archive MPQ V1 (65 536)", files.len()));
    }
    let total = files.len();
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .block_size(3) // secteurs de 4 Ko, comme les archives Blizzard
        .default_compression(compression::flags::ZLIB)
        .listfile_option(ListfileOption::Generate);
    for (i, (path, name)) in files.iter().enumerate() {
        progress(i + 1, total);
        builder = builder.add_file(path, name);
    }
    if out.exists() {
        std::fs::remove_file(out).map_err(|e| format!("{} : {e}", out.display()))?;
    }
    builder.build(out).map_err(|e| format!("construction MPQ : {e}"))?;
    Ok(total)
}

/// Lecture de contrôle : nombre de fichiers listés et taille de l'archive.
pub fn inspect(path: &Path) -> Result<(usize, u64), String> {
    let mut a = wow_mpq::Archive::open(path).map_err(|e| format!("ouverture : {e}"))?;
    let list = a.list().map_err(|e| format!("liste : {e}"))?;
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    Ok((list.len(), size))
}
