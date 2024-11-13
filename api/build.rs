fn main() {
    println!("cargo::rerun-if-env-changed=GITHUB_ACTIONS");

    let is_on_actions = std::env::var("GITHUB_ACTIONS")
        .ok()
        .and_then(|i| i.to_lowercase().parse::<bool>().ok())
        .unwrap_or(false);

    println!("cargo::rustc-check-cfg=cfg(on_ci)");
    if is_on_actions {
        println!("cargo::rustc-cfg=on_ci")
    }
}
