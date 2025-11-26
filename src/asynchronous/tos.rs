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
use crate::asynchronous::bucket::BucketAPI;
use crate::asynchronous::common::DataTransferListener;
use crate::asynchronous::control::ControlAPI;
use crate::asynchronous::http::HttpResponse;
use crate::asynchronous::internal::{AsyncInputTranslator, OutputParser};
use crate::asynchronous::multipart::MultipartAPI;
use crate::asynchronous::object::ObjectAPI;
use crate::asynchronous::paginator::PaginatorAPI;
use crate::asynchronous::reader::StreamVec;
use crate::auth::{pre_signed_policy_url, pre_signed_post_signature, pre_signed_url, sign_header, PreSignedPolicyURLInput, PreSignedPolicyURLOutput, PreSignedPostSignatureInput, PreSignedPostSignatureOutput, PreSignedURLInput, PreSignedURLOutput, SignerAPI};
use crate::bucket::{CreateBucketInput, CreateBucketOutput, DeleteBucketCORSInput, DeleteBucketCORSOutput, DeleteBucketCustomDomainInput, DeleteBucketCustomDomainOutput, DeleteBucketEncryptionInput, DeleteBucketEncryptionOutput, DeleteBucketInput, DeleteBucketInventoryInput, DeleteBucketInventoryOutput, DeleteBucketLifecycleInput, DeleteBucketLifecycleOutput, DeleteBucketMirrorBackInput, DeleteBucketMirrorBackOutput, DeleteBucketOutput, DeleteBucketPolicyInput, DeleteBucketPolicyOutput, DeleteBucketRealTimeLogInput, DeleteBucketRealTimeLogOutput, DeleteBucketRenameInput, DeleteBucketRenameOutput, DeleteBucketReplicationInput, DeleteBucketReplicationOutput, DeleteBucketTaggingInput, DeleteBucketTaggingOutput, DeleteBucketWebsiteInput, DeleteBucketWebsiteOutput, DoesBucketExistInput, GetBucketACLInput, GetBucketACLOutput, GetBucketAccessMonitorInput, GetBucketAccessMonitorOutput, GetBucketCORSInput, GetBucketCORSOutput, GetBucketEncryptionInput, GetBucketEncryptionOutput, GetBucketInfoInput, GetBucketInfoOutput, GetBucketInventoryInput, GetBucketInventoryOutput, GetBucketLifecycleInput, GetBucketLifecycleOutput, GetBucketLocationInput, GetBucketLocationOutput, GetBucketMirrorBackInput, GetBucketMirrorBackOutput, GetBucketNotificationType2Input, GetBucketNotificationType2Output, GetBucketPolicyInput, GetBucketPolicyOutput, GetBucketRealTimeLogInput, GetBucketRealTimeLogOutput, GetBucketRenameInput, GetBucketRenameOutput, GetBucketReplicationInput, GetBucketReplicationOutput, GetBucketTaggingInput, GetBucketTaggingOutput, GetBucketTrashInput, GetBucketTrashOutput, GetBucketTypeInput, GetBucketTypeOutput, GetBucketVersioningInput, GetBucketVersioningOutput, GetBucketWebsiteInput, GetBucketWebsiteOutput, HeadBucketInput, HeadBucketOutput, ListBucketCustomDomainInput, ListBucketCustomDomainOutput, ListBucketInventoryInput, ListBucketInventoryOutput, ListBucketsInput, ListBucketsOutput, PutBucketACLInput, PutBucketACLOutput, PutBucketAccessMonitorInput, PutBucketAccessMonitorOutput, PutBucketCORSInput, PutBucketCORSOutput, PutBucketCustomDomainInput, PutBucketCustomDomainOutput, PutBucketEncryptionInput, PutBucketEncryptionOutput, PutBucketInventoryInput, PutBucketInventoryOutput, PutBucketLifecycleInput, PutBucketLifecycleOutput, PutBucketMirrorBackInput, PutBucketMirrorBackOutput, PutBucketNotificationType2Input, PutBucketNotificationType2Output, PutBucketPolicyInput, PutBucketPolicyOutput, PutBucketRealTimeLogInput, PutBucketRealTimeLogOutput, PutBucketRenameInput, PutBucketRenameOutput, PutBucketReplicationInput, PutBucketReplicationOutput, PutBucketStorageClassInput, PutBucketStorageClassOutput, PutBucketTaggingInput, PutBucketTaggingOutput, PutBucketTrashInput, PutBucketTrashOutput, PutBucketVersioningInput, PutBucketVersioningOutput, PutBucketWebsiteInput, PutBucketWebsiteOutput};
use crate::common::{GenericInput, RequestInfoTrait};
use crate::config::ConfigHolder;
use crate::constant::{ALL_UPLOAD_OPERATIONS, BASE_DELAY_MS, CREDENTIALS_EXPIRES, DEFAULT_MAX_KEYS, GET_OBJECT_TO_FILE_OPERATION, HEADER_CONTENT_LENGTH, HEADER_CONTENT_LENGTH_LOWER, HEADER_EXPECT, HEADER_SDK_RETRY_COUNT, MAX_DELAY_MS, SCHEMA_HTTP, SCHEMA_HTTPS};
use crate::control::{DeleteQosPolicyInput, DeleteQosPolicyOutput, GetQosPolicyInput, GetQosPolicyOutput, PutQosPolicyInput, PutQosPolicyOutput};
use crate::credential::{CommonCredentials, CommonCredentialsProvider, Credentials, CredentialsProvider, EnvCredentialsProvider, StaticCredentialsProvider};
use crate::enumeration::BucketType;
use crate::error::{GenericError, TosError};
use crate::http::{HttpRequest, RequestContext};
use crate::internal::{auto_recognize_content_type, build_certificate, build_identity, check_bucket_and_key, check_need_retry, exceed_high_latency_log_threshold, get_request_url, AdditionalContext, InputTranslator, MockAsyncInputTranslator};
use crate::multipart::{AbortMultipartUploadInput, AbortMultipartUploadOutput, CompleteMultipartUploadInput, CompleteMultipartUploadOutput, CreateMultipartUploadInput, CreateMultipartUploadOutput, ListMultipartUploadsInput, ListMultipartUploadsOutput, ListPartsInput, ListPartsOutput, UploadPartCopyInput, UploadPartCopyOutput, UploadPartFromBufferInput, UploadPartInput, UploadPartOutput};
use crate::object::{AppendObjectBasicInput, AppendObjectFromBufferInput, AppendObjectInput, AppendObjectOutput, CopyObjectInput, CopyObjectOutput, DeleteMultiObjectsInput, DeleteMultiObjectsOutput, DeleteObjectInput, DeleteObjectOutput, DeleteObjectTaggingInput, DeleteObjectTaggingOutput, DoesObjectExistInput, FetchObjectInput, FetchObjectOutput, GetFetchTaskInput, GetFetchTaskOutput, GetFileStatusInput, GetFileStatusOutput, GetObjectACLInput, GetObjectACLOutput, GetObjectInput, GetObjectOutput, GetObjectTaggingInput, GetObjectTaggingOutput, GetSymlinkInput, GetSymlinkOutput, HeadObjectInput, HeadObjectOutput, ListObjectVersionsInput, ListObjectVersionsOutput, ListObjectsType2Input, ListObjectsType2Output, ModifyObjectFromBufferInput, ModifyObjectInput, ModifyObjectOutput, PutFetchTaskInput, PutFetchTaskOutput, PutObjectACLInput, PutObjectACLOutput, PutObjectBasicInput, PutObjectFromBufferInput, PutObjectInput, PutObjectOutput, PutObjectTaggingInput, PutObjectTaggingOutput, PutSymlinkInput, PutSymlinkOutput, RenameObjectInput, RenameObjectOutput, RestoreObjectInput, RestoreObjectOutput, SetObjectMetaInput, SetObjectMetaOutput, SetObjectTimeInput, SetObjectTimeOutput};
use crate::reader::{InternalReader, MultifunctionalReader};
use crate::tos::ConfigAware;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::future::BoxFuture;
use futures_core::Stream;
use reqwest::{redirect, Body, Client, Proxy, RequestBuilder};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicI8, AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tracing::log::{info, warn};

#[async_trait]
pub trait AsyncRuntime {
    type JoinError: Error;
    async fn sleep(&self, duration: Duration);
    fn spawn<'a, F>(&self, future: F) -> BoxFuture<'a, Result<F::Output, Self::JoinError>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn block_on<F: Future>(&self, future: F) -> F::Output;
}


#[derive(Debug, Clone, Default)]
pub struct TosClientBuilder<P, C, S>
{
    ak: String,
    sk: String,
    security_token: String,
    region: String,
    endpoint: String,
    control_endpoint: String,
    credentials_provider: Option<P>,
    config_holder: ConfigHolder,
    async_runtime: S,
    c: PhantomData<C>,
}

impl<P, C, S> TosClientBuilder<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    pub fn build(mut self) -> Result<TosClientImpl<P, C, S>, TosError> {
        self.config_holder.check(self.endpoint, self.region)?;
        self.config_holder.check_control(self.control_endpoint)?;
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

        let async_runtime = Arc::new(self.async_runtime);

        #[cfg(feature = "tokio-runtime")]
        if self.config_holder.dns_cache_time > 0 {
            let port;
            if let Some(pt) = self.config_holder.port {
                port = pt;
            } else if self.config_holder.schema == SCHEMA_HTTPS {
                port = 443;
            } else {
                port = 80;
            }
            client = client.dns_resolver(Arc::new(crate::asynchronous::dns::InternalDnsResolver::new(self.config_holder.dns_cache_time, port)));
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
                #[cfg(not(feature = "tokio-runtime"))]
                {
                    Ok(TosClientImpl {
                        client,
                        config_holder: ArcSwap::from(Arc::new(self.config_holder)),
                        credentials_provider: ArcSwap::from(Arc::new(cp)),
                        credentials_can_refresh,
                        async_runtime,
                        c: self.c,
                        closed: Arc::new(AtomicI8::new(0)),
                    })
                }

                #[cfg(feature = "tokio-runtime")]
                {
                    let tos_client = TosClientImpl {
                        client,
                        config_holder: ArcSwap::from(Arc::new(self.config_holder)),
                        credentials_provider: ArcSwap::from(Arc::new(cp)),
                        credentials_can_refresh,
                        async_runtime,
                        c: self.c,
                        closed: Arc::new(AtomicI8::new(0)),
                        inner_credentials: Arc::new(tokio::sync::RwLock::new(None)),
                        cached_buckets: tokio::sync::RwLock::new(HashMap::new()),
                    };
                    if !credentials_can_refresh {
                        tos_client.async_refresh_credentials();
                    }
                    Ok(tos_client)
                }
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

    pub fn control_endpoint(mut self, control_endpoint: impl Into<String>) -> Self {
        self.control_endpoint = control_endpoint.into();
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
    #[cfg(feature = "tokio-runtime")]
    pub fn dns_cache_time(mut self, dns_cache_time: isize) -> Self {
        self.config_holder.dns_cache_time = dns_cache_time;
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
    pub fn async_runtime(mut self, async_runtime: impl Into<S>) -> Self {
        self.async_runtime = async_runtime.into();
        self
    }
}

pub fn builder<S>() -> TosClientBuilder<CommonCredentialsProvider<CommonCredentials>, CommonCredentials, S>
where
    S: AsyncRuntime + Default,
{
    TosClientBuilder::default()
}

pub fn builder_common<P, C, S>() -> TosClientBuilder<P, C, S>
where
    S: AsyncRuntime + Default,
    P: CredentialsProvider<C> + Send + Sync + Default + 'static,
    C: Credentials + Send + Sync + Default + 'static,
{
    TosClientBuilder::default()
}

pub fn static_credentials_provider(ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>)
                                   -> StaticCredentialsProvider<CommonCredentials> {
    StaticCredentialsProvider::new(ak, sk, security_token).unwrap()
}

pub fn env_credentials_provider() -> EnvCredentialsProvider<CommonCredentials> {
    EnvCredentialsProvider::new().unwrap()
}

pub struct BufferStream {
    inner: Option<Vec<u8>>,
}

impl Stream for BufferStream {
    type Item = Result<Bytes, crate::error::CommonError>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.inner.is_none() {
            return Poll::Ready(None);
        }
        Poll::Ready(Some(Ok(Bytes::from(self.inner.take().unwrap()))))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            None => (0, None),
            Some(v) => (0, Some(v.len()))
        }
    }
}

pub fn new_stream(data: impl AsRef<[u8]>) -> BufferStream {
    BufferStream { inner: Some(data.as_ref().to_owned()) }
}

pub fn new_stream_nocopy(data: impl Into<Vec<u8>>) -> BufferStream {
    BufferStream { inner: Some(data.into()) }
}

#[async_trait]
pub trait TosClient: BucketAPI + ObjectAPI + MultipartAPI + PaginatorAPI + ControlAPI + SignerAPI + ConfigAware {
    fn refresh_credentials(&self, ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>) -> bool;
    fn refresh_endpoint_region(&self, endpoint: impl Into<String>, region: impl Into<String>) -> bool;
}

#[cfg(feature = "tokio-runtime")]
pub(crate) type BucketCache = (GetBucketTypeOutput, chrono::DateTime<chrono::Utc>);

#[derive(Debug)]
pub struct TosClientImpl<P, C, S> {
    pub(crate) client: Client,
    pub(crate) config_holder: ArcSwap<ConfigHolder>,
    pub(crate) credentials_provider: ArcSwap<P>,
    pub(crate) async_runtime: Arc<S>,
    pub(crate) c: PhantomData<C>,
    pub(crate) credentials_can_refresh: bool,
    pub(crate) closed: Arc<AtomicI8>,

    #[cfg(feature = "tokio-runtime")]
    pub(crate) inner_credentials: Arc<tokio::sync::RwLock<Option<CommonCredentials>>>,
    #[cfg(feature = "tokio-runtime")]
    pub(crate) cached_buckets: tokio::sync::RwLock<HashMap<String, BucketCache>>,
}

unsafe impl<P, C, S> Sync for TosClientImpl<P, C, S> {}

impl<P, C, S> ConfigAware for TosClientImpl<P, C, S> {
    fn is_custom_domain(&self) -> bool {
        self.config_holder.load().is_custom_domain
    }
}
#[async_trait]
impl<P, C, S> ObjectAPI for TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    async fn put_object<B>(&self, input: &PutObjectInput<B>) -> Result<PutObjectOutput, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Sync + Unpin + 'static,
    {
        self.do_request(input).await
    }

    async fn put_object_from_buffer(&self, input: &PutObjectFromBufferInput) -> Result<PutObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    #[cfg(feature = "tokio-runtime")]
    async fn put_object_from_file(&self, input: &crate::object::PutObjectFromFileInput) -> Result<PutObjectOutput, TosError> {
        self.do_request_af::<_, _, crate::asynchronous::file::FileReader>(input).await
    }

    async fn get_object(&self, input: &GetObjectInput) -> Result<GetObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }
    #[cfg(feature = "tokio-runtime")]
    async fn get_object_to_file(&self, input: &crate::object::GetObjectToFileInput) -> Result<crate::object::GetObjectToFileOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }
    async fn delete_object(&self, input: &DeleteObjectInput) -> Result<DeleteObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn head_object(&self, input: &HeadObjectInput) -> Result<HeadObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn list_objects_type2(&self, input: &ListObjectsType2Input) -> Result<ListObjectsType2Output, TosError> {
        if input.list_only_once {
            return self.do_request::<_, _, InternalReader<StreamVec>>(input).await;
        }

        let mut input = input.clone();
        if input.max_keys <= 0 {
            input.max_keys = DEFAULT_MAX_KEYS;
        }
        let mut _output: Option<ListObjectsType2Output> = None;
        loop {
            let mut temp_output = self.do_request::<ListObjectsType2Input, ListObjectsType2Output, InternalReader<StreamVec>>(&input).await?;
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

    async fn copy_object(&self, input: &CopyObjectInput) -> Result<CopyObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_multi_objects(&self, input: &DeleteMultiObjectsInput) -> Result<DeleteMultiObjectsOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_object_acl(&self, input: &GetObjectACLInput) -> Result<GetObjectACLOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn list_object_versions(&self, input: &ListObjectVersionsInput) -> Result<ListObjectVersionsOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_object_acl(&self, input: &PutObjectACLInput) -> Result<PutObjectACLOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn set_object_meta(&self, input: &SetObjectMetaInput) -> Result<SetObjectMetaOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }
    async fn append_object<B>(&self, input: &AppendObjectInput<B>) -> Result<AppendObjectOutput, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Sync + Unpin + 'static,
    {
        let mut hinput = GetBucketTypeInput::new(input.bucket());
        hinput.set_request_host(input.request_host());
        if let Some(request_date) = input.request_date() {
            hinput.set_request_date(request_date);
        }
        if let Some(bt) = self.get_bucket_type(&hinput).await?.bucket_type() {
            if bt == &BucketType::BucketTypeHns {
                let (if_match, forbid_overwrite) = self.check_object_status(input.bucket(), input.key(),
                                                                            input.offset(), input.content_length(), &input.inner.generic_input).await?;
                if forbid_overwrite {
                    let mut minput: PutObjectInput<B> = PutObjectInput::default();
                    minput.set_request_host(input.request_host());
                    if let Some(request_date) = input.request_date() {
                        minput.set_request_date(request_date);
                    }
                    minput.inner = self.trans_append_object_input(&input.inner);
                    minput.set_forbid_overwrite(forbid_overwrite);
                    if let Some(adts) = input.async_data_transfer_listener() {
                        minput.set_async_data_transfer_listener(adts.clone());
                    }
                    if let Some(b) = input.content.take() {
                        minput.set_content(b);
                    }
                    let output = self.put_object(&minput).await?;
                    return Ok(AppendObjectOutput {
                        request_info: output.request_info,
                        next_append_offset: input.content_length(),
                        hash_crc64ecma: output.hash_crc64ecma,
                    });
                }

                let mut minput: ModifyObjectInput<B> = ModifyObjectInput::new(input.bucket(), input.key());
                minput.set_request_host(input.request_host());
                if let Some(request_date) = input.request_date() {
                    minput.set_request_date(request_date);
                }
                minput.pre_hash_crc64ecma = input.pre_hash_crc64ecma();
                minput.set_if_match(if_match);
                minput.set_offset(input.offset());
                minput.set_content_length(input.content_length());
                minput.set_notification_custom_parameters(input.notification_custom_parameters());
                minput.set_traffic_limit(input.traffic_limit());
                if let Some(adts) = input.async_data_transfer_listener() {
                    minput.set_async_data_transfer_listener(adts.clone());
                }
                if let Some(b) = input.content.take() {
                    minput.set_content(b);
                }
                let output = self.modify_object(&minput).await?;
                return Ok(AppendObjectOutput {
                    request_info: output.request_info,
                    next_append_offset: output.next_modify_offset,
                    hash_crc64ecma: output.hash_crc64ecma,
                });
            }
        }
        self.do_request(input).await
    }

    async fn append_object_from_buffer(&self, input: &AppendObjectFromBufferInput) -> Result<AppendObjectOutput, TosError> {
        let mut hinput = GetBucketTypeInput::new(input.bucket());
        hinput.set_request_host(input.request_host());
        if let Some(request_date) = input.request_date() {
            hinput.set_request_date(request_date);
        }
        if let Some(bt) = self.get_bucket_type(&hinput).await?.bucket_type() {
            if bt == &BucketType::BucketTypeHns {
                let (if_match, forbid_overwrite) = self.check_object_status(input.bucket(), input.key(),
                                                                            input.offset(), input.content_length(), &input.inner.generic_input).await?;
                if forbid_overwrite {
                    let mut minput = PutObjectFromBufferInput::default();
                    minput.set_request_host(input.request_host());
                    if let Some(request_date) = input.request_date() {
                        minput.set_request_date(request_date);
                    }
                    minput.inner = self.trans_append_object_input(&input.inner);
                    minput.set_forbid_overwrite(forbid_overwrite);
                    if let Some(adts) = input.async_data_transfer_listener() {
                        minput.set_async_data_transfer_listener(adts.clone());
                    }
                    if let Some(b) = input.content() {
                        minput.set_content(b);
                    }
                    let output = self.put_object_from_buffer(&minput).await?;
                    return Ok(AppendObjectOutput {
                        request_info: output.request_info,
                        next_append_offset: input.content_length(),
                        hash_crc64ecma: output.hash_crc64ecma,
                    });
                }

                let mut minput = ModifyObjectFromBufferInput::new(input.bucket(), input.key());
                minput.set_request_host(input.request_host());
                if let Some(request_date) = input.request_date() {
                    minput.set_request_date(request_date);
                }
                minput.pre_hash_crc64ecma = input.pre_hash_crc64ecma();
                minput.set_if_match(if_match);
                minput.set_offset(input.offset());
                minput.set_content_length(input.content_length());
                minput.set_notification_custom_parameters(input.notification_custom_parameters());
                minput.set_traffic_limit(input.traffic_limit());
                if let Some(adts) = input.async_data_transfer_listener() {
                    minput.set_async_data_transfer_listener(adts.clone());
                }
                if let Some(b) = input.content() {
                    minput.set_content(b);
                }
                let output = self.modify_object_from_buffer(&minput).await?;
                return Ok(AppendObjectOutput {
                    request_info: output.request_info,
                    next_append_offset: output.next_modify_offset,
                    hash_crc64ecma: output.hash_crc64ecma,
                });
            }
        }
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn fetch_object(&self, input: &FetchObjectInput) -> Result<FetchObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_fetch_task(&self, input: &PutFetchTaskInput) -> Result<PutFetchTaskOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_fetch_task(&self, input: &GetFetchTaskInput) -> Result<GetFetchTaskOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_object_tagging(&self, input: &PutObjectTaggingInput) -> Result<PutObjectTaggingOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_object_tagging(&self, input: &GetObjectTaggingInput) -> Result<GetObjectTaggingOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_object_tagging(&self, input: &DeleteObjectTaggingInput) -> Result<DeleteObjectTaggingOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn rename_object(&self, input: &RenameObjectInput) -> Result<RenameObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn restore_object(&self, input: &RestoreObjectInput) -> Result<RestoreObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_symlink(&self, input: &PutSymlinkInput) -> Result<PutSymlinkOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_symlink(&self, input: &GetSymlinkInput) -> Result<GetSymlinkOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_file_status(&self, input: &GetFileStatusInput) -> Result<GetFileStatusOutput, TosError> {
        let mut hinput = GetBucketTypeInput::new(input.bucket());
        hinput.set_request_host(input.request_host());
        if let Some(request_date) = input.request_date() {
            hinput.set_request_date(request_date);
        }
        if let Some(bt) = self.get_bucket_type(&hinput).await?.bucket_type() {
            if bt == &BucketType::BucketTypeHns {
                let mut hinput = HeadObjectInput::new(&input.bucket, &input.key);
                hinput.set_request_host(input.request_host());
                if let Some(request_date) = input.request_date() {
                    hinput.set_request_date(request_date);
                }
                let o = self.head_object(&hinput).await?;
                return Ok(GetFileStatusOutput {
                    request_info: o.request_info,
                    key: input.key.clone(),
                    size: o.content_length,
                    last_modified_string: None,
                    last_modified: o.last_modified,
                    crc32: "".to_string(),
                    crc64: o.hash_crc64ecma.to_string(),
                    etag: o.etag,
                    object_type: o.object_type,
                });
            }
        }
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn does_object_exist(&self, input: &DoesObjectExistInput) -> Result<bool, TosError> {
        let mut hinput = HeadObjectInput::new_with_version_id(input.bucket(), input.key(), input.version_id());
        if let Some(request_date) = input.request_date() {
            hinput.set_request_date(request_date);
        }
        hinput.set_request_host(input.request_host());
        match self.head_object(&hinput).await {
            Ok(_) => Ok(true),
            Err(ex) => {
                if let Some(err) = ex.as_server_error() {
                    if err.status_code() == 400 && err.ec() == "0015-00000008" {
                        return Ok(true);
                    }
                    if err.status_code() == 404 && err.ec() == "0017-00000003" {
                        return Ok(false);
                    }
                }
                Err(ex)
            }
        }
    }

    async fn set_object_time(&self, input: &SetObjectTimeInput) -> Result<SetObjectTimeOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }
}

#[cfg(feature = "asynchronous")]
#[async_trait]
impl<P, C, S> BucketAPI for TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    async fn create_bucket(&self, input: &CreateBucketInput) -> Result<CreateBucketOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn head_bucket(&self, input: &HeadBucketInput) -> Result<HeadBucketOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket(&self, input: &DeleteBucketInput) -> Result<DeleteBucketOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn list_buckets(&self, input: &ListBucketsInput) -> Result<ListBucketsOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_cors(&self, input: &PutBucketCORSInput) -> Result<PutBucketCORSOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_cors(&self, input: &GetBucketCORSInput) -> Result<GetBucketCORSOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_cors(&self, input: &DeleteBucketCORSInput) -> Result<DeleteBucketCORSOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_storage_class(&self, input: &PutBucketStorageClassInput) -> Result<PutBucketStorageClassOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_location(&self, input: &GetBucketLocationInput) -> Result<GetBucketLocationOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_lifecycle(&self, input: &PutBucketLifecycleInput) -> Result<PutBucketLifecycleOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_lifecycle(&self, input: &GetBucketLifecycleInput) -> Result<GetBucketLifecycleOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_lifecycle(&self, input: &DeleteBucketLifecycleInput) -> Result<DeleteBucketLifecycleOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_policy(&self, input: &PutBucketPolicyInput) -> Result<PutBucketPolicyOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_policy(&self, input: &GetBucketPolicyInput) -> Result<GetBucketPolicyOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_policy(&self, input: &DeleteBucketPolicyInput) -> Result<DeleteBucketPolicyOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_mirror_back(&self, input: &PutBucketMirrorBackInput) -> Result<PutBucketMirrorBackOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_mirror_back(&self, input: &GetBucketMirrorBackInput) -> Result<GetBucketMirrorBackOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_mirror_back(&self, input: &DeleteBucketMirrorBackInput) -> Result<DeleteBucketMirrorBackOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_acl(&self, input: &PutBucketACLInput) -> Result<PutBucketACLOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_acl(&self, input: &GetBucketACLInput) -> Result<GetBucketACLOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_replication(&self, input: &PutBucketReplicationInput) -> Result<PutBucketReplicationOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_replication(&self, input: &GetBucketReplicationInput) -> Result<GetBucketReplicationOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_replication(&self, input: &DeleteBucketReplicationInput) -> Result<DeleteBucketReplicationOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_versioning(&self, input: &PutBucketVersioningInput) -> Result<PutBucketVersioningOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_versioning(&self, input: &GetBucketVersioningInput) -> Result<GetBucketVersioningOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_website(&self, input: &PutBucketWebsiteInput) -> Result<PutBucketWebsiteOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_website(&self, input: &GetBucketWebsiteInput) -> Result<GetBucketWebsiteOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_website(&self, input: &DeleteBucketWebsiteInput) -> Result<DeleteBucketWebsiteOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_custom_domain(&self, input: &PutBucketCustomDomainInput) -> Result<PutBucketCustomDomainOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }
    async fn list_bucket_custom_domain(&self, input: &ListBucketCustomDomainInput) -> Result<ListBucketCustomDomainOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_custom_domain(&self, input: &DeleteBucketCustomDomainInput) -> Result<DeleteBucketCustomDomainOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_real_time_log(&self, input: &PutBucketRealTimeLogInput) -> Result<PutBucketRealTimeLogOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_real_time_log(&self, input: &GetBucketRealTimeLogInput) -> Result<GetBucketRealTimeLogOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_real_time_log(&self, input: &DeleteBucketRealTimeLogInput) -> Result<DeleteBucketRealTimeLogOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_rename(&self, input: &PutBucketRenameInput) -> Result<PutBucketRenameOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_rename(&self, input: &GetBucketRenameInput) -> Result<GetBucketRenameOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_rename(&self, input: &DeleteBucketRenameInput) -> Result<DeleteBucketRenameOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_encryption(&self, input: &PutBucketEncryptionInput) -> Result<PutBucketEncryptionOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_encryption(&self, input: &GetBucketEncryptionInput) -> Result<GetBucketEncryptionOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_encryption(&self, input: &DeleteBucketEncryptionInput) -> Result<DeleteBucketEncryptionOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_tagging(&self, input: &PutBucketTaggingInput) -> Result<PutBucketTaggingOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_tagging(&self, input: &GetBucketTaggingInput) -> Result<GetBucketTaggingOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_tagging(&self, input: &DeleteBucketTaggingInput) -> Result<DeleteBucketTaggingOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_notification_type2(&self, input: &PutBucketNotificationType2Input) -> Result<PutBucketNotificationType2Output, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_notification_type2(&self, input: &GetBucketNotificationType2Input) -> Result<GetBucketNotificationType2Output, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_inventory(&self, input: &PutBucketInventoryInput) -> Result<PutBucketInventoryOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_inventory(&self, input: &GetBucketInventoryInput) -> Result<GetBucketInventoryOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn list_bucket_inventory(&self, input: &ListBucketInventoryInput) -> Result<ListBucketInventoryOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_bucket_inventory(&self, input: &DeleteBucketInventoryInput) -> Result<DeleteBucketInventoryOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    #[cfg(not(feature = "tokio-runtime"))]
    async fn get_bucket_type(&self, input: &GetBucketTypeInput) -> Result<GetBucketTypeOutput, TosError> {
        let mut hinput = HeadBucketInput::new(input.bucket());
        hinput.set_request_host(input.request_host());
        if let Some(request_date) = input.request_date() {
            hinput.set_request_date(request_date);
        }
        let output = self.head_bucket(&hinput).await?;
        Ok(GetBucketTypeOutput {
            request_info: output.request_info,
            region: output.region,
            storage_class: output.storage_class,
            az_redundancy: output.az_redundancy,
            project_name: output.project_name,
            bucket_type: output.bucket_type,
            expire_at: Default::default(),
        })
    }

    async fn does_bucket_exist(&self, input: &DoesBucketExistInput) -> Result<bool, TosError> {
        let mut hinput = HeadBucketInput::new(input.bucket());
        if let Some(request_date) = input.request_date() {
            hinput.set_request_date(request_date);
        }
        hinput.set_request_host(input.request_host());
        match self.head_bucket(&hinput).await {
            Ok(_) => Ok(true),
            Err(ex) => {
                if let Some(err) = ex.as_server_error() {
                    if err.status_code() == 404 && err.ec() == "0006-00000001" {
                        return Ok(false);
                    }
                }
                Err(ex)
            }
        }
    }

    async fn get_bucket_info(&self, input: &GetBucketInfoInput) -> Result<GetBucketInfoOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_access_monitor(&self, input: &PutBucketAccessMonitorInput) -> Result<PutBucketAccessMonitorOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_access_monitor(&self, input: &GetBucketAccessMonitorInput) -> Result<GetBucketAccessMonitorOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn put_bucket_trash(&self, input: &PutBucketTrashInput) -> Result<PutBucketTrashOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_bucket_trash(&self, input: &GetBucketTrashInput) -> Result<GetBucketTrashOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    #[cfg(feature = "tokio-runtime")]
    async fn get_bucket_type(&self, input: &GetBucketTypeInput) -> Result<GetBucketTypeOutput, TosError> {
        crate::internal::check_bucket(input.bucket())?;

        {
            let cached_buckets = self.cached_buckets.read().await;
            if let Some((output, ddl)) = cached_buckets.get(input.bucket()) {
                if ddl >= &chrono::Utc::now() {
                    let mut output = output.clone();
                    output.expire_at = *ddl;
                    return Ok(output);
                }
            }
        }

        let mut cached_buckets = self.cached_buckets.write().await;
        if let Some((output, ddl)) = cached_buckets.get(input.bucket()) {
            if ddl >= &chrono::Utc::now() {
                let mut output = output.clone();
                output.expire_at = *ddl;
                return Ok(output);
            }
        }
        let mut hinput = HeadBucketInput::new(input.bucket());
        hinput.set_request_host(input.request_host());
        if let Some(request_date) = input.request_date() {
            hinput.set_request_date(request_date);
        }
        let output = self.head_bucket(&hinput).await?;
        let mut rng = rand::thread_rng();
        let ddl = std::ops::Add::add(chrono::Utc::now(), chrono::Duration::minutes(rand::Rng::gen_range(&mut rng, 10..15) as i64));
        let output = GetBucketTypeOutput {
            request_info: output.request_info,
            region: output.region,
            storage_class: output.storage_class,
            az_redundancy: output.az_redundancy,
            project_name: output.project_name,
            bucket_type: output.bucket_type,
            expire_at: ddl,
        };
        cached_buckets.insert(input.bucket.clone(), (output.clone(), ddl));
        Ok(output)
    }
}

#[async_trait]
impl<P, C, S> MultipartAPI for TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    async fn create_multipart_upload(&self, input: &CreateMultipartUploadInput) -> Result<CreateMultipartUploadOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn upload_part<B>(&self, input: &UploadPartInput<B>) -> Result<UploadPartOutput, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Sync + Unpin + 'static,
    {
        self.do_request(input).await
    }

    async fn upload_part_from_buffer(&self, input: &UploadPartFromBufferInput) -> Result<UploadPartOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    #[cfg(feature = "tokio-runtime")]
    async fn upload_part_from_file(&self, input: &crate::multipart::UploadPartFromFileInput) -> Result<UploadPartOutput, TosError> {
        self.do_request_af::<_, _, crate::asynchronous::file::FileReader>(input).await
    }

    async fn complete_multipart_upload(&self, input: &CompleteMultipartUploadInput) -> Result<CompleteMultipartUploadOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn abort_multipart_upload(&self, input: &AbortMultipartUploadInput) -> Result<AbortMultipartUploadOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn upload_part_copy(&self, input: &UploadPartCopyInput) -> Result<UploadPartCopyOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn list_multipart_uploads(&self, input: &ListMultipartUploadsInput) -> Result<ListMultipartUploadsOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn list_parts(&self, input: &ListPartsInput) -> Result<ListPartsOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }
}

impl<P, C, S> SignerAPI for TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    fn pre_signed_url(&self, input: &PreSignedURLInput) -> Result<PreSignedURLOutput, TosError> {
        let (ak, sk, security_token) = self.sync_load_credentials()?;
        pre_signed_url(&self.config_holder, &ak, &sk, &security_token, input)
    }

    fn pre_signed_post_signature(&self, input: &PreSignedPostSignatureInput) -> Result<PreSignedPostSignatureOutput, TosError> {
        let (ak, sk, security_token) = self.sync_load_credentials()?;
        pre_signed_post_signature(&self.config_holder, &ak, &sk, &security_token, input)
    }

    fn pre_signed_policy_url(&self, input: &PreSignedPolicyURLInput) -> Result<PreSignedPolicyURLOutput, TosError> {
        let (ak, sk, security_token) = self.sync_load_credentials()?;
        pre_signed_policy_url(&self.config_holder, &ak, &sk, &security_token, input)
    }
}

#[async_trait]
impl<P, C, S> ControlAPI for TosClientImpl<P, C, S>
where
    C: 'static + Credentials + Send + Sync,
    P: 'static + CredentialsProvider<C> + Send + Sync,
    S: 'static + AsyncRuntime + Send + Sync,
{
    async fn put_qos_policy(&self, input: &PutQosPolicyInput) -> Result<PutQosPolicyOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn get_qos_policy(&self, input: &GetQosPolicyInput) -> Result<GetQosPolicyOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn delete_qos_policy(&self, input: &DeleteQosPolicyInput) -> Result<DeleteQosPolicyOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }
}

#[async_trait]
impl<P, C, S> TosClient for TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
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
            client_crt: c.ca_crt.clone(),
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

impl<P, C, S> TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    async fn load_credentials(&self) -> Result<(String, String, String), TosError> {
        #[cfg(feature = "tokio-runtime")]
        {
            let inner_credentials = self.inner_credentials.read().await;
            if let Some(c) = inner_credentials.as_ref() {
                return Ok((c.ak().to_string(), c.sk().to_string(), c.security_token().to_string()));
            }
        }
        self.sync_load_credentials()
    }

    fn sync_load_credentials(&self) -> Result<(String, String, String), TosError> {
        let cred = self.credentials_provider.load();
        match cred.credentials(CREDENTIALS_EXPIRES) {
            Err(ex) => Err(TosError::client_error_with_cause("load credentials error", GenericError::DefaultError(ex.to_string()))),
            Ok(c) => Ok((c.ak().to_string(), c.sk().to_string(), c.security_token().to_string())),
        }
    }

    #[cfg(feature = "tokio-runtime")]
    fn async_refresh_credentials(&self) {
        let closed = self.closed.clone();
        let async_runtime = self.async_runtime.clone();
        let credential_provider = self.credentials_provider.load().clone();
        let inner_credentials = self.inner_credentials.clone();
        let _ = self.async_runtime.spawn(async move {
            loop {
                if closed.load(Ordering::Acquire) == 1 {
                    return;
                }
                async_runtime.sleep(Duration::from_secs(crate::constant::CREDENTIALS_REFRESH_INTERVAL)).await;
                match credential_provider.credentials(CREDENTIALS_EXPIRES) {
                    Err(ex) => warn!("async load credentials error, {}", ex),
                    Ok(c) => {
                        let mut inner_credentials = inner_credentials.write().await;
                        *inner_credentials = Some(CommonCredentials {
                            ak: c.ak().to_string(),
                            sk: c.sk().to_string(),
                            security_token: c.security_token().to_string(),
                        });
                    }
                }
            }
        });
    }


    async fn modify_object<B>(&self, input: &ModifyObjectInput<B>) -> Result<ModifyObjectOutput, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Sync + Unpin + 'static,
    {
        self.do_request(input).await
    }

    async fn modify_object_from_buffer(&self, input: &ModifyObjectFromBufferInput) -> Result<ModifyObjectOutput, TosError> {
        self.do_request::<_, _, InternalReader<StreamVec>>(input).await
    }

    async fn do_request<T, K, B>(&self, input: &T) -> Result<K, TosError>
    where
        T: InputTranslator<B>,
        K: OutputParser + RequestInfoTrait + Send,
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Unpin + 'static,
    {
        self.do_request_common::<T, MockAsyncInputTranslator, K, B>(Some(input), None).await
    }

    pub(crate) async fn do_request_common<T, F, K, B>(&self, input: Option<&T>, input2: Option<&F>) -> Result<K, TosError>
    where
        T: InputTranslator<B>,
        F: AsyncInputTranslator<B>,
        K: OutputParser + RequestInfoTrait + Send,
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Unpin + 'static,
    {
        let config_holder = self.config_holder.load();
        let operation;
        if input.is_some() {
            operation = check_bucket_and_key(input.unwrap(), config_holder.is_custom_domain)?;
        } else {
            operation = check_bucket_and_key(input2.unwrap(), config_holder.is_custom_domain)?;
        }
        let mut retry_count = 0;
        let max_retry_count = config_holder.max_retry_count;
        loop {
            let start = Instant::now();
            let mut ac = AdditionalContext::new();
            let result = self.do_request_once::<T, F, K, B>(input, input2, retry_count, config_holder.clone(), &mut ac).await;
            let elapsed_ms = start.elapsed().as_millis();
            let exceed = exceed_high_latency_log_threshold(config_holder.high_latency_log_threshold, elapsed_ms, ac.request_size, operation);
            match result {
                Ok(k) => {
                    if exceed {
                        warn!("high latency request {} succeed, http status: {}, request id: {}, cost: {} ms", operation, k.status_code(), k.request_id(), elapsed_ms)
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
                    self.sleep_for_retry(retry_count, retry_after).await;
                    retry_count += 1;
                }
            }
        }
    }


    async fn sleep_for_retry(&self, retry_count: isize, retry_after: isize) {
        let mut delay = BASE_DELAY_MS * 2u64.pow(retry_count as u32);
        if delay > MAX_DELAY_MS {
            delay = MAX_DELAY_MS;
        }
        let retry_after = retry_after as u64 * 1000;
        if retry_after > delay {
            delay = retry_after;
        }

        self.async_runtime.sleep(Duration::from_millis(delay)).await;
    }

    async fn do_request_once<'a, 'b, T, F, K, B>(&self, input: Option<&'b T>, input2: Option<&'b F>, retry_count: isize,
                                                 config_holder: Arc<ConfigHolder>, ac: &mut AdditionalContext<'a>) -> Result<K, TosError>
    where
        T: InputTranslator<B>,
        F: AsyncInputTranslator<B>,
        K: OutputParser + Send,
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Unpin + 'static,
        'b: 'a,
    {
        let mut request;
        if input.is_some() {
            let input = input.unwrap();
            request = input.trans(config_holder)?;
            ac.request_host = input.request_host();
            ac.request_date = input.request_date();
            ac.request_header = input.request_header();
            ac.request_query = input.request_query();
            ac.is_control_operation = input.is_control_operation();
        } else {
            let input2 = input2.unwrap();
            request = input2.trans(config_holder).await?;
            ac.request_host = input2.request_host();
            ac.request_date = input2.request_date();
            ac.request_header = input2.request_header();
            ac.request_query = input2.request_query();
            ac.is_control_operation = input2.is_control_operation();
        }

        let body = request.body.take();
        request.retry_count = retry_count;
        let response = self.do_request_by_client(&mut request, body, ac).await?;
        if request.operation == GET_OBJECT_TO_FILE_OPERATION {
            if let Some(cl) = response.content_length() {
                ac.request_size = cl as i64;
            }
            let result = K::check_and_parse(request, response).await;
            return result;
        }
        K::check_and_parse(request, response).await
    }

    async fn do_request_by_client<'a, 'c, B>(&self, request: &mut HttpRequest<'c, B>, body: Option<B>, ac: &mut AdditionalContext<'a>) -> Result<HttpResponse, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Unpin + 'static,
        'a: 'c,
    {
        let config_holder = self.config_holder.load();
        if ac.is_control_operation && (config_holder.schema_control == "" || config_holder.domain_control == "") {
            return Err(TosError::client_error("request control operation but control endpoint is empty"));
        }

        let (ak, sk, security_token) = self.load_credentials().await?;
        auto_recognize_content_type(request, config_holder.auto_recognize_content_type);
        sign_header(request, &ak, &sk, &security_token, config_holder.as_ref(), ac)?;
        request.enable_crc = config_holder.enable_crc;

        let request_url = get_request_url(request, config_holder.as_ref(), ac.is_control_operation);
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
        let crc64 = Arc::new(AtomicU64::new(0));
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
                        if let Some(ref adts) = rc.async_data_transfer_listener {
                            reader.set_async_data_transfer_listener(adts.clone());
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
                    if let Some(ref adts) = rc.async_data_transfer_listener {
                        reader.set_async_data_transfer_listener(adts.clone());
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
                let result = self.client.execute(req).await;
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

    fn add_body<B>(&self, rb: RequestBuilder, body: B, _: i64) -> RequestBuilder
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + 'static,
    {
        rb.body(Body::wrap_stream(body))
    }

    async fn check_object_status(&self, bucket: &str, key: &str, offset: i64, content_length: i64, ginput: &GenericInput) -> Result<(String, bool), TosError> {
        let mut if_match = String::new();
        let mut forbid_overwrite = false;
        if offset <= 0 && content_length > 0 {
            let mut hinput = HeadObjectInput::new(bucket, key);
            hinput.set_request_host(ginput.request_host.as_str());
            if let Some(request_date) = ginput.request_date {
                hinput.set_request_date(request_date);
            }
            match self.head_object(&hinput).await {
                Ok(output) => {
                    if output.content_length > 0 {
                        return Err(TosError::client_error(format!("invalid offset,  expected {}, actual {}", output.content_length, offset)));
                    }
                    if_match = output.etag().to_string();
                }
                Err(ex) => {
                    if let Some(err) = ex.as_server_error() {
                        if err.status_code() == 404 && err.ec() == "0017-00000003" {
                            forbid_overwrite = true;
                        }
                    }

                    if !forbid_overwrite {
                        return Err(ex);
                    }
                }
            }
        }
        Ok((if_match, forbid_overwrite))
    }

    fn trans_append_object_input(&self, input: &AppendObjectBasicInput) -> PutObjectBasicInput {
        let mut minput = PutObjectBasicInput::default();
        minput.bucket = input.bucket.clone();
        minput.key = input.key.clone();
        minput.set_cache_control(input.cache_control());
        minput.set_content_disposition(input.content_disposition());
        minput.set_content_encoding(input.content_encoding());
        minput.set_content_language(input.content_language());
        minput.set_content_type(input.content_type());
        if let Some(expires) = input.expires() {
            minput.set_expires(expires);
        }
        if let Some(acl) = input.acl() {
            minput.set_acl(acl);
        }
        minput.set_grant_full_control(input.grant_full_control());
        minput.set_grant_read(input.grant_read());
        minput.set_grant_read_acp(input.grant_read_acp());
        minput.set_grant_write_acp(input.grant_write_acp());
        minput.set_website_redirect_location(input.website_redirect_location());
        minput.object_expires = input.object_expires;
        minput.meta = input.meta.clone();
        if let Some(sc) = input.storage_class() {
            minput.set_storage_class(sc);
        }
        minput.set_content_length(input.content_length());
        minput.notification_custom_parameters = input.notification_custom_parameters.clone();
        minput.traffic_limit = input.traffic_limit;
        minput
    }
}

impl<P, C, S> TosClientImpl<P, C, S> {
    pub fn close(&self) {
        let _ = self.closed.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Relaxed);
    }
}

impl<P, C, S> Drop for TosClientImpl<P, C, S> {
    fn drop(&mut self) {
        self.close();
    }
}
