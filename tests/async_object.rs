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
use bytes::BytesMut;
use chrono::Utc;
use scopeguard::defer;
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::runtime::Handle;
use tokio::{fs, runtime};
use tokio_util::io::ReaderStream;

use crate::common::asynchronous::{create_async_context, read_to_buf, read_to_string, AsyncContext};
use crate::common::{base64_md5, gen_random_string, hex_md5};
use ve_tos_rust_sdk::asynchronous::bucket::BucketAPI;
use ve_tos_rust_sdk::asynchronous::common::DataTransferListener;
use ve_tos_rust_sdk::asynchronous::object::{ObjectAPI, ObjectContent};
use ve_tos_rust_sdk::asynchronous::tos::{new_stream, BufferStream};
use ve_tos_rust_sdk::bucket::{CreateBucketInput, DeleteBucketRenameInput, GetBucketRenameInput, PutBucketRenameInput};
use ve_tos_rust_sdk::common::{init_tracing_log, DataTransferStatus, DataTransferType, Grant, Grantee, Owner, RateLimiter, Tag, TagSet};
use ve_tos_rust_sdk::enumeration::MetadataDirectiveType::{MetadataDirectiveCopy, MetadataDirectiveReplace};
use ve_tos_rust_sdk::enumeration::StorageClassType::{StorageClassIa, StorageClassStandard};
use ve_tos_rust_sdk::enumeration::{ACLType, CannedType, GranteeType, PermissionType, StorageClassType, TierType};
use ve_tos_rust_sdk::object::{AppendObjectFromBufferInput, AppendObjectInput, CopyObjectInput, DeleteMultiObjectsInput, DeleteObjectInput, DeleteObjectTaggingInput, DoesObjectExistInput, FetchObjectInput, GetFetchTaskInput, GetObjectACLInput, GetObjectInput, GetObjectOutput, GetObjectTaggingInput, GetObjectToFileInput, GetSymlinkInput, HeadObjectInput, ListObjectVersionsInput, ListObjectsType2Input, ObjectTobeDeleted, PutFetchTaskInput, PutObjectACLInput, PutObjectFromBufferInput, PutObjectFromFileInput, PutObjectInput, PutObjectTaggingInput, PutSymlinkInput, RenameObjectInput, RestoreJobParameters, RestoreObjectInput, SetObjectMetaInput};

mod common;

#[test]
fn test_main() {
    let _guard = init_tracing_log("info", "temp/logs", "app.log");
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
        test_put_object(&context).await;
        test_copy_object(&context).await;
        test_get_object(&context).await;
        test_delete_multi_objects(&context).await;
        test_put_object_acl(&context).await;
        test_set_object_meta_tags(&context).await;
        test_list_objects(&context).await;
        test_append_object(&context).await;
        test_put_object_from_file(&context).await;
        test_fetch_object(&context).await;
        test_rename_object(&context).await;
        test_restore_object(&context).await;
        test_symlink(&context).await;
        test_multi_contents(&context).await;
    });
}

async fn test_multi_contents(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";
    let mut input = PutObjectFromBufferInput::new(bucket, key);
    input.append_content_nocopy(data);
    input.append_content_nocopy(data);

    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let ginput = GetObjectInput::new(bucket, key);
    let mut o = client.get_object(&ginput).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let buf = read_to_buf(o.content().unwrap()).await;
    let mut new_data = String::with_capacity(data.len() * 3);
    new_data.push_str(data);
    new_data.push_str(data);
    let readed_data = String::from_utf8(buf).unwrap();
    assert_eq!(readed_data, new_data);
    println!("{}", readed_data);
}

async fn test_symlink(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";
    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_type("image/jpeg");
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let new_key = gen_random_string(10);
    let new_key = new_key.as_str();
    let mut input = PutSymlinkInput::new(context.fixed_bucket(), new_key, key);
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_forbid_overwrite(true);
    input.set_tagging("tag1=value1&tag2=value2");
    input.set_content_disposition("attachment; filename=中文.txt");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    let o = client.put_symlink(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let ginput = GetObjectInput::new(bucket, new_key);
    let mut o = client.get_object(&ginput).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let buf = read_to_buf(o.content().unwrap()).await;
    assert_eq!(String::from_utf8(buf).unwrap(), data);
    assert_eq!(o.object_type(), "Symlink");
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassStandard);
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");

    let ginput = HeadObjectInput::new(bucket, new_key);
    let mut o = client.head_object(&ginput).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.object_type(), "Symlink");
    assert!(o.content_length() > 0);
    assert_eq!(o.symlink_target_size(), data.len() as i64);
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassStandard);
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");

    let o = client.get_object_tagging(&GetObjectTaggingInput::new(bucket, new_key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.tag_set().tags().len(), 2);
    assert_eq!(o.tag_set().tags().get(0).unwrap().key(), "tag1");
    assert_eq!(o.tag_set().tags().get(0).unwrap().value(), "value1");
    assert_eq!(o.tag_set().tags().get(1).unwrap().key(), "tag2");
    assert_eq!(o.tag_set().tags().get(1).unwrap().value(), "value2");

    let o = client.get_symlink(&GetSymlinkInput::new(context.fixed_bucket(), new_key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.symlink_target_bucket(), context.fixed_bucket());
    assert_eq!(o.symlink_target_key(), key);
    assert!(o.last_modified().is_some());

    let o = client.delete_object(&DeleteObjectInput::new(context.fixed_bucket(), new_key)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_symlink(&GetSymlinkInput::new(context.fixed_bucket(), new_key)).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}

async fn test_rename_object(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";
    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_type("image/jpeg");
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let new_key = gen_random_string(10);
    let new_key = new_key.as_str();
    let mut input = RenameObjectInput::new(bucket, key, new_key);
    input.set_forbid_overwrite(true);
    input.set_recursive_mkdir(true);
    input.set_notification_custom_parameters("test");
    let e = client.rename_object(&input).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);

    let o = client.put_bucket_rename(&PutBucketRenameInput::new(bucket, true)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_rename(&GetBucketRenameInput::new(bucket)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rename_enable(), true);

    tokio::time::sleep(Duration::from_secs(10)).await;

    let mut input = RenameObjectInput::new(bucket, key, new_key);
    input.set_forbid_overwrite(true);
    input.set_recursive_mkdir(true);
    input.set_notification_custom_parameters("test");
    let o = client.rename_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let md5 = base64_md5(data);
    let mut ginput = GetObjectInput::new(bucket, new_key);
    let mut o = client.get_object(&ginput).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.content_type(), "image/jpeg");
    let source = base64_md5(read_to_buf(o.content().unwrap()).await);
    assert_eq!(source, md5.clone());

    ginput.set_key(key);
    let e = client.get_object(&ginput).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);

    let o = client.delete_bucket_rename(&DeleteBucketRenameInput::new(bucket)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_rename(&GetBucketRenameInput::new(bucket)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rename_enable(), false);
}
async fn test_restore_object(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";
    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_storage_class(StorageClassType::StorageClassArchive);
    input.set_content_type("image/jpeg");
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 403);

    let mut input = RestoreObjectInput::new(bucket, key);
    input.set_notification_custom_parameters("test");
    input.set_days(1);
    input.set_restore_job_parameters(RestoreJobParameters::new(TierType::TierStandard));
    let o = client.restore_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.head_object(&HeadObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.restore_info().is_some());
    assert_eq!(o.restore_info().as_ref().unwrap().restore_param().as_ref().unwrap().tier().as_ref().unwrap(), &TierType::TierStandard);
    assert_eq!(o.restore_info().as_ref().unwrap().restore_param().as_ref().unwrap().expiry_days(), 1);
}

async fn test_fetch_object(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let src_key = gen_random_string(10);
    let data = "hello world";
    let mut input = PutObjectFromBufferInput::new_with_content(bucket, src_key.as_str(), data);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_type("image/jpeg");
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.status_code(), 200);

    let md5 = base64_md5(data);

    let mut input = FetchObjectInput::new(bucket, key);
    input.set_ignore_same_key(true);
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_url(format!("https://{}.tos-cn-boe.volces.com/{}", bucket, src_key));
    input.set_forbid_overwrite(true);
    input.set_content_md5(md5.clone());
    input.set_object_expires(1);
    input.set_tagging("tag1=value1&tag2=value2");
    input.set_content_disposition("attachment; filename=中文.txt");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");

    let o = client.fetch_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.source_content_length(), data.len() as i64);
    assert_eq!(o.source_content_type(), "text/plain");
    assert_eq!(o.md5(), hex_md5(data));

    let ginput = GetObjectInput::new(bucket, key);
    let mut o = client.get_object(&ginput).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let source = base64_md5(read_to_buf(o.content().unwrap()).await);
    assert_eq!(source, md5.clone());

    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    // assert_eq!(o.website_redirect_location(), input.website_redirect_location());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");

    let o = client.get_object_tagging(&GetObjectTaggingInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.tag_set().tags().len(), 2);
    assert_eq!(o.tag_set().tags().get(0).unwrap().key(), "tag1");
    assert_eq!(o.tag_set().tags().get(0).unwrap().value(), "value1");
    assert_eq!(o.tag_set().tags().get(1).unwrap().key(), "tag2");
    assert_eq!(o.tag_set().tags().get(1).unwrap().value(), "value2");

    let key2 = gen_random_string(10);
    let key2 = key2.as_str();

    let mut input = PutFetchTaskInput::new(bucket, key2);
    input.set_ignore_same_key(true);
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_url(format!("https://{}.tos-cn-boe.volces.com/{}", bucket, src_key));
    input.set_forbid_overwrite(true);
    input.set_content_md5(md5.clone());
    input.set_object_expires(1);
    input.set_tagging("tag1=value1&tag2=value2");
    input.set_content_disposition("attachment; filename=中文.txt");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");

    let o = client.put_fetch_task(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.task_id().len() > 0);

    let task_id = o.task_id();
    let mut state = String::new();
    loop {
        let o = client.get_fetch_task(&GetFetchTaskInput::new(bucket, task_id)).await.unwrap();
        assert!(o.request_id().len() > 0);
        if o.err() != "" || o.state() != "Running" {
            state = o.state().to_string();
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert_eq!(state, "Succeed");

    let ginput = GetObjectInput::new(bucket, key2);
    let mut o = client.get_object(&ginput).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let source = base64_md5(read_to_buf(o.content().unwrap()).await);
    assert_eq!(source, md5.clone());

    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);
    // assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    // assert_eq!(o.content_disposition(), input.content_disposition());
    // assert_eq!(o.content_encoding(), input.content_encoding());
    // assert_eq!(o.content_language(), input.content_language());
    // assert_eq!(o.cache_control(), input.cache_control());
    // assert_eq!(o.content_type(), input.content_type());
    // assert_eq!(o.website_redirect_location(), input.website_redirect_location());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");

    // let o = client.get_object_tagging(&GetObjectTaggingInput::new(bucket, key)).await.unwrap();
    // assert!(o.request_id().len() > 0);
    // assert_eq!(o.tag_set().tags().len(), 2);
    // assert_eq!(o.tag_set().tags().get(0).unwrap().key(), "tag1");
    // assert_eq!(o.tag_set().tags().get(0).unwrap().value(), "value1");
    // assert_eq!(o.tag_set().tags().get(1).unwrap().key(), "tag2");
    // assert_eq!(o.tag_set().tags().get(1).unwrap().value(), "value2");
}

async fn test_put_object_from_file(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let folder = env::current_dir().unwrap().display().to_string();
    let file_path = folder.clone() + "/tests/1.jpg";
    let mut input = PutObjectFromFileInput::new(bucket, key);
    input.set_file_path(file_path.clone());
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_disposition("test-disposition");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");

    let (s, r) = async_channel::unbounded::<DataTransferStatus>();
    Handle::current().spawn(async move {
        loop {
            match r.recv().await {
                Err(_) => {
                    break;
                }
                Ok(item) => {
                    match item.data_transfer_type() {
                        DataTransferType::DataTransferStarted => {
                            println!("{}-{}, started", item.bucket(), item.key());
                        }
                        DataTransferType::DataTransferRW => {
                            println!("{}-{}, {}%", item.bucket(), item.key(), (item.consumed_bytes() as f64 / item.total_bytes() as f64) * 100.0);
                        }
                        DataTransferType::DataTransferSucceed => {
                            println!("{}-{}, succeed", item.bucket(), item.key());
                        }
                        DataTransferType::DataTransferFailed => {
                            println!("{}-{}, failed", item.bucket(), item.key());
                        }
                    }
                }
            }
        }
    });

    input.set_async_data_transfer_listener(s.clone());
    let rl = Arc::new(RateLimiter::new(30 * 1024 * 1024, 1024 * 1024));
    input.set_rate_limiter(rl.clone());
    let o = client.put_object_from_file(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut ginput = GetObjectInput::new(bucket, key);
    ginput.set_async_data_transfer_listener(s.clone());
    ginput.set_rate_limiter(rl.clone());
    let mut o = client.get_object(&ginput).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let fd = File::open(file_path).await.unwrap();
    let source = base64_md5(read_to_buf(o.content().unwrap()).await);
    let target = base64_md5(read_to_buf(&mut ReaderStream::new(fd)).await);
    println!("{source}");
    assert_eq!(source, target);

    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    assert_eq!(o.website_redirect_location(), input.website_redirect_location());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");
    let mut idx = 0;
    for mut dst in vec![
        folder.clone() + "/tests/temp/2.jpg",
        folder.clone() + "/tests/temp/a/b/c/3.jpg",
        folder.clone() + "/tests/temp"] {
        let p = Path::new(&dst);
        if p.exists() && p.is_dir() {
            dst = dst + "/" + key;
        }
        let mut ginput = GetObjectToFileInput::new(bucket, key, dst.clone());
        if idx == 0 {
            ginput.set_async_data_transfer_listener(s.clone());
        }
        ginput.set_rate_limiter(rl.clone());
        let o = client.get_object_to_file(&ginput).await.unwrap();
        assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);
        assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
        assert_eq!(o.content_disposition(), input.content_disposition());
        assert_eq!(o.content_encoding(), input.content_encoding());
        assert_eq!(o.content_language(), input.content_language());
        assert_eq!(o.cache_control(), input.cache_control());
        assert_eq!(o.content_type(), input.content_type());
        assert_eq!(o.website_redirect_location(), input.website_redirect_location());
        assert_eq!(o.meta().len(), 2);
        assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
        assert_eq!(o.meta().get("中文键").unwrap(), "中文值");
        let fd = File::open(dst.clone()).await.unwrap();
        let target = base64_md5(read_to_buf(&mut ReaderStream::new(fd)).await);
        assert_eq!(source, target);
        idx += 1;
    }
    fs::remove_dir_all(folder.clone() + "/tests/temp").await.unwrap();
}

async fn test_append_object(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data_len = 20000;
    let data = gen_random_string(data_len);

    let stream_data = new_stream(data.clone());
    let mut input = AppendObjectInput::<BufferStream>::new_with_content(bucket, key, stream_data);
    input.set_content_length(data_len as i64);
    let o = client.append_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.content_length(), data_len as i64);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);
    assert_eq!(o.meta().len(), 0);

    let o = client.delete_object(&DeleteObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = AppendObjectFromBufferInput::new_with_content(bucket, key, data.clone());
    input.set_storage_class(StorageClassStandard);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_disposition("attachment; filename=中文.txt");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");
    let o = client.append_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);
    let next_append_offset = o.next_append_offset();

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.content_length(), data_len as i64);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassStandard);
    assert_eq!(o.content_disposition(), "attachment; filename=中文.txt");
    assert_eq!(o.content_type(), "text/plain");
    assert_eq!(o.content_encoding(), "test-encoding");
    assert_eq!(o.content_language(), "test-language");
    assert_eq!(o.cache_control(), "test-cache-control");
    assert_eq!(o.website_redirect_location(), "http://test-website-redirection-location");

    let mut input = AppendObjectFromBufferInput::new_with_offset_content(bucket, key, next_append_offset, data.clone());
    input.set_pre_hash_crc64ecma(o.hash_crc64ecma());
    let o = client
        .append_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);

    let mut input = AppendObjectFromBufferInput::new_with_offset_content(bucket, key, o.next_append_offset(), data.clone());
    input.set_pre_hash_crc64ecma(o.hash_crc64ecma());
    let o = client
        .append_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.content_length(), data_len as i64 * 3);
    let buf = read_to_string(o.content().unwrap()).await;
    let mut new_data = String::with_capacity(data_len * 3);
    new_data.push_str(data.as_str());
    new_data.push_str(data.as_str());
    new_data.push_str(data.as_str());
    assert_eq!(buf, new_data);

    let e = client
        .append_object_from_buffer(&AppendObjectFromBufferInput::new_with_content(context.non_exists_bucket(), key, data)).await.expect_err("");
    assert!(e.is_server_error());
    assert_eq!(e.as_server_error().unwrap().code(), "NoSuchBucket");
}


async fn test_list_objects(context: &AsyncContext) {
    let client = context.client();
    let e = client.list_objects_type2(&ListObjectsType2Input::new(context.non_exists_bucket())).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.list_object_versions(&ListObjectVersionsInput::new(context.non_exists_bucket())).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let mut input = DeleteMultiObjectsInput::new(context.non_exists_bucket());
    input.add_object(ObjectTobeDeleted::new("test-key"));
    let e = client.delete_multi_objects(&input).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let bucket1 = gen_random_string(40);
    let bucket1 = bucket1.as_str();
    let o = client.create_bucket(&CreateBucketInput::new(bucket1)).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket1).await;

    let prefix = gen_random_string(5) + "/";
    let mut keys_map = HashMap::with_capacity(100);
    for _ in 0..100 {
        let key = gen_random_string(10);
        keys_map.insert(key, 1);
    }
    let mut keys_array = Vec::with_capacity(keys_map.len());
    for (k, _) in keys_map {
        keys_array.push(k);
    }
    let mut keys = Vec::with_capacity(keys_array.len());
    for (idx, key) in keys_array.iter().enumerate() {
        keys.push(prefix.clone() + key);
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, keys[idx].as_str(), "hello world")).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    let (s, r) = async_channel::unbounded();
    let cli = client.clone();
    let bkt1 = bucket1.to_string();

    Handle::current().spawn(async move {
        let mut input = ListObjectsType2Input::new(bkt1);
        input.set_max_keys(10);
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
    });


    let mut keys_from_server = Vec::with_capacity(keys_array.len());
    while let Ok(result) = r.recv().await {
        for content in result.contents() {
            keys_from_server.push(content.key().to_owned());
        }
    }

    assert_eq!(keys.len(), keys_from_server.len());
    keys.sort();
    keys_from_server.sort();
    assert_eq!(keys, keys_from_server);

    let mut objects = Vec::with_capacity(keys.len());
    for i in 0..50 {
        objects.push(ObjectTobeDeleted::new(keys[i].as_str()));
    }

    let o = client.delete_multi_objects(&DeleteMultiObjectsInput::new_with_objects(bucket1, objects)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.deleted().len(), 50);
    assert_eq!(o.error().len(), 0);

    keys = keys.as_slice()[50..].to_vec();

    let mut keys_from_server = Vec::with_capacity(50);
    let mut input = ListObjectsType2Input::new(bucket1);
    input.set_prefix(prefix.clone());
    input.set_max_keys(10);
    loop {
        let o = client.list_objects_type2(&input).await.unwrap();
        for content in o.contents() {
            keys_from_server.push(content.key().to_string());
            assert!(content.hash_crc64ecma() > 0);
        }

        if !o.is_truncated() {
            break;
        }
        input.set_continuation_token(o.next_continuation_token());
    }

    keys_from_server.sort();
    keys.sort();
    assert_eq!(keys_from_server, keys);

    let mut keys = Vec::with_capacity(100);
    let prefix2 = gen_random_string(5) + "/";
    for i in 1..3 {
        let key = prefix2.clone() + "abc/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "abc/123/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "bcd/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "bcd/456/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "cde/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).await.unwrap();
        assert!(o.request_id().len() > 0);
    }
    for i in 1..3 {
        let key = prefix2.clone() + "cde/789/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    let mut keys_from_server = Vec::with_capacity(100);
    let mut common_prefixes = Vec::with_capacity(10);
    common_prefixes.push(prefix2);
    while common_prefixes.len() > 0 {
        let prefix = common_prefixes.remove(0);
        list_by_prefix(context, bucket1, prefix, &mut keys_from_server, &mut common_prefixes).await;
    }
    keys.sort();
    keys_from_server.sort();
    assert_eq!(keys, keys_from_server);
}

async fn list_by_prefix(context: &AsyncContext, bucket: &str, prefix: String, keys_from_server: &mut Vec<String>, common_prefixes: &mut Vec<String>) {
    let client = context.client();
    let mut input = ListObjectsType2Input::new(bucket);
    input.set_max_keys(1000);
    input.set_delimiter("/");
    input.set_prefix(prefix);
    loop {
        let o = client.list_objects_type2(&input).await.unwrap();
        for content in o.contents() {
            keys_from_server.push(content.key().to_string());
        }

        for common_prefix in o.common_prefixes() {
            common_prefixes.push(common_prefix.prefix().to_string());
        }

        if !o.is_truncated() {
            break;
        }

        input.set_continuation_token(o.next_continuation_token());
    }
}


async fn test_set_object_meta_tags(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let stream_data = new_stream(data);
    let input = &mut PutObjectInput::<BufferStream>::new_with_content(bucket, key, stream_data);
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_disposition("test-disposition");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");
    input.set_content_md5(base64_md5(data));
    let o = client.put_object(input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.status_code(), 200);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);

    assert_eq!(o.content_length(), data.len() as i64);
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);

    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    assert_eq!(o.website_redirect_location(), input.website_redirect_location());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");

    let mut input = GetObjectInput::new(bucket, key);
    input.set_response_content_disposition("attachment; filename=\"中文.txt");
    let mut o = client.get_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()).await, data);
    assert_eq!(o.content_disposition(), "attachment; filename=\"中文.txt");

    let mut input = SetObjectMetaInput::new(bucket, key);
    input.set_content_disposition("test-disposition-new");
    let expires = Utc::now() + Duration::from_secs(7200);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("ccc".to_string(), "ddd".to_string()), ("中文键-new".to_string(), "中文值-new".to_string())]));
    input.set_content_encoding("test-encoding-new");
    input.set_content_language("test-language-new");
    input.set_cache_control("test-cache-control-new");
    input.set_content_type("text/plain-new");
    input.set_metadata_directive(MetadataDirectiveReplace);

    let o = client.set_object_meta(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.head_object(&HeadObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    assert_eq!(o.website_redirect_location(), "http://test-website-redirection-location");
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("ccc").unwrap(), "ddd");
    assert_eq!(o.meta().get("中文键-new").unwrap(), "中文值-new");

    let e = client.head_object(&HeadObjectInput::new(context.non_exists_bucket(), key)).await.expect_err("");
    assert!(e.is_server_error());
    assert_eq!(e.as_server_error().unwrap().status_code(), 404);

    let mut input = SetObjectMetaInput::new(context.non_exists_bucket(), key);
    input.set_meta(HashMap::from([("ccc".to_string(), "ddd".to_string()), ("中文键-new".to_string(), "中文值-new".to_string())]));
    let e = client.set_object_meta(&input).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let mut input = SetObjectMetaInput::new(bucket, "abc");
    input.set_meta(HashMap::from([("ccc".to_string(), "ddd".to_string()), ("中文键-new".to_string(), "中文值-new".to_string())]));
    let e = client.set_object_meta(&input).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");

    let e = client.delete_object(&DeleteObjectInput::new(context.non_exists_bucket(), key)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");


    let e = client.set_object_meta(&SetObjectMetaInput::new_with_version_id(bucket, key, "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);
    assert_eq!(ex.code(), "InvalidArgument");

    let e = client.head_object(&HeadObjectInput::new_with_version_id(bucket, key, "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);

    let e = client.delete_object(&DeleteObjectInput::new_with_version_id(bucket, key, "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);
    assert_eq!(ex.code(), "InvalidArgument");

    let mut input = PutObjectTaggingInput::new(bucket, key);
    input.set_tag_set(TagSet::new(vec![Tag::new("key1".to_string(), "value1".to_string()), Tag::new("key2".to_string(), "value2".to_string())]));
    let o = client.put_object_tagging(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_tagging(&GetObjectTaggingInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.tag_set().tags().len(), 2);
    assert_eq!(o.tag_set().tags().get(0).unwrap().key(), "key1");
    assert_eq!(o.tag_set().tags().get(0).unwrap().value(), "value1");
    assert_eq!(o.tag_set().tags().get(1).unwrap().key(), "key2");
    assert_eq!(o.tag_set().tags().get(1).unwrap().value(), "value2");

    let input = DeleteObjectTaggingInput::new(bucket, key);
    let o = client.delete_object_tagging(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_tagging(&GetObjectTaggingInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.tag_set().tags().len(), 0);
}

async fn test_put_object_acl(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_acl(ACLType::ACLPublicRead);
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.grants().len(), 1);
    assert_eq!(o.grants()[0].permission(), &PermissionType::PermissionRead);
    assert_eq!(o.grants()[0].grantee().grantee_type(), &GranteeType::GranteeGroup);
    assert_eq!(o.grants()[0].grantee().canned().as_ref().unwrap(), &CannedType::CannedAllUsers);

    let o = client.put_object_acl(&PutObjectACLInput::new_with_acl(bucket, key, ACLType::ACLPublicReadWrite)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.grants().len(), 2);

    for grant in o.grants() {
        assert!(grant.permission() == &PermissionType::PermissionRead || grant.permission() == &PermissionType::PermissionWrite);
        assert_eq!(grant.grantee().grantee_type(), &GranteeType::GranteeGroup);
        assert_eq!(grant.grantee().canned().as_ref().unwrap(), &CannedType::CannedAllUsers);
    }

    let owner_id = o.owner().id();
    let mut input = PutObjectACLInput::new(bucket, key);
    input.set_owner(Owner::new(owner_id));
    input.set_grants(Vec::from([Grant::new(Grantee::new_with_canned(GranteeType::GranteeGroup, CannedType::CannedAuthenticatedUsers), PermissionType::PermissionRead)]));
    let o = client.put_object_acl(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.grants().len(), 1);
    assert_eq!(o.grants()[0].permission(), &PermissionType::PermissionRead);
    assert_eq!(o.grants()[0].grantee().grantee_type(), &GranteeType::GranteeGroup);
    assert_eq!(o.grants()[0].grantee().canned().as_ref().unwrap(), &CannedType::CannedAuthenticatedUsers);

    let mut input = PutObjectACLInput::new(bucket, key);
    input.set_owner(Owner::new(owner_id));
    input.set_grants(Vec::from([Grant::new(Grantee::new_with_id(GranteeType::GranteeUser, owner_id), PermissionType::PermissionFullControl)]));
    let o = client.put_object_acl(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.grants().len(), 1);
    assert_eq!(o.grants()[0].permission(), &PermissionType::PermissionFullControl);
    assert_eq!(o.grants()[0].grantee().grantee_type(), &GranteeType::GranteeUser);
    assert_eq!(o.grants()[0].grantee().id(), owner_id);

    let e = client.put_object_acl(&PutObjectACLInput::new_with_acl(context.non_exists_bucket(), key, ACLType::ACLPublicReadWrite)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.put_object_acl(&PutObjectACLInput::new_with_acl(bucket, gen_random_string(400), ACLType::ACLPublicReadWrite)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");

    let e = client.get_object_acl(&GetObjectACLInput::new(context.non_exists_bucket(), key)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.get_object_acl(&GetObjectACLInput::new(bucket, gen_random_string(400))).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");
}

async fn test_delete_multi_objects(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let data = "hello world";
    let mut keys = Vec::new();
    for i in 0..5 {
        let key = String::from("key") + i.to_string().as_str();
        let o = client.delete_object(&DeleteObjectInput::new(bucket, key.clone())).await.unwrap();
        assert!(o.request_id().len() > 0);
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket, key.clone(), data)).await.unwrap();
        assert!(o.request_id().len() > 0);
        keys.push(key);
    }

    let mut objects = Vec::with_capacity(keys.len());
    for key in keys.iter() {
        objects.push(ObjectTobeDeleted::new(key));
    }
    let o = client.delete_multi_objects(&DeleteMultiObjectsInput::new_with_objects(bucket, objects)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut key2 = Vec::new();
    for deleted in o.deleted().iter() {
        key2.push(deleted.key().to_string());
    }
    keys.sort();
    key2.sort();
    assert_eq!(keys, key2);

    for key in keys.iter() {
        let ex = client.head_object(&HeadObjectInput::new(bucket, key)).await.expect_err("");
        assert_eq!(ex.as_server_error().unwrap().status_code(), 404);
    }
}

async fn test_get_object(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let o = client.delete_object(&DeleteObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);

    assert_eq!(client.does_object_exist(&DoesObjectExistInput::new(bucket, key)).await.unwrap(), false);

    let e = client.get_object(&GetObjectInput::new(bucket, key)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");
    assert_eq!(ex.key(), key);

    let data = "hello world";
    let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket, key, data)).await.unwrap();
    assert!(o.request_id().len() > 0);

    assert_eq!(client.does_object_exist(&DoesObjectExistInput::new(bucket, key)).await.unwrap(), true);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()).await, data);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(&mut o).await, data);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(String::from_utf8(o.read_all().await.unwrap()).unwrap(), data);

    let file_path = env::current_dir().unwrap().display().to_string() + "/tests/1.jpg";
    let o = client.put_object_from_file(&PutObjectFromFileInput::new_with_file_path(bucket, key, file_path)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = GetObjectInput::new(bucket, key);
    input.set_process("image/info");
    let mut o = client.get_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    let result = read_to_string(o.content().unwrap()).await;
    assert!(result.len() > 0);
    println!("{}", result);
    assert!(result.contains("FileSize"));
    assert!(result.contains("Format"));
    assert!(result.contains("ImageHeight"));
    assert!(result.contains("ImageWidth"));
}

async fn test_copy_object(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("ccc".to_string(), "ddd".to_string())]));
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let dst_key = key.to_string() + "-bak";
    let input = CopyObjectInput::new(bucket, dst_key.as_str(), bucket, key);
    let o = client.copy_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, dst_key.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);

    let bucket2 = gen_random_string(40);
    let o = client.create_bucket(&CreateBucketInput::new(bucket2.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket2.as_str()).await;

    // copy 到另外一个桶
    let input = CopyObjectInput::new(bucket2.as_str(), dst_key.as_str(), bucket, key);
    let o = client.copy_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket2.as_str(), dst_key.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);

    // 包含所有参数 + directive copy
    let dst_key2 = gen_random_string(40);
    let mut input = CopyObjectInput::new(bucket2.as_str(), dst_key2.as_str(), bucket, key);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_disposition("attachment; filename=中文.txt");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_storage_class(StorageClassIa);
    input.set_website_redirect_location("http://test-website-redirection-location");
    input.set_metadata_directive(MetadataDirectiveCopy);
    let o = client.copy_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket2.as_str(), dst_key2.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("ccc").unwrap(), "ddd");

    input.set_metadata_directive(MetadataDirectiveReplace);
    let o = client.copy_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let mut o = client.get_object(&GetObjectInput::new(bucket2.as_str(), dst_key2.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);
    assert_eq!(o.content_disposition(), "attachment; filename=中文.txt");
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");
    assert_eq!(o.content_encoding(), "test-encoding");
    assert_eq!(o.content_language(), "test-language");
    assert_eq!(o.cache_control(), "test-cache-control");
    assert_eq!(o.content_type(), "text/plain");
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);
    assert_eq!(o.website_redirect_location(), "http://test-website-redirection-location");

    let e = client.copy_object(&CopyObjectInput::new_with_version_id(bucket2.as_str(), dst_key2.as_str(), bucket, key, "123")).await.expect_err("");
    assert!(e.is_server_error());
    assert_eq!(e.as_server_error().unwrap().code(), "InvalidArgument");
}

async fn test_put_object(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let data = "hello world";

    // 上传各种有效字符的对象
    let keys = ["  ", "a", "仅包含中文", "にほんご", "Ελληνικά", "Ελληνικά", "（!-_.*()/&$@=;:+ ,?\\{^}%`]>[~<#|'\"）", "//"];
    for k in keys {
        let o = client.put_object_from_buffer(&mut PutObjectFromBufferInput::new_with_content(bucket, k, data)).await.unwrap();
        assert!(o.request_id().len() > 0);
        assert!(o.etag().len() > 0);
        assert_eq!(o.status_code(), 200);

        let mut o = client.get_object(&GetObjectInput::new(bucket, k)).await.unwrap();
        assert!(o.request_id().len() > 0);
        assert!(o.etag().len() > 0);
        assert!(o.expiration().is_empty());
        let buf = read_to_string(o.content().unwrap()).await;
        assert_eq!(buf, data);

        let o = client.delete_object(&DeleteObjectInput::new(bucket, k)).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    let key = gen_random_string(10);
    let key = key.as_str();

    // 测试 buffer
    let mut buf1 = BytesMut::with_capacity(8 * 1024 * 1024);
    buf1.write_str("helloworld");
    let buf1 = buf1.freeze();
    let mut buf2 = BytesMut::with_capacity(8 * 1024 * 1024);
    buf2.write_str("hiworld");
    let buf2 = buf2.freeze();
    let bytes_list = vec![buf1, buf2];
    let bytes_list2 = bytes_list.clone();
    {
        let mut input = PutObjectFromBufferInput::new(bucket, key);
        input.set_content_with_bytes_list(bytes_list.into_iter());
        let o = client.put_object_from_buffer(&input).await.unwrap();
        assert!(o.request_id().len() > 0);

        let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
        assert!(o.request_id().len() > 0);
        assert!(o.etag().len() > 0);
        let buf = read_to_string(o.content().unwrap()).await;
        assert_eq!(buf, "helloworldhiworld");
    }
    let mut idx = 0;
    for buf in bytes_list2 {
        let buf_mut = buf.try_into_mut().unwrap();
        if idx == 0 {
            assert_eq!("helloworld", String::from_utf8_lossy(&buf_mut));
        } else {
            assert_eq!("hiworld", String::from_utf8_lossy(&buf_mut));
        }
        idx += 1;
    }


    let stream_data = new_stream(data);
    // 所有参数上传对象
    let mut input = PutObjectInput::<BufferStream>::new_with_content(bucket, key, stream_data);
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_content_disposition("attachment; filename=中文.txt");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");
    input.set_content_md5(base64_md5(data));
    input.set_object_expires(1);
    let o = client.put_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.status_code(), 200);

    let o = client.list_objects_type2(&ListObjectsType2Input::new(bucket)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.contents().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);

    assert_eq!(o.content_length(), data.len() as i64);
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    assert_eq!(o.website_redirect_location(), input.website_redirect_location());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");
    assert!(o.expiration().len() > 0);

    // 测试 head 接口
    let o = client.head_object(&HeadObjectInput::new(bucket, key)).await.unwrap();
    assert_eq!(o.content_length(), data.len() as i64);
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassIa);
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), expires.format("%Y%m%dT%H%M%SZ").to_string());
    assert_eq!(o.content_disposition(), input.content_disposition());
    assert_eq!(o.content_encoding(), input.content_encoding());
    assert_eq!(o.content_language(), input.content_language());
    assert_eq!(o.cache_control(), input.cache_control());
    assert_eq!(o.content_type(), input.content_type());
    assert_eq!(o.website_redirect_location(), input.website_redirect_location());
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("中文键").unwrap(), "中文值");
    assert!(o.expiration().len() > 0);

    // 测试流式上传
    let o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let key2 = gen_random_string(10);
    let key2 = key2.as_str();
    let o = client.put_object(&mut PutObjectInput::<GetObjectOutput>::new_with_content(bucket, key2, o)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key2)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.content_length(), data.len() as i64);
    let buf = read_to_string(o.content().unwrap()).await;
    assert_eq!(buf, data);
    //上传大小为 0 的对象
    let o = client.put_object(&mut PutObjectInput::<BufferStream>::new_with_content(bucket, key2, new_stream(""))).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let o = client.get_object(&GetObjectInput::new(bucket, key2)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.content_length(), 0);

    // 测试自动设置 MIME 类型
    let keys = ["test", "test.TXT", "test.xml", "test.jpg", "test.HTML", "test.json", "test.mp4"];
    let keys_cts = ["binary/octet-stream", "text/plain", "application/xml", "image/jpeg",
        "text/html", "application/json", "video/mp4"];
    let cts = ["text/plain", ""];
    for ct in cts {
        let mut index = 0;
        for k in keys {
            let mut input = PutObjectFromBufferInput::new(bucket, k);
            input.set_content_type(ct);
            let o = client.put_object_from_buffer(&input).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert!(o.etag().len() > 0);
        }
        for k in keys {
            let o = client.get_object(&GetObjectInput::new(bucket, k)).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert!(o.etag().len() > 0);
            assert_eq!(o.content_length(), 0);
            if ct == "" {
                assert_eq!(o.content_type(), keys_cts[index]);
            } else {
                assert_eq!(o.content_type(), ct);
            }
            index += 1;
        }
    }

    // 测试上传本地文件
    let file_path = env::current_dir().unwrap().display().to_string() + "/tests/1.jpg";
    let file = File::open(file_path.as_str()).await.unwrap();
    let stream_data = ReaderStream::new(file);

    let input = PutObjectInput::<ReaderStream<File>>::new_with_content(bucket, key, stream_data);
    let o = client.put_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = GetObjectInput::new(bucket, key);
    input.set_process("image/info");
    let mut o = client.get_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    let result = read_to_string(o.content().unwrap()).await;
    assert!(result.len() > 0);
    println!("{}", result);
    assert!(result.contains("FileSize"));
    assert!(result.contains("Format"));
    assert!(result.contains("ImageHeight"));
    assert!(result.contains("ImageWidth"));

    let source_md5 = base64_md5(read_to_buf(&mut ReaderStream::new(File::open(file_path).await.unwrap())).await);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let target_md5 = base64_md5(read_to_buf(o.content().unwrap()).await);
    assert_eq!(source_md5, target_md5);

    let mut data = String::with_capacity(65537);
    for i in 0..65537 {
        data.push_str(gen_random_string(1).as_str());
    }
    let o = client.put_object_from_buffer(&mut PutObjectFromBufferInput::new_with_content(bucket, key, data.clone())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.status_code(), 200);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let get_data = read_to_buf(o.content().unwrap()).await;
    assert_eq!(data, String::from_utf8(get_data).unwrap());
}

