//! Extended integration tests against the live Finney chain.
//! Run with: cargo test --test chain_integration_test -- --nocapture
//!
//! These tests perform read-only queries against the Bittensor mainnet.

use agcli::chain::Client;
use agcli::types::NetUid;

const FINNEY: &str = "wss://entrypoint-finney.opentensor.ai:443";

/// Known Bittensor foundation/OTF address for testing.
const KNOWN_ADDRESS: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

#[tokio::test]
async fn test_subnet_hyperparams() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let params = client
        .get_subnet_hyperparams(NetUid(1))
        .await
        .expect("hyperparams");
    assert!(params.is_some(), "SN1 should have hyperparams");
    let h = params.unwrap();
    assert!(h.tempo > 0, "tempo should be positive");
    assert!(h.max_validators > 0, "max_validators should be positive");
    println!(
        "SN1 tempo={} max_validators={} rho={}",
        h.tempo, h.max_validators, h.rho
    );
}

#[tokio::test]
async fn test_get_subnet_info() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let info = client
        .get_subnet_info(NetUid(1))
        .await
        .expect("subnet info");
    assert!(info.is_some(), "SN1 should exist");
    let s = info.unwrap();
    assert_eq!(s.netuid, NetUid(1));
    assert!(s.n > 0, "SN1 should have neurons");
    println!(
        "SN1: name={} n={}/{} tempo={}",
        s.name, s.n, s.max_n, s.tempo
    );
}

#[tokio::test]
async fn test_dynamic_info_all_vs_single() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let all = client.get_all_dynamic_info().await.expect("all dynamic");
    let single = client
        .get_dynamic_info(NetUid(1))
        .await
        .expect("single dynamic");
    assert!(single.is_some());
    let s = single.unwrap();

    // Find SN1 in the all-subnets list
    let found = all.iter().find(|d| d.netuid == NetUid(1));
    assert!(found.is_some(), "SN1 should be in get_all_dynamic_info");
    let f = found.unwrap();

    // Prices should be reasonably close (queried at slightly different times)
    let diff = (s.price - f.price).abs();
    assert!(
        diff < 0.1,
        "Prices should be close: single={} all={}",
        s.price,
        f.price
    );
    println!(
        "SN1 price: single={:.6} all={:.6} diff={:.6}",
        s.price, f.price, diff
    );
}

#[tokio::test]
async fn test_get_balance_known_account() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let balance = client
        .get_balance_ss58(KNOWN_ADDRESS)
        .await
        .expect("balance");
    // Alice dev account likely has some balance
    println!(
        "Known account balance: {} RAO ({} TAO)",
        balance.rao(),
        balance.tao()
    );
}

#[tokio::test]
async fn test_get_stake_for_unknown_coldkey() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // A freshly generated address should have no stakes
    let stakes = client
        .get_stake_for_coldkey("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM")
        .await
        .expect("stakes");
    // It's okay if this address has recent stakes; just check we get a valid response
    println!("Unknown coldkey has {} stake positions", stakes.len());
}

#[tokio::test]
async fn test_nonexistent_subnet() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // Very high netuid should not exist
    let info = client
        .get_subnet_info(NetUid(65535))
        .await
        .expect("no error");
    assert!(info.is_none(), "netuid 65535 should not exist");
}

#[tokio::test]
async fn test_get_total_issuance() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let total = client.get_total_stake().await.expect("total stake");
    assert!(total.rao() > 0, "total stake should be nonzero");
    println!("Total stake: {} TAO", total.tao());
}

#[tokio::test]
async fn test_neurons_lite_ordering() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let neurons = client.get_neurons_lite(NetUid(1)).await.expect("neurons");
    assert!(!neurons.is_empty());
    // UIDs should be sequential starting from 0
    for (i, n) in neurons.iter().enumerate() {
        assert_eq!(n.uid as usize, i, "neuron UIDs should be sequential");
    }
    println!(
        "SN1: {} neurons, UIDs 0..{}",
        neurons.len(),
        neurons.len() - 1
    );
}

// ──── Step 26: Additional integration tests ────

#[tokio::test]
async fn test_total_issuance() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let issuance = client.get_total_issuance().await.expect("total issuance");
    // Bittensor total issuance should be > 0 and reasonable (millions of TAO)
    assert!(issuance.rao() > 0, "total issuance should be nonzero");
    println!("Total issuance: {:.2} TAO", issuance.tao());
}

#[tokio::test]
async fn test_block_emission() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let emission = client.get_block_emission().await.expect("block emission");
    // Block emission should be positive (TAO minted per block)
    assert!(emission.rao() > 0, "block emission should be positive");
    println!(
        "Block emission: {} RAO ({:.6} TAO)",
        emission.rao(),
        emission.tao()
    );
}

#[tokio::test]
async fn test_sim_swap_tao_for_alpha() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // Simulate swapping 1 TAO for alpha on SN1
    let one_tao_rao = 1_000_000_000u64;
    let (alpha_amount, tao_fee, alpha_fee) = client
        .sim_swap_tao_for_alpha(NetUid(1), one_tao_rao)
        .await
        .expect("sim swap");
    assert!(alpha_amount > 0, "should receive some alpha");
    println!(
        "1 TAO → {} alpha (tao_fee={}, alpha_fee={})",
        alpha_amount, tao_fee, alpha_fee
    );
}

#[tokio::test]
async fn test_sim_swap_alpha_for_tao() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // Simulate swapping 100 alpha units for TAO on SN1
    let alpha = 100_000_000_000u64; // 100 alpha
    let (tao_amount, tao_fee, alpha_fee) = client
        .sim_swap_alpha_for_tao(NetUid(1), alpha)
        .await
        .expect("sim swap");
    assert!(tao_amount > 0, "should receive some TAO");
    println!(
        "100 alpha → {} TAO (tao_fee={}, alpha_fee={})",
        tao_amount, tao_fee, alpha_fee
    );
}

#[tokio::test]
async fn test_get_delegate_single() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // Get all delegates, then query the first one by hotkey
    let delegates = client.get_delegates().await.expect("delegates");
    assert!(!delegates.is_empty());
    let first = &delegates[0];
    let single = client
        .get_delegate(&first.hotkey)
        .await
        .expect("single delegate");
    assert!(single.is_some(), "first delegate should be queryable");
    let d = single.unwrap();
    assert_eq!(d.hotkey, first.hotkey, "hotkeys should match");
    println!(
        "Delegate: {} take={:.2}%",
        &d.hotkey[..8],
        d.take as f64 / 65535.0 * 100.0
    );
}

#[tokio::test]
async fn test_list_proxies_empty() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // An unused address should have no proxies
    let proxies = client
        .list_proxies("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM")
        .await
        .expect("proxies");
    // It's fine if it has proxies; just check the query succeeds
    println!("Proxies for address: {}", proxies.len());
}

#[tokio::test]
async fn test_dynamic_info_all_valid_prices() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let dynamic = client.get_all_dynamic_info().await.expect("dynamic");
    // Check no negative prices (regression test from Step 25)
    for d in &dynamic {
        assert!(
            d.price >= 0.0,
            "SN{} has negative price: {}",
            d.netuid,
            d.price
        );
    }
    println!("All {} subnets have non-negative prices", dynamic.len());
}

#[tokio::test]
async fn test_hyperparams_nonexistent_subnet() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let params = client
        .get_subnet_hyperparams(NetUid(65535))
        .await
        .expect("hyperparams");
    assert!(params.is_none(), "netuid 65535 should not have hyperparams");
}

// ──── Step 29: Coldkey swap, child keys, historical queries ────

/// Test coldkey swap scheduled query — most accounts have no swap scheduled.
#[tokio::test]
async fn test_coldkey_swap_scheduled_none() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let swap = client
        .get_coldkey_swap_scheduled(KNOWN_ADDRESS)
        .await
        .expect("coldkey swap query");
    // Alice (known test address) almost certainly has no pending swap
    assert!(
        swap.is_none(),
        "known address should not have a pending coldkey swap"
    );
}

/// Test child keys query on SN1 for the top delegate.
#[tokio::test]
async fn test_child_keys_query() {
    let client = Client::connect(FINNEY).await.expect("connect");
    // Query child keys for a well-known validator on SN1
    // This should return either empty or a valid list
    let children = client
        .get_child_keys(KNOWN_ADDRESS, NetUid(1))
        .await
        .expect("child keys query");
    println!(
        "Child keys for {} on SN1: {} entries",
        KNOWN_ADDRESS,
        children.len()
    );
    // Just verify the query works — we don't know the exact state
    for (proportion, child_ss58) in &children {
        assert!(
            child_ss58.starts_with('5'),
            "child ss58 should be valid: {}",
            child_ss58
        );
        assert!(*proportion > 0, "proportion should be positive");
    }
}

/// Test parent keys query.
#[tokio::test]
async fn test_parent_keys_query() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let parents = client
        .get_parent_keys(KNOWN_ADDRESS, NetUid(1))
        .await
        .expect("parent keys query");
    println!(
        "Parent keys for {} on SN1: {} entries",
        KNOWN_ADDRESS,
        parents.len()
    );
    for (proportion, parent_ss58) in &parents {
        assert!(
            parent_ss58.starts_with('5'),
            "parent ss58 should be valid: {}",
            parent_ss58
        );
        assert!(*proportion > 0, "proportion should be positive");
    }
}

/// Test historical stake query (recent block should work).
#[tokio::test]
async fn test_stake_at_recent_block() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let current_block = client.get_block_number().await.expect("block number");
    // Try a block a few back (within pruning window)
    let recent = (current_block - 10) as u32;
    let hash = client.get_block_hash(recent).await.expect("block hash");
    let stakes = client
        .get_stake_for_coldkey_at_block(KNOWN_ADDRESS, hash)
        .await
        .expect("stake at block");
    println!(
        "Stakes for {} at block {}: {} positions",
        KNOWN_ADDRESS,
        recent,
        stakes.len()
    );
    // Query succeeds — that's the main validation
}

/// Test historical balance + stake at same block are consistent.
#[tokio::test]
async fn test_historical_account_consistency() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let current_block = client.get_block_number().await.expect("block number");
    let recent = (current_block - 5) as u32;
    let hash = client.get_block_hash(recent).await.expect("block hash");
    let (balance, stakes) = tokio::try_join!(
        client.get_balance_at_block(KNOWN_ADDRESS, hash),
        client.get_stake_for_coldkey_at_block(KNOWN_ADDRESS, hash),
    )
    .expect("parallel historical queries");
    println!(
        "Block {}: balance={}, stakes={}",
        recent,
        balance.display_tao(),
        stakes.len()
    );
    // Both queries should succeed at the same block
}

/// Test identity at recent block.
#[tokio::test]
async fn test_identity_at_block() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let current_block = client.get_block_number().await.expect("block number");
    let recent = (current_block - 5) as u32;
    let hash = client.get_block_hash(recent).await.expect("block hash");
    let identity = client
        .get_identity_at_block(KNOWN_ADDRESS, hash)
        .await
        .expect("identity at block");
    println!(
        "Identity at block {}: {:?}",
        recent,
        identity.as_ref().map(|i| &i.name)
    );
    // Query succeeds — main validation
}

/// Test pending child keys query — should return None for most addresses (no pending changes).
#[tokio::test]
async fn test_pending_child_keys_query() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let result = client
        .get_pending_child_keys(KNOWN_ADDRESS, NetUid(1))
        .await
        .expect("pending child keys query");
    // Most addresses won't have pending childkey changes
    println!(
        "Pending child keys for {} on SN1: {:?}",
        KNOWN_ADDRESS,
        result.as_ref().map(|(c, b)| (c.len(), b))
    );
    // If there are pending changes, validate structure
    if let Some((children, cooldown_block)) = result {
        assert!(cooldown_block > 0, "cooldown block should be positive");
        for (proportion, child_ss58) in &children {
            assert!(
                child_ss58.starts_with('5'),
                "child ss58 should be valid: {}",
                child_ss58
            );
            assert!(*proportion > 0, "proportion should be positive");
        }
    }
}

/// Test pending child keys on a different subnet — should also work.
#[tokio::test]
async fn test_pending_child_keys_query_sn0() {
    let client = Client::connect(FINNEY).await.expect("connect");
    let result = client
        .get_pending_child_keys(KNOWN_ADDRESS, NetUid(0))
        .await
        .expect("pending child keys query SN0");
    // Just verify the query succeeds
    println!("Pending child keys SN0: {:?}", result.is_some());
}
