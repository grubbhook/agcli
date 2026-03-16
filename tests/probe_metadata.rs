//! Metadata probe — dumps all calls and storage items for key pallets.
//! Run: cargo test --test probe_metadata -- --nocapture

use agcli::chain::Client;

const LOCAL_WS: &str = "ws://127.0.0.1:9944";

#[tokio::test]
async fn probe_runtime_metadata() {
    let client = Client::connect(LOCAL_WS)
        .await
        .expect("connect to localnet");
    let metadata = client.metadata();

    println!("\n═══ Runtime Metadata Probe ═══\n");

    // List all pallets
    println!("── ALL PALLETS ──");
    for pallet in metadata.pallets() {
        let call_count = match pallet.call_variants() {
            Some(v) => v.len(),
            None => 0,
        };
        let storage_count = match pallet.storage() {
            Some(s) => s.entries().len(),
            None => 0,
        };
        if call_count > 0 || storage_count > 0 {
            println!(
                "  [{}] {} — {} calls, {} storage",
                pallet.index(),
                pallet.name(),
                call_count,
                storage_count
            );
        }
    }

    // Target pallets to detail
    let targets = [
        "AdminUtils",
        "SubtensorModule",
        "Crowdloan",
        "Sudo",
        "Scheduler",
        "Preimage",
        "Multisig",
        "Contracts",
        "Proxy",
        "Utility",
        "EVM",
        "Ethereum",
        "SafeMode",
        "Drand",
        "Swap",
    ];

    println!("\n── TARGET PALLET DETAILS ──\n");
    for name in &targets {
        match metadata.pallet_by_name(name) {
            Some(pallet) => {
                println!("✓ {} (index {})", name, pallet.index());
                if let Some(variants) = pallet.call_variants() {
                    println!("  Calls ({}):", variants.len());
                    for v in variants {
                        let fields: Vec<String> = v
                            .fields
                            .iter()
                            .map(|f| f.name.as_deref().unwrap_or("?").to_string())
                            .collect();
                        println!("    - {}({})", v.name, fields.join(", "));
                    }
                }
                if let Some(storage) = pallet.storage() {
                    println!("  Storage ({}):", storage.entries().len());
                    for entry in storage.entries() {
                        println!("    - {}", entry.name());
                    }
                }
                println!();
            }
            None => {
                println!("✗ {} — NOT FOUND", name);
            }
        }
    }

    // Search for subtoken-related items across all pallets
    println!("\n── SUBTOKEN/STAKE RELATED CALLS (all pallets) ──");
    for pallet in metadata.pallets() {
        if let Some(variants) = pallet.call_variants() {
            for v in variants {
                let nl = v.name.to_lowercase();
                if nl.contains("subtoken") || nl.contains("token_enabled") {
                    let fields: Vec<&str> = v
                        .fields
                        .iter()
                        .map(|f| f.name.as_deref().unwrap_or("?"))
                        .collect();
                    println!("  {}.{}({})", pallet.name(), v.name, fields.join(", "));
                }
            }
        }
    }

    println!("\n── SUBTOKEN STORAGE ITEMS ──");
    for pallet in metadata.pallets() {
        if let Some(storage) = pallet.storage() {
            for entry in storage.entries() {
                let nl = entry.name().to_lowercase();
                if nl.contains("subtoken") || nl.contains("token_enabled") {
                    println!("  {}.{}", pallet.name(), entry.name());
                }
            }
        }
    }

    // Dissolve-related
    println!("\n── DISSOLVE RELATED CALLS ──");
    for pallet in metadata.pallets() {
        if let Some(variants) = pallet.call_variants() {
            for v in variants {
                if v.name.to_lowercase().contains("dissolve") {
                    let fields: Vec<&str> = v
                        .fields
                        .iter()
                        .map(|f| f.name.as_deref().unwrap_or("?"))
                        .collect();
                    println!("  {}.{}({})", pallet.name(), v.name, fields.join(", "));
                }
            }
        }
    }

    // Coldkey/swap related
    println!("\n── COLDKEY/SWAP RELATED CALLS ──");
    for pallet in metadata.pallets() {
        if let Some(variants) = pallet.call_variants() {
            for v in variants {
                let nl = v.name.to_lowercase();
                if nl.contains("coldkey")
                    || nl.contains("swap_hotkey")
                    || nl.contains("schedule_swap")
                {
                    let fields: Vec<&str> = v
                        .fields
                        .iter()
                        .map(|f| f.name.as_deref().unwrap_or("?"))
                        .collect();
                    println!("  {}.{}({})", pallet.name(), v.name, fields.join(", "));
                }
            }
        }
    }

    println!("\n═══ Probe Complete ═══\n");
}
