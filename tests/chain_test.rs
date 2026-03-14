//! Integration test: connect to finney and query real chain data.
//! Run with: cargo test --test chain_test -- --nocapture

use agcli::chain::Client;
use agcli::types::NetUid;

const FINNEY: &str = "wss://entrypoint-finney.opentensor.ai:443";

#[tokio::test]
async fn test_connect_and_block_number() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let block = client.get_block_number().await.expect("block number");
    assert!(
        block > 1_000_000,
        "finney should be past block 1M, got {block}"
    );
    println!("Current block: {block}");
}

#[tokio::test]
async fn test_total_stake() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let stake = client.get_total_stake().await.expect("total stake");
    println!("Total stake: {stake}");
    assert!(stake.rao() > 0, "total stake should be nonzero");
}

#[tokio::test]
async fn test_total_networks() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let n = client.get_total_networks().await.expect("total networks");
    println!("Total networks: {n}");
    assert!(n > 50, "finney should have >50 subnets, got {n}");
}

#[tokio::test]
async fn test_get_balance() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // Query a known address (opentensor foundation)
    let balance = client
        .get_balance_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
        .await
        .expect("balance");
    println!("Alice balance: {balance}");
}

#[tokio::test]
async fn test_get_all_subnets() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let subnets = client.get_all_subnets().await.expect("subnets");
    println!("Got {} subnets", subnets.len());
    assert!(!subnets.is_empty(), "should have subnets");
    for s in subnets.iter().take(5) {
        println!(
            "  {} n={} tempo={} owner={}",
            s.name,
            s.n,
            s.tempo,
            &s.owner[..8]
        );
    }
}

#[tokio::test]
async fn test_get_neurons_lite() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let neurons = client.get_neurons_lite(NetUid(1)).await.expect("neurons");
    println!("SN1 neurons: {}", neurons.len());
    assert!(!neurons.is_empty(), "SN1 should have neurons");
    let first = &neurons[0];
    println!(
        "  UID={} hotkey={} stake={} rank={:.4}",
        first.uid,
        &first.hotkey[..8],
        first.stake,
        first.rank
    );
}

#[tokio::test]
async fn test_get_delegates() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let delegates = client.get_delegates().await.expect("delegates");
    println!("Got {} delegates", delegates.len());
    assert!(!delegates.is_empty(), "should have delegates");
}

#[tokio::test]
async fn test_get_stake_for_coldkey() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let stakes = client
        .get_stake_for_coldkey("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
        .await
        .expect("stakes");
    println!("Stakes for Alice: {}", stakes.len());
}

#[tokio::test]
async fn test_get_all_dynamic_info() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let dynamic = client.get_all_dynamic_info().await.expect("dynamic info");
    println!("Got {} dynamic subnet infos", dynamic.len());
    assert!(!dynamic.is_empty(), "should have dynamic info");
    for d in dynamic.iter().take(5) {
        println!(
            "  SN{} \"{}\" price={:.6} tao_in={} alpha_in={} alpha_out={}",
            d.netuid, d.name, d.price, d.tao_in, d.alpha_in, d.alpha_out
        );
    }
}

#[tokio::test]
async fn test_get_dynamic_info_single() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let dynamic = client
        .get_dynamic_info(NetUid(1))
        .await
        .expect("dynamic info");
    assert!(dynamic.is_some(), "SN1 should have dynamic info");
    let d = dynamic.unwrap();
    println!(
        "SN1: \"{}\" symbol={} price={:.6}",
        d.name, d.symbol, d.price
    );
    assert!(d.price > 0.0, "SN1 price should be positive");
}
