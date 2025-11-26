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
use super::bucket::*;
use super::config::ConfigHolder;
use super::credential::{CommonCredentials, CommonCredentialsProvider, Credentials, CredentialsProvider};
use super::error::{ErrorResponse, GenericError, TosError};
use super::internal::{auto_recognize_content_type, build_certificate, build_identity, check_bucket_and_key, check_need_retry, exceed_high_latency_log_threshold, get_request_url, parse_json, sleep_for_retry, trans_header_value, AdditionalContext, InputTranslator, OutputParser};
use crate::auth::{pre_signed_policy_url, pre_signed_post_signature, pre_signed_url, sign_header, PreSignedPolicyURLInput, PreSignedPolicyURLOutput, PreSignedPostSignatureInput, PreSignedPostSignatureOutput, PreSignedURLInput, PreSignedURLOutput, SignerAPI};
use crate::common::{Meta, RequestInfo, RequestInfoTrait};
use crate::constant::*;
use crate::enumeration::HttpMethodType::HttpMethodHead;
use crate::http::{HttpRequest, HttpResponse, RequestContext};
use crate::multipart::*;
use crate::object::*;
use crate::reader::{InternalReader, MultifunctionalReader};
use arc_swap::ArcSwap;
use reqwest::blocking::{Body, Client, RequestBuilder};
use reqwest::{redirect, Proxy};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Cursor, Read};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::log::{info, warn};

#[derive(Debug, Clone, Default)]
pub struct TosClientBuilder<P, C>
{
    ak: String,
    sk: String,
    security_token: String,
    region: String,
    endpoint: String,
    credentials_provider: Option<P>,
    config_holder: ConfigHolder,
    c: PhantomData<C>,
}

impl<P, C> TosClientBuilder<P, C>
where
    P: CredentialsProvider<C> + Debug,
    C: Credentials + Debug,
{
    pub fn build(mut self) -> Result<TosClientImpl<P, C>, TosError> {
        self.config_holder.check(self.endpoint, self.region)?;
        self.config_holder.gen_user_agent();
        let mut client = Client::builder()
            .user_agent(self.config_holder.user_agent.as_str())
            .tcp_nodelay(true)
            .tcp_keepalive(None)
            .no_gzip()
            .no_deflate()
            .no_brotli()
            .connect_timeout(Duration::from_millis(self.config_holder.connection_timeout as u64))
            .pool_idle_timeout(Duration::from_millis(self.config_holder.idle_connection_time as u64))
            .pool_max_idle_per_host(self.config_holder.max_connections as usize);
        if self.config_holder.request_timeout > 0 {
            client = client.timeout(Duration::from_millis(self.config_holder.request_timeout as u64));
        }

        if self.config_holder.follow_redirect_times > 0 {
            client = client.redirect(redirect::Policy::limited(self.config_holder.follow_redirect_times as usize));
        } else {
            client = client.redirect(redirect::Policy::none());
        }

        if self.config_holder.proxy_host != "" {
            let mut proxy_url = self.config_holder.proxy_host.as_str();
            while proxy_url.len() > 0 && proxy_url.ends_with("/") {
                proxy_url = &proxy_url[0..proxy_url.len() - 1];
            }

            if proxy_url != "" {
                let mut proxy_url = proxy_url.to_lowercase();
                if !proxy_url.starts_with(SCHEMA_HTTP) && !proxy_url.starts_with(SCHEMA_HTTPS) {
                    proxy_url = format!("{}{}", SCHEMA_HTTP, proxy_url);
                }
                if self.config_holder.proxy_port >= 0 {
                    proxy_url = format!("{}:{}", proxy_url, self.config_holder.proxy_port);
                }

                let (domain, schema, _) = self.config_holder.parse_domain(proxy_url.as_str())?;
                if self.config_holder.proxy_username != "" && self.config_holder.proxy_password != "" {
                    proxy_url = format!("{}//{}:{}@{}", schema, self.config_holder.proxy_username, self.config_holder.proxy_password, domain);
                } else {
                    proxy_url = format!("{}//{}", schema, domain);
                }
                match Proxy::http(proxy_url.as_str()) {
                    Err(e) => return Err(TosError::client_error_with_cause("build http proxy error", GenericError::DefaultError(e.to_string()))),
                    Ok(proxy) => {
                        client = client.proxy(proxy);
                    }
                }

                match Proxy::https(proxy_url) {
                    Err(e) => return Err(TosError::client_error_with_cause("build https proxy error", GenericError::DefaultError(e.to_string()))),
                    Ok(proxy) => {
                        client = client.proxy(proxy);
                    }
                }
            } else {
                client = client.no_proxy();
            }
        } else {
            client = client.no_proxy();
        }

        if !self.config_holder.enable_verify_ssl {
            client = client.danger_accept_invalid_certs(true);
            #[cfg(feature = "use-native-tls")]
            {
                client = client.danger_accept_invalid_hostnames(true);
            }
        }

        #[cfg(any(feature = "use-native-tls", feature = "use-rustls"))]
        if self.config_holder.ca_crt != "" {
            client = client.add_root_certificate(build_certificate(self.config_holder.ca_crt.as_str())?);
        }

        #[cfg(any(feature = "use-native-tls", feature = "use-rustls"))]
        if self.config_holder.client_crt != "" && self.config_holder.client_key != "" {
            client = client.identity(build_identity(self.config_holder.client_crt.as_str(), self.config_holder.client_key.as_str())?);
        }

        let cp;
        let mut credentials_can_refresh = false;
        match self.credentials_provider {
            Some(p) => {
                cp = p;
            }
            None => {
                match C::new(self.ak, self.sk, self.security_token) {
                    Err(ex) => return Err(TosError::client_error_with_cause("create credentials error",
                                                                            GenericError::DefaultError(ex.to_string()))),
                    Ok(c) => {
                        match P::new(c) {
                            Err(ex) => return Err(TosError::client_error_with_cause("create credentials provider error",
                                                                                    GenericError::DefaultError(ex.to_string()))),
                            Ok(p) => {
                                credentials_can_refresh = true;
                                cp = p;
                            }
                        }
                    }
                }
            }
        }

        match client.build() {
            Ok(client) => {
                Ok(TosClientImpl {
                    client,
                    config_holder: ArcSwap::from(Arc::new(self.config_holder)),
                    credentials_provider: ArcSwap::from(Arc::new(cp)),
                    credentials_can_refresh,
                    c: self.c,
                })
            }
            Err(e) => {
                Err(TosError::client_error_with_cause("build tos client error", GenericError::DefaultError(e.to_string())))
            }
        }
    }

    pub fn build_as_trait(self) -> Result<impl TosClient, TosError> {
        let client = self.build()?;
        Ok(client)
    }

    pub fn ak(mut self, ak: impl Into<String>) -> Self {
        self.ak = ak.into();
        self
    }

    pub fn sk(mut self, sk: impl Into<String>) -> Self {
        self.sk = sk.into();
        self
    }

    pub fn security_token(mut self, security_token: impl Into<String>) -> Self {
        self.security_token = security_token.into();
        self
    }

    pub fn credentials_provider(mut self, p: P) -> Self {
        self.credentials_provider = Some(p);
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn request_timeout(mut self, request_timeout: isize) -> Self {
        if request_timeout > 0 {
            self.config_holder.request_timeout = request_timeout;
        }
        self
    }

    pub fn connection_timeout(mut self, connection_timeout: isize) -> Self {
        if connection_timeout > 0 {
            self.config_holder.connection_timeout = connection_timeout;
        }
        self
    }

    pub fn max_connections(mut self, max_connections: isize) -> Self {
        if max_connections > 0 {
            self.config_holder.max_connections = max_connections;
        }
        self
    }
    pub fn idle_connection_time(mut self, idle_connection_time: isize) -> Self {
        if idle_connection_time > 0 {
            self.config_holder.idle_connection_time = idle_connection_time;
        }
        self
    }

    pub fn enable_crc(mut self, enable_crc: bool) -> Self {
        self.config_holder.enable_crc = enable_crc;
        self
    }

    pub fn enable_verify_ssl(mut self, enable_verify_ssl: bool) -> Self {
        self.config_holder.enable_verify_ssl = enable_verify_ssl;
        self
    }

    pub fn max_retry_count(mut self, max_retry_count: isize) -> Self {
        self.config_holder.max_retry_count = max_retry_count;
        self
    }
    pub fn auto_recognize_content_type(mut self, auto_recognize_content_type: bool) -> Self {
        self.config_holder.auto_recognize_content_type = auto_recognize_content_type;
        self
    }
    pub fn is_custom_domain(mut self, is_custom_domain: bool) -> Self {
        self.config_holder.is_custom_domain = is_custom_domain;
        self
    }
    pub fn proxy_host(mut self, proxy_host: impl Into<String>) -> Self {
        self.config_holder.proxy_host = proxy_host.into();
        self
    }
    pub fn proxy_port(mut self, proxy_host: isize) -> Self {
        self.config_holder.proxy_port = proxy_host.into();
        self
    }
    pub fn proxy_username(mut self, proxy_username: impl Into<String>) -> Self {
        self.config_holder.proxy_username = proxy_username.into();
        self
    }
    pub fn proxy_password(mut self, proxy_password: impl Into<String>) -> Self {
        self.config_holder.proxy_password = proxy_password.into();
        self
    }
    pub fn disable_encoding_meta(mut self, disable_encoding_meta: bool) -> Self {
        self.config_holder.disable_encoding_meta = disable_encoding_meta;
        self
    }
    pub fn expect_100_continue_threshold(mut self, expect_100_continue_threshold: isize) -> Self {
        self.config_holder.expect_100_continue_threshold = expect_100_continue_threshold;
        self
    }
    pub fn high_latency_log_threshold(mut self, high_latency_log_threshold: isize) -> Self {
        self.config_holder.high_latency_log_threshold = high_latency_log_threshold;
        self
    }
    pub fn user_agent_product_name(mut self, user_agent_product_name: impl Into<String>) -> Self {
        self.config_holder.user_agent_product_name = user_agent_product_name.into();
        self
    }
    pub fn user_agent_soft_name(mut self, user_agent_soft_name: impl Into<String>) -> Self {
        self.config_holder.user_agent_soft_name = user_agent_soft_name.into();
        self
    }
    pub fn user_agent_soft_version(mut self, user_agent_soft_version: impl Into<String>) -> Self {
        self.config_holder.user_agent_soft_version = user_agent_soft_version.into();
        self
    }
    pub fn user_agent_customized_key_values(mut self, user_agent_customized_key_values: impl Into<HashMap<String, String>>) -> Self {
        self.config_holder.user_agent_customized_key_values = Some(user_agent_customized_key_values.into());
        self
    }

    pub fn follow_redirect_times(mut self, follow_redirect_times: isize) -> Self {
        self.config_holder.follow_redirect_times = follow_redirect_times;
        self
    }
    #[cfg(any(feature = "use-native-tls", feature = "use-rustls"))]
    pub fn client_crt(mut self, client_crt: impl Into<String>) -> Self {
        self.config_holder.client_crt = client_crt.into();
        self
    }
    #[cfg(any(feature = "use-native-tls", feature = "use-rustls"))]
    pub fn client_key(mut self, client_key: impl Into<String>) -> Self {
        self.config_holder.client_key = client_key.into();
        self
    }
    #[cfg(any(feature = "use-native-tls", feature = "use-rustls"))]
    pub fn ca_crt(mut self, ca_crt: impl Into<String>) -> Self {
        self.config_holder.ca_crt = ca_crt.into();
        self
    }
}

pub fn builder() -> TosClientBuilder<CommonCredentialsProvider<CommonCredentials>, CommonCredentials> {
    TosClientBuilder::default()
}

pub trait ConfigAware {
    fn is_custom_domain(&self) -> bool;
}

pub trait TosClient: BucketAPI + ObjectAPI + MultipartAPI + SignerAPI + ConfigAware + Debug {
    fn refresh_credentials(&self, ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>) -> bool;
    fn refresh_endpoint_region(&self, endpoint: impl Into<String>, region: impl Into<String>) -> bool;
}

pub type DefaultTosClient = TosClientImpl<CommonCredentialsProvider<CommonCredentials>, CommonCredentials>;

#[derive(Debug)]
pub struct TosClientImpl<P, C> {
    pub(crate) client: Client,
    pub(crate) config_holder: ArcSwap<ConfigHolder>,
    pub(crate) credentials_provider: ArcSwap<P>,
    pub(crate) credentials_can_refresh: bool,
    c: PhantomData<C>,
}


impl<P, C> BucketAPI for TosClientImpl<P, C>
where
    C: Credentials,
    P: CredentialsProvider<C>,
{
    fn create_bucket(&self, input: &CreateBucketInput) -> Result<CreateBucketOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn head_bucket(&self, input: &HeadBucketInput) -> Result<HeadBucketOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn delete_bucket(&self, input: &DeleteBucketInput) -> Result<DeleteBucketOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn list_buckets(&self, input: &ListBucketsInput) -> Result<ListBucketsOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }
}

impl<P, C> ObjectAPI for TosClientImpl<P, C>
where
    C: Credentials,
    P: CredentialsProvider<C>,
{
    fn copy_object(&self, input: &CopyObjectInput) -> Result<CopyObjectOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn delete_object(&self, input: &DeleteObjectInput) -> Result<DeleteObjectOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn delete_multi_objects(&self, input: &DeleteMultiObjectsInput) -> Result<DeleteMultiObjectsOutput, TosError> {
        self.do_request::<DeleteMultiObjectsInput, DeleteMultiObjectsOutput, InternalReader<Cursor<Vec<u8>>>>(input)
    }

    fn get_object(&self, input: &GetObjectInput) -> Result<GetObjectOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn get_object_to_file(&self, input: &GetObjectToFileInput) -> Result<GetObjectToFileOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn get_object_acl(&self, input: &GetObjectACLInput) -> Result<GetObjectACLOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn head_object(&self, input: &HeadObjectInput) -> Result<HeadObjectOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn append_object<B>(&self, input: &AppendObjectInput<B>) -> Result<AppendObjectOutput, TosError>
    where
        B: Read + Send + 'static,
    {
        self.do_request(input)
    }

    fn append_object_from_buffer(&self, input: &AppendObjectFromBufferInput) -> Result<AppendObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<Cursor<Vec<u8>>>>(input)
    }

    fn list_objects(&self, input: &ListObjectsInput) -> Result<ListObjectsOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn list_objects_type2(&self, input: &ListObjectsType2Input) -> Result<ListObjectsType2Output, TosError> {
        if input.list_only_once {
            return self.do_request::<_, _, Cursor<String>>(input);
        }

        let mut input = input.clone();
        if input.max_keys <= 0 {
            input.max_keys = DEFAULT_MAX_KEYS;
        }
        let mut _output: Option<ListObjectsType2Output> = None;
        loop {
            let mut temp_output = self.do_request::<ListObjectsType2Input, ListObjectsType2Output, Cursor<String>>(&input)?;
            if _output.is_none() {
                _output = Some(temp_output);
            } else {
                let output = _output.as_mut().unwrap();
                output.key_count += temp_output.key_count;
                output.is_truncated = temp_output.is_truncated;
                output.next_continuation_token = temp_output.next_continuation_token;
                output.contents.append(&mut temp_output.contents);
                output.common_prefixes.append(&mut temp_output.common_prefixes);
            }

            let output = _output.as_ref().unwrap();
            if !output.is_truncated || output.contents.len() + output.common_prefixes.len() >= input.max_keys as usize || output.key_count >= input.max_keys {
                break;
            }
            input.continuation_token = output.next_continuation_token.clone();
            input.max_keys = input.max_keys - output.key_count;
        }

        Ok(_output.unwrap())
    }

    fn list_object_versions(&self, input: &ListObjectVersionsInput) -> Result<ListObjectVersionsOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn put_object<B>(&self, input: &PutObjectInput<B>) -> Result<PutObjectOutput, TosError>
    where
        B: Read + Send + 'static,
    {
        self.do_request(input)
    }

    fn put_object_from_buffer(&self, input: &PutObjectFromBufferInput) -> Result<PutObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<Cursor<Vec<u8>>>>(input)
    }

    fn put_object_from_file(&self, input: &PutObjectFromFileInput) -> Result<PutObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<File>>(input)
    }

    fn put_object_acl(&self, input: &PutObjectACLInput) -> Result<PutObjectACLOutput, TosError> {
        self.do_request::<_, _, InternalReader<Cursor<Vec<u8>>>>(input)
    }

    fn set_object_meta(&self, input: &SetObjectMetaInput) -> Result<SetObjectMetaOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }
}

impl<P, C> MultipartAPI for TosClientImpl<P, C>
where
    C: Credentials,
    P: CredentialsProvider<C>,
{
    fn create_multipart_upload(&self, input: &CreateMultipartUploadInput) -> Result<CreateMultipartUploadOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn upload_part<B>(&self, input: &UploadPartInput<B>) -> Result<UploadPartOutput, TosError>
    where
        B: Read + Send + 'static,
    {
        self.do_request(input)
    }

    fn upload_part_from_buffer(&self, input: &UploadPartFromBufferInput) -> Result<UploadPartOutput, TosError> {
        self.do_request::<_, _, InternalReader<Cursor<Vec<u8>>>>(input)
    }

    fn upload_part_from_file(&self, input: &UploadPartFromFileInput) -> Result<UploadPartOutput, TosError> {
        self.do_request::<_, _, InternalReader<File>>(input)
    }

    fn complete_multipart_upload(&self, input: &CompleteMultipartUploadInput) -> Result<CompleteMultipartUploadOutput, TosError> {
        self.do_request::<_, _, InternalReader<Cursor<Vec<u8>>>>(input)
    }

    fn abort_multipart_upload(&self, input: &AbortMultipartUploadInput) -> Result<AbortMultipartUploadOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn upload_part_copy(&self, input: &UploadPartCopyInput) -> Result<UploadPartCopyOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn list_multipart_uploads(&self, input: &ListMultipartUploadsInput) -> Result<ListMultipartUploadsOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }

    fn list_parts(&self, input: &ListPartsInput) -> Result<ListPartsOutput, TosError> {
        self.do_request::<_, _, Cursor<String>>(input)
    }
}

impl<P, C> ConfigAware for TosClientImpl<P, C>
{
    fn is_custom_domain(&self) -> bool {
        self.config_holder.load().is_custom_domain
    }
}

impl<P, C> SignerAPI for TosClientImpl<P, C>
where
    C: Credentials + Debug,
    P: CredentialsProvider<C> + Debug,
{
    fn pre_signed_url(&self, input: &PreSignedURLInput) -> Result<PreSignedURLOutput, TosError> {
        let (ak, sk, security_token) = self.load_credentials()?;
        pre_signed_url(&self.config_holder, &ak, &sk, &security_token, input)
    }

    fn pre_signed_post_signature(&self, input: &PreSignedPostSignatureInput) -> Result<PreSignedPostSignatureOutput, TosError> {
        let (ak, sk, security_token) = self.load_credentials()?;
        pre_signed_post_signature(&self.config_holder, &ak, &sk, &security_token, input)
    }

    fn pre_signed_policy_url(&self, input: &PreSignedPolicyURLInput) -> Result<PreSignedPolicyURLOutput, TosError> {
        let (ak, sk, security_token) = self.load_credentials()?;
        pre_signed_policy_url(&self.config_holder, &ak, &sk, &security_token, input)
    }
}

impl<P, C> TosClient for TosClientImpl<P, C>
where
    P: CredentialsProvider<C> + Debug,
    C: Credentials + Debug,
{
    fn refresh_credentials(&self, ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>) -> bool {
        if !self.credentials_can_refresh {
            return false;
        }

        match C::new(ak, sk, security_token) {
            Err(_) => false,
            Ok(c) => {
                match P::new(c) {
                    Err(_) => false,
                    Ok(p) => {
                        self.credentials_provider.store(Arc::new(p));
                        true
                    }
                }
            }
        }
    }

    fn refresh_endpoint_region(&self, endpoint: impl Into<String>, region: impl Into<String>) -> bool {
        let c = self.config_holder.load();
        let mut config_holder = ConfigHolder {
            max_retry_count: c.max_retry_count,
            request_timeout: c.request_timeout,
            connection_timeout: c.connection_timeout,
            max_connections: c.max_connections,
            idle_connection_time: c.idle_connection_time,
            enable_crc: c.enable_crc,
            enable_verify_ssl: c.enable_verify_ssl,
            auto_recognize_content_type: c.auto_recognize_content_type,
            is_custom_domain: c.is_custom_domain,
            dns_cache_time: c.dns_cache_time,
            proxy_host: c.proxy_host.to_string(),
            proxy_port: c.proxy_port,
            proxy_username: c.proxy_username.clone(),
            proxy_password: c.proxy_password.clone(),
            disable_encoding_meta: c.disable_encoding_meta,
            expect_100_continue_threshold: c.expect_100_continue_threshold,
            high_latency_log_threshold: c.high_latency_log_threshold,
            user_agent_product_name: c.user_agent_product_name.clone(),
            user_agent_soft_name: c.user_agent_soft_name.clone(),
            user_agent_soft_version: c.user_agent_soft_version.clone(),
            user_agent_customized_key_values: c.user_agent_customized_key_values.clone(),
            follow_redirect_times: c.follow_redirect_times,
            client_crt: c.client_crt.clone(),
            client_key: c.client_key.clone(),
            ca_crt: c.ca_crt.clone(),
            user_agent: c.user_agent.clone(),
            region: "".to_string(),
            schema: "".to_string(),
            domain: "".to_string(),
            port: None,
            schema_control: c.schema_control.clone(),
            domain_control: c.domain_control.clone(),
        };
        if let Err(_) = config_holder.check(endpoint, region) {
            return false;
        }
        self.config_holder.store(Arc::new(config_holder));
        true
    }
}

impl<P, C> TosClientImpl<P, C>
where
    P: CredentialsProvider<C>,
    C: Credentials,
{
    fn load_credentials(&self) -> Result<(String, String, String), TosError> {
        let cred = self.credentials_provider.load();
        match cred.credentials(CREDENTIALS_EXPIRES) {
            Err(ex) => Err(TosError::client_error_with_cause("load credentials error", GenericError::DefaultError(ex.to_string()))),
            Ok(c) => Ok((c.ak().to_string(), c.sk().to_string(), c.security_token().to_string())),
        }
    }

    fn do_request<T, K, B>(&self, input: &T) -> Result<K, TosError>
    where
        T: InputTranslator<B>,
        K: OutputParser + RequestInfoTrait,
        B: Read + Send + 'static,
    {
        let config_holder = self.config_holder.load();
        let operation = check_bucket_and_key(input, config_holder.is_custom_domain)?;
        let mut retry_count = 0;
        let max_retry_count = config_holder.max_retry_count;
        loop {
            let start = Instant::now();
            let mut ac = AdditionalContext::new();
            let result = self.do_request_once::<T, K, B>(input, retry_count, config_holder.clone(), &mut ac);
            let elapsed_ms = start.elapsed().as_millis();
            let exceed = exceed_high_latency_log_threshold(config_holder.high_latency_log_threshold, elapsed_ms, ac.request_size, operation);
            match result {
                Ok(k) => {
                    if exceed {
                        warn!("high latency request {} succeed, http status: {}, request id: {}, cost: {} ms", operation, k.status_code(), k.request_id(), elapsed_ms);
                    } else {
                        info!("do {} succeed, http status: {}, request id: {}, cost: {} ms", operation, k.status_code(), k.request_id(), elapsed_ms);
                    }
                    return Ok(k);
                }
                Err(mut e) => {
                    match &e {
                        TosError::TosClientError { .. } => {
                            if exceed {
                                warn!("high latency request {} failed, cost: {} ms", operation, elapsed_ms);
                            } else {
                                warn!("do {} failed, cost: {} ms", operation, elapsed_ms);
                            }
                        }
                        TosError::TosServerError { status_code, request_id, ec, .. } => {
                            if exceed {
                                if status_code.to_owned() < 500 {
                                    warn!("high latency request {} finished, http status: {}, request id: {}, ec: {}, cost: {} ms", operation, status_code,
                                    request_id, ec, elapsed_ms);
                                } else {
                                    warn!("high latency request {} finished, http status: {}, request id: {}, ec: {}, cost: {} ms", operation, status_code,
                                    request_id, ec, elapsed_ms);
                                }
                            } else {
                                if status_code.to_owned() < 500 {
                                    warn!("do {} finished, http status: {}, request id: {}, ec: {}, cost: {} ms", operation, status_code,
                                    request_id, ec, elapsed_ms);
                                } else {
                                    info!("do {} finished, http status: {}, request id: {}, ec: {}, cost: {} ms", operation, status_code,
                                    request_id, ec, elapsed_ms);
                                }
                            }
                        }
                    }

                    let (retry_after, need_retry) = check_need_retry(&e, retry_count, max_retry_count, operation);
                    if !need_retry {
                        if let Some(request_url) = ac.request_url {
                            e.set_request_url(request_url);
                        }
                        return Err(e);
                    }
                    sleep_for_retry(retry_count, retry_after);
                    retry_count += 1;
                }
            }
        }
    }

    fn do_request_once<'a, 'b, T, K, B>(&self, input: &'b T, retry_count: isize, config_holder: Arc<ConfigHolder>, ac: &mut AdditionalContext<'a>) -> Result<K, TosError>
    where
        T: InputTranslator<B>,
        K: OutputParser,
        B: Read + Send + 'static,
        'b: 'a,
    {
        let mut request = input.trans(config_holder)?;
        ac.request_host = input.request_host();
        ac.request_date = input.request_date();
        ac.request_header = input.request_header();
        ac.request_query = input.request_query();
        let body = request.body.take();
        request.retry_count = retry_count;
        let mut response = self.do_request_by_client(&mut request, body, ac)?;
        let (request_info, meta) = self.check_response(&request, &mut response)?;
        if request.operation == GET_OBJECT_TO_FILE_OPERATION {
            if let Some(cl) = response.content_length() {
                ac.request_size = cl as i64;
            }
            let result = K::parse(request, response, request_info, meta);
            return result;
        }
        K::parse(request, response, request_info, meta)
    }

    fn do_request_by_client<'a, 'c, B>(&self, request: &mut HttpRequest<'c, B>, body: Option<B>, ac: &mut AdditionalContext<'a>) -> Result<HttpResponse, TosError>
    where
        B: Read + Send + 'static,
        'a: 'c,
    {
        let (ak, sk, security_token) = self.load_credentials()?;
        let config_holder = self.config_holder.load();
        auto_recognize_content_type(request, config_holder.auto_recognize_content_type);
        sign_header(request, &ak, &sk, &security_token, config_holder.as_ref(), ac)?;
        request.enable_crc = config_holder.enable_crc;
        let request_url = get_request_url(request, config_holder.as_ref(), false);
        ac.request_url = Some(request_url.clone());
        let mut rb = self.client.request(request.method.as_http_method(), request_url);
        let mut cl = -1i64;
        for kv in &request.header {
            if *kv.0 == HEADER_CONTENT_LENGTH || *kv.0 == HEADER_CONTENT_LENGTH_LOWER {
                if let Ok(x) = kv.1.parse::<i64>() {
                    cl = x;
                }
            }
            rb = rb.header(*kv.0, kv.1);
        }

        if let Some(meta) = &request.meta {
            for kv in meta {
                rb = rb.header(kv.0, kv.1);
            }
        }

        if request.retry_count > 0 {
            rb = rb.header(HEADER_SDK_RETRY_COUNT, format!("attempt={}; max={}", request.retry_count, config_holder.max_retry_count));
        }
        if config_holder.expect_100_continue_threshold > 0 && cl > config_holder.expect_100_continue_threshold as i64 {
            rb = rb.header(HEADER_EXPECT, "100-continue");
        }
        let is_upload_operation = ALL_UPLOAD_OPERATIONS.contains_key(request.operation);
        let calc_crc = config_holder.enable_crc && is_upload_operation;
        let crc64 = Arc::new(AtomicU64::new(0u64));
        // add body
        if let Some(bd) = body {
            if calc_crc {
                let mut reader = MultifunctionalReader::new(bd, Some(crc64.clone()), cl, request);
                if let Some(ref rc) = request.request_context {
                    if let Some(init_crc64) = rc.init_crc64 {
                        reader.init_crc64 = Some(init_crc64);
                    }

                    if is_upload_operation {
                        if let Some(ref rl) = rc.rate_limiter {
                            reader.set_rate_limiter(rl.clone());
                        }

                        if let Some(ref dts) = rc.data_transfer_listener {
                            reader.set_data_transfer_listener(dts.clone());
                        }
                    }
                }
                rb = self.add_body(rb, reader, cl);
            } else if is_upload_operation {
                if let Some(ref rc) = request.request_context {
                    let mut reader = MultifunctionalReader::new(bd, None, cl, request);
                    if let Some(ref rl) = rc.rate_limiter {
                        reader.set_rate_limiter(rl.clone());
                    }

                    if let Some(ref dts) = rc.data_transfer_listener {
                        reader.set_data_transfer_listener(dts.clone());
                    }
                    rb = self.add_body(rb, reader, cl);
                } else {
                    rb = self.add_body(rb, bd, cl);
                }
            } else {
                rb = self.add_body(rb, bd, cl);
            }
        } else if cl == -1 {
            rb = rb.header(HEADER_CONTENT_LENGTH, 0);
        }

        match rb.build() {
            Ok(req) => {
                let result = self.client.execute(req);
                ac.request_size = cl;
                match result {
                    Ok(resp) => {
                        if calc_crc {
                            let result = crc64.load(Ordering::Acquire);
                            if request.request_context.is_none() {
                                let mut rc = RequestContext::default();
                                rc.crc64 = Some(result);
                                request.request_context = Some(rc)
                            } else {
                                request.request_context.as_mut().unwrap().crc64 = Some(result);
                            }
                        }
                        Ok(resp)
                    }
                    Err(e) => {
                        Err(TosError::client_error_with_cause("do request error", GenericError::HttpRequestError(e.to_string())))
                    }
                }
            }
            Err(e) => {
                Err(TosError::client_error_with_cause("build request error", GenericError::DefaultError(e.to_string())))
            }
        }
    }

    fn add_body<B>(&self, mut rb: RequestBuilder, bd: B, cl: i64) -> RequestBuilder
    where
        B: Read + Send + 'static,
    {
        if cl >= 0 {
            rb = rb.body(Body::sized(bd, cl as u64));
        } else {
            rb = rb.body(Body::new(bd));
        }
        rb
    }

    fn check_response<B>(&self, request: &HttpRequest<B>, response: &mut HttpResponse) -> Result<(RequestInfo, Meta), TosError> {
        let status_code = response.status().as_u16();
        let mut header = HashMap::<String, String>::with_capacity(response.headers().len());
        let mut request_id = "".to_string();
        let mut id2 = "".to_string();
        let mut ec = "".to_string();
        let mut k;
        let mut v;
        let mut meta = Meta::new();
        for (key, value) in response.headers() {
            k = key.to_string();
            v = trans_header_value(value);
            if k == HEADER_REQUEST_ID {
                request_id = trans_header_value(value);
            } else if k == HEADER_ID2 {
                id2 = trans_header_value(value);
            } else if k == HEADER_EC {
                ec = trans_header_value(value);
            } else if k.starts_with(HEADER_PREFIX_META) {
                if let Ok(dk) = urlencoding::decode(&k[HEADER_PREFIX_META.len()..]) {
                    if let Ok(dv) = urlencoding::decode(v.as_str()) {
                        meta.insert(dk.to_string(), dv.to_string());
                    }
                }
            }
            header.insert(k, v);
        }

        let request_info = RequestInfo {
            status_code: status_code as isize,
            request_id,
            id2,
            header,
        };

        if status_code >= 300 {
            if request.method != HttpMethodHead {
                if let Ok(error_response) = parse_json::<ErrorResponse>(response) {
                    // println!("{}", error_response.canonical_request);
                    // println!("{}", error_response.string_to_sign);
                    // println!("{}", error_response.signature_provided);
                    return Err(TosError::server_error_with_code(error_response.code, error_response.ec, error_response.key, error_response.message,
                                                                error_response.host_id, error_response.resource, request_info));
                }
            }
            return Err(TosError::server_error_with_code("", ec, "", String::from("unexpected status code: ") + response.status().as_str(),
                                                        "", "", request_info));
        }

        Ok((request_info, meta))
    }
}