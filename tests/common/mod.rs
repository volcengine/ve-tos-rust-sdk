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
#![allow(
    dead_code,
)]

use std::env;
use std::io::Read;
use std::sync::{Arc, Mutex};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use md5::{Digest, Md5};
use once_cell::sync::Lazy;
use rand::Rng;

use ve_tos_rust_sdk::bucket::{BucketAPI, CreateBucketInput, DeleteBucketInput, HeadBucketInput};
use ve_tos_rust_sdk::multipart::{AbortMultipartUploadInput, ListMultipartUploadsInput, MultipartAPI};
use ve_tos_rust_sdk::object::{DeleteObjectInput, ListObjectVersionsInput, ObjectAPI};
use ve_tos_rust_sdk::tos;
use ve_tos_rust_sdk::tos::{DefaultTosClient, TosClient};

#[cfg(feature = "asynchronous")]
pub mod asynchronous;

pub fn create_context() -> Context {
    let mut ctx = Context::default();
    let mut non_exists_bucket;
    loop {
        non_exists_bucket = gen_random_string(30);
        if let Err(_) = ctx.client.head_bucket(&HeadBucketInput::new(non_exists_bucket.clone())) {
            ctx.non_exists_bucket = non_exists_bucket;
            break;
        }
    }

    let mut fixed_bucket;
    loop {
        fixed_bucket = gen_random_string(10);
        match ctx.client.create_bucket(&CreateBucketInput::new(fixed_bucket.clone())) {
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
                    panic!("unexpected status code");
                }
            }
        }
    }
    ctx
}

pub static CONTEXT: Lazy<Context> = Lazy::new(|| {
    create_context()
});

pub struct Context {
    client: Arc<DefaultTosClient>,
    https_client: Arc<DefaultTosClient>,
    buckets: Mutex<Vec<String>>,
    fixed_bucket: String,
    non_exists_bucket: String,
}

impl Default for Context {
    fn default() -> Self {
        let ak = env::var("TOS_ACCESS_KEY").unwrap_or("".to_string());
        let sk = env::var("TOS_SECRET_KEY").unwrap_or("".to_string());
        let ep = env::var("TOS_ENDPOINT").unwrap_or("".to_string());
        let https_ep = env::var("TOS_HTTPS_ENDPOINT").unwrap_or("".to_string());
        // let ep = "http://tos-cn-chongqing-sdv.volces.com".to_string();
        // let https_ep = "https://tos-cn-chongqing-sdv.volces.com".to_string();
        let client = tos::builder()
            .connection_timeout(3000)
            .request_timeout(60000)
            .max_retry_count(0)
            .ak(ak.clone())
            .sk(sk.clone())
            .region("test-region")
            .endpoint(ep.clone())
            .build().unwrap();
        let https_client = tos::builder()
            .connection_timeout(3000)
            .request_timeout(60000)
            .max_retry_count(0)
            .ak(ak.clone())
            .sk(sk.clone())
            .region("test-region")
            .endpoint(https_ep.clone())
            .build().unwrap();
        Self {
            client: Arc::new(client),
            https_client: Arc::new(https_client),
            buckets: Mutex::new(vec![]),
            fixed_bucket: "".to_string(),
            non_exists_bucket: "".to_string(),
        }
    }
}

impl Context {
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
    pub fn add_bucket(&self, bucket: impl Into<String>) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.push(bucket.into());
    }
    fn clean_bucket(&self, bucket: &str) {
        let mut can_delete_bucket = true;
        let mut input = ListObjectVersionsInput::new(bucket);
        input.set_max_keys(1000);
        'outer: loop {
            match self.client.list_object_versions(&input) {
                Ok(o) => {
                    for version in o.versions() {
                        if let Err(_) = self.client.delete_object(&DeleteObjectInput::new_with_version_id(bucket, version.key(), version.version_id())) {
                            can_delete_bucket = false;
                            break 'outer;
                        }
                    }

                    for delete_marker in o.delete_markers() {
                        if let Err(_) = self.client.delete_object(&DeleteObjectInput::new_with_version_id(bucket, delete_marker.key(), delete_marker.version_id())) {
                            can_delete_bucket = false;
                            break 'outer;
                        }
                    }

                    if !o.is_truncated() {
                        break;
                    }

                    input.set_key_marker(o.next_key_marker());
                    input.set_version_id_marker(o.next_version_id_marker());
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
            match self.client.list_multipart_uploads(&input) {
                Ok(o) => {
                    for upload in o.uploads() {
                        if let Err(_) = self.client.abort_multipart_upload(&AbortMultipartUploadInput::new(bucket, upload.key(), upload.upload_id())) {
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
            let _ = self.client.delete_bucket(&DeleteBucketInput::new(bucket));
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let buckets = self.buckets.lock().unwrap();
        for bucket in buckets.iter() {
            self.clean_bucket(bucket);
        }

        if self.fixed_bucket != "" {
            self.clean_bucket(self.fixed_bucket.as_str());
        }
    }
}

pub fn gen_random_string(len: usize) -> String {
    let mut result = String::with_capacity(len);
    let characters = "0123456789abcdefghijklmnopqrstuvwxyz".as_bytes();
    let mut ra = rand::thread_rng();
    for _ in 0..len {
        let a = ra.gen_range(0..characters.len());
        result.push(characters[a] as char);
    }

    result
}

pub fn base64_md5(data: impl AsRef<[u8]>) -> String {
    BASE64_STANDARD.encode(Md5::digest(data))
}

pub fn base64(data: impl AsRef<[u8]>) -> String {
    BASE64_STANDARD.encode(data)
}

pub fn hex_md5(data: impl AsRef<[u8]>) -> String {
    hex::encode(Md5::digest(data))
}

pub fn read_to_string(r: &mut dyn Read) -> String {
    let mut buf = String::new();
    r.read_to_string(&mut buf).unwrap();
    buf
}

pub fn read_to_buf(r: &mut dyn Read) -> Vec<u8> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).unwrap();
    buf
}

