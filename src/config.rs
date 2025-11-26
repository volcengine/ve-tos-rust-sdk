/*
 * Copyright (2024) Volcengine
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
use std::collections::HashMap;
use url::Url;

use super::constant::*;
use super::error::{GenericError, TosError};
use super::internal::url_encode_with_safe;

#[derive(Debug, Clone)]
pub(crate) struct ConfigHolder {
    pub(crate) max_retry_count: isize,
    pub(crate) request_timeout: isize,
    pub(crate) connection_timeout: isize,
    pub(crate) max_connections: isize,
    pub(crate) idle_connection_time: isize,
    pub(crate) enable_crc: bool,
    pub(crate) enable_verify_ssl: bool,
    pub(crate) auto_recognize_content_type: bool,
    pub(crate) is_custom_domain: bool,
    pub(crate) dns_cache_time: isize,
    pub(crate) proxy_host: String,
    pub(crate) proxy_port: isize,
    pub(crate) proxy_username: String,
    pub(crate) proxy_password: String,
    pub(crate) disable_encoding_meta: bool,
    pub(crate) expect_100_continue_threshold: isize,
    pub(crate) high_latency_log_threshold: isize,
    pub(crate) user_agent_product_name: String,
    pub(crate) user_agent_soft_name: String,
    pub(crate) user_agent_soft_version: String,
    pub(crate) user_agent_customized_key_values: Option<HashMap<String, String>>,
    pub(crate) follow_redirect_times: isize,
    pub(crate) client_crt: String,
    pub(crate) client_key: String,
    pub(crate) ca_crt: String,

    pub(crate) user_agent: String,
    pub(crate) region: String,
    pub(crate) schema: String,
    pub(crate) domain: String,
    pub(crate) port: Option<isize>,

    pub(crate) schema_control: String,
    pub(crate) domain_control: String,
}

impl Default for ConfigHolder {
    fn default() -> Self {
        Self {
            user_agent: "".to_string(),
            region: DEFAULT_REGION.to_string(),
            max_retry_count: DEFAULT_MAX_RETRY_COUNT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            connection_timeout: DEFAULT_CONNECTION_TIMEOUT,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            idle_connection_time: DEFAULT_IDLE_CONNECTION_TIME,
            enable_crc: true,
            enable_verify_ssl: true,
            auto_recognize_content_type: true,
            is_custom_domain: false,
            dns_cache_time: DEFAULT_DNS_CACHE_TIME,
            proxy_host: "".to_string(),
            proxy_port: -1,
            proxy_username: "".to_string(),
            proxy_password: "".to_string(),
            disable_encoding_meta: false,
            expect_100_continue_threshold: DEFAULT_EXPECT_100_CONTINUE_THRESHOLD,
            high_latency_log_threshold: DEFAULT_HIGH_LATENCY_LOG_THRESHOLD,
            user_agent_product_name: "".to_string(),
            user_agent_soft_name: "".to_string(),
            user_agent_soft_version: "".to_string(),
            user_agent_customized_key_values: None,
            follow_redirect_times: 0,
            client_crt: "".to_string(),
            client_key: "".to_string(),
            schema: "".to_string(),
            domain: "".to_string(),
            port: None,
            schema_control: "".to_string(),
            domain_control: "".to_string(),
            ca_crt: "".to_string(),
        }
    }
}


impl ConfigHolder {
    pub(crate) fn check(&mut self, endpoint: impl Into<String>, region: impl Into<String>) -> Result<(), TosError> {
        let region = region.into().trim().to_owned();
        if region == "" {
            return TosError::client_error_result("no region specified");
        }
        let mut endpoint = endpoint.into().trim().to_owned().to_lowercase();
        if endpoint == "" {
            if let Some(value) = REGION_ENDPOINTS.get(region.as_str()) {
                endpoint = (*value).to_string();
            }
        }

        if endpoint == "" {
            return TosError::client_error_result("no endpoint specified");
        }

        let (schema, domain, port) = self.split_endpoint(endpoint.as_str())?;
        if TOS_S3_ENDPOINTS.contains_key(domain.as_str()) {
            return TosError::client_error_result("invalid endpoint, please use TOS endpoint rather than S3 endpoint");
        }

        self.region = region.to_owned();
        self.schema = schema;
        self.domain = domain;
        self.port = port;
        Ok(())
    }

    pub(crate) fn check_control(&mut self, control_endpoint: impl Into<String>) -> Result<(), TosError> {
        let region = self.region.as_str();
        if region == "" {
            return TosError::client_error_result("no region specified");
        }
        let mut control_endpoint = control_endpoint.into().trim().to_owned().to_lowercase();
        if control_endpoint == "" {
            if let Some(value) = REGION_CONTROL_ENDPOINTS.get(region) {
                control_endpoint = (*value).to_string();
            }
        }

        if control_endpoint != "" {
            let (schema, domain, port) = self.split_endpoint(control_endpoint.as_str())?;
            if port != self.port {
                return TosError::client_error_result("mismatched port between control endpoint and endpoint");
            }

            self.schema_control = schema;
            self.domain_control = domain;
        }
        Ok(())
    }

    pub(crate) fn gen_user_agent(&mut self) {
        if self.user_agent_product_name == "" && self.user_agent_soft_name == "" && self.user_agent_soft_version == ""
            && (self.user_agent_customized_key_values.is_none() || self.user_agent_customized_key_values.as_ref().unwrap().is_empty()) {
            self.user_agent = String::from("ve-tos-rust-sdk/".to_string() + env!("CARGO_PKG_VERSION") +
                " (" + std::env::consts::OS + "/" + std::env::consts::ARCH + ")");
        } else {
            let mut product_name = self.user_agent_product_name.as_str();
            if product_name == "" {
                product_name = UNDEFINED;
            }
            let mut soft_name = self.user_agent_soft_name.as_str();
            if soft_name == "" {
                soft_name = UNDEFINED;
            }
            let mut soft_version = self.user_agent_soft_version.as_str();
            if soft_version == "" {
                soft_version = UNDEFINED;
            }
            let mut user_agent = String::from(format!("ve-tos-rust-sdk/{} ({}/{}) -- {}/{}/{}",
                                                      env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH,
                                                      product_name, soft_name, soft_version));

            if let Some(kv) = self.user_agent_customized_key_values.as_ref() {
                user_agent.push('(');
                let mut keys = Vec::<&str>::with_capacity(kv.len());
                for key in kv.keys() {
                    keys.push(key);
                }
                keys.sort();
                for (idx, key) in keys.iter().enumerate() {
                    user_agent.push_str(key);
                    user_agent.push('/');
                    user_agent.push_str(kv.get(*key).unwrap());
                    if idx != keys.len() - 1 {
                        user_agent.push(';');
                    }
                }
                user_agent.push(')');
            }

            self.user_agent = user_agent;
        }
    }

    pub(crate) fn split_endpoint(&self, endpoint: &str) -> Result<(String, String, Option<isize>), TosError> {
        let mut endpoint = endpoint;
        while endpoint.len() > 0 && endpoint.ends_with("/") {
            endpoint = &endpoint[0..endpoint.len() - 1];
        }

        endpoint = endpoint.trim();
        if endpoint.len() == 0 {
            return Err(TosError::client_error("invalid endpoint"));
        }
        let mut schema = String::with_capacity(SCHEMA_HTTP.len());
        let domain;
        let port;
        if endpoint.starts_with(SCHEMA_HTTP) {
            schema.push_str(SCHEMA_HTTP);
            (domain, _, port) = self.parse_domain(endpoint)?;
        } else if endpoint.starts_with(SCHEMA_HTTPS) {
            schema.push_str(SCHEMA_HTTPS);
            (domain, _, port) = self.parse_domain(endpoint)?;
        } else {
            schema.push_str(SCHEMA_HTTPS);
            (domain, _, port) = self.parse_domain((SCHEMA_HTTPS.to_owned() + endpoint).as_str())?;
        }

        Ok((schema, domain, port))
    }

    pub(crate) fn parse_domain(&self, input: &str) -> Result<(String, String, Option<isize>), TosError> {
        let mut domain = String::with_capacity(input.len());
        match Url::parse(input) {
            Ok(u) => {
                if let Some(host) = u.host() {
                    let pt;
                    if let Some(port) = u.port() {
                        domain.push_str(&format!("{}:{}", host, port));
                        pt = Some(port as isize);
                    } else {
                        domain.push_str(host.to_string().as_str());
                        pt = None;
                    }
                    Ok((domain, u.scheme().to_string(), pt))
                } else {
                    Err(TosError::client_error("no host error"))
                }
            }
            Err(e) => {
                Err(TosError::client_error_with_cause("parse domain error", GenericError::UrlParseError(e)))
            }
        }
    }

    pub(crate) fn get_host(&self, bucket: &str) -> String {
        self.get_host_with_domain(bucket, "", self.is_custom_domain)
    }

    pub(crate) fn get_host_with_domain(&self, bucket: &str, domain: &str, is_custom_domain: bool) -> String {
        let mut domain = domain;
        if domain == "" {
            domain = &self.domain;
        }
        let mut host = String::with_capacity(bucket.len() + domain.len() + 1);
        if bucket != "" && !is_custom_domain {
            host += bucket;
            host += ".";
        }
        host += domain;
        host
    }

    pub(crate) fn get_endpoint(&self, bucket: &str, key: &str) -> String {
        self.get_endpoint_with_domain(bucket, key, "", "", true, self.is_custom_domain)
    }

    pub(crate) fn get_endpoint_with_domain(&self, bucket: &str, key: &str, schema: &str, domain: &str, must_add_key: bool, is_custom_domain: bool) -> String {
        let mut schema = schema;
        if schema == "" {
            schema = &self.schema;
        }
        let mut domain = domain;
        if domain == "" {
            domain = &self.domain;
        }
        let mut endpoint = String::with_capacity(schema.len() + domain.len() + bucket.len() + key.len() * 2 + 3);
        endpoint += schema;

        if bucket != "" && !is_custom_domain {
            endpoint += bucket;
            endpoint += ".";
        }
        endpoint += domain;

        if key != "" && must_add_key {
            endpoint += "/";
            endpoint += url_encode_with_safe(key, "/").as_str();
        }

        endpoint
    }
}