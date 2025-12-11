/*
 * Copyright (c) 2025 Beijing Volcano Engine Technology Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::asynchronous::tos::AsyncRuntime;
use crate::constant::DNS_CACHE_REFRESH_INTERVAL;
use crate::error::TosError;
use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use hickory_resolver::lookup_ip::LookupIp;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::{ResolveError, Resolver};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use reqwest::dns::{Addrs, Resolve, Resolving};
use serde::de::StdError;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ops::Add;
use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockWriteGuard};
use tracing::warn;

pub(crate) struct DnsCacheItem {
    pub(crate) host: String,
    pub(crate) addrs: Vec<SocketAddr>,
    pub(crate) ddl: DateTime<Utc>,
    pub(crate) immortal: bool,
    pub(crate) last_update_at: DateTime<Utc>,
}
pub(crate) type DnsCache = Arc<ArcSwap<DnsCacheItem>>;

pub(crate) struct InternalDnsResolver {
    pub(crate) dns_cache_time: isize,
    pub(crate) port: isize,
    pub(crate) resolver: Arc<Resolver<TokioConnectionProvider>>,
    pub(crate) cached_addrs: Arc<RwLock<HashMap<String, DnsCache>>>,
}

impl InternalDnsResolver {
    pub(crate) fn new<S>(dns_cache_time: isize, port: isize, async_runtime: Arc<S>, closed: Arc<AtomicI8>) -> Self
    where
        S: AsyncRuntime + Send + Sync + 'static,
    {
        let cached_addrs = Arc::new(RwLock::new(HashMap::<String, DnsCache>::new()));
        let resolver = Arc::new(Resolver::builder_tokio().unwrap().build());

        let async_runtime2 = async_runtime.clone();
        let cached_addrs2 = cached_addrs.clone();
        let resolver2 = resolver.clone();
        let _ = async_runtime.spawn(async move {
            loop {
                if closed.load(Ordering::Acquire) == 1 {
                    return;
                }
                async_runtime2.sleep(Duration::from_secs(DNS_CACHE_REFRESH_INTERVAL)).await;
                let mut cached_addrs_values;
                {
                    let cached_addrs2 = cached_addrs2.read().await;
                    cached_addrs_values = Vec::with_capacity(cached_addrs2.len());
                    for cached_addr in cached_addrs2.values() {
                        cached_addrs_values.push(cached_addr.clone());
                    }
                }

                for cached_addrs_value in cached_addrs_values {
                    let cached_addrs_value2 = cached_addrs_value.load();
                    if (Utc::now() - cached_addrs_value2.last_update_at).num_milliseconds() <= 1000 {
                        continue;
                    }
                    match resolver2.lookup_ip(cached_addrs_value2.host.as_str()).await {
                        Err(ex) => {
                            warn!("async resolve from {} is failed, {}", cached_addrs_value2.host, ex.to_string());
                            cached_addrs_value.store(Arc::new(DnsCacheItem {
                                host: cached_addrs_value2.host.clone(),
                                addrs: cached_addrs_value2.addrs.clone(),
                                ddl: cached_addrs_value2.ddl,
                                immortal: true,
                                last_update_at: cached_addrs_value2.last_update_at,
                            }));
                        }
                        Ok(ips) => {
                            let mut addrs = Vec::<SocketAddr>::with_capacity(10);
                            for ip in ips.iter() {
                                match ip {
                                    IpAddr::V4(ipv4) => addrs.push(SocketAddr::V4(SocketAddrV4::new(ipv4, port as u16))),
                                    IpAddr::V6(ipv6) => addrs.push(SocketAddr::V6(SocketAddrV6::new(ipv6, port as u16, 0, 0)))
                                }
                            }

                            if addrs.len() == 0 {
                                warn!("async resolve from {} is empty", cached_addrs_value2.host);
                                cached_addrs_value.store(Arc::new(DnsCacheItem {
                                    host: cached_addrs_value2.host.clone(),
                                    addrs: cached_addrs_value2.addrs.clone(),
                                    ddl: cached_addrs_value2.ddl,
                                    immortal: true,
                                    last_update_at: cached_addrs_value2.last_update_at,
                                }));
                                return;
                            }

                            let mut rng = thread_rng();
                            let dns_cache_time = dns_cache_time + rng.gen_range(0..5) as isize;
                            let now = Utc::now();
                            cached_addrs_value.store(Arc::new(DnsCacheItem {
                                host: cached_addrs_value2.host.clone(),
                                addrs: addrs.clone(),
                                ddl: now.add(Duration::from_secs((dns_cache_time * 60) as u64)),
                                immortal: false,
                                last_update_at: now,
                            }));
                        }
                    }
                }
            }
        });

        Self {
            dns_cache_time,
            port,
            resolver,
            cached_addrs,
        }
    }
}

impl Resolve for InternalDnsResolver
{
    fn resolve(&self, name: hyper::client::connect::dns::Name) -> Resolving {
        let resolver = self.resolver.clone();
        let port = self.port as u16;
        let name = name.clone();
        let dns_cache_time = self.dns_cache_time;
        let cached_addrs = self.cached_addrs.clone();
        Box::pin(async move {
            {
                let cached_addrs = cached_addrs.read().await;
                if let Some(dns_cache) = cached_addrs.get(name.as_str()) {
                    let dns_cache = dns_cache.load();
                    if dns_cache.addrs.len() > 0 && (dns_cache.ddl >= Utc::now() || dns_cache.immortal) {
                        // println!("{}", "return cached addr");
                        let addrs = Box::new(shuffle(dns_cache.addrs.clone())) as Box<dyn Iterator<Item=SocketAddr> + Send>;
                        return Ok(addrs);
                    }
                }
            }

            let cached_addrs = cached_addrs.write().await;
            if let Some(dns_cache) = cached_addrs.get(name.as_str()) {
                let dns_cache = dns_cache.load();
                if dns_cache.addrs.len() > 0 && (dns_cache.ddl >= Utc::now() || dns_cache.immortal) {
                    // println!("{}", "return cached addr");
                    let addrs = Box::new(shuffle(dns_cache.addrs.clone())) as Box<dyn Iterator<Item=SocketAddr> + Send>;
                    return Ok(addrs);
                }
            }
            trans(resolver.lookup_ip(name.as_str()).await, port, name, dns_cache_time, cached_addrs)
        })
    }
}

pub(crate) fn shuffle(mut addrs: Vec<SocketAddr>) -> impl Iterator<Item=SocketAddr> {
    addrs.shuffle(&mut thread_rng());
    addrs.into_iter()
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

pub(crate) fn trans(result: Result<LookupIp, ResolveError>, port: u16, name: hyper::client::connect::dns::Name,
                    dns_cache_time: isize, mut cached_addrs: RwLockWriteGuard<HashMap<String, DnsCache>>) -> Result<Addrs, BoxError> {
    match result {
        Err(ex) => Err(Box::new(TosError::client_error(format!("resolve from {} is failed, {}", name, ex.to_string()))) as Box<dyn StdError + Send + Sync>),
        Ok(ips) => {
            let mut addrs = Vec::<SocketAddr>::with_capacity(10);
            for ip in ips.iter() {
                match ip {
                    IpAddr::V4(ipv4) => addrs.push(SocketAddr::V4(SocketAddrV4::new(ipv4, port))),
                    IpAddr::V6(ipv6) => addrs.push(SocketAddr::V6(SocketAddrV6::new(ipv6, port, 0, 0)))
                }
            }

            if addrs.len() == 0 {
                let ex = Box::new(TosError::client_error(format!("resolve from {} is empty", name))) as Box<dyn StdError + Send + Sync>;
                return Err(ex);
            }

            let mut rng = thread_rng();
            let dns_cache_time = dns_cache_time + rng.gen_range(0..5) as isize;
            let now = Utc::now();
            cached_addrs.insert(name.to_string(), Arc::new(ArcSwap::new(Arc::new(DnsCacheItem {
                host: name.to_string(),
                addrs: addrs.clone(),
                ddl: now.add(Duration::from_secs((dns_cache_time * 60) as u64)),
                immortal: false,
                last_update_at: now,
            }))));
            Ok(Box::new(addrs.into_iter()) as Box<dyn Iterator<Item=SocketAddr> + Send>)
        }
    }
}
