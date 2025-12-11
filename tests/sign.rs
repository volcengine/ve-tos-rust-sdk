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
use crate::common::asynchronous::{create_async_context, AsyncContext, TokioRuntime};
use crate::common::gen_random_string;
use reqwest::{redirect, Body, Client};
use scopeguard::defer;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::runtime;
use tracing::log::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use ve_tos_rust_sdk::asynchronous::auth::SignerAPI;
use ve_tos_rust_sdk::asynchronous::object::ObjectAPI;
use ve_tos_rust_sdk::asynchronous::tos;
use ve_tos_rust_sdk::auth::{ContentLengthRange, PolicySignatureCondition, PostSignatureCondition, PostSignatureMultiValuesCondition, PreSignedPolicyURLInput};
use ve_tos_rust_sdk::auth::{PreSignedPostSignatureInput, PreSignedURLInput};
use ve_tos_rust_sdk::enumeration::HttpMethodType::{HttpMethodDelete, HttpMethodGet, HttpMethodHead, HttpMethodPut};
use ve_tos_rust_sdk::object::PutObjectFromBufferInput;

mod common;


#[test]
fn test_main() {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let file_appender = tracing_appender::rolling::daily("temp/logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_writer(non_blocking)
        .with_env_filter(EnvFilter::new("info"))
        .with_ansi(false).init();

    let context = rt.block_on(async {
        return create_async_context().await;
    });

    defer! {
        rt.block_on(async {
            context.tear_down().await;
        });
    }

    rt.block_on(async {
        info!("Application started");
        debug!("Loading configuration...");
        warn!("This is a warning message");
        error!("Critical error occurred!");
        let client = tos::builder::<TokioRuntime>()
            .connection_timeout(3000)
            .request_timeout(60000)
            .max_retry_count(0)
            .region("test-region")
            .endpoint("tos-s3-cn-beijing.volces.com")
            .build();
        assert!(client.is_err());
        test_pre_sign(&context).await;
        test_post_sign(&context).await;
        test_policy_sign(&context).await;
    })
}

async fn test_policy_sign(context: &AsyncContext) {
    let client = context.client();
    let prefix = gen_random_string(10);
    let mut keys = Vec::with_capacity(10);
    let cnt = 1;
    for i in 0..cnt {
        let key = format!("{}/{}", prefix, gen_random_string(10));
        keys.push(key.clone());
        let mut input = PutObjectFromBufferInput::new(context.fixed_bucket(), key);
        input.set_content("hello world");
        let o = client.put_object_from_buffer(&input).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    let mut input = PreSignedPolicyURLInput::new(context.fixed_bucket());
    input.set_conditions(vec![PolicySignatureCondition::new_with_operator("key", prefix, "starts-with")]);
    let o = client.pre_signed_policy_url(&input).await.unwrap();
    println!("{}", o.signed_query());
    println!("{}", o.get_signed_url_for_list(None::<HashMap<&str, &str>>));
    println!("{}", o.get_signed_url_for_get_or_head("key1", None::<HashMap<&str, &str>>));


    let http_client = Client::builder()
        .tcp_nodelay(true)
        .tcp_keepalive(None)
        .redirect(redirect::Policy::none())
        .no_gzip()
        .no_deflate()
        .no_brotli()
        .danger_accept_invalid_certs(true).build().unwrap();
    for key in keys.iter() {
        let signed_url = o.get_signed_url_for_get_or_head(key, None::<HashMap<&str, &str>>);
        let req = http_client.head(signed_url.clone()).build().unwrap();
        let resp = http_client.execute(req).await.unwrap();
        assert!(resp.status().is_success());

        let req = http_client.get(signed_url).build().unwrap();
        let resp = http_client.execute(req).await.unwrap();
        assert!(resp.status().is_success());
        let cl = resp.headers().get("content-length").unwrap();
        assert_eq!(cl.to_str().unwrap().parse::<usize>().unwrap(), "hello world".len());
        let data = String::from_utf8(Vec::from(resp.bytes().await.unwrap())).unwrap();
        assert_eq!(data, "hello world".to_string());
    }
}

#[derive(Deserialize)]
pub(crate) struct InnerError {
    #[serde(rename = "StringToSign")]
    pub(crate) string_to_sign: String,
    #[serde(rename = "canonical_request")]
    pub(crate) canonical_request: String,
}

async fn test_post_sign(context: &AsyncContext) {
    let client = context.client();
    let key = gen_random_string(10);
    let mut input = PreSignedPostSignatureInput::new(context.fixed_bucket());
    input.set_key(key.as_str());
    input.set_expires(3600);
    input.set_content_length_range(ContentLengthRange::new(100, 200));
    let mut cond = PostSignatureCondition::new("Content-Type", "text/plain");
    cond.set_operator("starts-with");
    input.set_conditions(Vec::from([cond]));

    let cond = PostSignatureMultiValuesCondition::new("acl",
                                                      vec!["public-read".to_string(), "public-read-write".to_string()], "in");
    input.set_multi_values_conditions(Vec::from([cond]));
    let output = client.pre_signed_post_signature(&input).await.unwrap();
    println!("{}", context.fixed_bucket());
    println!("{}", key);
    println!("{}", output.origin_policy());
    println!("{}", output.policy());
    println!("{}", output.algorithm());
    println!("{}", output.date());
    println!("{}", output.credential());
    println!("{}", output.signature());
}

async fn test_pre_sign(context: &AsyncContext) {
    let http_client = Client::builder()
        .tcp_nodelay(true)
        .tcp_keepalive(None)
        .redirect(redirect::Policy::none())
        .no_gzip()
        .no_deflate()
        .no_brotli()
        .danger_accept_invalid_certs(true).build().unwrap();

    let client = context.client();
    let bucket = context.fixed_bucket();
    let mut input = PreSignedURLInput::new(bucket);
    input.set_http_method(HttpMethodHead);
    let output = client.pre_signed_url(&input).await.unwrap();
    let signed_url = output.signed_url();
    assert!(signed_url.len() > 0);

    println!("{}", signed_url);
    let req = http_client.head(signed_url).build().unwrap();
    let resp = http_client.execute(req).await.unwrap();
    assert!(resp.status().is_success());

    let key = gen_random_string(5);
    let mut input = PreSignedURLInput::new(bucket);
    input.set_http_method(HttpMethodPut);
    input.set_key(key.clone());

    let output = client.pre_signed_url(&input).await.unwrap();
    let signed_url = output.signed_url();
    assert!(signed_url.len() > 0);

    let req = http_client.put(signed_url).body(Body::from("helloworld")).build().unwrap();
    let resp = http_client.execute(req).await.unwrap();
    assert!(resp.status().is_success());

    let mut input = PreSignedURLInput::new(bucket);
    input.set_http_method(HttpMethodGet);
    input.set_key(key.clone());

    let output = client.pre_signed_url(&input).await.unwrap();
    let signed_url = output.signed_url();
    assert!(signed_url.len() > 0);

    let req = http_client.get(signed_url).build().unwrap();
    let resp = http_client.execute(req).await.unwrap();
    assert!(resp.status().is_success());
    let cl = resp.headers().get("content-length").unwrap();
    assert_eq!(cl.to_str().unwrap().parse::<usize>().unwrap(), "helloworld".len());
    let data = String::from_utf8(Vec::from(resp.bytes().await.unwrap())).unwrap();
    println!("{}", data);
    assert_eq!(data, "helloworld".to_string());

    let mut input = PreSignedURLInput::new(bucket);
    input.set_http_method(HttpMethodDelete);
    input.set_key(key.clone());

    let output = client.pre_signed_url(&input).await.unwrap();
    let signed_url = output.signed_url();
    assert!(signed_url.len() > 0);

    let req = http_client.delete(signed_url).build().unwrap();
    let resp = http_client.execute(req).await.unwrap();
    assert!(resp.status().is_success());

    let mut input = PreSignedURLInput::new(bucket);
    input.set_http_method(HttpMethodHead);
    input.set_key(key.clone());

    let output = client.pre_signed_url(&input).await.unwrap();
    let signed_url = output.signed_url();
    assert!(signed_url.len() > 0);

    let req = http_client.head(signed_url).build().unwrap();
    let resp = http_client.execute(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}