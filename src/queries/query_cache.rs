//! In-memory TTL cache for repeated chain queries.
//!
//! Caches expensive read-only queries (subnet list, dynamic info)
//! with a short TTL (default 30s) to avoid redundant chain calls
//! within the same command session.

use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::types::chain_data::{DynamicInfo, SubnetInfo};

/// Default cache TTL in seconds.
const DEFAULT_TTL_SECS: u64 = 30;

/// Shared query cache for chain data that changes slowly.
#[derive(Clone)]
pub struct QueryCache {
    /// Cached subnet list (all subnets).
    subnets: Cache<(), Arc<Vec<SubnetInfo>>>,
    /// Cached dynamic info for all subnets.
    all_dynamic: Cache<(), Arc<Vec<DynamicInfo>>>,
    /// Cached dynamic info per subnet.
    dynamic_by_netuid: Cache<u16, Arc<DynamicInfo>>,
}

impl QueryCache {
    /// Create a new cache with the default TTL.
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(DEFAULT_TTL_SECS))
    }

    /// Create a cache with a custom TTL.
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            subnets: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1)
                .build(),
            all_dynamic: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1)
                .build(),
            dynamic_by_netuid: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(100)
                .build(),
        }
    }

    /// Get or fetch all subnets.
    pub async fn get_all_subnets<F, Fut>(&self, fetch: F) -> anyhow::Result<Arc<Vec<SubnetInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Vec<SubnetInfo>>>,
    {
        if let Some(cached) = self.subnets.get(&()).await {
            tracing::debug!("cache hit: all_subnets");
            return Ok(cached);
        }
        tracing::debug!("cache miss: all_subnets — fetching from chain");
        let start = std::time::Instant::now();
        let data = Arc::new(fetch().await?);
        tracing::debug!(elapsed_ms = start.elapsed().as_millis() as u64, count = data.len(), "fetched all_subnets");
        self.subnets.insert((), data.clone()).await;
        Ok(data)
    }

    /// Get or fetch all dynamic info.
    pub async fn get_all_dynamic_info<F, Fut>(
        &self,
        fetch: F,
    ) -> anyhow::Result<Arc<Vec<DynamicInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Vec<DynamicInfo>>>,
    {
        if let Some(cached) = self.all_dynamic.get(&()).await {
            tracing::debug!("cache hit: all_dynamic_info");
            return Ok(cached);
        }
        tracing::debug!("cache miss: all_dynamic_info — fetching from chain");
        let start = std::time::Instant::now();
        let data = Arc::new(fetch().await?);
        tracing::debug!(elapsed_ms = start.elapsed().as_millis() as u64, count = data.len(), "fetched all_dynamic_info");
        self.all_dynamic.insert((), data.clone()).await;
        // Also populate per-netuid cache
        for d in data.iter() {
            self.dynamic_by_netuid
                .insert(d.netuid.0, Arc::new(d.clone()))
                .await;
        }
        Ok(data)
    }

    /// Get or fetch dynamic info for a specific subnet.
    pub async fn get_dynamic_info<F, Fut>(
        &self,
        netuid: u16,
        fetch: F,
    ) -> anyhow::Result<Option<Arc<DynamicInfo>>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<Option<DynamicInfo>>>,
    {
        if let Some(cached) = self.dynamic_by_netuid.get(&netuid).await {
            tracing::debug!(netuid, "cache hit: dynamic_info");
            return Ok(Some(cached));
        }
        tracing::debug!(netuid, "cache miss: dynamic_info — fetching from chain");
        let start = std::time::Instant::now();
        match fetch().await? {
            Some(data) => {
                tracing::debug!(netuid, elapsed_ms = start.elapsed().as_millis() as u64, "fetched dynamic_info");
                let arc = Arc::new(data);
                self.dynamic_by_netuid.insert(netuid, arc.clone()).await;
                Ok(Some(arc))
            }
            None => Ok(None),
        }
    }

    /// Invalidate all cached data.
    pub async fn invalidate_all(&self) {
        self.subnets.invalidate_all();
        self.all_dynamic.invalidate_all();
        self.dynamic_by_netuid.invalidate_all();
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
        let cache = QueryCache::new();
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
        let cache = QueryCache::new();
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

    /// Stress test: sequential reads should coalesce to a single fetch.
    /// Concurrent reads may each miss, but the final cached value is consistent.
    #[tokio::test]
    async fn cache_concurrent_readers_consistent() {
        let cache = Arc::new(QueryCache::new());
        let call_count = Arc::new(AtomicU32::new(0));

        // Warm the cache with one fetch
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
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Now 50 concurrent readers should all hit cache
        let mut handles = Vec::new();
        for _ in 0..50 {
            let c = cache.clone();
            let count = call_count.clone();
            handles.push(tokio::spawn(async move {
                c.get_all_subnets(|| {
                    let cc = count.clone();
                    async move {
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

        // All 50 reads should hit cache — no additional fetches
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
        let cache = QueryCache::new();

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
        let cache = QueryCache::new();

        // First call: fetch fails
        let result = cache
            .get_all_subnets(|| async { Err(anyhow::anyhow!("network error")) })
            .await;
        assert!(result.is_err());

        // Second call: fetch succeeds — cache should not be poisoned
        let result = cache
            .get_all_subnets(|| async { Ok(vec![]) })
            .await;
        assert!(result.is_ok());
    }
}
