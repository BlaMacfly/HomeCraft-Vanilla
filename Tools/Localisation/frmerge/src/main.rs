//! frmerge : complète les traductions françaises (loc2) de la base monde CMaNGOS-tbc (TBC-DB) à partir de
//! sources locales, en ne remplissant QUE les textes français absents et seulement quand le texte anglais source est
//! identique au texte anglais de TBC-DB (même entrée, même phrase) : jamais d'écrasement, jamais de traduction d'un
//! texte différent. Sources :
//!   - VMaNGOS mangos.sql (Vanilla) : mangos_string (messages serveur, format printf identique) ;
//!   - AzerothCore/SylvaniaCore acore_world.sql : acore_string (format {} converti), item/creature/gameobject/
//!     quest/page_text + *_locale (frFR) ;
//!   - mod-playerbots : ai_playerbot_texts (base + mise à jour française).
//! Sortie : un fichier SQL d'UPDATE / INSERT … ON DUPLICATE KEY UPDATE conditionnels, et un rapport sur stderr.
//! Usage : frmerge <tbcmangos.sql> <vmangos mangos.sql> <acore_world.sql> <bots base.sql> <bots fr.sql> <sortie.sql>

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};

type Row = Vec<Option<String>>;

#[derive(Default)]
struct Table {
    cols: Vec<String>,
    rows: Vec<Row>,
}
impl Table {
    fn idx(&self, c: &str) -> usize {
        self.cols.iter().position(|x| x.eq_ignore_ascii_case(c)).unwrap_or_else(|| panic!("colonne {c} absente ({:?})", self.cols))
    }
}

const BS: u8 = 92; // antislash
const Q: u8 = 39; // apostrophe

fn parse_tuples(s: &[u8], mut f: impl FnMut(Row)) {
    let mut i = 0;
    while i < s.len() {
        if s[i] == Q {
            // chaîne hors tuple (ne devrait pas arriver) : l'ignorer
            i += 1;
            continue;
        }
        if s[i] != b'(' {
            i += 1;
            continue;
        }
        i += 1;
        let mut row = Vec::new();
        loop {
            while i < s.len() && (s[i] == b' ' || s[i] == b'\n' || s[i] == b'\r' || s[i] == b'\t') {
                i += 1;
            }
            if i >= s.len() {
                break;
            }
            if s[i] == Q {
                i += 1;
                let mut v = Vec::new();
                while i < s.len() {
                    let c = s[i];
                    if c == BS {
                        let n = s[i + 1];
                        v.push(match n { b'n' => b'\n', b'r' => b'\r', b't' => b'\t', b'0' => 0, _ => n });
                        i += 2;
                    } else if c == Q && i + 1 < s.len() && s[i + 1] == Q {
                        v.push(Q);
                        i += 2;
                    } else if c == Q {
                        i += 1;
                        break;
                    } else {
                        v.push(c);
                        i += 1;
                    }
                }
                row.push(Some(String::from_utf8_lossy(&v).into_owned()));
            } else {
                let st = i;
                while i < s.len() && s[i] != b',' && s[i] != b')' {
                    i += 1;
                }
                let t = String::from_utf8_lossy(&s[st..i]).trim().to_string();
                row.push(if t.eq_ignore_ascii_case("NULL") { None } else { Some(t) });
            }
            while i < s.len() && (s[i] == b' ' || s[i] == b'\n' || s[i] == b'\r' || s[i] == b'\t') {
                i += 1;
            }
            if i >= s.len() {
                break;
            }
            if s[i] == b',' {
                i += 1;
                continue;
            }
            if s[i] == b')' {
                i += 1;
                break;
            }
        }
        f(row);
    }
}

/// Charge les tables demandées d'un dump (CREATE TABLE pour les colonnes, INSERT éventuellement multi-lignes).
fn load(path: &str, wanted: &[&str]) -> HashMap<String, Table> {
    let r = BufReader::with_capacity(1 << 22, std::fs::File::open(path).unwrap_or_else(|e| panic!("{path} : {e}")));
    let mut out: HashMap<String, Table> = HashMap::new();
    let mut cur_create: Option<String> = None;
    let mut stmt: Option<(String, Vec<u8>)> = None;
    for line in r.split(b'\n') {
        let line = line.unwrap();
        if let Some((t, buf)) = stmt.as_mut() {
            buf.push(b'\n');
            buf.extend_from_slice(&line);
            if line.trim_ascii_end().ends_with(b";") {
                let (t, buf) = (t.clone(), std::mem::take(buf));
                stmt = None;
                insert(&mut out, &t, &buf);
            }
            continue;
        }
        let low = line.to_ascii_lowercase();
        if low.starts_with(b"create table") {
            let s = String::from_utf8_lossy(&line).into_owned();
            let name = s.split('`').nth(1).unwrap_or("").to_string();
            if wanted.contains(&name.as_str()) {
                out.entry(name.clone()).or_default().cols.clear();
                cur_create = Some(name);
            } else {
                cur_create = None;
            }
            continue;
        }
        if let Some(t) = cur_create.clone() {
            let s = String::from_utf8_lossy(&line).into_owned();
            let ts = s.trim_start();
            if ts.starts_with('`') {
                out.get_mut(&t).unwrap().cols.push(ts.split('`').nth(1).unwrap().to_string());
            } else if ts.starts_with(')') {
                cur_create = None;
            }
            continue;
        }
        if low.starts_with(b"insert into `") {
            let s = String::from_utf8_lossy(&line[13..]).into_owned();
            let name = s.split('`').next().unwrap().to_string();
            if !wanted.contains(&name.as_str()) {
                continue;
            }
            if line.trim_ascii_end().ends_with(b";") {
                insert(&mut out, &name, &line);
            } else {
                stmt = Some((name, line.clone()));
            }
        }
    }
    out
}

fn insert(out: &mut HashMap<String, Table>, t: &str, stmt: &[u8]) {
    let tab = out.entry(t.to_string()).or_default();
    // Liste de colonnes explicite : INSERT INTO `t` (`a`, `b`) VALUES …
    let vpos = stmt.windows(6).position(|w| w.eq_ignore_ascii_case(b"VALUES")).expect("VALUES");
    let head = String::from_utf8_lossy(&stmt[..vpos]).into_owned();
    let explicit: Option<Vec<String>> = head.find('(').map(|p| head[p..].split('`').skip(1).step_by(2).map(str::to_string).collect());
    let order: Option<Vec<usize>> = explicit.map(|ex| {
        if tab.cols.is_empty() {
            tab.cols = ex.clone();
        }
        ex.iter().map(|c| tab.cols.iter().position(|x| x == c).expect("colonne inconnue")).collect()
    });
    let ncols = tab.cols.len();
    parse_tuples(&stmt[vpos + 6..], |row| {
        match &order {
            Some(ord) => {
                let mut r: Row = vec![None; ncols];
                for (k, v) in row.into_iter().enumerate() {
                    if let Some(&dst) = ord.get(k) {
                        r[dst] = v;
                    }
                }
                tab.rows.push(r);
            }
            None => tab.rows.push(row),
        }
    });
}

fn v<'a>(r: &'a Row, i: usize) -> &'a str {
    r.get(i).and_then(|x| x.as_deref()).unwrap_or("")
}
fn norm(s: &str) -> String {
    s.replace("\r\n", "\n").trim().to_string()
}
/// Comparaison tolérante des textes anglais : variables du client sans distinction de casse ($N/$n, $C/$c, $B/$b,
/// interprétées de la même façon par le client) et suites d'espaces réduites à une seule.
fn loose(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    let mut prev_dollar = false;
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() && c != '\n' {
            if !prev_space {
                o.push(' ');
            }
            prev_space = true;
            prev_dollar = false;
            continue;
        }
        prev_space = false;
        o.push(if prev_dollar { c.to_ascii_lowercase() } else { c });
        prev_dollar = c == '$';
    }
    o
}
/// Garde-fou sur les « traductions » des sources : certaines entrées frFR d'AzerothCore sont en réalité de l'anglais
/// (parfois avec des entités HTML, ex. quête 16 « Thanks, &lt;name&gt;! &nbsp;… »). Rejet si le texte contient une
/// entité HTML, s'il est identique à l'anglais, ou s'il compte plus de mots-outils anglais que français.
fn probably_french(fr: &str, en: &str) -> bool {
    if ["&lt;", "&gt;", "&nbsp;", "&amp;", "&quot;"].iter().any(|e| fr.contains(e)) {
        return false;
    }
    if loose(&norm(fr)) == loose(&norm(en)) {
        // Noms propres identiques dans les deux langues (« Thrall ») : acceptés seulement s'ils sont courts.
        return fr.split_whitespace().count() <= 2;
    }
    let words: Vec<String> = fr.split(|c: char| !c.is_alphabetic() && c != '\'').filter(|w| !w.is_empty()).map(|w| w.to_lowercase()).collect();
    if words.len() < 4 {
        return true;
    }
    const EN: [&str; 16] = ["the", "you", "your", "and", "is", "are", "of", "to", "with", "have", "this", "that", "for", "will", "what", "we"];
    const FR: [&str; 20] = ["le", "la", "les", "vous", "votre", "vos", "et", "est", "de", "des", "du", "un", "une", "pour", "avec", "nous", "que", "qui", "pas", "ce"];
    let en_n = words.iter().filter(|w| EN.contains(&w.as_str())).count();
    let fr_n = words.iter().filter(|w| FR.contains(&w.as_str())).count();
    fr_n >= en_n
}
fn empty(s: &str) -> bool {
    s.trim().is_empty()
}
/// utf8mb3 : pas de caractères hors plan multilingue de base.
fn bmp_only(s: &str) -> bool {
    s.chars().all(|c| (c as u32) <= 0xFFFF)
}
fn sql(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('\'');
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '\'' => o.push_str("\\'"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\0' => o.push_str("\\0"),
            _ => o.push(c),
        }
    }
    o.push('\'');
    o
}

/// Spécificateurs printf dans l'ordre (%% exclu).
fn printf_specs(s: &str) -> Vec<String> {
    let b: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '%' {
            if i + 1 < b.len() && b[i + 1] == '%' {
                i += 2;
                continue;
            }
            let st = i;
            i += 1;
            while i < b.len() && "-+ #0123456789.*hlLqjzt".contains(b[i]) {
                i += 1;
            }
            if i < b.len() && "diouxXeEfFgGcspn".contains(b[i]) {
                out.push(b[st..=i].iter().collect());
                i += 1;
                continue;
            }
            continue;
        }
        i += 1;
    }
    out
}
/// Remplace chaque spécificateur printf par \u{1} (gabarit de comparaison).
fn printf_skeleton(s: &str) -> String {
    let mut o = s.replace("%%", "\u{2}");
    for sp in printf_specs(s) {
        o = o.replacen(&sp, "\u{1}", 1);
    }
    o
}
/// Remplace chaque {…} (format fmt d'AzerothCore) par \u{1}.
fn fmt_skeleton(s: &str) -> (String, usize) {
    let mut o = String::new();
    let mut n = 0;
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '{' {
            if it.peek() == Some(&'{') {
                it.next();
                o.push('{');
                continue;
            }
            for d in it.by_ref() {
                if d == '}' {
                    break;
                }
            }
            o.push('\u{1}');
            n += 1;
        } else if c == '}' && it.peek() == Some(&'}') {
            it.next();
            o.push('}');
        } else {
            o.push(c);
        }
    }
    (o, n)
}

struct Out {
    sql: Vec<String>,
    report: Vec<String>,
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (tbc, vm, ac, bots_base, bots_fr, out_path) = (&a[1], &a[2], &a[3], &a[4], &a[5], &a[6]);
    let mut out = Out { sql: Vec::new(), report: Vec::new() };

    eprintln!("lecture de TBC-DB…");
    let t = load(tbc, &["mangos_string", "ai_playerbot_texts", "quest_template", "locales_quest", "item_template", "locales_item", "creature_template", "locales_creature", "gameobject_template", "locales_gameobject", "page_text", "locales_page_text"]);
    eprintln!("lecture de VMaNGOS…");
    let vmt = load(vm, &["mangos_string"]);
    eprintln!("lecture d'AzerothCore…");
    let act = load(ac, &["acore_string", "quest_template", "quest_template_locale", "quest_offer_reward", "quest_offer_reward_locale", "quest_request_items", "quest_request_items_locale", "item_template", "item_template_locale", "creature_template", "creature_template_locale", "gameobject_template", "gameobject_template_locale", "page_text", "page_text_locale"]);
    eprintln!("lecture des textes PlayerBots…");
    let bb = load(bots_base, &["ai_playerbot_texts"]);

    mangos_string(&t, &vmt, &act, &mut out);
    playerbot_texts(&t, &bb, bots_fr, &mut out);
    quests(&t, &act, &mut out);
    simple_locale(&t, &act, &mut out, "objets", "item_template", "entry", &[("name", "name_loc2"), ("description", "description_loc2")], "locales_item", "item_template", "entry", &[("name", "Name"), ("description", "Description")], "item_template_locale", "ID");
    simple_locale(&t, &act, &mut out, "créatures", "creature_template", "Entry", &[("Name", "name_loc2"), ("SubName", "subname_loc2")], "locales_creature", "creature_template", "entry", &[("name", "Name"), ("subname", "Title")], "creature_template_locale", "entry");
    simple_locale(&t, &act, &mut out, "objets du monde", "gameobject_template", "entry", &[("name", "name_loc2")], "locales_gameobject", "gameobject_template", "entry", &[("name", "name")], "gameobject_template_locale", "entry");
    simple_locale(&t, &act, &mut out, "pages de livres", "page_text", "entry", &[("text", "Text_loc2")], "locales_page_text", "page_text", "ID", &[("Text", "Text")], "page_text_locale", "ID");

    let mut f = std::fs::File::create(out_path).unwrap();
    writeln!(f, "-- HomeCraft Edition BC : compléments de traduction française (loc2 = frFR) de la base monde TBC-DB.").unwrap();
    writeln!(f, "-- Généré par l'outil frmerge (Documentation/09-Localisation-Inventaire.md). Ne remplit que les textes français").unwrap();
    writeln!(f, "-- absents, et seulement quand le texte anglais source est identique à celui de TBC-DB. Sources : VMaNGOS (GPL-2),").unwrap();
    writeln!(f, "-- AzerothCore / SylvaniaCore (GPL-2), mod-playerbots (GPL-2). Appliqué après l'import de tbcmangos.").unwrap();
    writeln!(f, "SET NAMES utf8mb3;").unwrap();
    for l in &out.sql {
        writeln!(f, "{l}").unwrap();
    }
    for r in &out.report {
        eprintln!("{r}");
        println!("{r}");
    }
}

fn mangos_string(t: &HashMap<String, Table>, vm: &HashMap<String, Table>, ac: &HashMap<String, Table>, out: &mut Out) {
    let ms = &t["mangos_string"];
    let (ie, id, il) = (ms.idx("entry"), ms.idx("content_default"), ms.idx("content_loc2"));
    // VMaNGOS : anglais -> français (même format printf)
    let vms = &vm["mangos_string"];
    let (vd, vl) = (vms.idx("content_default"), vms.idx("content_loc2"));
    let mut vmap: HashMap<String, String> = HashMap::new();
    for r in &vms.rows {
        if !empty(v(r, vl)) {
            vmap.entry(norm(v(r, vd))).or_insert_with(|| v(r, vl).to_string());
        }
    }
    // AzerothCore : gabarit anglais -> français ({} à convertir)
    let acs = &ac["acore_string"];
    let (ad, al) = (acs.idx("content_default"), acs.idx("locale_frFR"));
    let mut amap: HashMap<String, String> = HashMap::new();
    for r in &acs.rows {
        if !empty(v(r, al)) {
            amap.entry(fmt_skeleton(&norm(v(r, ad))).0).or_insert_with(|| v(r, al).to_string());
        }
    }
    let (mut total, mut already, mut from_vm, mut from_ac, mut rejected) = (0, 0, 0, 0, 0);
    for r in &ms.rows {
        total += 1;
        if !empty(v(r, il)) {
            already += 1;
            continue;
        }
        let en = norm(v(r, id));
        let specs = printf_specs(&en);
        let mut fr: Option<String> = None;
        if let Some(f) = vmap.get(&en) {
            if printf_specs(f) == specs {
                fr = Some(f.clone());
                from_vm += 1;
            } else {
                rejected += 1;
            }
        }
        if fr.is_none() {
            if let Some(f) = amap.get(&printf_skeleton(&en)) {
                let (sk, n) = fmt_skeleton(f);
                if n == specs.len() && !sk.contains('%') {
                    let mut s = String::new();
                    let mut k = 0;
                    for c in sk.chars() {
                        if c == '\u{1}' {
                            s.push_str(&specs[k]);
                            k += 1;
                        } else {
                            s.push(c);
                        }
                    }
                    fr = Some(s);
                    from_ac += 1;
                } else {
                    rejected += 1;
                }
            }
        }
        if let Some(f) = fr.filter(|f| bmp_only(f) && probably_french(f, &en)) {
            out.sql.push(format!("UPDATE `mangos_string` SET `content_loc2`={} WHERE `entry`={} AND COALESCE(`content_loc2`,'')='';", sql(&f), v(r, ie)));
        }
    }
    out.report.push(format!(
        "mangos_string : {total} messages, {already} déjà en français, {from_vm} complétés depuis VMaNGOS, {from_ac} depuis AzerothCore (format converti), {rejected} rejetés (paramètres différents), reste {} en anglais",
        total - already - from_vm - from_ac
    ));
}

fn playerbot_texts(t: &HashMap<String, Table>, bb: &HashMap<String, Table>, bots_fr: &str, out: &mut Out) {
    let pt = &t["ai_playerbot_texts"];
    let (pi, pn, px, pl) = (pt.idx("id"), pt.idx("name"), pt.idx("text"), pt.idx("text_loc2"));
    let base = &bb["ai_playerbot_texts"];
    let (bi, bn, bx, bl) = (base.idx("id"), base.idx("name"), base.idx("text"), base.idx("text_loc2"));
    // Mise à jour française de mod-playerbots : UPDATE … SET `text_loc2`='…' WHERE `id`=N;
    let mut upd: HashMap<String, String> = HashMap::new();
    let content = std::fs::read(bots_fr).unwrap();
    for line in content.split(|&c| c == b'\n') {
        let s = String::from_utf8_lossy(line);
        if let (Some(a), Some(w)) = (s.find("`text_loc2`='"), s.rfind("WHERE `id`=")) {
            let id: String = s[w + 11..].chars().take_while(|c| c.is_ascii_digit()).collect();
            let mut vals = Vec::new();
            parse_tuples(format!("({})", &s[a + 12..w].trim().trim_end_matches(',')).as_bytes(), |r| vals = r);
            if let Some(Some(fr)) = vals.into_iter().next() {
                upd.insert(id, fr);
            }
        }
    }
    let mut by_text: HashMap<(String, String), String> = HashMap::new();
    for r in &base.rows {
        let fr = upd.get(v(r, bi)).cloned().unwrap_or_else(|| v(r, bl).to_string());
        if !empty(&fr) {
            by_text.entry((v(r, bn).to_string(), norm(v(r, bx)))).or_insert(fr);
        }
    }
    let (mut total, mut already, mut done) = (0, 0, 0);
    for r in &pt.rows {
        total += 1;
        if !empty(v(r, pl)) {
            already += 1;
            continue;
        }
        if let Some(fr) = by_text.get(&(v(r, pn).to_string(), norm(v(r, px)))).filter(|f| bmp_only(f) && f.chars().count() <= 1024 && probably_french(f, v(r, px))) {
            out.sql.push(format!("UPDATE `ai_playerbot_texts` SET `text_loc2`={} WHERE `id`={} AND `text_loc2`='';", sql(fr), v(r, pi)));
            done += 1;
        }
    }
    out.report.push(format!("ai_playerbot_texts : {total} phrases des bots, {already} déjà en français, {done} complétées depuis mod-playerbots, reste {} en anglais", total - already - done));
}

/// Index « entrée -> ligne » d'une table locale AzerothCore filtrée sur frFR.
fn fr_index<'a>(t: &'a Table, key: &str) -> HashMap<String, &'a Row> {
    let (ik, il) = (t.idx(key), t.idx("locale"));
    t.rows.iter().filter(|r| v(r, il) == "frFR").map(|r| (v(r, ik).to_string(), r)).collect()
}
fn index<'a>(t: &'a Table, key: &str) -> HashMap<String, &'a Row> {
    let ik = t.idx(key);
    t.rows.iter().map(|r| (v(r, ik).to_string(), r)).collect()
}

/// Complète une table locales_* CMaNGOS colonne par colonne. `pairs` : (colonne anglaise TBC, colonne loc2 TBC) ;
/// `ac_pairs` : (colonne anglaise AC, colonne française AC) dans le même ordre.
#[allow(clippy::too_many_arguments)]
fn simple_locale(
    t: &HashMap<String, Table>, ac: &HashMap<String, Table>, out: &mut Out, label: &str,
    base: &str, key: &str, pairs: &[(&str, &str)], loc: &str,
    ac_base: &str, ac_key: &str, ac_pairs: &[(&str, &str)], ac_loc: &str, ac_loc_key: &str,
) {
    let (bt, lt) = (&t[base], &t[loc]);
    let lidx = index(lt, "entry");
    let (abt, alt) = (&ac[ac_base], &ac[ac_loc]);
    let aidx = index(abt, ac_key);
    let afr = fr_index(alt, ac_loc_key);
    for (k, (en_col, loc_col)) in pairs.iter().enumerate() {
        let (ie, ien) = (bt.idx(key), bt.idx(en_col));
        let ilc = lt.idx(loc_col);
        let (iaen, iafr) = (abt.idx(ac_pairs[k].0), alt.idx(ac_pairs[k].1));
        let (mut refn, mut already, mut done) = (0, 0, 0);
        for r in &bt.rows {
            let en = norm(v(r, ien));
            if en.is_empty() || !en.chars().any(char::is_alphabetic) {
                continue;
            }
            refn += 1;
            let entry = v(r, ie);
            if lidx.get(entry).map(|lr| !empty(v(lr, ilc))).unwrap_or(false) {
                already += 1;
                continue;
            }
            if std::env::var("FRMISS").is_ok() {
                eprintln!("MISS\t{label}\t{en_col}\t{entry}\t{}", en.chars().take(80).collect::<String>());
            }
            let (Some(ar), Some(fr_row)) = (aidx.get(entry), afr.get(entry)) else { continue };
            let fr = v(fr_row, iafr);
            if std::env::var("FRDBG").is_ok() && norm(v(ar, iaen)) != en && !empty(fr) && done < 4000 && (entry.len() % 97 == 0 || entry.ends_with("07")) { eprintln!("DBG {en_col} {entry}
  TBC: {:?}
  AC : {:?}", &en.chars().take(160).collect::<String>(), &norm(v(ar, iaen)).chars().take(160).collect::<String>()); }
            let fr = v(fr_row, iafr);
            if loose(&norm(v(ar, iaen))) != loose(&en) || empty(fr) || !bmp_only(fr) || !probably_french(fr, &en) {
                continue;
            }
            out.sql.push(format!(
                "INSERT INTO `{loc}` (`entry`,`{loc_col}`) VALUES ({entry},{}) ON DUPLICATE KEY UPDATE `{loc_col}`=IF(COALESCE(`{loc_col}`,'')='',VALUES(`{loc_col}`),`{loc_col}`);",
                sql(fr)
            ));
            done += 1;
        }
        out.report.push(format!("{label} — {en_col} : {refn} textes de référence, {already} déjà en français, {done} complétés depuis AzerothCore, reste {} en anglais", refn - already - done));
    }
}

fn quests(t: &HashMap<String, Table>, ac: &HashMap<String, Table>, out: &mut Out) {
    let (qt, lq) = (&t["quest_template"], &t["locales_quest"]);
    let lidx = index(lq, "entry");
    let aq = index(&ac["quest_template"], "ID");
    let aqfr = fr_index(&ac["quest_template_locale"], "ID");
    let aor = index(&ac["quest_offer_reward"], "ID");
    let aorfr = fr_index(&ac["quest_offer_reward_locale"], "ID");
    let ari = index(&ac["quest_request_items"], "ID");
    let arifr = fr_index(&ac["quest_request_items_locale"], "ID");
    // (colonne TBC, table AC anglaise, colonne AC anglaise, table AC française, colonne AC française)
    let fields: [(&str, &str, &str, &str, &str); 10] = [
        ("Title", "quest_template", "LogTitle", "quest_template_locale", "Title"),
        ("Details", "quest_template", "QuestDescription", "quest_template_locale", "Details"),
        ("Objectives", "quest_template", "LogDescription", "quest_template_locale", "Objectives"),
        ("EndText", "quest_template", "AreaDescription", "quest_template_locale", "EndText"),
        ("ObjectiveText1", "quest_template", "ObjectiveText1", "quest_template_locale", "ObjectiveText1"),
        ("ObjectiveText2", "quest_template", "ObjectiveText2", "quest_template_locale", "ObjectiveText2"),
        ("ObjectiveText3", "quest_template", "ObjectiveText3", "quest_template_locale", "ObjectiveText3"),
        ("ObjectiveText4", "quest_template", "ObjectiveText4", "quest_template_locale", "ObjectiveText4"),
        ("OfferRewardText", "quest_offer_reward", "RewardText", "quest_offer_reward_locale", "RewardText"),
        ("RequestItemsText", "quest_request_items", "CompletionText", "quest_request_items_locale", "CompletionText"),
    ];
    for (col, aen_t, aen_c, afr_t, afr_c) in fields {
        let (ie, ien) = (qt.idx("entry"), qt.idx(col));
        let loc_col = format!("{col}_loc2");
        let ilc = lq.idx(&loc_col);
        let (aen_idx, afr_idx) = match aen_t {
            "quest_template" => (&aq, &aqfr),
            "quest_offer_reward" => (&aor, &aorfr),
            _ => (&ari, &arifr),
        };
        let (iaen, iafr) = (ac[aen_t].idx(aen_c), ac[afr_t].idx(afr_c));
        let (mut refn, mut already, mut done) = (0, 0, 0);
        for r in &qt.rows {
            let en = norm(v(r, ien));
            if en.is_empty() || !en.chars().any(char::is_alphabetic) {
                continue;
            }
            refn += 1;
            let entry = v(r, ie);
            if lidx.get(entry).map(|lr| !empty(v(lr, ilc))).unwrap_or(false) {
                already += 1;
                continue;
            }
            let (Some(ar), Some(fr_row)) = (aen_idx.get(entry), afr_idx.get(entry)) else { continue };
            let fr = v(fr_row, iafr);
            if std::env::var("FRDBG").is_ok() && norm(v(ar, iaen)) != en && !empty(fr) && entry.ends_with('7') {
                eprintln!("DBG {col} {entry}\n  TBC: {:?}\n  AC : {:?}", en.chars().take(170).collect::<String>(), norm(v(ar, iaen)).chars().take(170).collect::<String>());
            }
            if loose(&norm(v(ar, iaen))) != loose(&en) || empty(fr) || !bmp_only(fr) || !probably_french(fr, &en) {
                continue;
            }
            out.sql.push(format!(
                "INSERT INTO `locales_quest` (`entry`,`{loc_col}`) VALUES ({entry},{}) ON DUPLICATE KEY UPDATE `{loc_col}`=IF(COALESCE(`{loc_col}`,'')='',VALUES(`{loc_col}`),`{loc_col}`);",
                sql(fr)
            ));
            done += 1;
        }
        out.report.push(format!("quêtes — {col} : {refn} textes de référence, {already} déjà en français, {done} complétés depuis AzerothCore, reste {} en anglais", refn - already - done));
    }
}
