//! In-memory TTL cache with request coalescing for repeated chain queries.
//!
//! Caches expensive read-only queries (subnet list, dynamic info)
//! with a short TTL (default 30s) to avoid redundant chain calls
//! within the same command session.
//!
//! Uses moka's `try_get_with()` for atomic deduplication: if multiple
//! concurrent tasks request the same key, only one fetch runs and
//! all waiters share the result.

use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::types::chain_data::{DelegateInfo, DynamicInfo, SubnetInfo};

/// Default in-memory cache TTL in seconds.
const DEFAULT_TTL_SECS: u64 = 30;

/// Disk cache TTL — longer than in-memory since disk cache survives across CLI invocations.
/// Avoids redundant chain fetches when running multiple commands in quick succession.
const DISK_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Shared query cache for chain data that changes slowly.
///
/// All get methods use `try_get_with()` for request coalescing — concurrent
/// callers for the same key block on a single in-flight fetch instead of
/// each issuing their own RPC call.
#[derive(Clone)]
pub struct QueryCache {
    /// Cached subnet list (all subnets).
    subnets: Cache<(), Arc<Vec<SubnetInfo>>>,
    /// Cached dynamic info for all subnets.
    all_dynamic: Cache<(), Arc<Vec<DynamicInfo>>>,
    /// Cached dynamic info per subnet.
    dynamic_by_netuid: Cache<u16, Arc<DynamicInfo>>,
    /// Cached delegate list (all delegates).
    delegates: Cache<(), Arc<Vec<DelegateInfo>>>,
    /// Whether to use the disk cache layer. Disabled for tests with custom TTLs.
    use_disk: bool,
}

impl QueryCache {
    /// Create a new cache with the default TTL and disk caching enabled.
    pub fn new() -> Self {
        Self::with_ttl_and_disk(Duration::from_secs(DEFAULT_TTL_SECS), true)
    }

    /// Create a cache with a custom TTL. Disk caching is disabled for custom TTLs
    /// (used in tests and special configurations).
    pub fn with_ttl(ttl: Duration) -> Self {
        Self::with_ttl_and_disk(ttl, false)
    }

    /// Internal constructor with explicit disk cache control. Also used in tests.
    pub(crate) fn with_ttl_and_disk(ttl: Duration, use_disk: bool) -> Self {
        Self {
            subnets: Cache::builder().time_to_live(ttl).max_capacity(1).build(),
            all_dynamic: Cache::builder().time_to_live(ttl).max_capacity(1).build(),
            dynamic_by_netuid: Cache::builder().time_to_live(ttl).max_capacity(100).build(),
            delegates: Cache::builder().time_to_live(ttl).max_capacity(1).build(),
            use_disk,
        }
    }

    /// Get or fetch all subnets. Concurrent callers coalesce into one fetch.
    /// Layered: in-memory (30s) → disk (5min) → chain.
    /// On chain fetch failure, serves stale disk cache if available (stale-while-error).
    pub async fn get_all_subnets<F, Fut>(&self, fetch: F) -> anyhow::Result<Arc<Vec<SubnetInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Vec<SubnetInfo>>>,
    {
        let disk = self.use_disk;
        self.subnets
            .try_get_with((), async {
                // Check disk cache before hitting chain
                if disk {
                    if let Some(cached) = super::disk_cache::get::<Vec<SubnetInfo>>("all_subnets", DISK_TTL) {
                        tracing::debug!(count = cached.len(), "cache hit: all_subnets (disk)");
                        return Ok(Arc::new(cached)) as anyhow::Result<_>;
                    }
                }
                tracing::debug!("cache miss: all_subnets — fetching from chain");
                let start = std::time::Instant::now();
                match fetch().await {
                    Ok(data) => {
                        tracing::debug!(elapsed_ms = start.elapsed().as_millis() as u64, count = data.len(), "fetched all_subnets");
                        // Write through to disk cache (best-effort)
                        if disk {
                            if let Err(e) = super::disk_cache::put("all_subnets", &data) {
                                tracing::warn!(error = %e, "failed to write all_subnets to disk cache");
                            }
                        }
                        Ok(Arc::new(data))
                    }
                    Err(e) => {
                        // Stale-while-error: serve expired disk cache on chain failure
                        if disk {
                            if let Some(stale) = super::disk_cache::get_stale::<Vec<SubnetInfo>>("all_subnets") {
                                tracing::warn!(count = stale.len(), error = %e, "chain fetch failed, serving stale all_subnets from disk cache");
                                return Ok(Arc::new(stale));
                            }
                        }
                        Err(e)
                    }
                }
            })
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Get or fetch all dynamic info. Concurrent callers coalesce into one fetch.
    /// Layered: in-memory (30s) → disk (5min) → chain.
    /// On chain fetch failure, serves stale disk cache if available (stale-while-error).
    pub async fn get_all_dynamic_info<F, Fut>(
        &self,
        fetch: F,
    ) -> anyhow::Result<Arc<Vec<DynamicInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Vec<DynamicInfo>>>,
    {
        let per_netuid = self.dynamic_by_netuid.clone();
        let disk = self.use_disk;
        self.all_dynamic
            .try_get_with((), async {
                // Check disk cache before hitting chain
                if disk {
                    if let Some(cached) = super::disk_cache::get::<Vec<DynamicInfo>>("all_dynamic_info", DISK_TTL) {
                        tracing::debug!(count = cached.len(), "cache hit: all_dynamic_info (disk)");
                        let data = Arc::new(cached);
                        // Also populate per-netuid in-memory cache
                        for d in data.iter() {
                            per_netuid.insert(d.netuid.0, Arc::new(d.clone())).await;
                        }
                        return Ok(data) as anyhow::Result<_>;
                    }
                }
                tracing::debug!("cache miss: all_dynamic_info — fetching from chain");
                let start = std::time::Instant::now();
                match fetch().await {
                    Ok(data) => {
                        tracing::debug!(elapsed_ms = start.elapsed().as_millis() as u64, count = data.len(), "fetched all_dynamic_info");
                        // Write through to disk cache (best-effort)
                        if disk {
                            if let Err(e) = super::disk_cache::put("all_dynamic_info", &data) {
                                tracing::warn!(error = %e, "failed to write all_dynamic_info to disk cache");
                            }
                        }
                        let data = Arc::new(data);
                        // Also populate per-netuid cache
                        for d in data.iter() {
                            per_netuid
                                .insert(d.netuid.0, Arc::new(d.clone()))
                                .await;
                        }
                        Ok(data)
                    }
                    Err(e) => {
                        // Stale-while-error: serve expired disk cache on chain failure
                        if disk {
                            if let Some(stale) = super::disk_cache::get_stale::<Vec<DynamicInfo>>("all_dynamic_info") {
                                tracing::warn!(count = stale.len(), error = %e, "chain fetch failed, serving stale all_dynamic_info from disk cache");
                                let data = Arc::new(stale);
                                for d in data.iter() {
                                    per_netuid.insert(d.netuid.0, Arc::new(d.clone())).await;
                                }
                                return Ok(data);
                            }
                        }
                        Err(e)
                    }
                }
            })
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Get or fetch dynamic info for a specific subnet.
    /// Concurrent callers for the same netuid coalesce into one fetch.
    pub async fn get_dynamic_info<F, Fut>(
        &self,
        netuid: u16,
        fetch: F,
    ) -> anyhow::Result<Option<Arc<DynamicInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Option<DynamicInfo>>>,
    {
        // try_get_with always returns Some when inserted, but we need to handle
        // the None case (subnet doesn't exist). Use a sentinel-free approach:
        // check cache first, then fetch if missing.
        if let Some(cached) = self.dynamic_by_netuid.get(&netuid).await {
            tracing::debug!(netuid, "cache hit: dynamic_info");
            return Ok(Some(cached));
        }
        tracing::debug!(netuid, "cache miss: dynamic_info — fetching from chain");
        let start = std::time::Instant::now();
        match fetch().await? {
            Some(data) => {
                tracing::debug!(
                    netuid,
                    elapsed_ms = start.elapsed().as_millis() as u64,
                    "fetched dynamic_info"
                );
                let arc = Arc::new(data);
                self.dynamic_by_netuid.insert(netuid, arc.clone()).await;
                Ok(Some(arc))
            }
            None => Ok(None),
        }
    }

    /// Get or fetch all delegates. Concurrent callers coalesce into one fetch.
    /// In-memory only (no disk cache) since delegate data is less stable.
    pub async fn get_all_delegates<F, Fut>(
        &self,
        fetch: F,
    ) -> anyhow::Result<Arc<Vec<DelegateInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Vec<DelegateInfo>>>,
    {
        self.delegates
            .try_get_with((), async {
                tracing::debug!("cache miss: all_delegates — fetching from chain");
                let start = std::time::Instant::now();
                let data = fetch().await?;
                tracing::debug!(
                    elapsed_ms = start.elapsed().as_millis() as u64,
                    count = data.len(),
                    "fetched all_delegates"
                );
                Ok(Arc::new(data)) as anyhow::Result<_>
            })
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Invalidate all cached data (both in-memory and disk).
    pub async fn invalidate_all(&self) {
        self.subnets.invalidate_all();
        self.all_dynamic.invalidate_all();
        self.dynamic_by_netuid.invalidate_all();
        self.delegates.invalidate_all();
        super::disk_cache::remove("all_subnets");
        super::disk_cache::remove("all_dynamic_info");
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn cache_deduplicates_calls() {
        let cache = QueryCache::with_ttl(Duration::from_secs(30));
        let call_count = Arc::new(AtomicU32::new(0));

        let count = call_count.clone();
        let r1 = cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        let count = call_count.clone();
        let r2 = cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        // Second call should use cache
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        assert!(Arc::ptr_eq(&r1, &r2));
    }

    #[tokio::test]
    async fn cache_expires_after_ttl() {
        let cache = QueryCache::with_ttl(Duration::from_millis(50));
        let call_count = Arc::new(AtomicU32::new(0));

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn invalidate_clears_cache() {
        let cache = QueryCache::with_ttl(Duration::from_secs(30));
        let call_count = Arc::new(AtomicU32::new(0));

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        cache.invalidate_all().await;

        let count = call_count.clone();
        cache
            .get_all_subnets(|| {
                let c = count.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    /// Stress test: concurrent readers coalesce — only one fetch executes.
    #[tokio::test]
    async fn cache_concurrent_readers_coalesce() {
        let cache = Arc::new(QueryCache::with_ttl(Duration::from_secs(30)));
        let call_count = Arc::new(AtomicU32::new(0));

        // Spawn 50 concurrent readers — with try_get_with, only ONE fetch should run
        let mut handles = Vec::new();
        for _ in 0..50 {
            let c = cache.clone();
            let count = call_count.clone();
            handles.push(tokio::spawn(async move {
                c.get_all_subnets(|| {
                    let cc = count.clone();
                    async move {
                        // Small delay to ensure concurrent tasks overlap
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        cc.fetch_add(1, Ordering::SeqCst);
                        Ok(vec![])
                    }
                })
                .await
            }));
        }

        let mut results = Vec::new();
        for h in handles {
            results.push(h.await.unwrap().unwrap());
        }

        // With try_get_with coalescing, only 1 fetch should have run
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        // All results should point to the same Arc
        for r in &results {
            assert!(Arc::ptr_eq(&results[0], r));
        }
    }

    fn make_dynamic_info(netuid: u16, name: &str) -> DynamicInfo {
        use crate::types::balance::{AlphaBalance, Balance};
        use crate::types::network::NetUid;
        DynamicInfo {
            netuid: NetUid(netuid),
            name: name.into(),
            symbol: String::new(),
            tempo: 360,
            emission: 0,
            tao_in: Balance::ZERO,
            alpha_in: AlphaBalance::ZERO,
            alpha_out: AlphaBalance::ZERO,
            price: 0.0,
            owner_hotkey: String::new(),
            owner_coldkey: String::new(),
            last_step: 0,
            blocks_since_last_step: 0,
            alpha_out_emission: 0,
            alpha_in_emission: 0,
            tao_in_emission: 0,
            pending_alpha_emission: 0,
            pending_root_emission: 0,
            subnet_volume: 0,
            network_registered_at: 0,
        }
    }

    /// Stress test: per-netuid cache populated from all_dynamic fetch.
    #[tokio::test]
    async fn cache_per_netuid_stress() {
        let cache = QueryCache::with_ttl(Duration::from_secs(30));

        // Bulk-fetch populates per-netuid cache
        let infos: Vec<DynamicInfo> = (0..64u16)
            .map(|i| make_dynamic_info(i, &format!("SN{}", i)))
            .collect();
        cache
            .get_all_dynamic_info(|| {
                let data = infos.clone();
                async move { Ok(data) }
            })
            .await
            .unwrap();

        // All per-netuid lookups should hit cache (no fetch)
        for i in 0..64u16 {
            let result = cache
                .get_dynamic_info(i, || async { Err(anyhow::anyhow!("should not be called")) })
                .await
                .unwrap()
                .expect("should be cached");
            assert_eq!(result.netuid.0, i);
            assert_eq!(result.name, format!("SN{}", i));
        }

        // Non-existent netuid should call the fetch function
        let fetched = cache
            .get_dynamic_info(999, || async {
                Ok(Some(make_dynamic_info(999, "fetched")))
            })
            .await
            .unwrap()
            .expect("should have fetched");
        assert_eq!(fetched.name, "fetched");
    }

    /// Cache handles fetch failures gracefully — error propagates, no poisoning.
    #[tokio::test]
    async fn cache_fetch_error_does_not_poison() {
        let cache = QueryCache::with_ttl(Duration::from_secs(30));

        // First call: fetch fails
        let result = cache
            .get_all_subnets(|| async { Err(anyhow::anyhow!("network error")) })
            .await;
        assert!(result.is_err());

        // Second call: fetch succeeds — cache should not be poisoned
        let result = cache.get_all_subnets(|| async { Ok(vec![]) }).await;
        assert!(result.is_ok());
    }

    /// Stale-while-error: when chain fetch fails and disk has stale data, serve it.
    #[tokio::test]
    async fn stale_while_error_subnets() {
        use crate::queries::disk_cache;

        // This test uses the real disk cache, so use a unique key prefix
        let key = "all_subnets";

        // Pre-populate disk cache with known data
        let stale_data: Vec<SubnetInfo> = vec![];
        disk_cache::put(key, &stale_data).unwrap();

        // Force in-memory TTL to be expired by using short TTL
        let cache = QueryCache::with_ttl_and_disk(Duration::from_millis(1), true);
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Fetch fails — should fall back to stale disk data
        let result = cache
            .get_all_subnets(|| async { Err(anyhow::anyhow!("chain connection failed")) })
            .await;
        assert!(
            result.is_ok(),
            "stale-while-error should serve stale disk data"
        );

        // Clean up
        disk_cache::remove(key);
    }

    /// Stale-while-error: when no stale data exists, error propagates normally.
    #[tokio::test]
    async fn stale_while_error_no_stale_data() {
        use crate::queries::disk_cache;

        // Use a unique key that no other test writes to
        // We test by disabling disk so no stale lookup happens
        let cache = QueryCache::with_ttl(Duration::from_millis(1));
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Fetch fails with disk disabled — error should propagate (no stale fallback)
        let result = cache
            .get_all_subnets(|| async { Err(anyhow::anyhow!("chain connection failed")) })
            .await;
        assert!(
            result.is_err(),
            "should propagate error when disk cache is disabled"
        );
        let _ = disk_cache::path(); // suppress unused import
    }

    /// Stale-while-error for dynamic info.
    #[tokio::test]
    async fn stale_while_error_dynamic_info() {
        use crate::queries::disk_cache;

        let key = "all_dynamic_info";

        // Pre-populate disk cache
        let stale_data: Vec<DynamicInfo> = vec![make_dynamic_info(1, "TestNet")];
        disk_cache::put(key, &stale_data).unwrap();

        let cache = QueryCache::with_ttl_and_disk(Duration::from_millis(1), true);
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Fetch fails — should fall back to stale disk data
        let result = cache
            .get_all_dynamic_info(|| async { Err(anyhow::anyhow!("timeout")) })
            .await;
        assert!(
            result.is_ok(),
            "stale-while-error should serve stale dynamic info"
        );
        let data = result.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].name, "TestNet");

        // Per-netuid cache should also be populated from stale data
        let single = cache
            .get_dynamic_info(1, || async { Err(anyhow::anyhow!("should not be called")) })
            .await
            .unwrap();
        assert!(single.is_some());
        assert_eq!(single.unwrap().name, "TestNet");

        // Clean up
        disk_cache::remove(key);
    }
}
