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
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chrono::Utc;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{env, fs, thread};
use urlencoding::encode;

use common::gen_random_string;
use ve_tos_rust_sdk::bucket::{BucketAPI, CreateBucketInput};
use ve_tos_rust_sdk::common::{DataTransferListener, DataTransferStatus, DataTransferType, Grant, Grantee, Owner, RateLimiter};
use ve_tos_rust_sdk::enumeration::ACLType::ACLPublicRead;
use ve_tos_rust_sdk::enumeration::MetadataDirectiveType::{MetadataDirectiveCopy, MetadataDirectiveReplace};
use ve_tos_rust_sdk::enumeration::StorageClassType::{StorageClassIa, StorageClassStandard};
use ve_tos_rust_sdk::enumeration::{ACLType, CannedType, GranteeType, PermissionType};
use ve_tos_rust_sdk::object::{AppendObjectFromBufferInput, AppendObjectInput, CopyObjectInput, DeleteMultiObjectsInput, DeleteObjectInput, GetObjectACLInput, GetObjectInput, GetObjectOutput, GetObjectToFileInput, HeadObjectInput, ListObjectVersionsInput, ListObjectsInput, ObjectAPI, ObjectContent, ObjectTobeDeleted, PutObjectACLInput, PutObjectFromBufferInput, PutObjectFromFileInput, PutObjectInput, SetObjectMetaInput};

use crate::common::{base64, base64_md5, create_context, hex_md5, read_to_buf, read_to_string, Context};

mod common;

pub(crate) static CHINESE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\x{4e00}-\x{9fa5}]").unwrap());

pub(crate) fn url_encode_chinese(input: &str) -> String {
    let mut temp = String::with_capacity(input.len() * 2);
    let mut buf;
    for item in input.chars() {
        buf = item.to_string();
        if CHINESE_REGEX.is_match(buf.as_str()) {
            temp.push_str(encode(buf.as_str()).as_ref());
        } else {
            temp.push(item);
        }
    }
    temp
}


#[test]
fn test_main() {
    let context = create_context();
    test_put_object(&context);
    test_invalid_argument(&context);
    test_range(&context);
    test_ssec(&context);
    test_response_header(&context);
    test_cas(&context);
    test_append_object(&context);
    test_copy_object(&context);
    test_get_object(&context);
    test_list_objects(&context);
    test_put_object_acl(&context);
    test_set_object_meta(&context);
    test_put_object_from_file(&context);
    test_multi_contents(&context);
}

fn test_multi_contents(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";
    let mut input = PutObjectFromBufferInput::new(bucket, key);
    input.append_content_nocopy(data);
    input.append_content_nocopy(data);

    let o = client.put_object_from_buffer(&input).unwrap();
    assert!(o.request_id().len() > 0);

    let ginput = GetObjectInput::new(bucket, key);
    let mut o = client.get_object(&ginput).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let buf = read_to_string(o.content().unwrap());
    let mut new_data = String::with_capacity(data.len() * 3);
    new_data.push_str(data);
    new_data.push_str(data);
    assert_eq!(buf, new_data);
    println!("{}", buf);
}

fn test_put_object(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    // 上传各种有效字符的对象
    let keys = ["  ", "a", "仅包含中文", "にほんご", "Ελληνικά", "Ελληνικά", "（!-_.*()/&$@=;:+ ,?\\{^}%`]>[~<#|'\"）", "//"];
    for k in keys {
        let o = client.put_object(&PutObjectInput::<Cursor<&str>>::new_with_content(bucket, k, Cursor::new(data))).unwrap();
        assert!(o.request_id().len() > 0);
        assert!(o.etag().len() > 0);
        assert_eq!(o.status_code(), 200);

        let mut o = client.get_object(&GetObjectInput::new(bucket, k)).unwrap();
        assert!(o.request_id().len() > 0);
        assert!(o.etag().len() > 0);
        let buf = read_to_string(o.content().unwrap());
        assert_eq!(buf, data);

        let o = client.delete_object(&DeleteObjectInput::new(bucket, k)).unwrap();
        assert!(o.request_id().len() > 0);
    }

    // 所有参数上传对象
    let input = &mut PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLPublicRead);
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
    input.set_rate_limiter(RateLimiter::new(1024 * 1024 * 30, 1024 * 1024));
    let o = client.put_object_from_buffer(input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.status_code(), 200);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let buf = read_to_string(o.content().unwrap());
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
    // 测试流式上传
    let o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let key2 = gen_random_string(10);
    let key2 = key2.as_str();
    let o = client.put_object(&PutObjectInput::<GetObjectOutput>::new_with_content(bucket, key2, o)).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key2)).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.content_length(), data.len() as i64);
    let buf = read_to_string(o.content().unwrap());
    assert_eq!(buf, data);
    // 上传大小为 0 的对象
    let o = client.put_object(&PutObjectInput::<Cursor<&str>>::new_with_content(bucket, key2, Cursor::new(""))).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let o = client.get_object(&GetObjectInput::new(bucket, key2)).unwrap();
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
            let o = client.put_object_from_buffer(&input).unwrap();
            assert!(o.request_id().len() > 0);
            assert!(o.etag().len() > 0);
        }
        for k in keys {
            let o = client.get_object(&GetObjectInput::new(bucket, k)).unwrap();
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
}

fn test_invalid_argument(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    // 验证各种错误的对象名参数
    let e = client.put_object(&PutObjectInput::<Cursor<&str>>::new(bucket, "")).expect_err("");
    assert!(!e.is_server_error());

    let input = &mut PutObjectInput::<Cursor<&str>>::new_with_content(bucket, "abc", Cursor::new("hello world"));
    input.set_content_md5(BASE64_STANDARD.encode("hello world"));
    let e = client.put_object(input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);
    assert_eq!(ex.code(), "InvalidDigest");

    let non_exists_bucket = context.non_exists_bucket();
    let e = client.put_object(&PutObjectInput::<Cursor<&str>>::new(non_exists_bucket, "abc")).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.get_object(&GetObjectInput::new(bucket, gen_random_string(400))).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");

    let input = &mut GetObjectInput::new(bucket, gen_random_string(400));
    // version id 不合法
    input.set_version_id("123");
    let e = client.get_object(input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);
    assert_eq!(ex.code(), "InvalidArgument");
}

fn test_ssec(context: &Context) {
    let client = context.https_client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";
    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    let ssec_key = "01234567890123456789012345678901";
    input.set_ssec_algorithm("AES256");
    input.set_ssec_key(base64(ssec_key));
    input.set_ssec_key_md5(base64_md5(ssec_key));

    let o = client.put_object_from_buffer(&input).unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = GetObjectInput::new(bucket, key);
    input.set_ssec_algorithm("AES256");
    input.set_ssec_key(base64(ssec_key));
    input.set_ssec_key_md5(base64_md5(ssec_key));

    let mut o = client.get_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap());
    assert_eq!(buf, data);

    let e = client.get_object(&GetObjectInput::new(bucket, key)).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);
    assert_eq!(ex.code(), "InvalidRequest");
}

fn test_cas(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket, key, data)).unwrap();
    assert!(o.request_id().len() > 0);

    let etag = o.etag();
    let none_match_etag = hex_md5("hi world");
    let mut input = GetObjectInput::new(bucket, key);
    input.set_if_match(etag);
    input.set_if_none_match(none_match_etag.clone());
    input.set_if_modified_since(Utc::now() - Duration::from_secs(3600));
    input.set_if_unmodified_since(Utc::now() + Duration::from_secs(3600));
    let mut o = client.get_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()), data);
    let last_modified = o.last_modified().unwrap();


    let mut input = GetObjectInput::new(bucket, key);
    input.set_if_match(none_match_etag);
    let e = client.get_object(&input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 412);
    assert_eq!(ex.code(), "PreconditionFailed");

    let mut input = GetObjectInput::new(bucket, key);
    input.set_if_none_match(etag);
    let e = client.get_object(&input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 304);

    let mut input = GetObjectInput::new(bucket, key);
    input.set_if_unmodified_since(last_modified - Duration::from_secs(3600));
    let e = client.get_object(&input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 412);
    assert_eq!(ex.code(), "PreconditionFailed");

    let mut input = GetObjectInput::new(bucket, key);
    input.set_if_modified_since(last_modified + Duration::from_secs(3600));
    let e = client.get_object(&input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 304);
}

fn test_range(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket, key, data)).unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = GetObjectInput::new(bucket, key);
    input.set_range_start(0);
    input.set_range_end(0);
    let mut o = client.get_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()), &data[..1]);

    let mut input = GetObjectInput::new(bucket, key);
    input.set_range("bytes=3-4");
    let mut o = client.get_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()), &data[3..3 + (4 - 3 + 1)]);
}

fn test_response_header(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket, key, data)).unwrap();
    assert!(o.request_id().len() > 0);

    let now = Utc::now();
    let mut input = GetObjectInput::new(bucket, key);
    input.set_response_content_type("test-content-type");
    input.set_response_cache_control("test-cache-control");
    input.set_response_content_disposition("attachment; filename=中文.txt");
    input.set_response_content_encoding("test-content-encoding");
    input.set_response_content_language("test-content-language");
    input.set_response_expires(now);

    let mut o = client.get_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()), data);
    assert_eq!(o.content_type(), input.response_content_type());
    assert_eq!(o.cache_control(), input.response_cache_control());
    assert_eq!(o.content_disposition(), input.response_content_disposition());
    assert_eq!(o.content_encoding(), input.response_content_encoding());
    assert_eq!(o.content_language(), input.response_content_language());
    assert_eq!(o.expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string(), input.response_expires().unwrap().format("%Y%m%dT%H%M%SZ").to_string());
}

fn test_append_object(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = gen_random_string(20000);

    let mut input = AppendObjectInput::<Cursor<String>>::new_with_content(bucket, key, Cursor::new(data.clone()));
    input.set_content_length(data.len() as i64);
    let o = client.append_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.content_length(), data.len() as i64);
    let buf = read_to_string(o.content().unwrap());
    assert_eq!(buf, data);
    assert_eq!(o.meta().len(), 0);

    let o = client.delete_object(&DeleteObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = AppendObjectFromBufferInput::new_with_content(bucket, key, data.clone());
    input.set_storage_class(StorageClassStandard);
    input.set_acl(ACLPublicRead);
    input.set_content_disposition("attachment; filename=中文.txt");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");
    let o = client.append_object_from_buffer(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);
    let next_append_offset = o.next_append_offset();

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.content_length(), data.len() as i64);
    let buf = read_to_string(o.content().unwrap());
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
        .append_object_from_buffer(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);

    let mut input = AppendObjectFromBufferInput::new_with_offset_content(bucket, key, o.next_append_offset(), data.clone());
    input.set_pre_hash_crc64ecma(o.hash_crc64ecma());
    let o = client
        .append_object_from_buffer(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.next_append_offset() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.content_length(), data.len() as i64 * 3);
    let buf = read_to_string(o.content().unwrap());
    let mut new_data = String::with_capacity(data.len() * 3);
    new_data.push_str(data.as_str());
    new_data.push_str(data.as_str());
    new_data.push_str(data.as_str());
    assert_eq!(buf, new_data);

    let e = client
        .append_object_from_buffer(&AppendObjectFromBufferInput::new_with_content(context.non_exists_bucket(), key, data)).expect_err("");
    assert!(e.is_server_error());
    assert_eq!(e.as_server_error().unwrap().code(), "NoSuchBucket");
}

fn test_copy_object(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("ccc".to_string(), "ddd".to_string())]));
    let o = client.put_object_from_buffer(&input).unwrap();
    assert!(o.request_id().len() > 0);

    let dst_key = key.to_string() + "-bak";
    let input = CopyObjectInput::new(bucket, dst_key.as_str(), bucket, key);
    let o = client.copy_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, dst_key.as_str())).unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap());
    assert_eq!(buf, data);

    let bucket2 = gen_random_string(40);
    let o = client.create_bucket(&CreateBucketInput::new(bucket2.as_str())).unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket2.as_str());

    // copy 到另外一个桶
    let input = CopyObjectInput::new(bucket2.as_str(), dst_key.as_str(), bucket, key);
    let o = client.copy_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket2.as_str(), dst_key.as_str())).unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap());
    assert_eq!(buf, data);

    // 包含所有参数 + directive copy
    let dst_key2 = gen_random_string(40);
    let mut input = CopyObjectInput::new(bucket2.as_str(), dst_key2.as_str(), bucket, key);
    input.set_acl(ACLPublicRead);
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
    let o = client.copy_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket2.as_str(), dst_key2.as_str())).unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap());
    assert_eq!(buf, data);
    assert_eq!(o.meta().len(), 2);
    assert_eq!(o.meta().get("aaa").unwrap(), "bbb");
    assert_eq!(o.meta().get("ccc").unwrap(), "ddd");

    input.set_metadata_directive(MetadataDirectiveReplace);
    let o = client.copy_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let mut o = client.get_object(&GetObjectInput::new(bucket2.as_str(), dst_key2.as_str())).unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_string(o.content().unwrap());
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

    let e = client.copy_object(&CopyObjectInput::new_with_version_id(bucket2.as_str(), dst_key2.as_str(), bucket, key, "123")).expect_err("");
    assert!(e.is_server_error());
    assert_eq!(e.as_server_error().unwrap().code(), "InvalidArgument");
}

fn test_get_object(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let o = client.delete_object(&DeleteObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_object(&GetObjectInput::new(bucket, key)).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");

    let data = "hello world";
    let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket, key, data)).unwrap();
    assert!(o.request_id().len() > 0);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()), data);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(String::from_utf8(o.read_all().unwrap()).unwrap(), data);

    let file_path = env::current_dir().unwrap().display().to_string() + "/tests/1.jpg";

    let o = client.put_object_from_file(&PutObjectFromFileInput::new_with_file_path(bucket, key, file_path)).unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = GetObjectInput::new(bucket, key);
    input.set_process("image/info");
    input.set_rate_limiter(RateLimiter::new(30 * 1024 * 1024, 1024 * 1024));
    let mut o = client.get_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    let result = read_to_string(o.content().unwrap());
    assert!(result.len() > 0);
    println!("{}", result);
    assert!(result.contains("FileSize"));
    assert!(result.contains("Format"));
    assert!(result.contains("ImageHeight"));
    assert!(result.contains("ImageWidth"));
}


fn test_list_objects(context: &Context) {
    let client = context.client();
    let e = client.list_objects(&ListObjectsInput::new(context.non_exists_bucket())).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.list_object_versions(&ListObjectVersionsInput::new(context.non_exists_bucket())).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let mut input = DeleteMultiObjectsInput::new(context.non_exists_bucket());
    input.add_object(ObjectTobeDeleted::new("test-key"));
    let e = client.delete_multi_objects(&input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let bucket1 = gen_random_string(40);
    let bucket1 = bucket1.as_str();
    let o = client.create_bucket(&CreateBucketInput::new(bucket1)).unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket1);

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
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, keys[idx].as_str(), "hello world")).unwrap();
        assert!(o.request_id().len() > 0);
    }

    let (s, r) = mpsc::channel();
    let cli = client.clone();
    let bkt1 = bucket1.to_string();
    thread::spawn(move || {
        let mut input = ListObjectsInput::new(bkt1);
        input.set_max_keys(10);
        loop {
            let o = cli.list_objects(&input).unwrap();
            assert!(o.request_id().len() > 0);
            let is_truncated = o.is_truncated();
            input.set_marker(o.next_marker());
            s.send(o).unwrap();
            if !is_truncated {
                break;
            }
        }
    });


    let mut keys_from_server = Vec::with_capacity(keys_array.len());
    while let Ok(result) = r.recv() {
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

    let o = client.delete_multi_objects(&DeleteMultiObjectsInput::new_with_objects(bucket1, objects)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.deleted().len(), 50);
    assert_eq!(o.error().len(), 0);

    keys = keys.as_slice()[50..].to_vec();

    let mut keys_from_server = Vec::with_capacity(50);
    let mut input = ListObjectsInput::new(bucket1);
    input.set_prefix(prefix.clone());
    input.set_max_keys(10);
    loop {
        let o = client.list_objects(&input).unwrap();
        for content in o.contents() {
            keys_from_server.push(content.key().to_string());
            assert!(content.hash_crc64ecma() > 0);
        }

        if !o.is_truncated() {
            break;
        }
        input.set_marker(o.next_marker());
    }

    keys_from_server.sort();
    keys.sort();
    assert_eq!(keys_from_server, keys);

    let mut keys = Vec::with_capacity(100);
    let prefix2 = gen_random_string(5) + "/";
    for i in 1..3 {
        let key = prefix2.clone() + "abc/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "abc/123/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "bcd/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "bcd/456/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "cde/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).unwrap();
        assert!(o.request_id().len() > 0);
    }
    for i in 1..3 {
        let key = prefix2.clone() + "cde/789/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.put_object_from_buffer(&PutObjectFromBufferInput::new_with_content(bucket1, key, "hello world")).unwrap();
        assert!(o.request_id().len() > 0);
    }

    let mut keys_from_server = Vec::with_capacity(100);
    let mut common_prefixes = Vec::with_capacity(10);
    common_prefixes.push(prefix2);
    while common_prefixes.len() > 0 {
        let prefix = common_prefixes.remove(0);
        list_by_prefix(context, bucket1, prefix, &mut keys_from_server, &mut common_prefixes);
    }
    keys.sort();
    keys_from_server.sort();
    assert_eq!(keys, keys_from_server);
}

fn list_by_prefix(context: &Context, bucket: &str, prefix: String, keys_from_server: &mut Vec<String>, common_prefixes: &mut Vec<String>) {
    let client = context.client();
    let mut input = ListObjectsInput::new(bucket);
    input.set_max_keys(1000);
    input.set_delimiter("/");
    input.set_prefix(prefix);
    loop {
        let o = client.list_objects(&input).unwrap();
        for content in o.contents() {
            keys_from_server.push(content.key().to_string());
        }

        for common_prefix in o.common_prefixes() {
            common_prefixes.push(common_prefix.prefix().to_string());
        }

        if !o.is_truncated() {
            break;
        }

        input.set_marker(o.next_marker());
    }
}

fn test_put_object_acl(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let mut input = PutObjectFromBufferInput::new_with_content(bucket, key, data);
    input.set_acl(ACLType::ACLPublicRead);
    let o = client.put_object_from_buffer(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.grants().len(), 1);
    assert_eq!(o.grants()[0].permission(), &PermissionType::PermissionRead);
    assert_eq!(o.grants()[0].grantee().grantee_type(), &GranteeType::GranteeGroup);
    assert_eq!(o.grants()[0].grantee().canned().as_ref().unwrap(), &CannedType::CannedAllUsers);

    let o = client.put_object_acl(&PutObjectACLInput::new_with_acl(bucket, key, ACLType::ACLPublicReadWrite)).unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).unwrap();
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
    let o = client.put_object_acl(&input).unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.grants().len(), 1);
    assert_eq!(o.grants()[0].permission(), &PermissionType::PermissionRead);
    assert_eq!(o.grants()[0].grantee().grantee_type(), &GranteeType::GranteeGroup);
    assert_eq!(o.grants()[0].grantee().canned().as_ref().unwrap(), &CannedType::CannedAuthenticatedUsers);

    let mut input = PutObjectACLInput::new(bucket, key);
    input.set_owner(Owner::new(owner_id));
    input.set_grants(Vec::from([Grant::new(Grantee::new_with_id(GranteeType::GranteeUser, owner_id), PermissionType::PermissionFullControl)]));
    let o = client.put_object_acl(&input).unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_object_acl(&GetObjectACLInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.grants().len(), 1);
    assert_eq!(o.grants()[0].permission(), &PermissionType::PermissionFullControl);
    assert_eq!(o.grants()[0].grantee().grantee_type(), &GranteeType::GranteeUser);
    assert_eq!(o.grants()[0].grantee().id(), owner_id);

    let e = client.put_object_acl(&PutObjectACLInput::new_with_acl(context.non_exists_bucket(), key, ACLType::ACLPublicReadWrite)).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.put_object_acl(&PutObjectACLInput::new_with_acl(bucket, gen_random_string(400), ACLType::ACLPublicReadWrite)).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");

    let e = client.get_object_acl(&GetObjectACLInput::new(context.non_exists_bucket(), key)).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.get_object_acl(&GetObjectACLInput::new(bucket, gen_random_string(400))).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");
}

fn test_set_object_meta(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let data = "hello world";

    let input = &mut PutObjectInput::<Cursor<&str>>::new_with_content(bucket, key, Cursor::new(data));
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLPublicRead);
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
    let o = client.put_object(input).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    assert_eq!(o.status_code(), 200);

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let buf = read_to_string(o.content().unwrap());
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
    let mut o = client.get_object(&input).unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(read_to_string(o.content().unwrap()), data);
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

    let o = client.set_object_meta(&input).unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.head_object(&HeadObjectInput::new(bucket, key)).unwrap();
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

    let e = client.head_object(&HeadObjectInput::new(context.non_exists_bucket(), key)).expect_err("");
    assert!(e.is_server_error());
    assert_eq!(e.as_server_error().unwrap().status_code(), 404);

    let mut input = SetObjectMetaInput::new(context.non_exists_bucket(), key);
    input.set_meta(HashMap::from([("ccc".to_string(), "ddd".to_string()), ("中文键-new".to_string(), "中文值-new".to_string())]));
    let e = client.set_object_meta(&input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let mut input = SetObjectMetaInput::new(bucket, "abc");
    input.set_meta(HashMap::from([("ccc".to_string(), "ddd".to_string()), ("中文键-new".to_string(), "中文值-new".to_string())]));
    let e = client.set_object_meta(&input).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchKey");

    let e = client.delete_object(&DeleteObjectInput::new(context.non_exists_bucket(), key)).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");


    let e = client.set_object_meta(&SetObjectMetaInput::new_with_version_id(bucket, key, "123")).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);
    assert_eq!(ex.code(), "InvalidArgument");

    let e = client.head_object(&HeadObjectInput::new_with_version_id(bucket, key, "123")).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);

    let e = client.delete_object(&DeleteObjectInput::new_with_version_id(bucket, key, "123")).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 400);
    assert_eq!(ex.code(), "InvalidArgument");
}

fn test_put_object_from_file(context: &Context) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let folder = env::current_dir().unwrap().display().to_string();
    let file_path = folder.clone() + "/tests/1.jpg";
    let mut input = PutObjectFromFileInput::new(bucket, key);
    input.set_file_path(file_path.clone());
    input.set_storage_class(StorageClassIa);
    input.set_acl(ACLPublicRead);
    input.set_content_disposition("test-disposition");
    let expires = Utc::now() + Duration::from_secs(3600);
    input.set_expires(expires);
    input.set_meta(HashMap::from([("aaa".to_string(), "bbb".to_string()), ("中文键".to_string(), "中文值".to_string())]));
    input.set_content_encoding("test-encoding");
    input.set_content_language("test-language");
    input.set_cache_control("test-cache-control");
    input.set_content_type("text/plain");
    input.set_website_redirect_location("http://test-website-redirection-location");

    let (s, r) = mpsc::channel::<DataTransferStatus>();
    thread::spawn(move || {
        loop {
            match r.recv() {
                Err(e) => {
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

    input.set_data_transfer_listener(s.clone());
    let rl = Arc::new(RateLimiter::new(30 * 1024 * 1024, 1024 * 1024));
    input.set_rate_limiter(rl.clone());
    let o = client.put_object_from_file(&input).unwrap();
    assert!(o.request_id().len() > 0);

    let mut ginput = GetObjectInput::new(bucket, key);
    ginput.set_data_transfer_listener(s.clone());
    ginput.set_rate_limiter(rl.clone());
    let mut o = client.get_object(&ginput).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.etag().len() > 0);
    let mut fd = File::open(file_path).unwrap();
    let source = base64_md5(read_to_buf(o.content().unwrap()));
    let target = base64_md5(read_to_buf(&mut fd));
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
            ginput.set_data_transfer_listener(s.clone());
        }
        ginput.set_rate_limiter(rl.clone());
        let o = client.get_object_to_file(&ginput).unwrap();
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
        let mut fd = File::open(dst.clone()).unwrap();
        let target = base64_md5(read_to_buf(&mut fd));
        assert_eq!(source, target);
        idx += 1;
    }
    fs::remove_dir_all(folder.clone() + "/tests/temp").unwrap();
}