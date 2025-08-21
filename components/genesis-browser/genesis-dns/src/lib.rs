use std::net::IpAddr;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};

/// Genesis DNS Resolver - Blockchain-based domain resolution
pub struct GenesisDnsResolver {
    /// Genesis node URL
    genesis_node_url: String,
    /// HTTP client for API calls
    client: reqwest::Client,
    /// Cache for resolved domains
    cache: HashMap<String, DnsResult>,
    /// Enable traditional DNS fallback
    fallback_enabled: bool,
}

/// DNS resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResult {
    pub domain: String,
    pub ip_address: Option<IpAddr>,
    pub content_hash: Option<String>,
    pub resolver_type: ResolverType,
    pub ttl: u64,
    pub timestamp: u64,
}

impl std::fmt::Display for DnsResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ip) = &self.ip_address {
            write!(f, "{}", ip)
        } else if let Some(hash) = &self.content_hash {
            write!(f, "ipfs://{}", hash)
        } else {
            write!(f, "{}", self.domain)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolverType {
    Genesis,
    Traditional,
    IPFS,
    Hybrid,
}

/// Genesis domain information from blockchain
#[derive(Debug, Deserialize)]
struct GenesisDomain {
    name: String,
    owner: String,
    resolver: String,
    content_hash: Option<String>,
    ip_address: Option<String>,
    ttl: Option<u64>,
}

impl GenesisDnsResolver {
    pub fn new(genesis_node_url: String, fallback_enabled: bool) -> Self {
        Self {
            genesis_node_url,
            client: reqwest::Client::new(),
            cache: HashMap::new(),
            fallback_enabled,
        }
    }

    /// Initialize the DNS resolver
    pub async fn initialize(&mut self) -> Result<(), DnsError> {
        info!("🚀 Initializing Genesis DNS Resolver");
        // DNS resolver initialization logic here
        Ok(())
    }

    /// Resolve a domain using Genesis blockchain
    pub async fn resolve(&mut self, domain: &str) -> Result<DnsResult, DnsError> {
        info!("🔍 Resolving domain: {}", domain);

        // Check cache first
        if let Some(cached) = self.cache.get(domain) {
            if !self.is_cache_expired(cached) {
                debug!("📋 Cache hit for domain: {}", domain);
                return Ok(cached.clone());
            }
        }

        // Determine resolver strategy
        let result = if self.is_genesis_domain(domain) {
            self.resolve_genesis_domain(domain).await
        } else if self.fallback_enabled {
            self.resolve_traditional_domain(domain).await
        } else {
            Err(DnsError::UnsupportedDomain(domain.to_string()))
        };

        // Cache successful results
        if let Ok(ref result) = result {
            self.cache.insert(domain.to_string(), result.clone());
        }

        result
    }

    /// Check if domain is a Genesis blockchain domain
    fn is_genesis_domain(&self, domain: &str) -> bool {
        let genesis_tlds = [".genesis", ".free", ".web", ".defi", ".dao"];
        genesis_tlds.iter().any(|tld| domain.ends_with(tld))
    }

    /// Resolve Genesis blockchain domain
    async fn resolve_genesis_domain(&self, domain: &str) -> Result<DnsResult, DnsError> {
        info!("🌐 Resolving Genesis domain: {}", domain);

        let url = format!("{}/api/dns/resolve/{}", self.genesis_node_url, domain);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<GenesisDomain>().await {
                        Ok(genesis_domain) => {
                            info!("✅ Genesis domain resolved: {}", domain);
                            Ok(self.convert_genesis_domain(genesis_domain))
                        },
                        Err(e) => {
                            error!("❌ Failed to parse Genesis domain response: {}", e);
                            Err(DnsError::InvalidResponse(e.to_string()))
                        }
                    }
                } else {
                    warn!("⚠️ Genesis node returned error for {}: {}", domain, response.status());
                    Err(DnsError::NodeError(response.status().to_string()))
                }
            },
            Err(e) => {
                error!("❌ Failed to connect to Genesis node: {}", e);
                Err(DnsError::ConnectionError(e.to_string()))
            }
        }
    }

    /// Resolve traditional DNS domain (fallback)
    async fn resolve_traditional_domain(&self, domain: &str) -> Result<DnsResult, DnsError> {
        info!("🌍 Resolving traditional domain: {}", domain);

        #[cfg(feature = "traditional-fallback")]
        {
            use trust_dns_resolver::TokioAsyncResolver;
            use trust_dns_resolver::config::*;

            let resolver = TokioAsyncResolver::tokio(
                ResolverConfig::default(),
                ResolverOpts::default()
            );

            match resolver.lookup_ip(domain).await {
                Ok(response) => {
                    let ip = response.iter().next()
                        .ok_or_else(|| DnsError::NoResults(domain.to_string()))?;
                    
                    info!("✅ Traditional DNS resolved: {} -> {}", domain, ip);
                    
                    Ok(DnsResult {
                        domain: domain.to_string(),
                        ip_address: Some(ip),
                        content_hash: None,
                        resolver_type: ResolverType::Traditional,
                        ttl: 300, // 5 minutes default
                        timestamp: chrono::Utc::now().timestamp() as u64,
                    })
                },
                Err(e) => {
                    warn!("⚠️ Traditional DNS failed for {}: {}", domain, e);
                    Err(DnsError::ResolutionFailed(e.to_string()))
                }
            }
        }

        #[cfg(not(feature = "traditional-fallback"))]
        {
            Err(DnsError::UnsupportedDomain(domain.to_string()))
        }
    }

    /// Convert Genesis domain to DNS result
    fn convert_genesis_domain(&self, genesis_domain: GenesisDomain) -> DnsResult {
        let ip_address = genesis_domain.ip_address
            .and_then(|ip_str| ip_str.parse().ok());

        DnsResult {
            domain: genesis_domain.name,
            ip_address,
            content_hash: genesis_domain.content_hash,
            resolver_type: if ip_address.is_some() {
                ResolverType::Genesis
            } else {
                ResolverType::IPFS
            },
            ttl: genesis_domain.ttl.unwrap_or(3600), // 1 hour default
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// Check if cache entry is expired
    fn is_cache_expired(&self, result: &DnsResult) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        now > result.timestamp + result.ttl
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&mut self) {
        let now = chrono::Utc::now().timestamp() as u64;
        self.cache.retain(|_, result| now <= result.timestamp + result.ttl);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            genesis_domains: self.cache.values()
                .filter(|r| matches!(r.resolver_type, ResolverType::Genesis))
                .count(),
            traditional_domains: self.cache.values()
                .filter(|r| matches!(r.resolver_type, ResolverType::Traditional))
                .count(),
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub genesis_domains: usize,
    pub traditional_domains: usize,
}

/// DNS resolution errors
#[derive(Debug, thiserror::Error)]
pub enum DnsError {
    #[error("Unsupported domain: {0}")]
    UnsupportedDomain(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Node error: {0}")]
    NodeError(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),
    
    #[error("No results for domain: {0}")]
    NoResults(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_genesis_domain() {
        let resolver = GenesisDnsResolver::new("http://localhost:3000".to_string(), true);
        
        assert!(resolver.is_genesis_domain("test.genesis"));
        assert!(resolver.is_genesis_domain("freedom.free"));
        assert!(resolver.is_genesis_domain("mysite.web"));
        assert!(resolver.is_genesis_domain("token.defi"));
        assert!(!resolver.is_genesis_domain("google.com"));
        assert!(!resolver.is_genesis_domain("example.org"));
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let mut resolver = GenesisDnsResolver::new("http://localhost:3000".to_string(), true);
        
        // Mock a cache entry
        let result = DnsResult {
            domain: "test.genesis".to_string(),
            ip_address: Some("192.168.1.100".parse().unwrap()),
            content_hash: None,
            resolver_type: ResolverType::Genesis,
            ttl: 3600,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        resolver.cache.insert("test.genesis".to_string(), result.clone());
        
        // Test cache hit
        let stats = resolver.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.genesis_domains, 1);
    }
}