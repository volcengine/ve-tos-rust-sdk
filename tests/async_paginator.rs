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
use crate::common::asynchronous::create_async_context;
use async_trait::async_trait;
use scopeguard::defer;
use std::sync::Arc;
use tokio::runtime;
use ve_tos_rust_sdk::asynchronous::object::ObjectAPI;
use ve_tos_rust_sdk::asynchronous::paginator::ListObjectsType2Paginator;
use ve_tos_rust_sdk::asynchronous::paginator::PaginatorAPI;
use ve_tos_rust_sdk::error::TosError;
use ve_tos_rust_sdk::object::{ListObjectsType2Input, ListObjectsType2Output, PutObjectFromBufferInput};
use ve_tos_rust_sdk::paginator::NextPageInput;
use ve_tos_rust_sdk::paginator::Paginator;

mod common;


pub struct MyPaginator<P> {
    inner: P,
}

unsafe impl<P> Sync for MyPaginator<P> {}

impl<P> Paginator for MyPaginator<P>
where
    P: Paginator,
{
    fn has_next(&self) -> bool {
        self.inner.has_next()
    }

    fn current_prefix(&self) -> Arc<String> {
        self.inner.current_prefix()
    }

    fn close(&self) {
        self.inner.close()
    }
}

#[async_trait]
impl<P> ListObjectsType2Paginator for MyPaginator<P>
where
    P: ListObjectsType2Paginator,
{
    async fn next_page(&self, input: &NextPageInput) -> Result<ListObjectsType2Output, TosError> {
        self.inner.next_page(input).await
    }
}

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
        let client = context.client();
        let bucket = context.fixed_bucket();
        let data = "hello world";

        let mut origin_keys = Vec::with_capacity(100);
        let mut idx = 0;
        for pre in ["prefix1", "prefix2"] {
            let k = format!("{}/key{}", pre, idx);
            let o = client.put_object_from_buffer(&mut PutObjectFromBufferInput::new_with_content(bucket, k.clone(), data)).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert!(o.etag().len() > 0);
            assert_eq!(o.status_code(), 200);
            idx += 1;
            origin_keys.push(k);
            let mut subidx = 0;
            for subpre in ["subprefix1", "subprefix2", "subprefix3"] {
                let k = format!("{}/{}/key{}", pre, subpre, subidx);
                let o = client.put_object_from_buffer(&mut PutObjectFromBufferInput::new_with_content(bucket, k.clone(), data)).await.unwrap();
                assert!(o.request_id().len() > 0);
                assert!(o.etag().len() > 0);
                assert_eq!(o.status_code(), 200);
                subidx += 1;
                origin_keys.push(k);
            }
        }

        let mut root_page_keys = Vec::with_capacity(10);
        for i in 0..10 {
            let k = format!("key{}", i);
            let o = client.put_object_from_buffer(&mut PutObjectFromBufferInput::new_with_content(bucket, k.clone(), data)).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert!(o.etag().len() > 0);
            assert_eq!(o.status_code(), 200);
            origin_keys.push(k.clone());
            root_page_keys.push(k);
        }
        origin_keys.sort();


        let (s, r) = async_channel::bounded(2);
        let mut keys: Vec<String> = Vec::with_capacity(origin_keys.len());
        let bkt = bucket.to_string();
        let cli = client.clone();
        rt.spawn(async move {
            let mut input = ListObjectsType2Input::new(bkt);
            input.set_max_keys(3);
            loop {
                let o = cli.list_objects_type2(&input).await.unwrap();
                assert!(o.request_id().len() > 0);
                let is_truncated = o.is_truncated();
                input.set_continuation_token(o.next_continuation_token());
                s.send(o).await.unwrap();
                if !is_truncated {
                    break;
                }
            }
            s.close();
        });

        while let Ok(result) = r.recv().await {
            for content in result.contents() {
                keys.push(content.key().to_owned());
            }
        }

        keys.sort();
        assert_eq!(keys, origin_keys);


        let mut keys: Vec<String> = Vec::with_capacity(origin_keys.len());
        let input = ListObjectsType2Input::new(bucket);
        let paginator = client.new_list_objects_type2_paginator(&input, true).unwrap();
        let pg = MyPaginator { inner: paginator };
        while pg.has_next() {
            let o = pg.next_page(&NextPageInput {}).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert_eq!(o.status_code(), 200);
            for content in o.contents().iter() {
                keys.push(content.key().to_string());
            }
            println!("current_prefix: {}", pg.current_prefix());
        }
        pg.close();
        keys.sort();
        assert_eq!(keys, origin_keys);

        keys.clear();
        let mut input = ListObjectsType2Input::new(bucket);
        input.set_prefix("prefix1/");
        let paginator = client.new_list_objects_type2_paginator(&input, true).unwrap();
        while paginator.has_next() {
            let o = paginator.next_page(&NextPageInput {}).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert_eq!(o.status_code(), 200);
            for content in o.contents().iter() {
                keys.push(content.key().to_string());
            }
            println!("current_prefix: {}", paginator.current_prefix());
        }
        paginator.close();
        keys.sort();
        assert_eq!(keys, vec!["prefix1/key0", "prefix1/subprefix1/key0", "prefix1/subprefix2/key1", "prefix1/subprefix3/key2"]);

        keys.clear();
        let mut input = ListObjectsType2Input::new(bucket);
        input.set_delimiter("/");
        let paginator = client.new_list_objects_type2_paginator(&input, true).unwrap();
        while paginator.has_next() {
            let o = paginator.next_page(&NextPageInput {}).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert_eq!(o.status_code(), 200);
            for content in o.contents().iter() {
                keys.push(content.key().to_string());
            }
            println!("current_prefix with delimiter: {}", paginator.current_prefix());
        }
        paginator.close();
        keys.sort();
        assert_eq!(keys, origin_keys);

        keys.clear();
        let mut input = ListObjectsType2Input::new(bucket);
        input.set_prefix("prefix1/");
        input.set_delimiter("/");
        let paginator = client.new_list_objects_type2_paginator(&input, true).unwrap();
        while paginator.has_next() {
            let o = paginator.next_page(&NextPageInput {}).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert_eq!(o.status_code(), 200);
            for content in o.contents().iter() {
                keys.push(content.key().to_string());
            }
            println!("current_prefix with delimiter: {}", paginator.current_prefix());
        }
        paginator.close();
        keys.sort();
        assert_eq!(keys, vec!["prefix1/key0", "prefix1/subprefix1/key0", "prefix1/subprefix2/key1", "prefix1/subprefix3/key2"]);

        keys.clear();
        let mut input = ListObjectsType2Input::new(bucket);
        input.set_delimiter("/");
        let paginator = client.new_list_objects_type2_paginator(&input, false).unwrap();
        while paginator.has_next() {
            let o = paginator.next_page(&NextPageInput {}).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert_eq!(o.status_code(), 200);
            for content in o.contents().iter() {
                keys.push(content.key().to_string());
            }
            println!("current_prefix with delimiter: {}", paginator.current_prefix());
        }
        paginator.close();
        keys.sort();
        assert_eq!(keys, root_page_keys);
    });
}