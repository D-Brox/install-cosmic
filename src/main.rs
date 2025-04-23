use apt_parser::Packages;
use rust_apt::{
    new_cache,
    progress::{AcquireProgress, InstallProgress},
};
use std::collections::HashSet;
use surf::get;
use tempdir::TempDir;

#[tokio::main]
async fn main() {
    let mut cosmic_packages = HashSet::from([
        "cosmic-app-library",
        "cosmic-applets",
        "cosmic-bg",
        "cosmic-comp",
        "cosmic-edit",
        "cosmic-files",
        "cosmic-greeter",
        "cosmic-greeter-daemon",
        "cosmic-icons",
        "cosmic-idle",
        "cosmic-launcher",
        "cosmic-notifications",
        "cosmic-osd",
        "cosmic-panel",
        "cosmic-player",
        "cosmic-randr",
        "cosmic-screenshot",
        "cosmic-session",
        "cosmic-settings",
        "cosmic-settings-daemon",
        "cosmic-store",
        "cosmic-term",
        "cosmic-wallpapers",
        "cosmic-workspaces",
        "pop-fonts",
        "pop-icon-theme",
        "pop-launcher",
        "pop-sound-theme",
        "xdg-desktop-portal-cosmic",
    ]);
    let data = get("https://apt-origin.pop-os.org/release/dists/noble/main/binary-amd64/Packages")
        .await
        .expect("Failed to get Packages file")
        .body_string()
        .await
        .expect("Failed to read Packages file");

    let packages = Packages::from(&data);
    let tempdir = TempDir::new("install_cosmic").expect("Failed to create tempdir");
    let mut urls = vec![];
    for package in packages {
        if cosmic_packages.contains(package.package.as_str()) {
            let file = tempdir.path().join(format!("{}.deb", package.package));
            urls.push(file.to_str().unwrap().to_string());
            std::fs::write(
                file,
                get(format!(
                    "https://apt-origin.pop-os.org/release/{}",
                    package.filename
                ))
                .await
                .expect("Failed to get deb")
                .body_bytes()
                .await
                .expect("Failed to read deb"),
            ).expect("Failed to download deb");
        }
    }

    let cache = new_cache!(&urls).expect("Failed to load apt cache");
    for package in cosmic_packages {
        let pkg = cache.get(&package).expect("Failed to get package");
        pkg.mark_install(true, true);
        pkg.protect();
    }
    if let Err(err) = cache.resolve(false) {
        panic!("Failed to resolve: {err}")
    }

    let mut acquire_progress = AcquireProgress::apt();
    let mut install_progress = InstallProgress::apt();
    cache
        .commit(&mut acquire_progress, &mut install_progress)
        .expect("Failed do install");
}
