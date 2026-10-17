//! Ressources Windows de l'exécutable : icône (Explorateur, barre des tâches, raccourcis) et informations de version.
//! L'icône est générée par Build\New-LauncherImages.ps1 depuis assets\HomeCraft-WoTLK-original.png.

fn main() {
    println!("cargo:rerun-if-changed=assets/homecraft.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/homecraft.ico");
        res.set("ProductName", "HomeCraft Edition WoTLK");
        res.set("FileDescription", "HomeCraft Edition WoTLK");
        res.set("CompanyName", "BlaMacfly");
        res.set("LegalCopyright", "© 2026 BlaMacfly. Tous droits réservés.");
        res.set("OriginalFilename", "HomeCraft Edition WoTLK.exe");
        res.set_language(0x040c); // français (France)
        res.compile().expect("ressources Windows (winresource : rc.exe du SDK Windows requis)");
    }
}
