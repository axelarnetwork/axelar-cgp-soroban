use std::{env, path::Path, process::Command};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    dbg!(&out_dir);
    // let out_dir = env!("CARGO_MANIFEST_DIR");
    let dir_name = "_artefacts/";
    let out_dir = Path::new(&out_dir).join(dir_name);

    let old_ver = "0.1.0";
    let new_ver = "0.1.1";

    let old_contract_path = Path::new(&out_dir).join("axelar_gateway_old.wasm");
    let new_contract_path = Path::new(&out_dir).join("axelar_gateway_new.wasm");
    let axelar_gatway_path = Path::new(&out_dir).join("axelar_gateway.wasm");

    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", old_ver);
    let mut command_name = Command::new("stellar");
    let command = command_name.args([
        "contract",
        "build",
        "--package",
        "axelar-gateway",
        "--no-cache",
        "--out-dir",
        dir_name,
    ]);

    let _status = command.status().unwrap();

    std::fs::rename(&axelar_gatway_path, old_contract_path).unwrap();

    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", new_ver);
    let _status = command.status().unwrap();
    std::fs::rename(&axelar_gatway_path, new_contract_path).unwrap();
}
