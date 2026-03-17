use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::{env, process::Command};

use parity_scale_codec::Decode;
use subxt_codegen::CodegenBuilder;
use subxt_metadata::Metadata;
use subxt_utils_fetchmetadata::{self as fetch_metadata, MetadataVersion};

#[tokio::main]
async fn main() {
    let endpoint = env::var_os("METADATA_CHAIN_ENDPOINT")
        .map(|s| s.into_string().unwrap())
        .unwrap_or("wss://entrypoint-finney.opentensor.ai:443".into());

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let metadata_path = Path::new(&out_dir).join("metadata.rs");

    // If metadata already exists and SKIP_METADATA_FETCH is set, reuse it
    if metadata_path.exists() && env::var("SKIP_METADATA_FETCH").is_ok() {
        eprintln!("agcli: reusing cached metadata (SKIP_METADATA_FETCH set)");
        return;
    }

    eprintln!("agcli: fetching chain metadata from {endpoint}...");

    // Try V15 first (subxt 0.38 compatible), fall back to V14
    let url = endpoint.as_str().try_into().unwrap();
    let fetch_result = match fetch_metadata::from_url(url, MetadataVersion::Version(15)).await {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            eprintln!("agcli: V15 failed ({e}), trying V14...");
            let url = endpoint.as_str().try_into().unwrap();
            fetch_metadata::from_url(url, MetadataVersion::Version(14)).await
        }
    };

    let metadata_bytes = match fetch_result {
        Ok(bytes) => bytes,
        Err(e) => {
            // If fetch fails but we have cached metadata, reuse it
            if metadata_path.exists() {
                eprintln!(
                    "agcli: metadata fetch failed ({e}), reusing cached metadata at {}",
                    metadata_path.display()
                );
                return;
            }
            panic!("Failed to fetch metadata and no cache available: {e}");
        }
    };

    let mut slice: &[u8] = &metadata_bytes;
    let metadata = Metadata::decode(&mut slice).unwrap();

    let codegen = CodegenBuilder::new();
    let code = codegen.generate(metadata).unwrap();

    // Try to format with rustfmt; if not available, write directly
    match Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(process) => {
            write!(process.stdin.as_ref().unwrap(), "{code}").unwrap();
            let output = process.wait_with_output().unwrap();
            std::fs::write(&metadata_path, &output.stdout).unwrap();
        }
        Err(_) => {
            let mut file = File::create(&metadata_path).unwrap();
            write!(file, "{code}").unwrap();
        }
    }

    eprintln!(
        "agcli: metadata codegen complete → {}",
        metadata_path.display()
    );
}
