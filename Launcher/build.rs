//! Ressources Windows de l'exécutable : icône (affichée par l'Explorateur, la barre des tâches, les raccourcis)
//! et informations de version. L'icône est générée par Build\New-InstallerImages.ps1 depuis assets\logo-512.png.

fn main() {
    println!("cargo:rerun-if-changed=assets/homecraft.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/homecraft.ico");
        res.set("ProductName", "HomeCraft Edition BC");
        res.set("FileDescription", "HomeCraft Edition BC");
        res.set("CompanyName", "BlaMacfly");
        res.set("LegalCopyright", "© 2026 BlaMacfly. Tous droits réservés.");
        res.set("OriginalFilename", "HomeCraft Edition BC.exe");
        res.set_language(0x040c); // français (France)
        res.compile().expect("ressources Windows (winresource : rc.exe du SDK Windows requis)");
    }
}
