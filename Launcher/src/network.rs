//! HomeCraft Multijoueur : réseau privé entre un hôte et ses invités, entièrement dans le launcher (aucun programme
//! externe).
//!
//! Principe : l'hôte écoute sur un port TCP (11010 par défaut, ouvert automatiquement sur la box par UPnP si possible).
//! Chaque invitation contient un identifiant et une clé aléatoires propres à un joueur (32 octets), une date
//! d'expiration et les adresses de l'hôte ; rien de persistant : la clé n'existe que le temps de la partie, chez l'hôte
//! et dans le code transmis à ce joueur, et l'hôte peut la révoquer à tout moment.
//! Chaque connexion de l'invité (authentification 3724, monde 8085, canal de présence) ouvre une connexion TCP vers
//! l'hôte, authentifiée et chiffrée par le protocole Noise `NNpsk0` (X25519, ChaCha20-Poly1305, BLAKE2s : échange de
//! clés éphémères, donc confidentialité persistante, et clé de l'invitation obligatoire des deux côtés). L'en-tête en
//! clair (identifiant d'invitation, type de connexion) est lié au chiffrement (prologue) : le modifier fait échouer
//! l'échange. Côté hôte, un type de connexion ne mène qu'à 127.0.0.1:3724 ou 127.0.0.1:8085 : MariaDB, la console et
//! tout autre port local sont injoignables par construction. Côté invité, le launcher écoute sur 127.0.0.1:3724 et
//! 127.0.0.1:8085 : le client WoW se connecte « en local » sans modification ; rien de la machine de l'invité n'est
//! exposé au réseau.
//! Aucune connexion directe n'est garantie : si la box ou le pare-feu de l'hôte bloque le port et que l'UPnP est
//! indisponible, l'invité reste « à la recherche de l'hôte » et le launcher le dit (redirection manuelle du port).

use crate::paths::{Layout, REALMD_PORT, WORLD_PORT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const MAGIC: &[u8; 4] = b"HCL1";
const NOISE: &str = "Noise_NNpsk0_25519_ChaChaPoly_BLAKE2s";
const CODE_PREFIX: &str = "HC2-";
/// Taille maximale d'un message Noise (64 Kio), étiquette d'authentification comprise
const MAX_MSG: usize = 65_535;
const CHUNK: usize = MAX_MSG - 16;
/// Connexions simultanées au plus par invitation (un client WoW en ouvre deux, plus la présence)
const MAX_CONNS_PER_INVITE: usize = 12;

/// Mode invité actif : le serveur local ne doit pas démarrer (ports 3724/8085 occupés par les redirections).
static GUEST_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn guest_active() -> bool {
    GUEST_ACTIVE.load(Ordering::SeqCst)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Kind {
    Control = 0,
    Realm = 1,
    World = 2,
}

impl Kind {
    fn from(b: u8) -> Option<Kind> {
        match b {
            0 => Some(Kind::Control),
            1 => Some(Kind::Realm),
            2 => Some(Kind::World),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------------------------ réglages

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct NetSettings {
    /// Nom affiché aux autres joueurs
    pub player_name: String,
    /// Port d'écoute de l'hôte (TCP)
    pub listen_port: u16,
    /// Adresse publique ou nom DNS annoncé dans les invitations (vide : adresse donnée par la box via UPnP)
    pub public_address: String,
    /// Durée de validité d'une invitation, en heures
    pub invite_hours: u32,
    /// Ouverture automatique du port sur la box (UPnP)
    pub upnp: bool,
    /// Ports locaux de l'invité (3724/8085 ; autres valeurs : tests sur une seule machine)
    pub guest_realm_port: u16,
    pub guest_world_port: u16,
    /// Tests : l'invité n'essaie pas les adresses TCP directes (vérifie la liaison par perçage de NAT)
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub quic_only: bool,
}

impl Default for NetSettings {
    fn default() -> Self {
        NetSettings { player_name: String::new(), listen_port: 11_010, public_address: String::new(), invite_hours: 24, upnp: true, guest_realm_port: REALMD_PORT, guest_world_port: WORLD_PORT, quic_only: false }
    }
}

impl NetSettings {
    pub fn path(layout: &Layout) -> PathBuf {
        layout.user_data().join("network.json")
    }
    pub fn load(layout: &Layout) -> NetSettings {
        std::fs::read_to_string(Self::path(layout)).ok().and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
    }
    pub fn save(&self, layout: &Layout) -> Result<(), String> {
        std::fs::create_dir_all(layout.user_data()).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(layout), serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }
    fn display_name(&self) -> String {
        clean_name(&self.player_name, "Joueur-HomeCraft")
    }
}

fn clean_name(s: &str, default: &str) -> String {
    let n: String = s.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ').take(24).collect();
    let n = n.trim().to_string();
    if n.is_empty() { default.into() } else { n }
}

// ------------------------------------------------------------------------------------------------ invitation

/// Contenu d'un code d'invitation « HC2-… ». `k` est la clé propre à ce joueur pour cette partie, jamais réutilisée.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Invite {
    pub v: u8,
    /// identifiant de l'invitation (16 octets, hexadécimal)
    pub i: String,
    /// clé de l'invitation (32 octets, hexadécimal)
    pub k: String,
    /// adresses de l'hôte (hôte:port)
    pub p: Vec<String>,
    /// expiration (secondes Unix)
    pub x: u64,
    /// nom de l'hôte
    pub h: String,
    /// adresses UDP de l'hôte pour le perçage de NAT (publique selon STUN, réseau local)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub q: Vec<String>,
    /// empreinte SHA-256 du certificat QUIC de l'hôte (hexadécimal)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub f: String,
}

impl Invite {
    pub fn encode(&self) -> String {
        format!("{CODE_PREFIX}{}", b64url_encode(serde_json::to_string(self).unwrap().as_bytes()))
    }
    pub fn decode(code: &str) -> Result<Invite, String> {
        let body: String = code.trim().chars().filter(|c| !c.is_whitespace()).collect();
        let body = body.strip_prefix(CODE_PREFIX).ok_or("code d'invitation non reconnu (il commence par HC2-)")?;
        let raw = b64url_decode(body).ok_or("code d'invitation incomplet ou altéré")?;
        let inv: Invite = serde_json::from_slice(&raw).map_err(|_| "code d'invitation incomplet ou altéré")?;
        if inv.v != 2 || inv.id().is_none() || inv.key().is_none() || inv.p.is_empty() {
            return Err("code d'invitation incomplet ou altéré".into());
        }
        if inv.x <= now_unix() {
            return Err("cette invitation a expiré : demandez-en une nouvelle à l'hôte".into());
        }
        Ok(inv)
    }
    fn id(&self) -> Option<[u8; 16]> {
        from_hex(&self.i).and_then(|v| v.try_into().ok())
    }
    fn key(&self) -> Option<[u8; 32]> {
        from_hex(&self.k).and_then(|v| v.try_into().ok())
    }
}

/// Code de réponse de l'invité « HCR-… » : ses adresses UDP, signées avec la clé de son invitation (HMAC-SHA256).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Reply {
    pub i: String,
    pub c: Vec<String>,
    pub m: String,
}

impl Reply {
    fn mac(i: &str, c: &[String], key: &[u8; 32]) -> String {
        to_hex(&crate::nat::hmac_sha256(key, format!("HCR|{i}|{}", c.join(",")).as_bytes()))
    }
    pub fn new(i: &str, c: Vec<String>, key: &[u8; 32]) -> Reply {
        let m = Self::mac(i, &c, key);
        Reply { i: i.to_string(), c, m }
    }
    pub fn verify(&self, key: &[u8; 32]) -> bool {
        Self::mac(&self.i, &self.c, key) == self.m
    }
    pub fn encode(&self) -> String {
        format!("HCR-{}", b64url_encode(serde_json::to_string(self).unwrap().as_bytes()))
    }
    pub fn decode(code: &str) -> Result<Reply, String> {
        let body: String = code.trim().chars().filter(|c| !c.is_whitespace()).collect();
        let body = body.strip_prefix("HCR-").ok_or("code de réponse non reconnu (il commence par HCR-)")?;
        let raw = b64url_decode(body).ok_or("code de réponse incomplet ou altéré")?;
        serde_json::from_slice(&raw).map_err(|_| "code de réponse incomplet ou altéré".to_string())
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn b64url_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        for i in 0..chunk.len() + 1 {
            out.push(B64[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    out
}

fn b64url_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let vals: Vec<u32> = s.bytes().map(|c| B64.iter().position(|&x| x == c).map(|p| p as u32)).collect::<Option<_>>()?;
    for chunk in vals.chunks(4) {
        if chunk.len() == 1 {
            return None;
        }
        let mut n = 0u32;
        for (i, v) in chunk.iter().enumerate() {
            n |= v << (18 - 6 * i);
        }
        for i in 0..chunk.len() - 1 {
            out.push((n >> (16 - 8 * i)) as u8);
        }
    }
    Some(out)
}

fn to_hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

// ------------------------------------------------------------------------------------------------ canal chiffré

fn write_frame(s: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    let mut b = Vec::with_capacity(data.len() + 2);
    b.extend_from_slice(&(data.len() as u16).to_be_bytes());
    b.extend_from_slice(data);
    s.write_all(&b)
}

fn read_frame(s: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut l = [0u8; 2];
    s.read_exact(&mut l)?;
    let mut b = vec![0u8; u16::from_be_bytes(l) as usize];
    s.read_exact(&mut b)?;
    Ok(b)
}

fn prologue(id: &[u8; 16], kind: Kind) -> Vec<u8> {
    let mut p = MAGIC.to_vec();
    p.extend_from_slice(id);
    p.push(kind as u8);
    p
}

/// Échange Noise côté invité (initiateur) : en-tête en clair, puis deux messages.
fn handshake_initiator(s: &mut TcpStream, id: &[u8; 16], psk: &[u8; 32], kind: Kind) -> Result<snow::TransportState, String> {
    let pro = prologue(id, kind);
    let mut hs = snow::Builder::new(NOISE.parse().unwrap()).psk(0, psk).and_then(|b| b.prologue(&pro)).and_then(|b| b.build_initiator()).map_err(|e| e.to_string())?;
    s.write_all(&pro).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; MAX_MSG];
    let n = hs.write_message(&[], &mut buf).map_err(|e| e.to_string())?;
    write_frame(s, &buf[..n]).map_err(|e| e.to_string())?;
    let m = read_frame(s).map_err(|_| "refusé par l'hôte".to_string())?;
    hs.read_message(&m, &mut buf).map_err(|_| "refusé par l'hôte".to_string())?;
    hs.into_transport_mode().map_err(|e| e.to_string())
}

/// Échange Noise côté hôte (répondeur), clé de l'invitation déjà retrouvée par son identifiant.
fn handshake_responder(s: &mut TcpStream, pro: &[u8], psk: &[u8; 32]) -> Result<snow::TransportState, String> {
    let mut hs = snow::Builder::new(NOISE.parse().unwrap()).psk(0, psk).and_then(|b| b.prologue(pro)).and_then(|b| b.build_responder()).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; MAX_MSG];
    let m = read_frame(s).map_err(|e| e.to_string())?;
    hs.read_message(&m, &mut buf).map_err(|_| "clé d'invitation fausse".to_string())?;
    let n = hs.write_message(&[], &mut buf).map_err(|e| e.to_string())?;
    write_frame(s, &buf[..n]).map_err(|e| e.to_string())?;
    hs.into_transport_mode().map_err(|e| e.to_string())
}

/// Relaie une connexion locale en clair et une connexion chiffrée, dans les deux sens, jusqu'à la fermeture de l'une.
fn pipe(enc: TcpStream, plain: TcpStream, t: snow::TransportState) {
    let t = Arc::new(Mutex::new(t));
    let (mut enc_r, mut enc_w) = (enc.try_clone().unwrap(), enc);
    let (mut plain_r, mut plain_w) = (plain.try_clone().unwrap(), plain);
    let t2 = t.clone();
    let (close_a, close_b) = (enc_w.try_clone().unwrap(), plain_w.try_clone().unwrap());
    let up = std::thread::spawn(move || {
        let mut buf = vec![0u8; CHUNK];
        let mut out = vec![0u8; MAX_MSG];
        loop {
            let n = match plain_r.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            let len = match t2.lock().unwrap().write_message(&buf[..n], &mut out) {
                Ok(l) => l,
                Err(_) => break,
            };
            if write_frame(&mut enc_w, &out[..len]).is_err() {
                break;
            }
        }
        let _ = close_a.shutdown(Shutdown::Both);
        let _ = close_b.shutdown(Shutdown::Both);
    });
    let mut out = vec![0u8; MAX_MSG];
    loop {
        let m = match read_frame(&mut enc_r) {
            Ok(m) => m,
            Err(_) => break,
        };
        let len = match t.lock().unwrap().read_message(&m, &mut out) {
            Ok(l) => l,
            Err(_) => break,
        };
        if plain_w.write_all(&out[..len]).is_err() {
            break;
        }
    }
    let _ = enc_r.shutdown(Shutdown::Both);
    let _ = plain_w.shutdown(Shutdown::Both);
    let _ = up.join();
}

fn send_msg(s: &mut TcpStream, t: &Mutex<snow::TransportState>, v: &serde_json::Value) -> std::io::Result<()> {
    let mut out = vec![0u8; MAX_MSG];
    let len = t.lock().unwrap().write_message(v.to_string().as_bytes(), &mut out).map_err(|e| std::io::Error::other(e.to_string()))?;
    write_frame(s, &out[..len])
}

fn recv_msg(s: &mut TcpStream, t: &Mutex<snow::TransportState>) -> std::io::Result<serde_json::Value> {
    let m = read_frame(s)?;
    let mut out = vec![0u8; MAX_MSG];
    let len = t.lock().unwrap().read_message(&m, &mut out).map_err(|e| std::io::Error::other(e.to_string()))?;
    serde_json::from_slice(&out[..len]).map_err(|e| std::io::Error::other(e.to_string()))
}

// ------------------------------------------------------------------------------------------------ hôte

#[derive(Clone)]
struct HostInvite {
    id: [u8; 16],
    psk: [u8; 32],
    label: String,
    expires: u64,
    revoked: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Peer {
    pub name: String,
    pub invite: String,
    pub address: String,
    pub latency_ms: Option<f32>,
    pub local: bool,
}

struct HostShared {
    invites: Mutex<Vec<HostInvite>>,
    /// Connexions ouvertes par invitation (pour les couper à la révocation)
    conns: Mutex<Vec<(u64, [u8; 16], TcpStream)>>,
    next_conn: AtomicU64,
    /// Présence : identifiant d'invitation -> (nom, adresse, latence, dernière nouvelle)
    peers: Mutex<HashMap<[u8; 16], (String, String, Option<f32>, Instant)>>,
    stop: AtomicBool,
    /// Ports locaux atteignables : authentification, monde (tests : autres ports)
    targets: (u16, u16),
    log: Box<dyn Fn(String) + Send + Sync>,
}

pub struct Host {
    shared: Arc<HostShared>,
    pub port: u16,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Host {
    pub fn start(port: u16, targets: (u16, u16), log: Box<dyn Fn(String) + Send + Sync>) -> Result<Host, String> {
        let listener = TcpListener::bind(("0.0.0.0", port)).map_err(|e| format!("port {port} indisponible : {e}"))?;
        let port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let shared = Arc::new(HostShared { invites: Mutex::new(Vec::new()), conns: Mutex::new(Vec::new()), peers: Mutex::new(HashMap::new()), stop: AtomicBool::new(false), targets, log, next_conn: AtomicU64::new(1) });
        let sh = shared.clone();
        let thread = std::thread::Builder::new()
            .name("hote-reseau".into())
            .spawn(move || {
                while !sh.stop.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok((s, addr)) => {
                            let sh2 = sh.clone();
                            std::thread::spawn(move || host_conn(sh2, s, addr));
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(50)),
                        Err(_) => std::thread::sleep(Duration::from_millis(200)),
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Host { shared, port, thread: Some(thread) })
    }

    /// Nouvelle invitation : identifiant et clé aléatoires, propres à ce joueur.
    pub fn invite(&self, label: &str, ttl_secs: u64, endpoints: Vec<String>, host_name: &str) -> (String, Invite) {
        let id: [u8; 16] = rand::random();
        let psk: [u8; 32] = rand::random();
        let expires = now_unix() + ttl_secs;
        self.shared.invites.lock().unwrap().push(HostInvite { id, psk, label: label.to_string(), expires, revoked: false });
        (to_hex(&id), Invite { v: 2, i: to_hex(&id), k: to_hex(&psk), p: endpoints, x: expires, h: host_name.to_string(), q: Vec::new(), f: String::new() })
    }

    /// Révocation : la clé est effacée, les connexions ouvertes avec elle sont coupées tout de suite.
    pub fn revoke(&self, id_hex: &str) -> bool {
        let Some(id) = from_hex(id_hex).and_then(|v| <[u8; 16]>::try_from(v).ok()) else { return false };
        let mut found = false;
        for inv in self.shared.invites.lock().unwrap().iter_mut().filter(|i| i.id == id) {
            inv.revoked = true;
            inv.psk = [0u8; 32];
            found = true;
        }
        self.shared.conns.lock().unwrap().retain(|(_, i, s)| {
            if *i == id {
                let _ = s.shutdown(Shutdown::Both);
                false
            } else {
                true
            }
        });
        self.shared.peers.lock().unwrap().remove(&id);
        found
    }

    /// Clé d'une invitation encore valide (ni révoquée ni expirée)
    pub fn psk(&self, id_hex: &str) -> Option<([u8; 32], String)> {
        let id = from_hex(id_hex).and_then(|v| <[u8; 16]>::try_from(v).ok())?;
        self.shared.invites.lock().unwrap().iter().find(|i| i.id == id && !i.revoked && i.expires > now_unix()).map(|i| (i.psk, i.label.clone()))
    }

    pub fn peers(&self) -> Vec<Peer> {
        let invites = self.shared.invites.lock().unwrap().clone();
        self.shared
            .peers
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, p)| p.3.elapsed() < Duration::from_secs(15))
            .map(|(id, p)| Peer { name: p.0.clone(), address: p.1.clone(), latency_ms: p.2, invite: invites.iter().find(|i| i.id == *id).map(|i| i.label.clone()).unwrap_or_default(), local: false })
            .collect()
    }

    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        for (_, _, s) in self.shared.conns.lock().unwrap().drain(..) {
            let _ = s.shutdown(Shutdown::Both);
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.stop();
    }
}

fn host_conn(sh: Arc<HostShared>, mut s: TcpStream, addr: SocketAddr) {
    let _ = s.set_nonblocking(false);
    let _ = s.set_nodelay(true);
    let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
    let mut head = [0u8; 21];
    if s.read_exact(&mut head).is_err() || &head[..4] != MAGIC {
        return;
    }
    let id: [u8; 16] = head[4..20].try_into().unwrap();
    let Some(kind) = Kind::from(head[20]) else {
        (sh.log)(format!("connexion refusée de {addr} : type de connexion inconnu"));
        return;
    };
    let inv = sh.invites.lock().unwrap().iter().find(|i| i.id == id).cloned();
    let inv = match inv {
        Some(i) if i.revoked => {
            (sh.log)(format!("connexion refusée de {addr} : invitation révoquée ({})", i.label));
            return;
        }
        Some(i) if i.expires <= now_unix() => {
            (sh.log)(format!("connexion refusée de {addr} : invitation expirée ({})", i.label));
            return;
        }
        Some(i) => i,
        None => {
            (sh.log)(format!("connexion refusée de {addr} : invitation inconnue"));
            return;
        }
    };
    if sh.conns.lock().unwrap().iter().filter(|(_, i, _)| *i == id).count() >= MAX_CONNS_PER_INVITE {
        (sh.log)(format!("connexion refusée de {addr} : trop de connexions pour l'invitation {}", inv.label));
        return;
    }
    let t = match handshake_responder(&mut s, &head, &inv.psk) {
        Ok(t) => t,
        Err(e) => {
            (sh.log)(format!("connexion refusée de {addr} : {e} ({})", inv.label));
            return;
        }
    };
    let _ = s.set_read_timeout(None);
    let seq = sh.next_conn.fetch_add(1, Ordering::SeqCst);
    if let Ok(c) = s.try_clone() {
        sh.conns.lock().unwrap().push((seq, id, c));
    }
    match kind {
        Kind::Control => {
            let _ = s.set_read_timeout(Some(Duration::from_secs(20)));
            let t = Mutex::new(t);
            while !sh.stop.load(Ordering::SeqCst) {
                let Ok(m) = recv_msg(&mut s, &t) else { break };
                let name = clean_name(m["name"].as_str().unwrap_or(""), &inv.label);
                let rtt = m["rtt"].as_f64().map(|r| r as f32);
                sh.peers.lock().unwrap().insert(id, (name, addr.ip().to_string(), rtt, Instant::now()));
                if send_msg(&mut s, &t, &serde_json::json!({"t": "pong", "n": m["n"]})).is_err() {
                    break;
                }
            }
            sh.peers.lock().unwrap().remove(&id);
        }
        Kind::Realm | Kind::World => {
            let port = if kind == Kind::Realm { sh.targets.0 } else { sh.targets.1 };
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(local) => {
                    let _ = local.set_nodelay(true);
                    if let Ok(c) = s.try_clone() {
                        pipe(c, local, t);
                    }
                }
                Err(e) => (sh.log)(format!("serveur local injoignable (127.0.0.1:{port}) : {e}")),
            }
        }
    }
    let _ = s.shutdown(Shutdown::Both);
    sh.conns.lock().unwrap().retain(|(n, _, _)| *n != seq);
}

// ------------------------------------------------------------------------------------------------ invité

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum GuestLink {
    #[default]
    Searching,
    Connected,
    /// L'hôte refuse l'invitation (révoquée, expirée, clé fausse)
    Refused,
}

struct GuestShared {
    id: [u8; 16],
    psk: [u8; 32],
    endpoints: Vec<String>,
    /// Dernière adresse de l'hôte qui a répondu
    preferred: Mutex<Option<SocketAddr>>,
    stop: AtomicBool,
    link: Mutex<GuestLink>,
    rtt_ms: AtomicU64,
    name: String,
    log: Box<dyn Fn(String) + Send + Sync>,
    /// Pont local vers la liaison QUIC (perçage de NAT), quand elle est établie
    bridge: Option<Arc<Mutex<Option<SocketAddr>>>>,
    /// Tests : pas d'adresse TCP directe
    quic_only: bool,
}

pub struct Guest {
    shared: Arc<GuestShared>,
    threads: Vec<std::thread::JoinHandle<()>>,
    pub host_name: String,
}

impl Guest {
    pub fn start(inv: &Invite, local_ports: (u16, u16), name: &str, log: Box<dyn Fn(String) + Send + Sync>) -> Result<Guest, String> {
        Self::start_with(inv, local_ports, name, log, None, false)
    }

    pub fn start_with(inv: &Invite, local_ports: (u16, u16), name: &str, log: Box<dyn Fn(String) + Send + Sync>, bridge: Option<Arc<Mutex<Option<SocketAddr>>>>, quic_only: bool) -> Result<Guest, String> {
        let (id, psk) = (inv.id().ok_or("invitation invalide")?, inv.key().ok_or("invitation invalide")?);
        let mut listeners = Vec::new();
        for (port, kind) in [(local_ports.0, Kind::Realm), (local_ports.1, Kind::World)] {
            let l = TcpListener::bind(("127.0.0.1", port)).map_err(|_| format!("le port local {port} est occupé : arrêtez votre serveur HomeCraft (ou le programme qui l'utilise) avant de rejoindre une partie"))?;
            l.set_nonblocking(true).map_err(|e| e.to_string())?;
            listeners.push((l, kind));
        }
        let shared = Arc::new(GuestShared { id, psk, endpoints: inv.p.clone(), preferred: Mutex::new(None), stop: AtomicBool::new(false), link: Mutex::new(GuestLink::Searching), rtt_ms: AtomicU64::new(0), name: name.to_string(), log, bridge, quic_only });
        let mut threads = Vec::new();
        for (l, kind) in listeners {
            let sh = shared.clone();
            threads.push(std::thread::spawn(move || {
                while !sh.stop.load(Ordering::SeqCst) {
                    match l.accept() {
                        Ok((local, _)) => {
                            let sh2 = sh.clone();
                            std::thread::spawn(move || {
                                let _ = local.set_nonblocking(false);
                                let _ = local.set_nodelay(true);
                                match guest_connect(&sh2, kind) {
                                    Ok((s, t)) => pipe(s, local, t),
                                    Err(e) => {
                                        (sh2.log)(format!("connexion à l'hôte impossible : {e}"));
                                        let _ = local.shutdown(Shutdown::Both);
                                    }
                                }
                            });
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(50)),
                        Err(_) => std::thread::sleep(Duration::from_millis(200)),
                    }
                }
            }));
        }
        let sh = shared.clone();
        threads.push(std::thread::spawn(move || guest_control(sh)));
        Ok(Guest { shared, threads, host_name: inv.h.clone() })
    }

    pub fn link(&self) -> GuestLink {
        *self.shared.link.lock().unwrap()
    }

    pub fn rtt_ms(&self) -> Option<f32> {
        let r = self.shared.rtt_ms.load(Ordering::SeqCst);
        if r == 0 { None } else { Some(r as f32 / 1000.0) }
    }

    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

impl Drop for Guest {
    fn drop(&mut self) {
        self.stop();
    }
}

fn resolve(ep: &str) -> Vec<SocketAddr> {
    ep.to_socket_addrs().map(|a| a.collect()).unwrap_or_default()
}

/// Connexion chiffrée vers l'hôte : adresse qui a déjà répondu, sinon chacune des adresses de l'invitation.
fn guest_connect(sh: &GuestShared, kind: Kind) -> Result<(TcpStream, snow::TransportState), String> {
    let mut candidates: Vec<SocketAddr> = Vec::new();
    if let Some(b) = sh.bridge.as_ref().and_then(|b| *b.lock().unwrap()) {
        candidates.push(b);
    }
    if let Some(p) = *sh.preferred.lock().unwrap() {
        if !candidates.contains(&p) {
            candidates.push(p);
        }
    }
    for ep in sh.endpoints.iter().filter(|_| !sh.quic_only) {
        for a in resolve(ep) {
            if !candidates.contains(&a) {
                candidates.push(a);
            }
        }
    }
    let mut last = "aucune adresse de l'hôte ne répond".to_string();
    for a in candidates {
        let Ok(mut s) = TcpStream::connect_timeout(&a, Duration::from_secs(4)) else { continue };
        let _ = s.set_nodelay(true);
        let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
        match handshake_initiator(&mut s, &sh.id, &sh.psk, kind) {
            Ok(t) => {
                let _ = s.set_read_timeout(None);
                *sh.preferred.lock().unwrap() = Some(a);
                return Ok((s, t));
            }
            Err(e) => {
                last = e;
                if last == "refusé par l'hôte" {
                    *sh.link.lock().unwrap() = GuestLink::Refused;
                    return Err(last);
                }
            }
        }
    }
    Err(last)
}

/// Canal de présence : nom du joueur, latence ; reconnexion toutes les 5 s en cas de coupure.
fn guest_control(sh: Arc<GuestShared>) {
    let mut seq = 0u64;
    while !sh.stop.load(Ordering::SeqCst) {
        match guest_connect(&sh, Kind::Control) {
            Ok((mut s, t)) => {
                (sh.log)("relié à l'hôte".into());
                *sh.link.lock().unwrap() = GuestLink::Connected;
                let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
                let t = Mutex::new(t);
                while !sh.stop.load(Ordering::SeqCst) {
                    seq += 1;
                    let rtt = sh.rtt_ms.load(Ordering::SeqCst);
                    let start = Instant::now();
                    let msg = serde_json::json!({"t": "ping", "n": seq, "name": sh.name, "rtt": if rtt == 0 { serde_json::Value::Null } else { (rtt as f64 / 1000.0).into() }});
                    if send_msg(&mut s, &t, &msg).is_err() || recv_msg(&mut s, &t).is_err() {
                        break;
                    }
                    sh.rtt_ms.store((start.elapsed().as_micros() as u64).max(1), Ordering::SeqCst);
                    for _ in 0..30 {
                        if sh.stop.load(Ordering::SeqCst) {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                let _ = s.shutdown(Shutdown::Both);
                if !sh.stop.load(Ordering::SeqCst) {
                    (sh.log)("liaison avec l'hôte perdue, nouvelle tentative".into());
                    *sh.link.lock().unwrap() = GuestLink::Searching;
                }
            }
            Err(_) => {
                if *sh.link.lock().unwrap() != GuestLink::Refused {
                    *sh.link.lock().unwrap() = GuestLink::Searching;
                }
            }
        }
        for _ in 0..50 {
            if sh.stop.load(Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

// ------------------------------------------------------------------------------------------------ UPnP, adresses

/// Adresse IPv4 de cette machine sur le réseau local (interface de la route par défaut ; aucun paquet n'est envoyé).
fn lan_ipv4() -> Option<std::net::Ipv4Addr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?;
    match s.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(v) if !v.is_loopback() && !v.is_unspecified() => Some(v),
        _ => None,
    }
}

/// Ouvre le port TCP sur la box (UPnP) ; renvoie l'adresse publique annoncée par la box.
fn upnp_open(port: u16) -> Result<String, String> {
    let lan = lan_ipv4().ok_or("adresse locale introuvable")?;
    let gw = igd_next::search_gateway({ let mut o = igd_next::SearchOptions::default(); o.timeout = Some(Duration::from_secs(4)); o }).map_err(|e| format!("aucune box UPnP trouvée ({e})"))?;
    gw.add_port(igd_next::PortMappingProtocol::TCP, port, SocketAddr::new(lan.into(), port), 0, "HomeCraft Multijoueur").map_err(|e| format!("la box refuse l'ouverture du port ({e})"))?;
    let ip = gw.get_external_ip().map_err(|e| format!("adresse publique inconnue ({e})"))?;
    Ok(ip.to_string())
}

fn upnp_close(port: u16) {
    if let Ok(gw) = igd_next::search_gateway({ let mut o = igd_next::SearchOptions::default(); o.timeout = Some(Duration::from_secs(3)); o }) {
        let _ = gw.remove_port(igd_next::PortMappingProtocol::TCP, port);
    }
}

// ------------------------------------------------------------------------------------------------ état et commandes

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Role {
    #[default]
    Off,
    Host,
    Guest,
}

#[derive(Clone, Debug)]
pub struct InviteRow {
    pub id: String,
    pub label: String,
    pub expires: u64,
    pub revoked: bool,
}

#[derive(Clone, Debug, Default)]
pub struct NetStatus {
    pub role: Role,
    pub busy: bool,
    pub message: String,
    pub error: Option<String>,
    pub peers: Vec<Peer>,
    pub invites: Vec<InviteRow>,
    /// Dernier code créé (libellé, code) : affiché pour être copié, jamais journalisé
    pub last_code: Option<(String, String)>,
    /// Invité : état de la liaison avec l'hôte et latence
    pub guest_link: GuestLink,
    pub latency_ms: Option<f32>,
    pub public_ips: Vec<String>,
    /// Hôte : résultat de l'ouverture automatique du port
    pub upnp: Option<Result<String, String>>,
    pub host_name: String,
    pub listen_port: u16,
    /// Invité : code de réponse à renvoyer à l'hôte si sa box n'est pas joignable directement
    pub reply_code: Option<String>,
    /// Hôte : perçage de NAT disponible (adresse publique UDP trouvée)
    pub nat_ready: bool,
}

pub enum NetCmd {
    Host,
    Join(String),
    Leave,
    Invite(String),
    Revoke(String),
    /// Code de réponse d'un invité : perçage de NAT vers lui
    Reply(String),
    Settings(NetSettings),
    Shutdown,
}

#[derive(Clone)]
pub struct Network {
    pub status: Arc<Mutex<NetStatus>>,
    tx: Sender<NetCmd>,
}

impl Network {
    pub fn send(&self, c: NetCmd) {
        let _ = self.tx.send(c);
    }
    /// Fermeture du launcher : partie fermée proprement (port UPnP refermé).
    pub fn shutdown_and_wait(&self) {
        self.send(NetCmd::Shutdown);
        let start = Instant::now();
        while self.status.lock().unwrap().role != Role::Off && start.elapsed() < Duration::from_secs(8) {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

struct Worker {
    layout: Layout,
    settings: NetSettings,
    status: Arc<Mutex<NetStatus>>,
    rx: Receiver<NetCmd>,
    host: Option<Host>,
    guest: Option<Guest>,
    invites: Vec<InviteRow>,
    upnp_port: Option<u16>,
    quic_host: Option<crate::nat::QuicHost>,
    quic_guest: Option<crate::nat::QuicGuest>,
}

pub fn spawn(layout: Layout) -> Network {
    let status = Arc::new(Mutex::new(NetStatus::default()));
    let (tx, rx) = std::sync::mpsc::channel();
    let st = status.clone();
    std::thread::Builder::new()
        .name("reseau".into())
        .spawn(move || {
            let settings = NetSettings::load(&layout);
            let mut w = Worker { layout, settings, status: st, rx, host: None, guest: None, invites: Vec::new(), upnp_port: None, quic_host: None, quic_guest: None };
            w.run();
        })
        .expect("thread réseau");
    Network { status, tx }
}

fn logger_arc(layout: &Layout) -> Arc<dyn Fn(String) + Send + Sync> {
    let l = logger(layout);
    Arc::new(move |s| l(s))
}

fn logger(layout: &Layout) -> Box<dyn Fn(String) + Send + Sync> {
    let dir = layout.logs().join("network");
    Box::new(move |line: String| {
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("homecraft-network.log")) {
            let _ = writeln!(f, "{} {line}", crate::util::time_hms());
        }
    })
}

impl Worker {
    fn run(&mut self) {
        loop {
            match self.rx.recv_timeout(Duration::from_secs(2)) {
                Ok(NetCmd::Shutdown) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    self.leave();
                    return;
                }
                Ok(c) => self.handle(c),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            }
            self.poll();
        }
    }

    fn set(&self, f: impl FnOnce(&mut NetStatus)) {
        f(&mut self.status.lock().unwrap());
    }

    fn log(&self, line: &str) {
        logger(&self.layout)(line.to_string());
    }

    fn handle(&mut self, c: NetCmd) {
        self.set(|s| s.error = None);
        let r = match c {
            NetCmd::Host => self.host(),
            NetCmd::Join(code) => self.join(&code),
            NetCmd::Leave => {
                self.leave();
                self.set(|s| s.message = "Partie multijoueur fermée.".into());
                Ok(())
            }
            NetCmd::Invite(label) => self.invite(&label),
            NetCmd::Revoke(id) => self.revoke(&id),
            NetCmd::Reply(code) => self.reply(&code),
            NetCmd::Settings(s) => {
                let r = s.save(&self.layout).map_err(|e| format!("network.json : {e}"));
                self.settings = s;
                r
            }
            NetCmd::Shutdown => Ok(()),
        };
        if let Err(e) = r {
            self.log(&format!("erreur : {e}"));
            self.set(|s| {
                s.busy = false;
                s.error = Some(e);
            });
        }
    }

    fn host(&mut self) -> Result<(), String> {
        if self.host.is_some() || self.guest.is_some() {
            return Err("une partie multijoueur est déjà ouverte".into());
        }
        let port = self.settings.listen_port;
        let host = Host::start(port, (REALMD_PORT, WORLD_PORT), logger(&self.layout))?;
        self.log(&format!("partie ouverte sur le port {port}"));
        self.host = Some(host);
        self.invites.clear();
        self.set(|s| {
            s.role = Role::Host;
            s.invites.clear();
            s.last_code = None;
            s.upnp = None;
            s.listen_port = port;
            s.public_ips.clear();
            s.message = "Partie ouverte : créez une invitation pour chaque joueur.".into();
        });
        if self.settings.upnp {
            self.set(|s| s.busy = true);
            let r = upnp_open(port);
            match &r {
                Ok(ip) => {
                    self.log(&format!("port {port} ouvert sur la box (UPnP), adresse publique {ip}"));
                    self.upnp_port = Some(port);
                }
                Err(e) => self.log(&format!("UPnP : {e} — ouvrez le port {port} (TCP) sur la box si les invités sont hors de votre réseau local")),
            }
            self.set(|s| {
                s.busy = false;
                if let Ok(ip) = &r {
                    s.public_ips = vec![ip.clone()];
                }
                s.upnp = Some(r);
            });
        }
        // Perçage de NAT en secours : point QUIC sur le même numéro de port (UDP), adresse publique selon STUN.
        match crate::nat::QuicHost::start(port, port, logger_arc(&self.layout)) {
            Ok(q) => {
                self.log(&format!("perçage de NAT prêt (UDP {}{})", q.port, q.public.map(|a| format!(", adresse publique {a}")).unwrap_or_else(|| ", adresse publique inconnue".into())));
                let ready = q.public.is_some();
                self.quic_host = Some(q);
                self.set(|s| s.nat_ready = ready);
            }
            Err(e) => self.log(&format!("perçage de NAT indisponible : {e}")),
        }
        Ok(())
    }

    /// Adresses annoncées dans les invitations : adresse publique (saisie, sinon donnée par la box), puis réseau local.
    fn endpoints(&self) -> Vec<String> {
        let port = self.host.as_ref().map(|h| h.port).unwrap_or(self.settings.listen_port);
        let mut out: Vec<String> = Vec::new();
        let manual = self.settings.public_address.trim();
        if !manual.is_empty() {
            out.push(if manual.contains(':') && !manual.starts_with('[') && manual.matches(':').count() == 1 { manual.to_string() } else { format!("{manual}:{port}") });
        }
        for ip in self.status.lock().unwrap().public_ips.clone() {
            out.push(format!("{ip}:{port}"));
        }
        if let Some(l) = lan_ipv4() {
            out.push(format!("{l}:{port}"));
        }
        out.dedup();
        out
    }

    fn invite(&mut self, label: &str) -> Result<(), String> {
        let Some(host) = self.host.as_ref() else { return Err("ouvrez d'abord une partie".into()) };
        let label: String = label.trim().chars().filter(|c| !c.is_control()).take(32).collect();
        let label = if label.is_empty() { format!("Invité {}", self.invites.len() + 1) } else { label };
        let ttl = (self.settings.invite_hours.clamp(1, 24 * 30) as u64) * 3600;
        let endpoints = self.endpoints();
        if endpoints.is_empty() {
            return Err("aucune adresse de l'hôte n'a été trouvée : saisissez l'adresse publique dans les réglages".into());
        }
        let (id, mut inv) = host.invite(&label, ttl, endpoints, &self.settings.display_name());
        if let Some(q) = &self.quic_host {
            if let Some(p) = q.public {
                inv.q.push(p.to_string());
            }
            if let Some(l) = lan_ipv4() {
                inv.q.push(format!("{l}:{}", q.port));
            }
            inv.f = to_hex(&q.fingerprint);
        }
        let code = inv.encode();
        // Tests automatiques : code écrit dans le dossier désigné (jamais dans les journaux).
        if let Ok(d) = std::env::var("HOMECRAFT_NET_CODE_DIR") {
            let _ = std::fs::write(std::path::Path::new(&d).join(format!("{label}.txt")), &code);
        }
        self.invites.push(InviteRow { id: id.clone(), label: label.clone(), expires: inv.x, revoked: false });
        self.log(&format!("invitation créée : {label} ({id}), valable {} h, {} adresse(s)", ttl / 3600, inv.p.len()));
        let rows = self.invites.clone();
        self.set(|s| {
            s.invites = rows;
            s.last_code = Some((label, code));
            s.message = "Invitation créée : copiez le code et transmettez-le en privé.".into();
        });
        Ok(())
    }

    fn revoke(&mut self, id: &str) -> Result<(), String> {
        let Some(host) = self.host.as_ref() else { return Err("aucune partie ouverte".into()) };
        if !host.revoke(id) {
            return Err("invitation inconnue".into());
        }
        if let Some(r) = self.invites.iter_mut().find(|r| r.id == id) {
            r.revoked = true;
            let label = r.label.clone();
            self.log(&format!("invitation révoquée : {label} ({id})"));
        }
        let rows = self.invites.clone();
        self.set(|s| {
            s.invites = rows;
            s.last_code = None;
            s.message = "Invitation révoquée : ce joueur est déconnecté et ne peut plus revenir avec ce code.".into();
        });
        Ok(())
    }

    /// Code de réponse d'un invité (« HCR-… ») : signature vérifiée avec la clé de son invitation, puis perçage.
    fn reply(&mut self, code: &str) -> Result<(), String> {
        let (Some(host), Some(q)) = (self.host.as_ref(), self.quic_host.as_ref()) else { return Err("ouvrez d'abord une partie".into()) };
        let r = Reply::decode(code)?;
        let (psk, label) = host.psk(&r.i).ok_or("ce code de réponse ne correspond à aucune invitation valide")?;
        if !r.verify(&psk) {
            return Err("code de réponse altéré".into());
        }
        let targets: Vec<SocketAddr> = r.c.iter().filter_map(|a| a.parse().ok()).collect();
        if targets.is_empty() {
            return Err("code de réponse sans adresse".into());
        }
        q.punch(targets.clone());
        self.log(&format!("perçage de NAT vers {label} ({} adresse(s))", targets.len()));
        self.set(|s| s.message = format!("Liaison en cours avec {label} : elle s'établit en quelques secondes si les deux box le permettent."));
        Ok(())
    }

    fn join(&mut self, code: &str) -> Result<(), String> {
        if self.host.is_some() || self.guest.is_some() {
            return Err("une partie multijoueur est déjà ouverte".into());
        }
        let inv = Invite::decode(code)?;
        // Perçage de NAT : point QUIC de l'invité et code de réponse (adresses de l'invité, signées avec la clé).
        let mut bridge = None;
        let mut reply = None;
        if let (false, Some(fp)) = (inv.q.is_empty(), from_hex(&inv.f).and_then(|v| <[u8; 32]>::try_from(v).ok())) {
            let targets: Vec<SocketAddr> = inv.q.iter().filter_map(|a| a.parse().ok()).collect();
            match crate::nat::QuicGuest::start(targets, fp, logger_arc(&self.layout)) {
                Ok(q) => {
                    let mut c: Vec<String> = q.public.iter().map(|a| a.to_string()).collect();
                    if let Some(l) = lan_ipv4() {
                        c.push(format!("{l}:{}", q.local_port));
                    }
                    reply = inv.key().map(|k| Reply::new(&inv.i, c, &k).encode());
                    bridge = Some(q.bridge.clone());
                    self.quic_guest = Some(q);
                }
                Err(e) => self.log(&format!("perçage de NAT indisponible : {e}")),
            }
        }
        let g = Guest::start_with(&inv, (self.settings.guest_realm_port, self.settings.guest_world_port), &self.settings.display_name(), logger(&self.layout), bridge, self.settings.quic_only)?;
        if let (Some(r), Ok(d)) = (&reply, std::env::var("HOMECRAFT_NET_CODE_DIR")) {
            let _ = std::fs::write(std::path::Path::new(&d).join("reponse.txt"), r);
        }
        GUEST_ACTIVE.store(true, Ordering::SeqCst);
        self.log(&format!("connexion à la partie de {} ({} adresse(s))", inv.h, inv.p.len()));
        let host = inv.h.clone();
        self.guest = Some(g);
        self.set(|s| {
            s.role = Role::Guest;
            s.host_name = host;
            s.guest_link = GuestLink::Searching;
            s.reply_code = reply;
            s.message = "Recherche de l'hôte…".into();
        });
        Ok(())
    }

    fn leave(&mut self) {
        if let Some(mut h) = self.host.take() {
            h.stop();
            self.log("partie fermée (toutes les invitations sont invalidées)");
        }
        if let Some(port) = self.upnp_port.take() {
            upnp_close(port);
            self.log(&format!("port {port} refermé sur la box (UPnP)"));
        }
        if let Some(mut q) = self.quic_host.take() {
            q.stop();
        }
        if let Some(mut g) = self.guest.take() {
            g.stop();
            self.log("partie de l'hôte quittée");
        }
        if let Some(mut q) = self.quic_guest.take() {
            q.stop();
        }
        GUEST_ACTIVE.store(false, Ordering::SeqCst);
        self.invites.clear();
        self.set(|s| {
            s.role = Role::Off;
            s.busy = false;
            s.peers.clear();
            s.invites.clear();
            s.last_code = None;
            s.guest_link = GuestLink::Searching;
            s.latency_ms = None;
            s.host_name.clear();
            s.upnp = None;
            s.reply_code = None;
            s.nat_ready = false;
        });
    }

    fn poll(&mut self) {
        if let Some(h) = &self.host {
            let peers = h.peers();
            self.set(|s| s.peers = peers);
        }
        if let Some(g) = &self.guest {
            let (link, rtt) = (g.link(), g.rtt_ms());
            self.set(|s| {
                s.guest_link = link;
                s.latency_ms = rtt;
                s.message = match link {
                    GuestLink::Connected => format!("Connecté à {}{}", s.host_name, rtt.map(|l| format!(" ({l:.0} ms)")).unwrap_or_default()),
                    GuestLink::Searching => "Recherche de l'hôte… (si cela dure, l'hôte doit ouvrir son port sur sa box)".into(),
                    GuestLink::Refused => "L'hôte refuse cette invitation (révoquée, expirée ou partie fermée) : demandez-en une nouvelle.".into(),
                };
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_aller_retour() {
        let inv = Invite { v: 2, i: "00".repeat(16), k: "ab".repeat(32), p: vec!["1.2.3.4:11010".into()], x: now_unix() + 60, h: "Hôte".into(), q: Vec::new(), f: String::new() };
        let code = inv.encode();
        assert!(code.starts_with("HC2-"));
        assert!(!code.contains('+') && !code.contains('/'));
        assert_eq!(Invite::decode(&code).unwrap(), inv);
        let wrapped = format!("  {}\n{} ", &code[..20], &code[20..]);
        assert_eq!(Invite::decode(&wrapped).unwrap(), inv);
        assert!(Invite::decode("HC2-!!!").is_err());
        assert!(Invite::decode(&code[..code.len() - 5]).is_err());
        assert!(Invite::decode(&Invite { x: 1, ..inv.clone() }.encode()).unwrap_err().contains("expiré"));
        assert!(Invite::decode(&Invite { k: "ab".into(), ..inv }.encode()).is_err());
    }

    #[test]
    fn base64_toutes_longueurs() {
        for n in 0..20u8 {
            let d: Vec<u8> = (0..n).map(|i| i.wrapping_mul(37)).collect();
            assert_eq!(b64url_decode(&b64url_encode(&d)).unwrap(), d);
        }
    }

    /// Serveur d'écho local : joue le rôle de realmd / mangosd.
    fn echo_server() -> u16 {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                std::thread::spawn(move || {
                    let mut s = s;
                    let mut buf = [0u8; 70_000];
                    while let Ok(n) = s.read(&mut buf) {
                        if n == 0 || s.write_all(&buf[..n]).is_err() {
                            break;
                        }
                    }
                });
            }
        });
        port
    }

    fn free_port() -> u16 {
        TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
    }

    fn quiet() -> Box<dyn Fn(String) + Send + Sync> {
        Box::new(|_| {})
    }

    fn roundtrip(port: u16, data: &[u8]) -> Option<Vec<u8>> {
        let mut c = TcpStream::connect(("127.0.0.1", port)).ok()?;
        c.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
        c.write_all(data).ok()?;
        let mut got = vec![0u8; data.len()];
        c.read_exact(&mut got).ok()?;
        Some(got)
    }

    fn wait_link(g: &Guest, want: GuestLink) -> bool {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            if g.link() == want {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }

    #[test]
    fn tunnel_de_bout_en_bout() {
        let (realm, world) = (echo_server(), echo_server());
        let host = Host::start(0, (realm, world), quiet()).unwrap();
        let (_, inv) = host.invite("Ami", 3600, vec![format!("127.0.0.1:{}", host.port)], "Hôte");
        let (lr, lw) = (free_port(), free_port());
        let g = Guest::start(&inv, (lr, lw), "Ami", quiet()).unwrap();
        assert!(wait_link(&g, GuestLink::Connected));
        // Gros volume (plusieurs messages Noise) dans les deux sens, sur les deux ports
        let big: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        assert_eq!(roundtrip(lr, &big).unwrap(), big);
        assert_eq!(roundtrip(lw, b"monde").unwrap(), b"monde");
        // Présence vue par l'hôte
        std::thread::sleep(Duration::from_millis(500));
        assert!(host.peers().iter().any(|p| p.name == "Ami" && p.invite == "Ami"));
    }

    #[test]
    fn mauvaise_cle_refusee() {
        let host = Host::start(0, (echo_server(), echo_server()), quiet()).unwrap();
        let (_, mut inv) = host.invite("Ami", 3600, vec![format!("127.0.0.1:{}", host.port)], "Hôte");
        inv.k = "11".repeat(32);
        let lr = free_port();
        let g = Guest::start(&inv, (lr, free_port()), "Pirate", quiet()).unwrap();
        assert!(wait_link(&g, GuestLink::Refused));
        assert!(roundtrip(lr, b"x").is_none());
    }

    #[test]
    fn revocation_coupe_et_refuse() {
        let host = Host::start(0, (echo_server(), echo_server()), quiet()).unwrap();
        let (id, inv) = host.invite("Ami", 3600, vec![format!("127.0.0.1:{}", host.port)], "Hôte");
        let lr = free_port();
        let g = Guest::start(&inv, (lr, free_port()), "Ami", quiet()).unwrap();
        assert!(wait_link(&g, GuestLink::Connected));
        let mut open = TcpStream::connect(("127.0.0.1", lr)).unwrap();
        open.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        open.write_all(b"avant").unwrap();
        let mut b = [0u8; 5];
        open.read_exact(&mut b).unwrap();
        assert!(host.revoke(&id));
        // La connexion ouverte est coupée, les suivantes sont refusées.
        let mut rest = [0u8; 1];
        assert!(matches!(open.read(&mut rest), Ok(0) | Err(_)));
        assert!(wait_link(&g, GuestLink::Refused));
        assert!(roundtrip(lr, b"apres").is_none());
    }

    #[test]
    fn invitation_expiree_refusee() {
        let host = Host::start(0, (echo_server(), echo_server()), quiet()).unwrap();
        let (_, inv) = host.invite("Ami", 0, vec![format!("127.0.0.1:{}", host.port)], "Hôte");
        let inv = Invite { x: now_unix() + 60, ..inv }; // date du code falsifiée : l'hôte garde la vraie
        let lr = free_port();
        let g = Guest::start(&inv, (lr, free_port()), "Ami", quiet()).unwrap();
        assert!(wait_link(&g, GuestLink::Refused));
    }

    #[test]
    fn type_inconnu_et_en_tete_invalide_refuses() {
        let host = Host::start(0, (echo_server(), echo_server()), quiet()).unwrap();
        let (id, inv) = host.invite("Ami", 3600, vec![], "Hôte");
        let psk: [u8; 32] = inv.key().unwrap();
        let idb: [u8; 16] = from_hex(&id).unwrap().try_into().unwrap();
        // Type 3 (par ex. une tentative vers MariaDB) : rien n'existe au-delà de l'authentification et du monde.
        let mut s = TcpStream::connect(("127.0.0.1", host.port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut pro = MAGIC.to_vec();
        pro.extend_from_slice(&idb);
        pro.push(3);
        s.write_all(&pro).unwrap();
        let mut hs = snow::Builder::new(NOISE.parse().unwrap()).psk(0, &psk).unwrap().prologue(&pro).unwrap().build_initiator().unwrap();
        let mut buf = vec![0u8; MAX_MSG];
        let n = hs.write_message(&[], &mut buf).unwrap();
        let _ = write_frame(&mut s, &buf[..n]);
        assert!(read_frame(&mut s).is_err());
        // Données quelconques sans en-tête
        let mut s = TcpStream::connect(("127.0.0.1", host.port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s.write_all(b"GET / HTTP/1.0\r\n\r\nxxxxxxxxxxxx").unwrap();
        let mut b = [0u8; 1];
        assert!(matches!(s.read(&mut b), Ok(0) | Err(_)));
    }

    #[test]
    fn code_de_reponse_signe() {
        let key = [5u8; 32];
        let r = Reply::new("ab".repeat(16).as_str(), vec!["86.1.2.3:4000".into(), "192.168.1.5:4000".into()], &key);
        let back = Reply::decode(&r.encode()).unwrap();
        assert!(back.verify(&key));
        assert!(!back.verify(&[6u8; 32]), "autre clé");
        let forged = Reply { c: vec!["6.6.6.6:1".into()], ..back };
        assert!(!forged.verify(&key), "adresses modifiées");
        assert!(Reply::decode("HC2-xxx").is_err());
    }

    #[test]
    fn liaison_par_percage_quic() {
        // Hôte : TCP (Noise) + QUIC ; invité : QUIC seulement (aucune adresse TCP essayée), comme derrière une box fermée.
        let (realm, world) = (echo_server(), echo_server());
        let host = Host::start(0, (realm, world), quiet()).unwrap();
        let qlog: Arc<dyn Fn(String) + Send + Sync> = Arc::new(|_| {});
        let qh = crate::nat::QuicHost::start(0, host.port, qlog.clone()).unwrap();
        let (_, mut inv) = host.invite("Ami", 3600, vec![format!("127.0.0.1:{}", host.port)], "Hôte");
        inv.q = vec![format!("127.0.0.1:{}", qh.port)];
        inv.f = to_hex(&qh.fingerprint);
        let qg = crate::nat::QuicGuest::start(vec![format!("127.0.0.1:{}", qh.port).parse().unwrap()], qh.fingerprint, qlog).unwrap();
        let (lr, lw) = (free_port(), free_port());
        let g = Guest::start_with(&inv, (lr, lw), "Ami", quiet(), Some(qg.bridge.clone()), true).unwrap();
        assert!(wait_link(&g, GuestLink::Connected));
        let big: Vec<u8> = (0..150_000u32).map(|i| (i % 249) as u8).collect();
        assert_eq!(roundtrip(lr, &big).unwrap(), big);
        assert_eq!(roundtrip(lw, b"monde").unwrap(), b"monde");
    }

    #[test]
    fn port_local_occupe_signale() {
        let busy = TcpListener::bind("127.0.0.1:0").unwrap();
        let p = busy.local_addr().unwrap().port();
        let inv = Invite { v: 2, i: "00".repeat(16), k: "ab".repeat(32), p: vec!["127.0.0.1:1".into()], x: now_unix() + 60, h: "H".into(), q: Vec::new(), f: String::new() };
        let e = Guest::start(&inv, (p, free_port()), "A", quiet()).err().unwrap();
        assert!(e.contains("occupé"));
    }
}
