//! Client World of Warcraft 1.12 minimal, sans interface, pour les tests d'intégration de HomeCraft.
//!
//! Il parle le vrai protocole du jeu (authentification SRP6 auprès de realmd, liste des royaumes, session du monde
//! avec chiffrement des en-têtes, création et connexion d'un personnage, discussion) afin de vérifier ce qu'aucun test
//! sans client ne peut prouver : plusieurs personnages d'un même compte connectés en même temps, absence de
//! déconnexion pour inactivité, réponses des bots. Chaque événement est écrit sur une ligne « EVENT … » de la sortie.
//!
//! Exemple : hc-wowclient --account TEST --password-env HC_PW --char Testun --stay 60 --afk
//! Codes de sortie : 0 succès, 2 authentification, 3 session du monde, 4 personnage, 5 déconnecté par le serveur.

use num_bigint::BigUint;
use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const N_HEX: &str = "894B645E89E1535BBDAD5B8B290650530801B18EBFBF5E8FAB3C82872A3E9BB7";
const BUILD: u16 = 5875;

const SMSG_AUTH_CHALLENGE: u16 = 0x1EC;
const CMSG_AUTH_SESSION: u32 = 0x1ED;
const SMSG_AUTH_RESPONSE: u16 = 0x1EE;
const CMSG_CHAR_ENUM: u32 = 0x037;
const SMSG_CHAR_ENUM: u16 = 0x03B;
const CMSG_CHAR_CREATE: u32 = 0x036;
const SMSG_CHAR_CREATE: u16 = 0x03A;
const CMSG_PLAYER_LOGIN: u32 = 0x03D;
const SMSG_CHARACTER_LOGIN_FAILED: u16 = 0x041;
const SMSG_LOGIN_VERIFY_WORLD: u16 = 0x236;
const CMSG_MESSAGECHAT: u32 = 0x095;
const SMSG_MESSAGECHAT: u16 = 0x096;
const CMSG_PING: u32 = 0x1DC;
const CMSG_LOGOUT_REQUEST: u32 = 0x04B;
const SMSG_LOGOUT_COMPLETE: u16 = 0x04D;

const CHAT_MSG_SAY: u32 = 0x00;
const CHAT_MSG_WHISPER: u32 = 0x06;
const CHAT_MSG_AFK: u32 = 0x14;

fn event(msg: impl AsRef<str>) {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    println!("EVENT {t} {}", msg.as_ref());
    let _ = std::io::stdout().flush();
}

fn fail(code: i32, msg: impl AsRef<str>) -> ! {
    event(format!("ERROR {}", msg.as_ref()));
    std::process::exit(code);
}

fn sha1(parts: &[&[u8]]) -> [u8; 20] {
    let mut h = Sha1::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// Entier → octets petit-boutistes de longueur fixe (les nombres SRP de WoW sont petit-boutistes).
fn le_bytes(n: &BigUint, len: usize) -> Vec<u8> {
    let mut v = n.to_bytes_le();
    v.resize(len, 0);
    v
}

struct Args {
    realm: String,
    account: String,
    password: String,
    character: String,
    race: u8,
    class: u8,
    stay: u64,
    afk: bool,
    say: Option<String>,
    whisper: Option<(String, String)>,
    expect_reply_from: Option<String>,
    /// Adresse du monde à utiliser à la place de celle annoncée par le royaume (tests du tunnel multijoueur sur une
    /// seule machine : les ports locaux de l'invité diffèrent de ceux de l'hôte)
    world_override: Option<String>,
}

fn parse_args() -> Args {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let password = match (get("--password"), get("--password-env")) {
        (Some(p), _) => p,
        (None, Some(var)) => std::env::var(&var).unwrap_or_else(|_| fail(1, format!("variable {var} absente"))),
        _ => fail(1, "--password ou --password-env requis"),
    };
    let whisper = a.iter().position(|x| x == "--whisper").and_then(|i| Some((a.get(i + 1)?.clone(), a.get(i + 2)?.clone())));
    Args {
        realm: get("--realm").unwrap_or_else(|| "127.0.0.1:3724".into()),
        account: get("--account").unwrap_or_else(|| fail(1, "--account requis")).to_ascii_uppercase(),
        password: password.to_ascii_uppercase(),
        character: get("--char").unwrap_or_else(|| fail(1, "--char requis")),
        race: get("--race").and_then(|v| v.parse().ok()).unwrap_or(1),
        class: get("--class").and_then(|v| v.parse().ok()).unwrap_or(1),
        stay: get("--stay").and_then(|v| v.parse().ok()).unwrap_or(10),
        afk: a.iter().any(|x| x == "--afk"),
        say: get("--say"),
        whisper,
        expect_reply_from: get("--expect-reply-from"),
        world_override: get("--world-override"),
    }
}

// ------------------------------------------------------------------ realmd (SRP6)

struct RealmLogin {
    key: [u8; 40],
    world_addr: String,
}

fn read_exact(s: &mut TcpStream, n: usize) -> Vec<u8> {
    let mut b = vec![0u8; n];
    s.read_exact(&mut b).unwrap_or_else(|e| fail(2, format!("lecture realmd : {e}")));
    b
}

fn realm_login(args: &Args) -> RealmLogin {
    let mut s = TcpStream::connect(&args.realm).unwrap_or_else(|e| fail(2, format!("connexion à realmd {} : {e}", args.realm)));
    s.set_read_timeout(Some(Duration::from_secs(15))).ok();

    // AUTH_LOGON_CHALLENGE
    let name = args.account.as_bytes();
    let mut body = Vec::new();
    body.extend_from_slice(b"WoW\0");
    body.extend_from_slice(&[1, 12, 1]);
    body.extend_from_slice(&BUILD.to_le_bytes());
    body.extend_from_slice(b"68x\0");
    body.extend_from_slice(b"niW\0");
    body.extend_from_slice(b"RFrf");
    body.extend_from_slice(&60u32.to_le_bytes());
    body.extend_from_slice(&[127, 0, 0, 1]);
    body.push(name.len() as u8);
    body.extend_from_slice(name);
    let mut pkt = vec![0x00, 0x03];
    pkt.extend_from_slice(&(body.len() as u16).to_le_bytes());
    pkt.extend_from_slice(&body);
    s.write_all(&pkt).unwrap_or_else(|e| fail(2, format!("envoi realmd : {e}")));

    let head = read_exact(&mut s, 3);
    if head[2] != 0 {
        fail(2, format!("refus de realmd au défi (code {}) : compte inconnu ?", head[2]));
    }
    let b_bytes = read_exact(&mut s, 32);
    let g_len = read_exact(&mut s, 1)[0] as usize;
    let g_bytes = read_exact(&mut s, g_len);
    let n_len = read_exact(&mut s, 1)[0] as usize;
    let n_bytes = read_exact(&mut s, n_len);
    let salt = read_exact(&mut s, 32);
    let _crc_salt = read_exact(&mut s, 16);
    let _security = read_exact(&mut s, 1);

    let n = BigUint::from_bytes_le(&n_bytes);
    if n != BigUint::parse_bytes(N_HEX.as_bytes(), 16).unwrap() {
        fail(2, "module N inattendu");
    }
    let g = BigUint::from_bytes_le(&g_bytes);
    let b = BigUint::from_bytes_le(&b_bytes);
    let k = BigUint::from(3u32);

    let a_priv = BigUint::from_bytes_le(&rand::random::<[u8; 19]>());
    let a_pub = g.modpow(&a_priv, &n);
    let a_le = le_bytes(&a_pub, 32);

    let inner = sha1(&[format!("{}:{}", args.account, args.password).as_bytes()]);
    let x = BigUint::from_bytes_le(&sha1(&[&salt, &inner]));
    let u = BigUint::from_bytes_le(&sha1(&[&a_le, &b_bytes]));
    let gx = g.modpow(&x, &n);
    let base = (&b + &k * &n - (&k * gx) % &n) % &n;
    let s_val = base.modpow(&(&a_priv + &u * &x), &n);
    let s_le = le_bytes(&s_val, 32);
    let even: Vec<u8> = s_le.iter().step_by(2).copied().collect();
    let odd: Vec<u8> = s_le.iter().skip(1).step_by(2).copied().collect();
    let (h1, h2) = (sha1(&[&even]), sha1(&[&odd]));
    let mut key = [0u8; 40];
    for i in 0..20 {
        key[2 * i] = h1[i];
        key[2 * i + 1] = h2[i];
    }
    let hn = sha1(&[&n_bytes]);
    let hg = sha1(&[&g_bytes]);
    let mut nxg = [0u8; 20];
    for i in 0..20 {
        nxg[i] = hn[i] ^ hg[i];
    }
    let m1 = sha1(&[&nxg, &sha1(&[name]), &salt, &a_le, &b_bytes, &key]);

    // AUTH_LOGON_PROOF
    let mut proof = vec![0x01];
    proof.extend_from_slice(&a_le);
    proof.extend_from_slice(&m1);
    proof.extend_from_slice(&[0u8; 20]);
    proof.extend_from_slice(&[0, 0]);
    s.write_all(&proof).unwrap_or_else(|e| fail(2, format!("envoi realmd : {e}")));
    let ph = read_exact(&mut s, 2);
    if ph[1] != 0 {
        fail(2, format!("mot de passe refusé par realmd (code {})", ph[1]));
    }
    let m2 = read_exact(&mut s, 20);
    let _flags = read_exact(&mut s, 4);
    if m2 != sha1(&[&a_le, &m1, &key]) {
        fail(2, "preuve du serveur invalide");
    }
    event(format!("realm_auth_ok account={}", args.account));

    // REALM_LIST
    s.write_all(&[0x10, 0, 0, 0, 0]).unwrap_or_else(|e| fail(2, format!("envoi realmd : {e}")));
    let rh = read_exact(&mut s, 3);
    let size = u16::from_le_bytes([rh[1], rh[2]]) as usize;
    let data = read_exact(&mut s, size);
    let mut p = 5usize; // u32 inutilisé + u8 nombre de royaumes
    let count = data[4];
    if count == 0 {
        fail(2, "aucun royaume");
    }
    let cstr = |p: &mut usize| -> String {
        let start = *p;
        while data[*p] != 0 {
            *p += 1;
        }
        let s = String::from_utf8_lossy(&data[start..*p]).into_owned();
        *p += 1;
        s
    };
    p += 4 + 1; // icône, drapeaux
    let realm_name = cstr(&mut p);
    let world_addr = cstr(&mut p);
    event(format!("realm name=\"{realm_name}\" address={world_addr}"));
    RealmLogin { key, world_addr }
}

// ------------------------------------------------------------------ monde

struct World {
    s: TcpStream,
    key: [u8; 40],
    send_i: usize,
    send_j: u8,
    crypt: bool,
}

struct Packet {
    opcode: u16,
    body: Vec<u8>,
}

impl World {
    fn send(&mut self, opcode: u32, body: &[u8]) {
        let size = (body.len() + 4) as u16;
        let mut head = vec![(size >> 8) as u8, size as u8];
        head.extend_from_slice(&opcode.to_le_bytes());
        if self.crypt {
            for b in head.iter_mut() {
                self.send_i %= 40;
                let x = (*b ^ self.key[self.send_i]).wrapping_add(self.send_j);
                self.send_i += 1;
                self.send_j = x;
                *b = x;
            }
        }
        let mut all = head;
        all.extend_from_slice(body);
        if let Err(e) = self.s.write_all(&all) {
            fail(5, format!("connexion au monde perdue à l'envoi : {e}"));
        }
    }
}

/// Lecture des paquets du serveur sur un fil dédié (en-têtes déchiffrés dès l'authentification).
fn spawn_reader(mut s: TcpStream, key: [u8; 40], crypt_from_first_after_challenge: bool, tx: mpsc::Sender<Option<Packet>>) {
    std::thread::spawn(move || {
        let (mut recv_i, mut recv_j) = (0usize, 0u8);
        let mut crypt = false;
        loop {
            let mut h = [0u8; 4];
            if s.read_exact(&mut h).is_err() {
                let _ = tx.send(None);
                return;
            }
            if crypt {
                for b in h.iter_mut() {
                    recv_i %= 40;
                    let x = b.wrapping_sub(recv_j) ^ key[recv_i];
                    recv_i += 1;
                    recv_j = *b;
                    *b = x;
                }
            }
            let size = u16::from_be_bytes([h[0], h[1]]) as usize;
            let opcode = u16::from_le_bytes([h[2], h[3]]);
            let mut body = vec![0u8; size.saturating_sub(2)];
            if s.read_exact(&mut body).is_err() {
                let _ = tx.send(None);
                return;
            }
            // Le défi arrive en clair ; tout ce qui suit est chiffré.
            if opcode == SMSG_AUTH_CHALLENGE && crypt_from_first_after_challenge {
                crypt = true;
            }
            if tx.send(Some(Packet { opcode, body })).is_err() {
                return;
            }
        }
    });
}

fn addon_info() -> Vec<u8> {
    let mut raw = Vec::new();
    raw.extend_from_slice(b"Blizzard_AuctionUI\0");
    raw.push(1);
    raw.extend_from_slice(&0x4C1C_776Du32.to_le_bytes());
    raw.extend_from_slice(&0u32.to_le_bytes());
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    z.write_all(&raw).unwrap();
    let packed = z.finish().unwrap();
    let mut out = (raw.len() as u32).to_le_bytes().to_vec();
    out.extend_from_slice(&packed);
    out
}

fn recv(rx: &mpsc::Receiver<Option<Packet>>, timeout: Duration) -> Option<Packet> {
    match rx.recv_timeout(timeout) {
        Ok(Some(p)) => Some(p),
        Ok(None) => fail(5, "connexion au monde fermée par le serveur"),
        Err(_) => None,
    }
}

fn wait_for(rx: &mpsc::Receiver<Option<Packet>>, opcodes: &[u16], timeout: Duration, what: &str, code: i32) -> Packet {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(p) = recv(rx, deadline - Instant::now()) {
            if opcodes.contains(&p.opcode) {
                return p;
            }
        }
    }
    fail(code, format!("délai dépassé en attendant {what}"));
}

fn cstring(b: &[u8], p: &mut usize) -> String {
    let start = *p;
    while *p < b.len() && b[*p] != 0 {
        *p += 1;
    }
    let s = String::from_utf8_lossy(&b[start..*p]).into_owned();
    *p += 1;
    s
}

/// Liste des personnages : (guid, nom).
fn parse_char_enum(b: &[u8]) -> Vec<(u64, String)> {
    let mut out = Vec::new();
    let count = b[0];
    let mut p = 1;
    for _ in 0..count {
        let guid = u64::from_le_bytes(b[p..p + 8].try_into().unwrap());
        p += 8;
        let name = cstring(b, &mut p);
        p += 3 + 5 + 1 + 4 + 4 + 12 + 4 + 4 + 1 + 12 + 20 * 5;
        out.push((guid, name));
    }
    out
}

/// Message du serveur : (type, guid de l'émetteur, texte).
fn parse_chat(b: &[u8]) -> Option<(u8, u64, String)> {
    let t = *b.first()?;
    let mut p = 5;
    let rd64 = |p: &mut usize| -> Option<u64> {
        let v = u64::from_le_bytes(b.get(*p..*p + 8)?.try_into().ok()?);
        *p += 8;
        Some(v)
    };
    let guid = match t as u32 {
        0x00 | 0x01 | 0x05 => {
            let g = rd64(&mut p)?;
            rd64(&mut p)?;
            g
        }
        0x0E => {
            cstring(b, &mut p);
            p += 4;
            rd64(&mut p)?
        }
        0x0B | 0x0C | 0x0D | 0x1A => return None,
        _ => rd64(&mut p)?,
    };
    p += 4;
    Some((t, guid, cstring(b, &mut p)))
}

fn chat(w: &mut World, kind: u32, target: Option<&str>, text: &str) {
    let mut body = kind.to_le_bytes().to_vec();
    body.extend_from_slice(&0u32.to_le_bytes());
    if let Some(t) = target {
        body.extend_from_slice(t.as_bytes());
        body.push(0);
    }
    body.extend_from_slice(text.as_bytes());
    body.push(0);
    w.send(CMSG_MESSAGECHAT, &body);
}

fn main() {
    let args = parse_args();
    let login = realm_login(&args);

    let addr = args.world_override.clone().unwrap_or_else(|| login.world_addr.clone());
    let s = TcpStream::connect(&addr).unwrap_or_else(|e| fail(3, format!("connexion au monde {addr} : {e}")));
    let (tx, rx) = mpsc::channel();
    spawn_reader(s.try_clone().unwrap(), login.key, true, tx);
    let mut w = World { s, key: login.key, send_i: 0, send_j: 0, crypt: false };

    let ch = wait_for(&rx, &[SMSG_AUTH_CHALLENGE], Duration::from_secs(15), "le défi du monde", 3);
    let server_seed = ch.body.get(0..4).unwrap_or_else(|| fail(3, "défi du monde invalide")).to_vec();
    let client_seed: [u8; 4] = rand::random();
    let digest = sha1(&[args.account.as_bytes(), &[0, 0, 0, 0], &client_seed, &server_seed, &login.key]);
    let mut body = (BUILD as u32).to_le_bytes().to_vec();
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(args.account.as_bytes());
    body.push(0);
    body.extend_from_slice(&client_seed);
    body.extend_from_slice(&digest);
    body.extend_from_slice(&addon_info());
    w.send(CMSG_AUTH_SESSION, &body);
    w.crypt = true;

    let r = wait_for(&rx, &[SMSG_AUTH_RESPONSE], Duration::from_secs(20), "la réponse d'authentification du monde", 3);
    if r.body.first() != Some(&0x0C) {
        fail(3, format!("session du monde refusée (code 0x{:02X})", r.body.first().copied().unwrap_or(0)));
    }
    event("world_auth_ok");

    // Personnage : créé s'il n'existe pas, puis connecté.
    w.send(CMSG_CHAR_ENUM, &[]);
    let e = wait_for(&rx, &[SMSG_CHAR_ENUM], Duration::from_secs(15), "la liste des personnages", 4);
    let mut chars = parse_char_enum(&e.body);
    let wanted = args.character.to_ascii_lowercase();
    if !chars.iter().any(|(_, n)| n.to_ascii_lowercase() == wanted) {
        let mut b = args.character.as_bytes().to_vec();
        b.push(0);
        b.extend_from_slice(&[args.race, args.class, 0, 0, 0, 0, 0, 0, 0]);
        w.send(CMSG_CHAR_CREATE, &b);
        let c = wait_for(&rx, &[SMSG_CHAR_CREATE], Duration::from_secs(15), "la création du personnage", 4);
        if c.body.first() != Some(&0x2E) {
            fail(4, format!("création du personnage {} refusée (code 0x{:02X})", args.character, c.body.first().copied().unwrap_or(0)));
        }
        event(format!("char_created name={}", args.character));
        w.send(CMSG_CHAR_ENUM, &[]);
        let e = wait_for(&rx, &[SMSG_CHAR_ENUM], Duration::from_secs(15), "la liste des personnages", 4);
        chars = parse_char_enum(&e.body);
    }
    let guid = chars.iter().find(|(_, n)| n.to_ascii_lowercase() == wanted).map(|(g, _)| *g).unwrap_or_else(|| fail(4, "personnage introuvable après création"));
    w.send(CMSG_PLAYER_LOGIN, &guid.to_le_bytes());
    let l = wait_for(&rx, &[SMSG_LOGIN_VERIFY_WORLD, SMSG_CHARACTER_LOGIN_FAILED], Duration::from_secs(30), "l'entrée dans le monde", 4);
    if l.opcode == SMSG_CHARACTER_LOGIN_FAILED {
        fail(4, format!("connexion du personnage refusée (code {})", l.body.first().copied().unwrap_or(0)));
    }
    event(format!("in_world name={} guid={guid}", args.character));

    if args.afk {
        chat(&mut w, CHAT_MSG_AFK, None, "");
        event("afk_set");
    }
    if let Some(t) = &args.say {
        chat(&mut w, CHAT_MSG_SAY, None, t);
    }
    if let Some((target, text)) = &args.whisper {
        chat(&mut w, CHAT_MSG_WHISPER, Some(target), text);
        event(format!("whisper_sent to={target}"));
    }

    // Présence : ping toutes les 30 s comme le vrai client ; tout message reçu est journalisé.
    let start = Instant::now();
    let mut last_ping = Instant::now();
    let mut seq = 1u32;
    let mut got_reply = false;
    while start.elapsed() < Duration::from_secs(args.stay) {
        if last_ping.elapsed() >= Duration::from_secs(30) {
            let mut b = seq.to_le_bytes().to_vec();
            b.extend_from_slice(&50u32.to_le_bytes());
            w.send(CMSG_PING, &b);
            seq += 1;
            last_ping = Instant::now();
        }
        let Some(p) = recv(&rx, Duration::from_millis(500)) else { continue };
        match p.opcode {
            SMSG_LOGOUT_COMPLETE => fail(5, "déconnecté du monde par le serveur (retour à la sélection des personnages)"),
            SMSG_MESSAGECHAT => {
                if let Some((t, g, text)) = parse_chat(&p.body) {
                    if t as u32 == CHAT_MSG_WHISPER || t as u32 == CHAT_MSG_SAY || t == 0x01 {
                        event(format!("chat type={t} from={g} text={text:?}"));
                        if t as u32 == CHAT_MSG_WHISPER && args.expect_reply_from.is_some() {
                            got_reply = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    event(format!("stayed seconds={}", args.stay));
    if args.expect_reply_from.is_some() && !got_reply {
        fail(6, "aucune réponse chuchotée reçue");
    }
    w.send(CMSG_LOGOUT_REQUEST, &[]);
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Some(p) = recv(&rx, Duration::from_millis(500)) {
            if p.opcode == SMSG_LOGOUT_COMPLETE {
                event("logout_complete");
                return;
            }
        }
    }
    event("logout_timeout");
}
