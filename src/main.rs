use apt_parser::Packages;
use indicatif::ProgressBar;
use rust_apt::{
    new_cache,
    progress::{AcquireProgress, InstallProgress},
};
use std::{collections::HashSet, path::Path};
use surf::get;
use tempdir::TempDir;

#[tokio::main]
async fn main() {
    let cosmic_packages = HashSet::from([
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

    let url = if cfg!(target_arch = "x86_64") {
        "https://apt-origin.pop-os.org/release/dists/noble/main/binary-amd64/Packages"
    } else if cfg!(target_arch = "aarch64") {
        "https://apt-origin.pop-os.org/release/dists/noble/main/binary-arm64/Packages"
    } else {
        panic!("Architecture not suported");
    };

    let data = get(url)
        .await
        .expect("Failed to get Packages file")
        .body_string()
        .await
        .expect("Failed to read Packages file");

    let packages = Packages::from(&data);
    // let tempdir = TempDir::new("install_cosmic").expect("Failed to create tempdir");

    let tempdir = Path::new("/tmp/install_cosmic");
    std::fs::create_dir(tempdir).expect("Failed to create dir");
    let mut urls = vec![];
    let pb = ProgressBar::new(cosmic_packages.len() as u64);
    pb.inc(0);
    for package in packages {
        if cosmic_packages.contains(package.package.as_str()) {
            let file = tempdir.join(format!("{}.deb", package.package));
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
            )
            .expect("Failed to download deb");
            pb.inc(1);
        }
    }

    let cache = new_cache!(&urls).expect("Failed to load apt cache");
    for package in cosmic_packages {
        let pkg = cache.get(&package).expect("Failed to get package");
        pkg.mark_install(true, true);
        pkg.protect();
    }
    if let Err(err) = cache.resolve(true) {
        panic!("Failed to resolve: {err}")
    }

    let mut acquire_progress = AcquireProgress::apt();
    let mut install_progress = InstallProgress::apt();
    cache
        .commit(&mut acquire_progress, &mut install_progress)
        .expect("Failed do install");
}
