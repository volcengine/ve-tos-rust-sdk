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
use std::env;
use std::future::Future;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures_core::future::BoxFuture;
use futures_core::Stream;
use futures_util::StreamExt;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use ve_tos_rust_sdk::asynchronous::bucket::BucketAPI;
use ve_tos_rust_sdk::asynchronous::multipart::MultipartAPI;
use ve_tos_rust_sdk::asynchronous::object::ObjectAPI;
use ve_tos_rust_sdk::asynchronous::tos;
use ve_tos_rust_sdk::asynchronous::tos::{AsyncRuntime, TosClient, TosClientImpl};
use ve_tos_rust_sdk::bucket::{CreateBucketInput, DeleteBucketInput, HeadBucketInput};
use ve_tos_rust_sdk::credential::{CommonCredentials, CommonCredentialsProvider};
use ve_tos_rust_sdk::multipart::{AbortMultipartUploadInput, ListMultipartUploadsInput};
use ve_tos_rust_sdk::object::{DeleteObjectInput, ListObjectsType2Input};

use crate::common::gen_random_string;

#[derive(Debug, Default)]
pub struct TokioRuntime {}

#[async_trait]
impl AsyncRuntime for TokioRuntime {
    type JoinError = tokio::task::JoinError;
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    fn spawn<'a, F>(&self, future: F) -> BoxFuture<'a, Result<F::Output, Self::JoinError>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        Box::pin(Handle::current().spawn(future))
    }

    fn block_on<F: Future>(&self, future: F) -> F::Output {
        Handle::current().block_on(future)
    }
}

type DefaultTosClient = TosClientImpl<CommonCredentialsProvider<CommonCredentials>, CommonCredentials, TokioRuntime>;

pub struct AsyncContext {
    client: Arc<DefaultTosClient>,
    https_client: Arc<DefaultTosClient>,
    buckets: Mutex<Vec<String>>,
    fixed_bucket: String,
    non_exists_bucket: String,
    released: atomic::AtomicI8,
}

impl Default for AsyncContext {
    fn default() -> Self {
        let ak = env::var("TOS_ACCESS_KEY").unwrap_or("".to_string());
        let sk = env::var("TOS_SECRET_KEY").unwrap_or("".to_string());
        let ep = env::var("TOS_ENDPOINT").unwrap_or("".to_string());
        let cep = env::var("TOS_CONTROL_ENDPOINT").unwrap_or("".to_string());

        let https_ep = env::var("TOS_HTTPS_ENDPOINT").unwrap_or("".to_string());
        let mut client = tos::builder::<TokioRuntime>()
            .connection_timeout(3000)
            .request_timeout(60000)
            .max_retry_count(0)
            .control_endpoint(cep.clone())
            .user_agent_product_name("test")
            .user_agent_soft_name("soft")
            .user_agent_soft_version("v1")
            .user_agent_customized_key_values(HashMap::from([("p1".to_string(), "p2".to_string()), ("p3".to_string(), "p4".to_string())]))
            .ak(ak.clone())
            .sk(sk.clone())
            .region("test-region")
            .endpoint(ep.clone());

        #[cfg(feature = "tokio-runtime")]
        {
            client = client.dns_cache_time(5);
        }
        let client = client.build().unwrap();

        let mut https_client = tos::builder::<TokioRuntime>()
            .connection_timeout(3000)
            .request_timeout(60000)
            .max_retry_count(0)
            .control_endpoint(cep)
            .ak(ak.clone())
            .sk(sk.clone())
            .region("test-region")
            .endpoint(https_ep.clone());

        #[cfg(feature = "tokio-runtime")]
        {
            https_client = https_client.dns_cache_time(5);
        }

        let https_client = https_client.build().unwrap();
        Self {
            client: Arc::new(client),
            https_client: Arc::new(https_client),
            buckets: Mutex::new(vec![]),
            fixed_bucket: "".to_string(),
            non_exists_bucket: "".to_string(),
            released: atomic::AtomicI8::new(0),
        }
    }
}

impl AsyncContext {
    pub fn client(&self) -> Arc<impl TosClient> {
        self.client.clone()
    }
    pub fn https_client(&self) -> Arc<impl TosClient> {
        self.https_client.clone()
    }
    pub fn fixed_bucket(&self) -> &str {
        &self.fixed_bucket
    }

    pub fn non_exists_bucket(&self) -> &str {
        &self.non_exists_bucket
    }

    pub async fn add_bucket(&self, bucket: impl Into<String>) {
        let mut buckets = self.buckets.lock().await;
        buckets.push(bucket.into());
    }

    pub async fn tear_down(&self) {
        if let Ok(_) = self.released.compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed) {
            let buckets = self.buckets.lock().await;
            for bucket in buckets.iter() {
                self.clean_bucket(bucket).await;
            }

            if self.fixed_bucket.as_str() != "" {
                self.clean_bucket(self.fixed_bucket.as_str()).await;
            }
        }
    }

    async fn clean_bucket(&self, bucket: &str) {
        let mut can_delete_bucket = true;
        let mut input = ListObjectsType2Input::new(bucket);
        input.set_max_keys(1000);
        'outer: loop {
            match self.client.list_objects_type2(&input).await {
                Ok(o) => {
                    for content in o.contents() {
                        if let Err(_) = self.client.delete_object(&DeleteObjectInput::new(bucket, content.key())).await {
                            can_delete_bucket = false;
                            break 'outer;
                        }
                    }

                    if !o.is_truncated() {
                        break;
                    }

                    input.set_continuation_token(o.next_continuation_token());
                }
                Err(_) => {
                    can_delete_bucket = false;
                    break;
                }
            }
        }

        let mut input = ListMultipartUploadsInput::new(bucket);
        input.set_max_uploads(1000);
        'outer: loop {
            match self.client.list_multipart_uploads(&input).await {
                Ok(o) => {
                    for upload in o.uploads() {
                        if let Err(_) = self.client.abort_multipart_upload(&AbortMultipartUploadInput::new(bucket, upload.key(), upload.upload_id())).await {
                            can_delete_bucket = false;
                            break 'outer;
                        }
                    }

                    if !o.is_truncated() {
                        break;
                    }

                    input.set_upload_id_marker(o.next_upload_id_marker());
                    input.set_key_marker(o.next_key_marker());
                }
                Err(_) => {
                    can_delete_bucket = false;
                    break;
                }
            }
        }

        if can_delete_bucket {
            let _ = self.client.delete_bucket(&DeleteBucketInput::new(bucket)).await;
        }
    }
}

pub async fn create_async_context() -> AsyncContext {
    let mut ctx = AsyncContext::default();
    let mut non_exists_bucket;
    loop {
        non_exists_bucket = gen_random_string(30);
        if let Err(_) = ctx.client.head_bucket(&HeadBucketInput::new(non_exists_bucket.clone())).await {
            ctx.non_exists_bucket = non_exists_bucket;
            break;
        }
    }

    let mut fixed_bucket;
    loop {
        fixed_bucket = gen_random_string(10);
        match ctx.client.create_bucket(&CreateBucketInput::new(fixed_bucket.clone())).await {
            Ok(_) => {
                ctx.fixed_bucket = fixed_bucket;
                break;
            }
            Err(e) => {
                if !e.is_server_error() {
                    panic!("{}", e.to_string());
                }

                let ex = e.as_server_error().unwrap();
                if ex.status_code() != 409 {
                    panic!("unexpected status code, {}", ex.code());
                }
            }
        }
    }
    ctx
}


pub async fn read_to_string<S: Stream<Item=Result<Bytes, std::io::Error>> + Unpin + ?Sized>(r: &mut S) -> String {
    String::from_utf8(read_to_buf(r).await).unwrap()
}

pub async fn read_to_buf<S: Stream<Item=Result<Bytes, std::io::Error>> + Unpin + ?Sized>(r: &mut S) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        match r.next().await {
            None => return buf,
            Some(result) => {
                let x = result.unwrap();
                buf.extend_from_slice(x.slice(0..x.len()).as_ref());
            }
        }
    }
}