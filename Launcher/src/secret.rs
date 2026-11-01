//! Secrets locaux (clés d'API, mots de passe des profils, secret du réseau hôte) chiffrés par DPAPI
//! (CryptProtectData, portée utilisateur Windows) : illisibles par un autre compte ou sur une autre machine,
//! et jamais écrits en clair. Stockés en hexadécimal dans les fichiers de UserData.

use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData};

/// Entropie propre à HomeCraft : un secret chiffré par une autre application avec DPAPI ne se déchiffre pas ici.
const ENTROPY: &[u8] = b"HomeCraft Edition Vanilla / secret local v1";

fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 }
}

fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    let v = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize) }.to_vec();
    unsafe { LocalFree(out.pbData as _) };
    v
}

pub fn protect(plain: &str) -> Result<String, String> {
    if plain.is_empty() {
        return Ok(String::new());
    }
    let input = blob(plain.as_bytes());
    let entropy = blob(ENTROPY);
    let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    let ok = unsafe { CryptProtectData(&input, std::ptr::null(), &entropy, std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut out) };
    if ok == 0 {
        return Err(format!("chiffrement DPAPI impossible ({})", std::io::Error::last_os_error()));
    }
    Ok(to_hex(&take(out)))
}

pub fn unprotect(hex: &str) -> Result<String, String> {
    if hex.is_empty() {
        return Ok(String::new());
    }
    let data = from_hex(hex).ok_or("secret enregistré illisible (format)")?;
    let input = blob(&data);
    let entropy = blob(ENTROPY);
    let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    let ok = unsafe { CryptUnprotectData(&input, std::ptr::null_mut(), &entropy, std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut out) };
    if ok == 0 {
        return Err("secret enregistré illisible (chiffré par un autre compte Windows ou une autre machine)".into());
    }
    String::from_utf8(take(out)).map_err(|_| "secret enregistré illisible (encodage)".into())
}

fn to_hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

/// Jeton aléatoire alphanumérique (secrets éphémères : réseau, pont IA).
pub fn random_token(len: usize) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    (0..len).map(|_| ALPHABET[(rand::random::<u32>() as usize) % ALPHABET.len()] as char).collect()
}

/// Masque un secret pour l'affichage (jamais dans les journaux) : « sk-…a1b2 ».
pub fn masked(s: &str) -> String {
    let n = s.chars().count();
    if n == 0 {
        return String::new();
    }
    if n <= 8 {
        return "••••".into();
    }
    let head: String = s.chars().take(3).collect();
    let tail: String = s.chars().skip(n - 4).collect();
    format!("{head}…{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour_dpapi() {
        let c = protect("sk-test-0123456789").unwrap();
        assert!(!c.contains("sk-test"));
        assert_eq!(unprotect(&c).unwrap(), "sk-test-0123456789");
        assert_eq!(protect("").unwrap(), "");
        assert!(unprotect("zz").is_err());
    }

    #[test]
    fn masquage() {
        assert_eq!(masked("sk-abcdefghij1234"), "sk-…1234");
        assert_eq!(masked("court"), "••••");
    }
}
