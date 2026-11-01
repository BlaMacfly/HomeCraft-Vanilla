//! Profils de lancement (UserData/profiles.json) : un bouton par profil sur l'accueil, qui lance le client avec le
//! compte (et, si le joueur l'a enregistré, le mot de passe) prérempli et le personnage préféré présélectionné.
//! Mot de passe : chiffré par DPAPI (`secret`), jamais en clair sur le disque, jamais journalisé, jamais transmis au
//! réseau multijoueur ; déchiffré seulement au moment du lancement, pour le seul processus du client.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub account: String,
    /// Mot de passe chiffré DPAPI (hexadécimal) ; vide = non enregistré (le joueur le saisit dans le client)
    pub password: String,
    /// Personnage préféré (présélectionné ; vide = dernier utilisé)
    pub character: String,
    /// Entrer directement en jeu avec ce personnage (client Benilla)
    pub enter_world: bool,
    /// Arguments supplémentaires de l'exécutable du client
    pub extra_args: String,
}

impl Profile {
    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }
    pub fn set_password(&mut self, plain: &str) -> Result<(), String> {
        self.password = crate::secret::protect(plain)?;
        Ok(())
    }
    pub fn password_plain(&self) -> Result<Option<String>, String> {
        if self.password.is_empty() {
            return Ok(None);
        }
        crate::secret::unprotect(&self.password).map(Some)
    }
    /// Arguments supplémentaires découpés (guillemets doubles acceptés)
    pub fn args(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = String::new();
        let mut quoted = false;
        for c in self.extra_args.chars() {
            match c {
                '"' => quoted = !quoted,
                c if c.is_whitespace() && !quoted => {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                }
                c => cur.push(c),
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        out
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Profiles {
    pub default_id: String,
    pub list: Vec<Profile>,
}

impl Profiles {
    pub fn path(user_data: &Path) -> PathBuf {
        user_data.join("profiles.json")
    }
    pub fn load(user_data: &Path) -> Profiles {
        std::fs::read_to_string(Self::path(user_data)).ok().and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
    }
    pub fn save(&self, user_data: &Path) -> Result<(), String> {
        std::fs::create_dir_all(user_data).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(user_data), serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }
    pub fn get(&self, id: &str) -> Option<&Profile> {
        self.list.iter().find(|p| p.id == id)
    }
    pub fn new_id() -> String {
        format!("p{}", crate::secret::random_token(10).to_ascii_lowercase())
    }
    /// Ajoute ou remplace (même identifiant) ; le premier profil devient le profil par défaut.
    pub fn upsert(&mut self, p: Profile) {
        match self.list.iter_mut().find(|x| x.id == p.id) {
            Some(x) => *x = p,
            None => self.list.push(p),
        }
        if self.default_id.is_empty() || self.get(&self.default_id.clone()).is_none() {
            self.default_id = self.list.first().map(|p| p.id.clone()).unwrap_or_default();
        }
    }
    pub fn remove(&mut self, id: &str) {
        self.list.retain(|p| p.id != id);
        if self.default_id == id {
            self.default_id = self.list.first().map(|p| p.id.clone()).unwrap_or_default();
        }
    }
}

/// Nom de compte valide pour le serveur (lettres, chiffres, 32 caractères au plus).
pub fn valid_account(a: &str) -> bool {
    !a.is_empty() && a.len() <= 32 && a.chars().all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mot_de_passe_jamais_en_clair() {
        let dir = std::env::temp_dir().join(format!("hc-profiles-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut ps = Profiles::default();
        let mut p = Profile { id: Profiles::new_id(), name: "Principal".into(), account: "JOUEUR".into(), character: "Taelia".into(), ..Default::default() };
        p.set_password("MotDePasseSecret42").unwrap();
        ps.upsert(p.clone());
        ps.save(&dir).unwrap();
        let raw = std::fs::read_to_string(Profiles::path(&dir)).unwrap();
        assert!(!raw.contains("MotDePasseSecret42"));
        let back = Profiles::load(&dir);
        assert_eq!(back.default_id, p.id);
        assert_eq!(back.get(&p.id).unwrap().password_plain().unwrap().as_deref(), Some("MotDePasseSecret42"));
        let mut back = back;
        back.remove(&p.id);
        assert!(back.list.is_empty() && back.default_id.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn arguments() {
        let p = Profile { extra_args: "-console \"un dossier\"  -x".into(), ..Default::default() };
        assert_eq!(p.args(), vec!["-console", "un dossier", "-x"]);
    }

    #[test]
    fn comptes() {
        assert!(valid_account("BLAMACFLY2"));
        assert!(!valid_account("a b"));
        assert!(!valid_account(""));
    }
}
