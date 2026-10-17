//! Couverture frFR d'un dump mysqldump : pour chaque table, nombre de lignes, et pour chaque colonne *_loc2
//! (ou table *_locale avec colonne locale='frFR'), nombre de valeurs non vides. Sortie TSV.
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};

fn parse_tuples(s: &[u8], mut f: impl FnMut(Vec<Option<String>>)) {
    let mut i = 0;
    while i < s.len() {
        if s[i] != b'(' { i += 1; continue; }
        i += 1;
        let mut row = Vec::new();
        loop {
            while i < s.len() && s[i] == b' ' { i += 1; }
            if s[i] == b'\'' {
                i += 1; let mut v = Vec::new();
                while i < s.len() {
                    match s[i] {
                        92u8 => { let c = s[i + 1]; v.push(match c { 110u8 => 10u8, 114u8 => 13u8, 116u8 => 9u8, 48u8 => 0u8, _ => c }); i += 2; }
                        b'\'' if i + 1 < s.len() && s[i + 1] == b'\'' => { v.push(b'\''); i += 2; }
                        b'\'' => { i += 1; break; }
                        c => { v.push(c); i += 1; }
                    }
                }
                row.push(Some(String::from_utf8_lossy(&v).into_owned()));
            } else {
                let st = i;
                while i < s.len() && s[i] != b',' && s[i] != b')' { i += 1; }
                let t = String::from_utf8_lossy(&s[st..i]).trim().to_string();
                row.push(if t == "NULL" { None } else { Some(t) });
            }
            if i >= s.len() { break; }
            if s[i] == b',' { i += 1; continue; }
            if s[i] == b')' { i += 1; break; }
        }
        f(row);
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("dump");
    let r = BufReader::with_capacity(1 << 20, std::fs::File::open(&path).unwrap());
    let mut cols: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut cur: Option<String> = None;
    let mut rows: BTreeMap<String, u64> = BTreeMap::new();
    let mut filled: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut locale_fr: BTreeMap<String, u64> = BTreeMap::new();
    for line in r.split(b'\n') {
        let line = line.unwrap();
        if line.starts_with(b"CREATE TABLE `") {
            let n = String::from_utf8_lossy(&line[14..]).split('`').next().unwrap().to_string();
            cols.insert(n.clone(), Vec::new()); cur = Some(n);
        } else if let Some(t) = cur.clone() {
            if line.starts_with(b"  `") { let c = String::from_utf8_lossy(&line[3..]).split('`').next().unwrap().to_string(); cols.get_mut(&t).unwrap().push(c); }
            else if line.starts_with(b")") { cur = None; }
        }
        if line.starts_with(b"INSERT INTO `") {
            let t = String::from_utf8_lossy(&line[13..]).split('`').next().unwrap().to_string();
            let cs = cols.get(&t).cloned().unwrap_or_default();
            let pos = line.windows(7).position(|w| w == b"VALUES ").unwrap() + 7;
            let li = cs.iter().position(|c| c.eq_ignore_ascii_case("locale"));
            parse_tuples(&line[pos..], |row| {
                *rows.entry(t.clone()).or_default() += 1;
                for (k, c) in cs.iter().enumerate() {
                    if c.ends_with("_loc2") || c == "text_loc2" {
                        if let Some(Some(v)) = row.get(k) { if !v.trim().is_empty() { *filled.entry((t.clone(), c.clone())).or_default() += 1; } }
                    }
                }
                if ["quest_template", "script_texts", "gossip_menu_option", "questgiver_greeting", "trainer_greeting", "areatrigger_teleport", "item_template", "page_text", "gossip_texts", "custom_texts"].contains(&t.as_str()) {
                    for (k, c) in cs.iter().enumerate() {
                        if let Some(Some(v)) = row.get(k) { if (!v.trim().is_empty() && v.chars().any(|ch| ch.is_alphabetic())) || (c.contains("broadcast") && v.parse::<i64>().map(|n| n > 0).unwrap_or(false)) { *filled.entry((t.clone(), format!("~{c}"))).or_default() += 1; } }
                    }
                }
                if let Some(li) = li { if let Some(Some(v)) = row.get(li) { if v == "frFR" { *locale_fr.entry(t.clone()).or_default() += 1; } } }
            });
        }
    }
    for (t, n) in &rows { println!("ROWS\t{t}\t{n}"); }
    for ((t, c), n) in &filled { println!("LOC2\t{t}\t{c}\t{n}"); }
    for (t, n) in &locale_fr { println!("FRFR\t{t}\t{n}"); }
}
