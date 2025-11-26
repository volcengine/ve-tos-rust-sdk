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
use crate::error::TosError;
use chrono::{DateTime, Duration, Utc};
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
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};

pub(crate) type DnsCache = (Vec<SocketAddr>, DateTime<Utc>);

pub(crate) struct InternalDnsResolver {
    pub(crate) dns_cache_time: isize,
    pub(crate) port: isize,
    pub(crate) resolver: Arc<Resolver<TokioConnectionProvider>>,
    pub(crate) cached_addrs: Arc<RwLock<HashMap<String, DnsCache>>>,
}

impl InternalDnsResolver {
    pub(crate) fn new(dns_cache_time: isize, port: isize) -> Self
    {
        let cached_addrs = Arc::new(RwLock::new(HashMap::new()));
        let resolver = Arc::new(Resolver::builder_tokio().unwrap().build());
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
                if let Some((addrs, ddl)) = cached_addrs.get(name.as_str()) {
                    if addrs.len() > 0 && ddl >= &Utc::now() {
                        // println!("{}", "return cached addr");
                        let addrs = Box::new(shuffle(addrs.clone())) as Box<dyn Iterator<Item=SocketAddr> + Send>;
                        return Ok(addrs);
                    }
                }
            }

            let cached_addrs = cached_addrs.write().await;
            if let Some((addrs, ddl)) = cached_addrs.get(name.as_str()) {
                if addrs.len() > 0 && ddl >= &Utc::now() {
                    // println!("{}", "return cached addr");
                    let addrs = Box::new(shuffle(addrs.clone())) as Box<dyn Iterator<Item=SocketAddr> + Send>;
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

            let mut rng = rand::thread_rng();
            let dns_cache_time = dns_cache_time + rng.gen_range(0..5) as isize;
            cached_addrs.insert(name.to_string(), (addrs.clone(), Utc::now().add(Duration::minutes(dns_cache_time as i64))));
            Ok(Box::new(addrs.into_iter()) as Box<dyn Iterator<Item=SocketAddr> + Send>)
        }
    }
}
