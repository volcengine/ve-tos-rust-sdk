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
use crate::common::asynchronous::{create_async_context, read_to_buf, AsyncContext};
use crate::common::{base64_md5, gen_random_string};
use scopeguard::defer;
use std::collections::{BTreeMap, HashMap};
use std::io::SeekFrom;
use std::os::unix::fs::MetadataExt;
use std::{env, fs};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::runtime;
use tokio_util::io::ReaderStream;
use ve_tos_rust_sdk::asynchronous::bucket::BucketAPI;
use ve_tos_rust_sdk::asynchronous::multipart::MultipartAPI;
use ve_tos_rust_sdk::asynchronous::object::ObjectAPI;
use ve_tos_rust_sdk::asynchronous::object::ObjectContent;
use ve_tos_rust_sdk::bucket::CreateBucketInput;
use ve_tos_rust_sdk::multipart::{AbortMultipartUploadInput, CompleteMultipartUploadInput, CreateMultipartUploadInput, ListMultipartUploadsInput, ListPartsInput, UploadPartCopyInput, UploadPartFromBufferInput, UploadPartFromFileInput, UploadPartInput, UploadedPart};
use ve_tos_rust_sdk::object::GetObjectInput;


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
        test_multipart_list(&context).await;
        test_multipart_upload(&context).await;
    });
}

async fn test_multipart_upload(context: &AsyncContext) {
    let client = context.client();
    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.upload_id().len() > 0);
    let upload_id = o.upload_id();
    let big_file_path = env::current_dir().unwrap().display().to_string() + "/tests/" + gen_random_string(10).as_str() + ".txt";
    let big_file_path = big_file_path.as_str();
    let mut meta = fs::metadata(big_file_path);
    if meta.is_err() {
        {
            let mut fd = File::options().write(true).truncate(true).create(true).open(big_file_path).await.unwrap();
            let random_str = gen_random_string(20);
            for _ in 0..500000 {
                fd.write(random_str.as_bytes()).await.unwrap();
            }
        }
        meta = fs::metadata(big_file_path);
    }
    let file_size = meta.unwrap().size();
    let source_md5 = base64_md5(read_to_buf(&mut ReaderStream::new(File::open(big_file_path).await.unwrap())).await);
    println!("{source_md5}");

    let first_part_size = 5 * 1024 * 1024;
    let mut etags = Vec::with_capacity(2);
    {
        let mut input = UploadPartInput::<ReaderStream<File>>::new(bucket, key, upload_id);
        input.set_part_number(1);
        let fd = File::open(big_file_path).await.unwrap();
        input.set_content(ReaderStream::new(fd));
        input.set_content_length(first_part_size);
        let o = client.upload_part(&input).await.unwrap();
        assert!(o.request_id().len() > 0);
        println!("upload part etag1 done, {}", o.etag());
        etags.push(o.etag().to_string());
    }
    {
        let mut input = UploadPartInput::<ReaderStream<File>>::new(bucket, key, upload_id);
        input.set_part_number(2);
        let mut fd = File::open(big_file_path).await.unwrap();
        fd.seek(SeekFrom::Start(first_part_size as u64)).await.unwrap();
        input.set_content(ReaderStream::new(fd));
        input.set_content_length(file_size as i64 - first_part_size);
        let o = client.upload_part(&input).await.unwrap();
        assert!(o.request_id().len() > 0);
        println!("upload part etag2 done, {}", o.etag());
        etags.push(o.etag().to_string());
    }

    let o = client.list_parts(&ListPartsInput::new(bucket, key, upload_id)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.parts().len(), 2);

    let o = client.complete_multipart_upload(
        &CompleteMultipartUploadInput::new_with_parts(bucket, key, upload_id, Vec::from([UploadedPart::new(1, etags[0].as_str()), UploadedPart::new(2, etags[1].as_str())]))).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.hash_crc64ecma() > 0);
    println!("complete parts done, {}", o.hash_crc64ecma());

    let mut o = client.get_object(&GetObjectInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let target_md5 = base64_md5(read_to_buf(o.content().unwrap()).await);
    assert_eq!(source_md5, target_md5);

    let key2 = gen_random_string(10);
    let key2 = key2.as_str();
    let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket, key2)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.upload_id().len() > 0);
    let upload_id2 = o.upload_id();
    let mut etags = Vec::with_capacity(2);
    let mut input = UploadPartFromFileInput::new(bucket, key2, upload_id2);
    input.set_part_number(1);
    input.set_file_path(big_file_path);
    input.set_part_size(first_part_size);
    let o = client.upload_part_from_file(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("upload part etag1 done, {}", o.etag());
    etags.push(o.etag().to_string());
    let mut input = UploadPartFromFileInput::new(bucket, key2, upload_id2);
    input.set_part_number(2);
    input.set_file_path(big_file_path);
    input.set_offset(first_part_size);
    input.set_part_size(file_size as i64 - first_part_size);
    let o = client.upload_part_from_file(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("upload part etag2 done, {}", o.etag());
    etags.push(o.etag().to_string());

    let o = client.complete_multipart_upload(
        &CompleteMultipartUploadInput::new_with_parts(bucket, key2, upload_id2, Vec::from([UploadedPart::new(1, etags[0].as_str()), UploadedPart::new(2, etags[1].as_str())]))).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.hash_crc64ecma() > 0);
    println!("complete parts done, {}", o.hash_crc64ecma());

    let mut o = client.get_object(&GetObjectInput::new(bucket, key2)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let target_md5 = base64_md5(read_to_buf(o.content().unwrap()).await);
    assert_eq!(source_md5, target_md5);

    fs::remove_file(big_file_path).unwrap();

    let key3 = gen_random_string(10);
    let key3 = key3.as_str();
    let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket, key3)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.upload_id().len() > 0);
    let upload_id3 = o.upload_id();
    let mut input = UploadPartCopyInput::new_with_part_number(bucket, key3, bucket, key, upload_id3, 1);
    input.set_copy_source_range_start(0);
    input.set_copy_source_range_end(first_part_size - 1);
    let o = client.upload_part_copy(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("upload part copy 1 done, {}", o.etag());

    let mut input = UploadPartCopyInput::new_with_part_number(bucket, key3, bucket, key, upload_id3, 2);
    input.set_copy_source_range_start(first_part_size);
    input.set_copy_source_range_end(file_size as i64 - 1);
    let o = client.upload_part_copy(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("upload part copy 2 done, {}", o.etag());

    let o = client.complete_multipart_upload(
        &CompleteMultipartUploadInput::new_with_complete_all(bucket, key3, upload_id3, true)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.hash_crc64ecma() > 0);
    println!("complete parts done, {}", o.hash_crc64ecma());
    for part in o.completed_parts().as_ref().unwrap() {
        println!("{}-{}", part.part_number(), part.etag());
    }

    let mut o = client.get_object(&GetObjectInput::new(bucket, key3)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let target_md5 = base64_md5(read_to_buf(o.content().unwrap()).await);
    assert_eq!(source_md5, target_md5);
}

async fn test_multipart_list(context: &AsyncContext) {
    let client = context.client();
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
    let mut upload_ids = BTreeMap::new();
    for (idx, key) in keys_array.iter().enumerate() {
        keys.push(prefix.clone() + key);
        let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket1, keys[idx].as_str())).await.unwrap();
        assert!(o.request_id().len() > 0);
        assert!(o.upload_id().len() > 0);
        upload_ids.insert(keys[idx].clone(), o.upload_id().to_string());
    }

    let mut keys_from_server = Vec::with_capacity(keys_array.len());
    let mut upload_ids_from_server = BTreeMap::new();
    let mut input = ListMultipartUploadsInput::new(bucket1);
    input.set_prefix(prefix.clone());
    input.set_max_uploads(10);

    loop {
        let o = client.list_multipart_uploads(&input).await.unwrap();
        for upload in o.uploads() {
            keys_from_server.push(upload.key().to_string());
            upload_ids_from_server.insert(upload.key().to_string(), upload.upload_id().to_string());
        }

        if !o.is_truncated() {
            break;
        }
        input.set_key_marker(o.next_key_marker());
        input.set_upload_id_marker(o.next_upload_id_marker());
    }


    keys.sort();
    keys_from_server.sort();
    assert_eq!(keys, keys_from_server);
    assert_eq!(upload_ids, upload_ids_from_server);

    for i in 0..50 {
        let o = client.abort_multipart_upload(&AbortMultipartUploadInput::new(bucket1, keys[i].as_str(), upload_ids.get(keys[i].as_str()).unwrap())).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    keys = keys.as_slice()[50..].to_vec();
    let mut keys_from_server = Vec::with_capacity(50);
    let mut input = ListMultipartUploadsInput::new(bucket1);
    input.set_prefix(prefix);
    input.set_max_uploads(10);

    loop {
        let o = client.list_multipart_uploads(&input).await.unwrap();
        for upload in o.uploads() {
            keys_from_server.push(upload.key().to_string());
        }

        if !o.is_truncated() {
            break;
        }
        input.set_key_marker(o.next_key_marker());
        input.set_upload_id_marker(o.next_upload_id_marker());
    }
    keys_from_server.sort();
    assert_eq!(keys, keys_from_server);

    let mut keys = Vec::with_capacity(100);
    let prefix2 = gen_random_string(5) + "/";
    for i in 1..3 {
        let key = prefix2.clone() + "abc/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket1, key)).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "abc/123/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket1, key)).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "bcd/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket1, key)).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "bcd/456/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket1, key)).await.unwrap();
        assert!(o.request_id().len() > 0);
    }

    for i in 1..3 {
        let key = prefix2.clone() + "cde/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket1, key)).await.unwrap();
        assert!(o.request_id().len() > 0);
    }
    for i in 1..3 {
        let key = prefix2.clone() + "cde/789/" + keys_array[i].as_str();
        keys.push(key.clone());
        let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket1, key)).await.unwrap();
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

    let e = client.create_multipart_upload(&CreateMultipartUploadInput::new(context.non_exists_bucket(), gen_random_string(3))).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.list_multipart_uploads(&ListMultipartUploadsInput::new(context.non_exists_bucket())).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.upload_part_from_buffer(&UploadPartFromBufferInput::new(context.non_exists_bucket(), gen_random_string(3), "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let e = client.abort_multipart_upload(&AbortMultipartUploadInput::new(context.non_exists_bucket(), gen_random_string(3), "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchBucket");

    let bucket = context.fixed_bucket();
    let key = gen_random_string(10);
    let key = key.as_str();
    let o = client.create_multipart_upload(&CreateMultipartUploadInput::new(bucket, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.upload_id().len() > 0);
    let upload_id = o.upload_id();

    let e = client.upload_part_from_buffer(&UploadPartFromBufferInput::new(bucket, gen_random_string(400), upload_id)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");

    let e = client.upload_part_from_buffer(&UploadPartFromBufferInput::new(bucket, key, "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");

    let e = client.list_parts(&ListPartsInput::new(bucket, gen_random_string(400), upload_id)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");

    let e = client.list_parts(&ListPartsInput::new(bucket, key, "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");

    let e = client.abort_multipart_upload(&AbortMultipartUploadInput::new(bucket, gen_random_string(400), upload_id)).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");

    let e = client.abort_multipart_upload(&AbortMultipartUploadInput::new(bucket, key, "123")).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");

    let e = client.complete_multipart_upload(&CompleteMultipartUploadInput::new_with_parts(bucket, gen_random_string(400), upload_id, Vec::from([UploadedPart::new(1, "123")]))).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");

    let e = client.complete_multipart_upload(&CompleteMultipartUploadInput::new_with_parts(bucket, key, "123", Vec::from([UploadedPart::new(1, "123")]))).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    assert_eq!(ex.code(), "NoSuchUpload");
}

async fn list_by_prefix(context: &AsyncContext, bucket: &str, prefix: String, keys_from_server: &mut Vec<String>, common_prefixes: &mut Vec<String>) {
    let client = context.client();
    let mut input = ListMultipartUploadsInput::new(bucket);
    input.set_max_uploads(1000);
    input.set_delimiter("/");
    input.set_prefix(prefix);
    loop {
        let o = client.list_multipart_uploads(&input).await.unwrap();
        for upload in o.uploads() {
            keys_from_server.push(upload.key().to_string());
        }

        for common_prefix in o.common_prefixes() {
            common_prefixes.push(common_prefix.prefix().to_string());
        }

        if !o.is_truncated() {
            break;
        }

        input.set_key_marker(o.next_key_marker());
        input.set_upload_id_marker(o.next_upload_id_marker());
    }
}