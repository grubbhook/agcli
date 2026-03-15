//! Extended integration tests against the live Finney chain.
//! Run with: cargo test --test chain_integration_test -- --nocapture
//!
//! All assertions run sequentially within a single tokio runtime,
//! sharing one WebSocket connection to avoid rate-limiting and
//! cross-runtime connection issues.

use agcli::chain::Client;
use agcli::types::NetUid;

const FINNEY: &str = "wss://entrypoint-finney.opentensor.ai:443";

/// Known Bittensor foundation/OTF address for testing.
const KNOWN_ADDRESS: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

#[tokio::test]
async fn extended_chain_queries() {
    let client = Client::connect(FINNEY).await.expect("connect to finney");

    // ── subnet hyperparams ──
    let params = client
        .get_subnet_hyperparams(NetUid(1))
        .await
        .expect("hyperparams")
        .expect("SN1 should have hyperparams");
    assert!(params.tempo > 0, "tempo should be positive");
    assert!(
        params.max_validators > 0,
        "max_validators should be positive"
    );
    println!(
        "[ok] SN1 tempo={} max_validators={} rho={}",
        params.tempo, params.max_validators, params.rho
    );

    // ── subnet info ──
    let s = client
        .get_subnet_info(NetUid(1))
        .await
        .expect("subnet info")
        .expect("SN1 should exist");
    assert_eq!(s.netuid, NetUid(1));
    assert!(s.n > 0, "SN1 should have neurons");
    println!(
        "[ok] SN1: name={} n={}/{} tempo={}",
        s.name, s.n, s.max_n, s.tempo
    );

    // ── dynamic info: all vs single ──
    let all = client.get_all_dynamic_info().await.expect("all dynamic");
    let single = client
        .get_dynamic_info(NetUid(1))
        .await
        .expect("single dynamic")
        .expect("SN1 dynamic should exist");
    let found = all
        .iter()
        .find(|d| d.netuid == NetUid(1))
        .expect("SN1 should be in get_all_dynamic_info");
    let diff = (single.price - found.price).abs();
    assert!(
        diff < 0.1,
        "Prices should be close: single={} all={}",
        single.price,
        found.price
    );
    println!(
        "[ok] SN1 price: single={:.6} all={:.6} diff={:.6}",
        single.price, found.price, diff
    );

    // ── balance for known account ──
    let balance = client
        .get_balance_ss58(KNOWN_ADDRESS)
        .await
        .expect("balance");
    println!(
        "[ok] Known account balance: {} RAO ({} TAO)",
        balance.rao(),
        balance.tao()
    );

    // ── stake for unknown coldkey ──
    let stakes = client
        .get_stake_for_coldkey("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM")
        .await
        .expect("stakes");
    println!("[ok] Unknown coldkey: {} stake positions", stakes.len());

    // ── nonexistent subnet ──
    let info = client
        .get_subnet_info(NetUid(65535))
        .await
        .expect("no error");
    assert!(info.is_none(), "netuid 65535 should not exist");
    println!("[ok] netuid 65535 correctly does not exist");

    // ── total stake ──
    let total = client.get_total_stake().await.expect("total stake");
    assert!(total.rao() > 0, "total stake should be nonzero");
    println!("[ok] Total stake: {} TAO", total.tao());

    // ── neurons lite ordering ──
    let neurons = client.get_neurons_lite(NetUid(1)).await.expect("neurons");
    assert!(!neurons.is_empty());
    for (i, n) in neurons.iter().enumerate() {
        assert_eq!(n.uid as usize, i, "neuron UIDs should be sequential");
    }
    println!(
        "[ok] SN1: {} neurons, UIDs 0..{}",
        neurons.len(),
        neurons.len() - 1
    );

    // ── total issuance ──
    let issuance = client.get_total_issuance().await.expect("total issuance");
    assert!(issuance.rao() > 0, "total issuance should be nonzero");
    println!("[ok] Total issuance: {:.2} TAO", issuance.tao());

    // ── block emission ──
    let emission = client.get_block_emission().await.expect("block emission");
    assert!(emission.rao() > 0, "block emission should be positive");
    println!(
        "[ok] Block emission: {} RAO ({:.6} TAO)",
        emission.rao(),
        emission.tao()
    );

    // ── sim swap tao→alpha ──
    let (alpha_amount, tao_fee, alpha_fee) = client
        .sim_swap_tao_for_alpha(NetUid(1), 1_000_000_000)
        .await
        .expect("sim swap");
    assert!(alpha_amount > 0, "should receive some alpha");
    println!(
        "[ok] 1 TAO -> {} alpha (tao_fee={}, alpha_fee={})",
        alpha_amount, tao_fee, alpha_fee
    );

    // ── sim swap alpha→tao ──
    let (tao_amount, tao_fee, alpha_fee) = client
        .sim_swap_alpha_for_tao(NetUid(1), 100_000_000_000)
        .await
        .expect("sim swap");
    assert!(tao_amount > 0, "should receive some TAO");
    println!(
        "[ok] 100 alpha -> {} TAO (tao_fee={}, alpha_fee={})",
        tao_amount, tao_fee, alpha_fee
    );

    // ── single delegate ──
    let delegates = client.get_delegates().await.expect("delegates");
    assert!(!delegates.is_empty());
    let first = &delegates[0];
    let d = client
        .get_delegate(&first.hotkey)
        .await
        .expect("single delegate")
        .expect("first delegate should be queryable");
    assert_eq!(d.hotkey, first.hotkey, "hotkeys should match");
    println!(
        "[ok] Delegate: {} take={:.2}%",
        &d.hotkey[..8],
        d.take / 65535.0 * 100.0
    );

    // ── proxies ──
    let proxies = client
        .list_proxies("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM")
        .await
        .expect("proxies");
    println!("[ok] Proxies for address: {}", proxies.len());

    // ── all dynamic prices non-negative ──
    for d in all.iter() {
        assert!(
            d.price >= 0.0,
            "SN{} has negative price: {}",
            d.netuid,
            d.price
        );
    }
    println!("[ok] All {} subnets have non-negative prices", all.len());

    // ── hyperparams nonexistent subnet ──
    let params = client
        .get_subnet_hyperparams(NetUid(65535))
        .await
        .expect("hyperparams");
    assert!(params.is_none(), "netuid 65535 should not have hyperparams");
    println!("[ok] netuid 65535 correctly has no hyperparams");

    // ── coldkey swap ──
    let swap = client
        .get_coldkey_swap_scheduled(KNOWN_ADDRESS)
        .await
        .expect("coldkey swap query");
    assert!(
        swap.is_none(),
        "known address should not have a pending coldkey swap"
    );
    println!("[ok] No pending coldkey swap for known address");

    // ── child keys ──
    let children = client
        .get_child_keys(KNOWN_ADDRESS, NetUid(1))
        .await
        .expect("child keys query");
    println!(
        "[ok] Child keys for {} on SN1: {} entries",
        KNOWN_ADDRESS,
        children.len()
    );
    for (proportion, child_ss58) in &children {
        assert!(
            child_ss58.starts_with('5'),
            "child ss58 should be valid: {}",
            child_ss58
        );
        assert!(*proportion > 0, "proportion should be positive");
    }

    // ── parent keys ──
    let parents = client
        .get_parent_keys(KNOWN_ADDRESS, NetUid(1))
        .await
        .expect("parent keys query");
    println!(
        "[ok] Parent keys for {} on SN1: {} entries",
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

    // ── historical: stake at recent block ──
    let current_block = client.get_block_number().await.expect("block number");
    let recent = (current_block - 10) as u32;
    let hash = client.get_block_hash(recent).await.expect("block hash");
    let stakes = client
        .get_stake_for_coldkey_at_block(KNOWN_ADDRESS, hash)
        .await
        .expect("stake at block");
    println!(
        "[ok] Stakes at block {}: {} positions",
        recent,
        stakes.len()
    );

    // ── historical: balance + stake consistency ──
    let recent2 = (current_block - 5) as u32;
    let hash2 = client.get_block_hash(recent2).await.expect("block hash");
    let (hist_balance, hist_stakes) = tokio::try_join!(
        client.get_balance_at_block(KNOWN_ADDRESS, hash2),
        client.get_stake_for_coldkey_at_block(KNOWN_ADDRESS, hash2),
    )
    .expect("parallel historical queries");
    println!(
        "[ok] Block {}: balance={}, stakes={}",
        recent2,
        hist_balance.display_tao(),
        hist_stakes.len()
    );

    // ── historical: identity at block ──
    let identity = client
        .get_identity_at_block(KNOWN_ADDRESS, hash2)
        .await
        .expect("identity at block");
    println!(
        "[ok] Identity at block {}: {:?}",
        recent2,
        identity.as_ref().map(|i| &i.name)
    );

    // ── pending child keys ──
    let result = client
        .get_pending_child_keys(KNOWN_ADDRESS, NetUid(1))
        .await
        .expect("pending child keys query");
    println!(
        "[ok] Pending child keys SN1: {:?}",
        result.as_ref().map(|(c, b)| (c.len(), b))
    );
    if let Some((children, cooldown_block)) = &result {
        assert!(*cooldown_block > 0, "cooldown block should be positive");
        for (proportion, child_ss58) in children {
            assert!(
                child_ss58.starts_with('5'),
                "child ss58 should be valid: {}",
                child_ss58
            );
            assert!(*proportion > 0, "proportion should be positive");
        }
    }

    // ── pending child keys SN0 ──
    let result0 = client
        .get_pending_child_keys(KNOWN_ADDRESS, NetUid(0))
        .await
        .expect("pending child keys query SN0");
    println!("[ok] Pending child keys SN0: {:?}", result0.is_some());

    println!("\n=== All extended chain queries passed ===");
}
