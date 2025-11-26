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
use crate::common::asynchronous::{create_async_context, AsyncContext, TokioRuntime};
use chrono::{Duration, Utc};
use scopeguard::defer;
use serde::de::StdError;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ops::Add;
use tokio::runtime;
use ve_tos_rust_sdk::asynchronous::bucket::BucketAPI;
use ve_tos_rust_sdk::asynchronous::tos;
use ve_tos_rust_sdk::bucket::{DeleteBucketTaggingInput, GetBucketLocationInput, GetBucketTaggingInput, GetBucketTypeInput, ListBucketsInput, PutBucketTaggingInput};
use ve_tos_rust_sdk::common::{Tag, TagSet};
use ve_tos_rust_sdk::credential::{CommonCredentials, Credentials, CredentialsProvider, EnvCredentialsProvider, StaticCredentialsProvider};
use ve_tos_rust_sdk::enumeration::StorageClassType;

mod common;

#[test]
fn test_main() {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let context = rt.block_on(async {
        return create_async_context().await;
    });

    defer! {
        rt.block_on(async {
            context.tear_down().await;
        });
    }

    rt.block_on(async {
        test_credential_provider(&context).await;
        test_bucket_tagging(&context).await;
        test_get_bucket_type(&context).await;
        test_request_host_and_date(&context).await;
        test_request_header_and_query(&context).await;
    })
}

#[derive(Default)]
struct CustomizeCredentials {
    ak: String,
    sk: String,
}

impl Credentials for CustomizeCredentials {
    fn ak(&self) -> &str {
        self.ak.as_str()
    }

    fn sk(&self) -> &str {
        self.sk.as_str()
    }

    fn security_token(&self) -> &str {
        ""
    }

    fn new(ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}

#[derive(Default)]
struct CustomizeCredentialProvider<C> {
    cred: C,
}

impl<C> CredentialsProvider<C> for CustomizeCredentialProvider<C>
where
    C: Credentials,
{
    fn credentials(&self, expires: i64) -> Result<&C, Box<dyn StdError + Send + Sync>> {
        Ok(&self.cred)
    }

    fn new(c: C) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        todo!()
    }
}

async fn test_credential_provider(context: &AsyncContext) {
    let ak = env::var("TOS_ACCESS_KEY").unwrap_or("".to_string());
    let sk = env::var("TOS_SECRET_KEY").unwrap_or("".to_string());
    let cred = CustomizeCredentials { ak: ak.clone(), sk: sk.clone() };
    let ep = env::var("TOS_ENDPOINT").unwrap_or("".to_string());
    let client = tos::builder_common::<CustomizeCredentialProvider<CustomizeCredentials>, CustomizeCredentials, TokioRuntime>()
        .region("test-region")
        .credentials_provider(CustomizeCredentialProvider { cred })
        .endpoint(ep.clone()).build().unwrap();

    let o = client.list_buckets(&ListBucketsInput::new()).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("{}", o.buckets().len());

    let client = tos::builder_common::<StaticCredentialsProvider<CommonCredentials>, CommonCredentials, TokioRuntime>()
        .region("test-region")
        .credentials_provider(tos::static_credentials_provider(ak, sk, ""))
        .endpoint(ep.clone()).build().unwrap();

    let o = client.list_buckets(&ListBucketsInput::new()).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("{}", o.buckets().len());

    let client = tos::builder_common::<EnvCredentialsProvider<CommonCredentials>, CommonCredentials, TokioRuntime>()
        .region("test-region")
        .credentials_provider(tos::env_credentials_provider())
        .endpoint(ep.clone()).build().unwrap();

    let o = client.list_buckets(&ListBucketsInput::new()).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("{}", o.buckets().len());
}

async fn test_request_header_and_query(context: &AsyncContext) {
    let client = context.client();
    let mut input = GetBucketLocationInput::new(context.fixed_bucket());
    input.set_request_header(HashMap::from([("test-header".to_string(), "123".to_string()), ("x-tos-test".to_string(), "888".to_string())]));
    input.set_request_query(HashMap::from([("test-query".to_string(), "456".to_string())]));
    let o = client.get_bucket_location(&mut input).await.unwrap();
    assert!(o.request_id().len() > 0);
}

async fn test_request_host_and_date(context: &AsyncContext) {
    let client = context.client();
    let mut input = GetBucketLocationInput::new(context.fixed_bucket());
    let o = client.get_bucket_location(&mut input).await.unwrap();
    assert!(o.request_id().len() > 0);

    input.set_request_date(Utc::now().add(Duration::days(10)));
    let e = client.get_bucket_location(&mut input).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 403);
    println!("{}", ex.code());

    input.set_request_date(Utc::now());
    input.set_request_host("www.abc.com");
    let e = client.get_bucket_location(&mut input).await.unwrap_err();
    println!("{:?}", e);
}

async fn test_get_bucket_type(context: &AsyncContext) {
    let client = context.client();
    let o = client.get_bucket_type(&GetBucketTypeInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassType::StorageClassStandard);
    println!("{:?}", o.bucket_type());

    let o = client.get_bucket_type(&GetBucketTypeInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassType::StorageClassStandard);
    println!("{:?}", o.bucket_type());
}

async fn test_bucket_tagging(context: &AsyncContext) {
    let client = context.client();
    let tag_set = TagSet::new(vec![Tag::new("key1".to_string(), "value1".to_string()), Tag::new("key2".to_string(), "value2".to_string())]);
    let input = PutBucketTaggingInput::new(context.fixed_bucket(), tag_set);
    let o = client.put_bucket_tagging(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_tagging(&GetBucketTaggingInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.tag_set().tags().len(), 2);
    assert_eq!(o.tag_set().tags().get(0).unwrap().key(), "key1");
    assert_eq!(o.tag_set().tags().get(0).unwrap().value(), "value1");
    assert_eq!(o.tag_set().tags().get(1).unwrap().key(), "key2");
    assert_eq!(o.tag_set().tags().get(1).unwrap().value(), "value2");

    let o = client.delete_bucket_tagging(&DeleteBucketTaggingInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_tagging(&GetBucketTaggingInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}