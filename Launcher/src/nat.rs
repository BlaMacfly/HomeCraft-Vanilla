//! Perçage de NAT, en secours quand la box de l'hôte n'ouvre pas son port (pas d'UPnP) : aucun réglage de box.
//!
//! 1. Chaque launcher découvre l'adresse publique de son socket UDP auprès de serveurs STUN publics (RFC 5389 : ils ne
//!    voient que l'adresse IP et le port, rien d'autre ne leur est envoyé).
//! 2. L'hôte met son adresse dans l'invitation ; l'invité renvoie un « code de réponse » avec la sienne (signé avec la
//!    clé de son invitation : l'hôte ne perce que vers un invité authentique).
//! 3. Les deux envoient des paquets l'un vers l'autre en même temps : chaque box voit une réponse à un envoi sortant et
//!    laisse passer. La liaison est alors une connexion QUIC (UDP fiable) ; le certificat de l'hôte, créé pour la
//!    partie, est épinglé par son empreinte dans l'invitation.
//! QUIC ne sert que de tuyau : chaque flux QUIC est raccordé, des deux côtés, à une connexion TCP locale qui passe par
//! le protocole Noise habituel (network.rs) ; authentification, chiffrement de bout en bout et cloisonnement sont donc
//! exactement les mêmes que par TCP direct.
//! Limite : derrière un NAT « symétrique » (certaines box 4G/5G, réseaux d'entreprise, CGNAT), le perçage échoue.

use sha2::{Digest, Sha256};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const STUN_SERVERS: [&str; 3] = ["stun.l.google.com:19302", "stun.cloudflare.com:3478", "stun.nextcloud.com:443"];
const MAGIC_COOKIE: u32 = 0x2112_A442;

// ------------------------------------------------------------------------------------------------ STUN

fn stun_request(txid: &[u8; 12]) -> Vec<u8> {
    let mut b = vec![0x00, 0x01, 0x00, 0x00];
    b.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    b.extend_from_slice(txid);
    b
}

/// Adresse publique (XOR-MAPPED-ADDRESS, ou MAPPED-ADDRESS) d'une réponse STUN « Binding success ».
pub fn parse_stun(resp: &[u8], txid: &[u8; 12]) -> Option<SocketAddr> {
    if resp.len() < 20 || resp[0..2] != [0x01, 0x01] || resp[4..8] != MAGIC_COOKIE.to_be_bytes() || &resp[8..20] != txid {
        return None;
    }
    let len = u16::from_be_bytes([resp[2], resp[3]]) as usize;
    let body = resp.get(20..20 + len)?;
    let mut i = 0;
    let mut mapped = None;
    while i + 4 <= body.len() {
        let t = u16::from_be_bytes([body[i], body[i + 1]]);
        let l = u16::from_be_bytes([body[i + 2], body[i + 3]]) as usize;
        let v = body.get(i + 4..i + 4 + l)?;
        if (t == 0x0020 || t == 0x0001) && l >= 8 && v[1] == 0x01 {
            let mut port = u16::from_be_bytes([v[2], v[3]]);
            let mut ip = u32::from_be_bytes([v[4], v[5], v[6], v[7]]);
            if t == 0x0020 {
                port ^= (MAGIC_COOKIE >> 16) as u16;
                ip ^= MAGIC_COOKIE;
            }
            let a = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(ip), port));
            if t == 0x0020 {
                return Some(a);
            }
            mapped = Some(a);
        }
        i += 4 + l.div_ceil(4) * 4;
    }
    mapped
}

/// Adresse publique de ce socket UDP vue par un serveur STUN (le premier qui répond).
pub fn stun_mapped(sock: &UdpSocket) -> Result<SocketAddr, String> {
    sock.set_read_timeout(Some(Duration::from_millis(1500))).map_err(|e| e.to_string())?;
    for server in STUN_SERVERS {
        let Some(addr) = server.to_socket_addrs().ok().and_then(|mut a| a.find(|a| a.is_ipv4())) else { continue };
        for _ in 0..2 {
            let txid: [u8; 12] = rand::random();
            if sock.send_to(&stun_request(&txid), addr).is_err() {
                break;
            }
            let mut buf = [0u8; 512];
            while let Ok((n, from)) = sock.recv_from(&mut buf) {
                if from == addr {
                    if let Some(a) = parse_stun(&buf[..n], &txid) {
                        let _ = sock.set_read_timeout(None);
                        return Ok(a);
                    }
                }
            }
        }
    }
    let _ = sock.set_read_timeout(None);
    Err("adresse publique introuvable (serveurs STUN injoignables)".into())
}

// ------------------------------------------------------------------------------------------------ signature du code de réponse

pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut inner = Sha256::new();
    inner.update(k.iter().map(|b| b ^ 0x36).collect::<Vec<u8>>());
    inner.update(msg);
    let mut outer = Sha256::new();
    outer.update(k.iter().map(|b| b ^ 0x5c).collect::<Vec<u8>>());
    outer.update(inner.finalize());
    outer.finalize().into()
}

// ------------------------------------------------------------------------------------------------ QUIC

fn transport() -> Arc<quinn::TransportConfig> {
    let mut t = quinn::TransportConfig::default();
    t.keep_alive_interval(Some(Duration::from_secs(5)));
    t.max_idle_timeout(Some(Duration::from_secs(60).try_into().unwrap()));
    Arc::new(t)
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread().worker_threads(2).thread_name("quic").enable_all().build().map_err(|e| e.to_string())
}

/// Raccorde un flux QUIC et une connexion TCP, dans les deux sens.
async fn splice(mut send: quinn::SendStream, mut recv: quinn::RecvStream, tcp: tokio::net::TcpStream) {
    let _ = tcp.set_nodelay(true);
    let (mut tr, mut tw) = tcp.into_split();
    let down = async {
        let _ = tokio::io::copy(&mut recv, &mut tw).await;
        let _ = tokio::io::AsyncWriteExt::shutdown(&mut tw).await;
    };
    let up = async {
        let _ = tokio::io::copy(&mut tr, &mut send).await;
        let _ = send.finish();
    };
    tokio::join!(down, up);
}

/// Côté hôte : point QUIC sur le port UDP de la partie ; chaque flux reçu est raccordé au port TCP local de l'hôte.
pub struct QuicHost {
    rt: Option<tokio::runtime::Runtime>,
    endpoint: quinn::Endpoint,
    punch: UdpSocket,
    stop: Arc<AtomicBool>,
    pub public: Option<SocketAddr>,
    pub port: u16,
    pub fingerprint: [u8; 32],
}

impl QuicHost {
    pub fn start(port: u16, tcp_target: u16, log: Arc<dyn Fn(String) + Send + Sync>) -> Result<QuicHost, String> {
        let sock = UdpSocket::bind(("0.0.0.0", port)).or_else(|_| UdpSocket::bind(("0.0.0.0", 0))).map_err(|e| e.to_string())?;
        let port = sock.local_addr().map_err(|e| e.to_string())?.port();
        let public = match stun_mapped(&sock) {
            Ok(a) => Some(a),
            Err(e) => {
                log(format!("perçage de NAT indisponible : {e}"));
                None
            }
        };
        let punch = sock.try_clone().map_err(|e| e.to_string())?;
        let ck = rcgen::generate_simple_self_signed(vec!["homecraft".to_string()]).map_err(|e| e.to_string())?;
        let der = ck.cert.der().to_vec();
        let fingerprint: [u8; 32] = Sha256::digest(&der).into();
        let key = rustls::pki_types::PrivatePkcs8KeyDer::from(ck.signing_key.serialize_der());
        let mut server = quinn::ServerConfig::with_single_cert(vec![rustls::pki_types::CertificateDer::from(der)], key.into()).map_err(|e| e.to_string())?;
        server.transport_config(transport());
        let rt = runtime()?;
        let endpoint = {
            let _g = rt.enter();
            quinn::Endpoint::new(quinn::EndpointConfig::default(), Some(server), sock, Arc::new(quinn::TokioRuntime)).map_err(|e| e.to_string())?
        };
        let ep = endpoint.clone();
        rt.spawn(async move {
            while let Some(incoming) = ep.accept().await {
                let log = log.clone();
                tokio::spawn(async move {
                    let Ok(conn) = incoming.await else { return };
                    log(format!("liaison QUIC établie avec {} (perçage de NAT)", conn.remote_address()));
                    while let Ok((send, recv)) = conn.accept_bi().await {
                        tokio::spawn(async move {
                            if let Ok(tcp) = tokio::net::TcpStream::connect(("127.0.0.1", tcp_target)).await {
                                splice(send, recv, tcp).await;
                            }
                        });
                    }
                });
            }
        });
        // Garde ouverte l'entrée de la box vers ce socket (elle expire sans trafic sortant au bout de 30 s à quelques
        // minutes selon les box) : une requête STUN toutes les 20 s, dont la réponse est ignorée par QUIC.
        let stop = Arc::new(AtomicBool::new(false));
        if let (Some(_), Ok(keep)) = (public, punch.try_clone()) {
            let st = stop.clone();
            std::thread::spawn(move || {
                let server = STUN_SERVERS[0].to_socket_addrs().ok().and_then(|mut a| a.find(|a| a.is_ipv4()));
                'veille: loop {
                    for _ in 0..200 {
                        if st.load(Ordering::SeqCst) {
                            break 'veille;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    if let Some(s) = server {
                        if keep.send_to(&stun_request(&rand::random()), s).is_err() {
                            break;
                        }
                    }
                }
            });
        }
        Ok(QuicHost { rt: Some(rt), endpoint, punch, stop, public, port, fingerprint })
    }

    /// Perce vers un invité : paquets envoyés à ses adresses pendant une minute (il tente de se connecter en même temps).
    pub fn punch(&self, targets: Vec<SocketAddr>) {
        let Ok(s) = self.punch.try_clone() else { return };
        std::thread::spawn(move || {
            for _ in 0..120 {
                for t in &targets {
                    let _ = s.send_to(b"homecraft", t);
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.endpoint.close(0u32.into(), b"partie fermee");
        if let Some(rt) = self.rt.take() {
            rt.shutdown_timeout(Duration::from_secs(2));
        }
    }
}

impl Drop for QuicHost {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Vérification du certificat de l'hôte par son empreinte (donnée par l'invitation), sans autorité de certification.
#[derive(Debug)]
struct Pinned {
    fp: [u8; 32],
    provider: Arc<rustls::crypto::CryptoProvider>,
}

impl rustls::client::danger::ServerCertVerifier for Pinned {
    fn verify_server_cert(&self, end_entity: &rustls::pki_types::CertificateDer<'_>, _: &[rustls::pki_types::CertificateDer<'_>], _: &rustls::pki_types::ServerName<'_>, _: &[u8], _: rustls::pki_types::UnixTime) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let fp: [u8; 32] = Sha256::digest(end_entity.as_ref()).into();
        if fp == self.fp { Ok(rustls::client::danger::ServerCertVerified::assertion()) } else { Err(rustls::Error::General("certificat de l'hôte inattendu".into())) }
    }
    fn verify_tls12_signature(&self, message: &[u8], cert: &rustls::pki_types::CertificateDer<'_>, dss: &rustls::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }
    fn verify_tls13_signature(&self, message: &[u8], cert: &rustls::pki_types::CertificateDer<'_>, dss: &rustls::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}

/// Côté invité : connexion QUIC vers l'hôte (tentée en boucle, ce qui perce la box de l'invité) et pont local : une
/// connexion TCP sur `bridge` devient un flux QUIC vers l'hôte.
pub struct QuicGuest {
    rt: Option<tokio::runtime::Runtime>,
    endpoint: quinn::Endpoint,
    stop: Arc<AtomicBool>,
    pub public: Option<SocketAddr>,
    pub local_port: u16,
    /// Adresse du pont local, renseignée tant que la liaison QUIC est établie
    pub bridge: Arc<Mutex<Option<SocketAddr>>>,
}

impl QuicGuest {
    pub fn start(host: Vec<SocketAddr>, fingerprint: [u8; 32], log: Arc<dyn Fn(String) + Send + Sync>) -> Result<QuicGuest, String> {
        let sock = UdpSocket::bind(("0.0.0.0", 0)).map_err(|e| e.to_string())?;
        let local_port = sock.local_addr().map_err(|e| e.to_string())?.port();
        let public = stun_mapped(&sock).ok();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let tls = rustls::ClientConfig::builder_with_provider(provider.clone())
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| e.to_string())?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Pinned { fp: fingerprint, provider }))
            .with_no_client_auth();
        let mut client = quinn::ClientConfig::new(Arc::new(quinn::crypto::rustls::QuicClientConfig::try_from(tls).map_err(|e| e.to_string())?));
        client.transport_config(transport());
        let rt = runtime()?;
        let (mut endpoint, listener) = {
            let _g = rt.enter();
            let ep = quinn::Endpoint::new(quinn::EndpointConfig::default(), None, sock, Arc::new(quinn::TokioRuntime)).map_err(|e| e.to_string())?;
            let l = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
            l.set_nonblocking(true).map_err(|e| e.to_string())?;
            (ep, tokio::net::TcpListener::from_std(l).map_err(|e| e.to_string())?)
        };
        endpoint.set_default_client_config(client);
        let bridge_addr = listener.local_addr().map_err(|e| e.to_string())?;
        let bridge = Arc::new(Mutex::new(None));
        let current: Arc<Mutex<Option<quinn::Connection>>> = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        // Connexion à l'hôte, renouvelée si elle se perd
        let (ep, cur, br, st, lg) = (endpoint.clone(), current.clone(), bridge.clone(), stop.clone(), log.clone());
        rt.spawn(async move {
            while !st.load(Ordering::SeqCst) {
                let mut conn = None;
                for a in &host {
                    if let Ok(connecting) = ep.connect(*a, "homecraft") {
                        if let Ok(Ok(c)) = tokio::time::timeout(Duration::from_secs(3), connecting).await {
                            conn = Some(c);
                            break;
                        }
                    }
                }
                match conn {
                    Some(c) => {
                        lg(format!("liaison QUIC établie avec {} (perçage de NAT)", c.remote_address()));
                        *cur.lock().unwrap() = Some(c.clone());
                        *br.lock().unwrap() = Some(bridge_addr);
                        c.closed().await;
                        *br.lock().unwrap() = None;
                        *cur.lock().unwrap() = None;
                        lg("liaison QUIC perdue".into());
                    }
                    None => tokio::time::sleep(Duration::from_millis(500)).await,
                }
            }
        });
        // Pont local : chaque connexion TCP devient un flux QUIC
        let cur = current.clone();
        rt.spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                let c = cur.lock().unwrap().clone();
                if let Some(c) = c {
                    tokio::spawn(async move {
                        if let Ok((send, recv)) = c.open_bi().await {
                            splice(send, recv, tcp).await;
                        }
                    });
                }
            }
        });
        Ok(QuicGuest { rt: Some(rt), endpoint, stop, public, local_port, bridge })
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.endpoint.close(0u32.into(), b"fin");
        if let Some(rt) = self.rt.take() {
            rt.shutdown_timeout(Duration::from_secs(2));
        }
    }
}

impl Drop for QuicGuest {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reponse_stun() {
        let txid = [7u8; 12];
        // Réponse construite à la main : XOR-MAPPED-ADDRESS 203.0.113.7:55935
        let ip = u32::from(Ipv4Addr::new(203, 0, 113, 7)) ^ MAGIC_COOKIE;
        let port = 55935u16 ^ (MAGIC_COOKIE >> 16) as u16;
        let mut attr = vec![0x00, 0x20, 0x00, 0x08, 0x00, 0x01];
        attr.extend_from_slice(&port.to_be_bytes());
        attr.extend_from_slice(&ip.to_be_bytes());
        let mut r = vec![0x01, 0x01];
        r.extend_from_slice(&(attr.len() as u16).to_be_bytes());
        r.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
        r.extend_from_slice(&txid);
        r.extend_from_slice(&attr);
        assert_eq!(parse_stun(&r, &txid), Some("203.0.113.7:55935".parse().unwrap()));
        assert_eq!(parse_stun(&r, &[0u8; 12]), None, "autre transaction");
        assert_eq!(parse_stun(&r[..10], &txid), None);
    }

    #[test]
    fn hmac_vecteur_rfc4231() {
        // RFC 4231, cas 2
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(mac.iter().map(|b| format!("{b:02x}")).collect::<String>(), "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
    }

    #[test]
    fn tuyau_quic_local() {
        // Serveur d'écho TCP joue le port TCP de l'hôte
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let target = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                std::thread::spawn(move || {
                    use std::io::{Read, Write};
                    let mut s = s;
                    let mut b = [0u8; 4096];
                    while let Ok(n) = s.read(&mut b) {
                        if n == 0 || s.write_all(&b[..n]).is_err() {
                            break;
                        }
                    }
                });
            }
        });
        let quiet: Arc<dyn Fn(String) + Send + Sync> = Arc::new(|_| {});
        let host = QuicHost::start(0, target, quiet.clone()).unwrap();
        let addr: SocketAddr = format!("127.0.0.1:{}", host.port).parse().unwrap();
        let guest = QuicGuest::start(vec![addr], host.fingerprint, quiet.clone()).unwrap();
        let start = std::time::Instant::now();
        while guest.bridge.lock().unwrap().is_none() && start.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(100));
        }
        let b = guest.bridge.lock().unwrap().expect("liaison QUIC");
        use std::io::{Read, Write};
        let mut c = std::net::TcpStream::connect(b).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let data: Vec<u8> = (0..100_000u32).map(|i| (i % 253) as u8).collect();
        c.write_all(&data).unwrap();
        let mut got = vec![0u8; data.len()];
        c.read_exact(&mut got).unwrap();
        assert_eq!(got, data);
        // Mauvaise empreinte : l'invité refuse le certificat, aucune liaison
        let bad = QuicGuest::start(vec![addr], [9u8; 32], quiet).unwrap();
        std::thread::sleep(Duration::from_secs(3));
        assert!(bad.bridge.lock().unwrap().is_none());
    }
}
