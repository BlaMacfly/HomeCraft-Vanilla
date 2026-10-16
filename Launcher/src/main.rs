//! HomeCraft Edition Vanilla — launcher (Rust, egui). © 2026 BlaMacfly. Licence propriétaire : voir LICENCE.txt.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod addons;
mod backup;
mod botgen;
mod client;
mod conf_catalog;
mod credentials;
mod db;
mod i18n;
mod mpqbuild;
mod orchestrator;
mod patches;
mod paths;
mod repair;
mod server;
mod settings;
mod ui;
mod util;

fn main() -> eframe::Result {
    let layout = paths::Layout::detect();
    let settings = settings::Settings::load(&layout.settings_file());
    let (shared, tx) = orchestrator::spawn(layout.clone(), settings.clone());

    // Options de test (Tests/) : --start démarre les services immédiatement, --quit-after N ferme après N secondes.
    let args: Vec<String> = std::env::args().collect();
    let auto_start = args.iter().any(|a| a == "--start");
    let quit_after = args.iter().position(|a| a == "--quit-after").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<u64>().ok());
    let start_tab = args.iter().position(|a| a == "--tab").and_then(|i| args.get(i + 1)).cloned();
    if auto_start {
        let _ = tx.send(orchestrator::Command::StartAll);
    }
    // --do <action> [argument] : backup | verify | repair | restore <zip> | diagnostic | reimportworld | mmaps … (tests d'intégration).
    if let Some(i) = args.iter().position(|a| a == "--do") {
        let cmd = match (args.get(i + 1).map(String::as_str), args.get(i + 2)) {
            (Some("backup"), _) => Some(orchestrator::Command::Backup("test automatique".into())),
            (Some("verify"), _) => Some(orchestrator::Command::Verify),
            (Some("repair"), _) => Some(orchestrator::Command::Repair),
            (Some("diagnostic"), _) => Some(orchestrator::Command::ExportDiagnostic),
            (Some("restore"), Some(p)) => Some(orchestrator::Command::Restore(std::path::PathBuf::from(p))),
            (Some("botgen"), Some(n)) => n.parse().ok().map(orchestrator::Command::GenerateBots),
            (Some("botdel"), _) => Some(orchestrator::Command::DeleteBots),
            (Some("patchoff"), Some(n)) => Some(orchestrator::Command::PatchSetActive(n.clone(), false)),
            (Some("patchon"), Some(n)) => Some(orchestrator::Command::PatchSetActive(n.clone(), true)),
            (Some("addonadd"), Some(p)) => Some(orchestrator::Command::AddonAdd(std::path::PathBuf::from(p))),
            (Some("addonoff"), Some(n)) => Some(orchestrator::Command::AddonSetActive(n.clone(), false)),
            (Some("addonon"), Some(n)) => Some(orchestrator::Command::AddonSetActive(n.clone(), true)),
            (Some("addondel"), Some(n)) => Some(orchestrator::Command::AddonRemove(n.clone())),
            (Some("accdel"), Some(n)) => Some(orchestrator::Command::DeleteAccount(n.clone())),
            (Some("mpqbuild"), Some(p)) => Some(orchestrator::Command::PatchBuild(std::path::PathBuf::from(p))),
            (Some("reimportworld"), _) => Some(orchestrator::Command::ReimportWorld),
            (Some("mmaps"), _) => Some(orchestrator::Command::GenerateMmaps),
            _ => None,
        };
        if let Some(c) = cmd {
            let _ = tx.send(c);
        }
    }
    // --install-to <dossier> : copie depuis un support en lecture seule sans passer par le sélecteur (tests).
    if let Some(i) = args.iter().position(|a| a == "--install-to") {
        if let Some(p) = args.get(i + 1) {
            let _ = tx.send(orchestrator::Command::InstallTo(std::path::PathBuf::from(p)));
        }
    }
    let mut viewport = eframe::egui::ViewportBuilder::default().with_inner_size([980.0, 700.0]).with_min_inner_size([760.0, 520.0]).with_title(i18n::fr::WINDOW_TITLE);
    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/logo-256.png")) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        viewport = viewport.with_icon(std::sync::Arc::new(eframe::egui::IconData { rgba: rgba.into_raw(), width, height }));
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    let tx_exit = tx.clone();
    let result = eframe::run_native(
        i18n::fr::WINDOW_TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(ui::App::new(cc, layout, settings, shared, tx, quit_after, start_tab)))),
    );
    // Fermeture : l'orchestrateur arrête les services qu'il a démarrés.
    let _ = tx_exit.send(orchestrator::Command::Shutdown);
    std::thread::sleep(std::time::Duration::from_millis(200));
    result
}
