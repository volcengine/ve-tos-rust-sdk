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
use crate::common::asynchronous::{create_async_context, read_to_buf, AsyncContext};
use crate::common::gen_random_string;
use chrono::{DateTime, Duration, Utc};
use scopeguard::defer;
use std::ops::Add;
use tokio::runtime;
use ve_tos_rust_sdk::asynchronous::bucket::BucketAPI;
use ve_tos_rust_sdk::asynchronous::object::{ObjectAPI, ObjectContent};
use ve_tos_rust_sdk::asynchronous::tos::{new_stream, BufferStream};
use ve_tos_rust_sdk::bucket::{AbortInCompleteMultipartUpload, AccessControlTranslation, AccessLogConfiguration, AccessTimeTransition, ApplyServerSideEncryptionByDefault, BucketEncryptionRule, BucketTrash, BucketTrashPrefixRule, CORSRule, CommonCredentialProvider, CommonSourceEndpoint, CommonStaticCredential, Condition, CreateBucketInput, CustomDomainRule, DeleteBucketCORSInput, DeleteBucketCustomDomainInput, DeleteBucketEncryptionInput, DeleteBucketInput, DeleteBucketInventoryInput, DeleteBucketLifecycleInput, DeleteBucketMirrorBackInput, DeleteBucketPolicyInput, DeleteBucketRealTimeLogInput, DeleteBucketReplicationInput, DeleteBucketTaggingInput, DeleteBucketWebsiteInput, Destination, DestinationHttpServer, DestinationVeFaaS, DoesBucketExistInput, EndpointCredentialProvider, ErrorDocument, Expiration, GetBucketACLInput, GetBucketAccessMonitorInput, GetBucketCORSInput, GetBucketEncryptionInput, GetBucketInfoInput, GetBucketInventoryInput, GetBucketLifecycleInput, GetBucketLocationInput, GetBucketMirrorBackInput, GetBucketNotificationType2Input, GetBucketPolicyInput, GetBucketRealTimeLogInput, GetBucketReplicationInput, GetBucketTaggingInput, GetBucketTrashInput, GetBucketVersioningInput, GetBucketWebsiteInput, HeadBucketInput, IndexDocument, InventoryDestination, InventoryFilter, InventoryOptionalFields, InventorySchedule, LifecycleRule, LifecycleRuleFilter, ListBucketCustomDomainInput, ListBucketInventoryInput, ListBucketsInput, MirrorBackRule, NonCurrentVersionAccessTimeTransition, NoncurrentVersionExpiration, NoncurrentVersionTransition, NotificationDestination, NotificationRule, PrivateSource, PublicSource, PutBucketACLInput, PutBucketAccessMonitorInput, PutBucketCORSInput, PutBucketCustomDomainInput, PutBucketEncryptionInput, PutBucketInventoryInput, PutBucketLifecycleInput, PutBucketMirrorBackInput, PutBucketNotificationType2Input, PutBucketPolicyInput, PutBucketRealTimeLogInput, PutBucketReplicationInput, PutBucketStorageClassInput, PutBucketTaggingInput, PutBucketTrashInput, PutBucketVersioningInput, PutBucketWebsiteInput, RealTimeLogConfiguration, Redirect, RedirectAllRequestsTo, ReplicationRule, RoutingRule, RoutingRuleCondition, RoutingRuleRedirect, SourceEndpoint, TOSBucketDestination, Transition};
use ve_tos_rust_sdk::common::{Owner, RequestInfoTrait, Tag, TagSet};
use ve_tos_rust_sdk::enumeration::{ACLType, AuthProtocolType, AzRedundancyType, BucketType, InventoryFormatType, InventoryFrequencyType, InventoryIncludedObjType, ProtocolType, RedirectType, StatusType, StorageClassInheritDirectiveType, StorageClassType, VersioningStatusType};
use ve_tos_rust_sdk::object::{AppendObjectFromBufferInput, AppendObjectInput, DeleteObjectInput, GetFileStatusInput, GetObjectInput, HeadObjectInput, ListObjectsType2Input, PutObjectFromBufferInput, SetObjectTimeInput};

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
        test_basic(&context).await;
        test_bucket_storage_class_and_cors(&context).await;
        test_bucket_lifecycle(&context).await;
        test_bucket_policy(&context).await;
        test_bucket_mirror_back(&context).await;
        test_bucket_acl(&context).await;
        test_bucket_replication(&context).await;
        test_bucket_versioning(&context).await;
        test_bucket_website(&context).await;
        test_bucket_custom_domain(&context).await;
        test_bucket_real_time_log(&context).await;
        test_bucket_encryption(&context).await;
        test_bucket_tagging(&context).await;
        test_bucket_notification(&context).await;
        test_bucket_inventory(&context).await;
        test_hns_bucket(&context).await;
    })
}

async fn test_hns_bucket(context: &AsyncContext) {
    let client = context.client();
    let bucket1 = gen_random_string(10);
    let bucket1 = bucket1.as_str();
    let o = client.create_bucket(&CreateBucketInput::new(bucket1)).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket1).await;

    let o = client.head_bucket(&HeadBucketInput::new(bucket1)).await.unwrap();
    if let Some(bt) = o.bucket_type().as_ref() {
        assert_eq!(bt, &BucketType::BucketTypeFns);
    }

    let bucket2 = gen_random_string(40);
    let bucket2 = bucket2.as_str();
    let mut input = CreateBucketInput::new(bucket2);
    input.set_bucket_type(BucketType::BucketTypeHns);
    let o = client.create_bucket(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket2).await;

    let o = client.head_bucket(&HeadBucketInput::new(bucket2)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.bucket_type().as_ref().unwrap(), &BucketType::BucketTypeHns);
    let mut input = ListBucketsInput::new();
    input.set_bucket_type(BucketType::BucketTypeFns);
    let o = client.list_buckets(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    let mut flag = false;
    for bucket in o.buckets() {
        assert_eq!(bucket.bucket_type(), &BucketType::BucketTypeFns);
        if bucket.name() == bucket1 {
            flag = true;
        }
    }
    assert!(flag);


    let mut input = ListBucketsInput::new();
    input.set_bucket_type(BucketType::BucketTypeHns);
    let o = client.list_buckets(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    let mut flag = false;
    for bucket in o.buckets() {
        assert_eq!(bucket.bucket_type(), &BucketType::BucketTypeHns);
        if bucket.name() == bucket2 {
            flag = true;
        }
    }
    assert!(flag);

    let folder = "folder/";
    let key = format!("folder/{}", gen_random_string(10));
    let key = key.as_str();
    let data = "helloworld";

    // fns
    let input = PutObjectFromBufferInput::new_with_content(bucket1, key, data);
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.head_object(&HeadObjectInput::new(bucket1, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(!o.is_directory());

    let o = client.get_file_status(&GetFileStatusInput::new(bucket1, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.size(), data.len() as i64);
    assert_eq!(o.key(), key);

    let o = client.get_file_status(&GetFileStatusInput::new(bucket1, folder)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.key(), key);

    // hns
    let input = PutObjectFromBufferInput::new_with_content(bucket2, key, data);
    let o = client.put_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_file_status(&GetFileStatusInput::new(bucket2, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.size(), data.len() as i64);
    assert_eq!(o.key(), key);

    let o = client.get_file_status(&GetFileStatusInput::new(bucket2, folder)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.key(), folder);

    let o = client.head_object(&HeadObjectInput::new(bucket2, folder)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.is_directory());
    let o = client.head_object(&HeadObjectInput::new(bucket2, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(!o.is_directory());

    let mut o = client.get_object(&GetObjectInput::new(bucket2, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(!o.is_directory());
    let buf = read_to_buf(o.content().unwrap()).await;
    assert_eq!(String::from_utf8(buf).unwrap(), data);

    println!("{:?}", o.last_modified());
    println!("{:?}", o.last_modified_timestamp());
    let timestamp = Utc::now().add(Duration::seconds(120));
    match client.set_object_time(&SetObjectTimeInput::new(bucket2, key, timestamp)).await {
        Ok(o) => {
            assert!(o.request_id().len() > 0);
            let o = client.head_object(&HeadObjectInput::new(bucket2, key)).await.unwrap();
            assert!(o.request_id().len() > 0);
            println!("{:?}", o.last_modified());
        }
        Err(ex) => {
            assert!(ex.is_server_error());
        }
    }

    let o = client.delete_object(&DeleteObjectInput::new(bucket1, key)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.delete_object(&DeleteObjectInput::new(bucket2, key)).await.unwrap();
    assert!(o.request_id().len() > 0);

    // test append fns
    let mut input = AppendObjectFromBufferInput::new(bucket1, key);
    input.set_content_length(data.len() as i64);
    input.set_content(data);
    let o = client.append_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.next_append_offset(), data.len() as i64);

    let mut o = client.get_object(&GetObjectInput::new(bucket1, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_buf(o.content().unwrap()).await;
    assert_eq!(String::from_utf8(buf).unwrap(), data);

    let stream_data = new_stream(data);
    let mut input = AppendObjectInput::<BufferStream>::new_with_content(bucket1, key, stream_data);
    input.set_content_length(data.len() as i64);
    input.set_offset(o.content_length());
    input.set_pre_hash_crc64ecma(o.hash_crc64ecma());

    let o = client.append_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.next_append_offset(), (data.len() * 2) as i64);

    let mut o = client.get_object(&GetObjectInput::new(bucket1, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_buf(o.content().unwrap()).await;
    assert_eq!(String::from_utf8(buf).unwrap(), String::from(data) + data);

    // test append hns
    let mut input = AppendObjectFromBufferInput::new(bucket2, key);
    input.set_content_length(data.len() as i64);
    input.set_content(data);
    let o = client.append_object_from_buffer(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.next_append_offset(), data.len() as i64);

    let mut o = client.get_object(&GetObjectInput::new(bucket2, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_buf(o.content().unwrap()).await;
    assert_eq!(String::from_utf8(buf).unwrap(), data);

    let stream_data = new_stream(data);
    let mut input = AppendObjectInput::<BufferStream>::new_with_content(bucket2, key, stream_data);
    input.set_content_length(data.len() as i64);
    input.set_offset(o.content_length());
    input.set_pre_hash_crc64ecma(o.hash_crc64ecma());

    let o = client.append_object(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.next_append_offset(), (data.len() * 2) as i64);

    let mut o = client.get_object(&GetObjectInput::new(bucket2, key)).await.unwrap();
    assert!(o.request_id().len() > 0);
    let buf = read_to_buf(o.content().unwrap()).await;
    assert_eq!(String::from_utf8(buf).unwrap(), String::from(data) + data);

    let mut input = ListObjectsType2Input::new(bucket2);
    input.set_delimiter("/");
    let o = client.list_objects_type2(&input).await.unwrap();
    for common_prefix in o.common_prefixes() {
        println!("{:?}", common_prefix.last_modified());
    }

    // clean
    let o = client.delete_object(&DeleteObjectInput::new(bucket1, key)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.delete_object(&DeleteObjectInput::new(bucket2, key)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.delete_object(&DeleteObjectInput::new(bucket2, folder)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut trash = BucketTrash::new("test1", 1, StatusType::StatusEnabled);
    let mut prefix_match_rules = Vec::with_capacity(3);
    prefix_match_rules.push(BucketTrashPrefixRule::new("test2", 1,
                                                       vec!["prefix1".to_string(), "prefix2".to_string()]));

    prefix_match_rules.push(BucketTrashPrefixRule::new("test3", 2,
                                                       vec!["prefix3".to_string(), "prefix4".to_string()]));
    trash.set_prefix_match_rules(prefix_match_rules);
    let o = client.put_bucket_trash(&PutBucketTrashInput::new(bucket2, trash)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_trash(&GetBucketTrashInput::new(bucket2)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.trash().status(), &StatusType::StatusEnabled);
    assert_eq!(o.trash().clean_interval(), 1);
    assert_eq!(o.trash().trash_path(), "test1");
    assert_eq!(o.trash().prefix_match_rules().len(), 2);
    for rule in o.trash().prefix_match_rules() {
        assert!(rule.trash_path() == "test2" || rule.trash_path() == "test3");
        if rule.trash_path() == "test2" {
            assert_eq!(rule.clean_interval(), 1);
            assert_eq!(rule.prefix_list(), &vec!["prefix1".to_string(), "prefix2".to_string()]);
        } else {
            assert_eq!(rule.clean_interval(), 2);
            assert_eq!(rule.prefix_list(), &vec!["prefix3".to_string(), "prefix4".to_string()]);
        }
    }
}

async fn test_bucket_inventory(context: &AsyncContext) {
    let client = context.client();
    let bucket1 = gen_random_string(10);
    let bucket1 = bucket1.as_str();
    let o = client.create_bucket(&CreateBucketInput::new(bucket1)).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket1).await;

    let o = client.get_bucket_acl(&GetBucketACLInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    let account_id = o.owner().id();

    let mut input = PutBucketInventoryInput::new(context.fixed_bucket(), "id1");
    input.set_is_enabled(true);
    input.set_is_un_compressed(true);
    input.set_schedule(InventorySchedule::new(InventoryFrequencyType::InventoryFrequencyWeekly));
    let mut dst = TOSBucketDestination::new(InventoryFormatType::InventoryFormatCsv);
    dst.set_bucket(bucket1);
    dst.set_prefix("prefix1");
    dst.set_role("TosArchiveTOSInventory");
    dst.set_account_id(account_id);
    let dst = InventoryDestination::new_with_tos_bucket_destination(dst);
    input.set_destination(dst);
    input.set_included_object_versions(InventoryIncludedObjType::InventoryIncludedObjAll);
    input.set_filter(InventoryFilter::new("prefix2"));
    input.set_optional_fields(InventoryOptionalFields::new(vec!["Size".to_string(), "ETag".to_string()]));
    let o = client.put_bucket_inventory(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let o = client.get_bucket_inventory(&GetBucketInventoryInput::new(context.fixed_bucket(), "id1")).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.id(), "id1");
    assert_eq!(o.is_enabled(), true);
    assert_eq!(o.is_un_compressed(), true);
    assert_eq!(o.schedule().as_ref().unwrap().frequency(), &InventoryFrequencyType::InventoryFrequencyWeekly);
    assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().bucket(), bucket1);
    assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().prefix(), "prefix1");
    assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().role(), "TosArchiveTOSInventory");
    assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().account_id(), account_id);
    assert_eq!(o.included_object_versions(), &InventoryIncludedObjType::InventoryIncludedObjAll);
    assert_eq!(o.filter().as_ref().unwrap().prefix(), "prefix2");
    assert_eq!(o.optional_fields().as_ref().unwrap().field().len(), 2);
    assert_eq!(o.optional_fields().as_ref().unwrap().field().get(0).unwrap(), "Size");
    assert_eq!(o.optional_fields().as_ref().unwrap().field().get(1).unwrap(), "ETag");

    let mut input = PutBucketInventoryInput::new(context.fixed_bucket(), "id2");
    input.set_is_enabled(true);
    input.set_schedule(InventorySchedule::new(InventoryFrequencyType::InventoryFrequencyDaily));
    let mut dst = TOSBucketDestination::new(InventoryFormatType::InventoryFormatCsv);
    dst.set_bucket(bucket1);
    dst.set_prefix("prefix3");
    dst.set_role("TosArchiveTOSInventory");
    dst.set_account_id(account_id);
    let dst = InventoryDestination::new_with_tos_bucket_destination(dst);
    input.set_destination(dst);
    input.set_included_object_versions(InventoryIncludedObjType::InventoryIncludedObjCurrent);
    input.set_filter(InventoryFilter::new("prefix4"));
    let o = client.put_bucket_inventory(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut input = ListBucketInventoryInput::new(context.fixed_bucket());
    input.set_continuation_token("");
    let o = client.list_bucket_inventory(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.configurations().len(), 2);
    assert_eq!(o.is_truncated(), false);
    assert_eq!(o.next_continuation_token(), "");

    for o in o.configurations() {
        assert!(o.id() == "id1" || o.id() == "id2");
        if o.id() == "id1" {
            assert_eq!(o.id(), "id1");
            assert_eq!(o.is_enabled(), true);
            assert_eq!(o.is_un_compressed(), true);
            assert_eq!(o.schedule().as_ref().unwrap().frequency(), &InventoryFrequencyType::InventoryFrequencyWeekly);
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().bucket(), bucket1);
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().prefix(), "prefix1");
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().role(), "TosArchiveTOSInventory");
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().account_id(), account_id);
            assert_eq!(o.included_object_versions(), &InventoryIncludedObjType::InventoryIncludedObjAll);
            assert_eq!(o.filter().as_ref().unwrap().prefix(), "prefix2");
            assert_eq!(o.optional_fields().as_ref().unwrap().field().len(), 2);
            assert_eq!(o.optional_fields().as_ref().unwrap().field().get(0).unwrap(), "Size");
            assert_eq!(o.optional_fields().as_ref().unwrap().field().get(1).unwrap(), "ETag");
        } else {
            assert_eq!(o.id(), "id2");
            assert_eq!(o.is_enabled(), true);
            assert_eq!(o.is_un_compressed(), false);
            assert_eq!(o.schedule().as_ref().unwrap().frequency(), &InventoryFrequencyType::InventoryFrequencyDaily);
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().bucket(), bucket1);
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().prefix(), "prefix3");
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().role(), "TosArchiveTOSInventory");
            assert_eq!(o.destination().as_ref().unwrap().tos_bucket_destination().as_ref().unwrap().account_id(), account_id);
            assert_eq!(o.included_object_versions(), &InventoryIncludedObjType::InventoryIncludedObjCurrent);
            assert_eq!(o.filter().as_ref().unwrap().prefix(), "prefix4");
            assert!(o.optional_fields().is_none());
        }
    }

    let o = client.delete_bucket_inventory(&DeleteBucketInventoryInput::new(context.fixed_bucket(), "id1")).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.delete_bucket_inventory(&DeleteBucketInventoryInput::new(context.fixed_bucket(), "id2")).await.unwrap();
    assert!(o.request_id().len() > 0);

    match client.list_bucket_inventory(&input).await {
        Ok(o) => {
            assert!(o.request_id().len() > 0);
            assert_eq!(o.configurations().len(), 0);
        }
        Err(e) => {
            assert!(e.is_server_error());
            let ex = e.as_server_error().unwrap();
            assert_eq!(ex.status_code(), 404);
        }
    }
}

async fn test_bucket_notification(context: &AsyncContext) {
    let client = context.client();
    let o = client.get_bucket_notification_type2(&GetBucketNotificationType2Input::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rules().len(), 0);

    let mut rules = Vec::with_capacity(2);
    let mut rule = NotificationRule::new("rule1");
    let mut dst = NotificationDestination::new();
    dst.set_http_server(vec![DestinationHttpServer::new("www.baidu.com")]);
    rule.set_destination(dst);
    // rules.push(rule);

    let mut rule = NotificationRule::new("rule2");
    let mut dst = NotificationDestination::new();
    dst.set_ve_faas(vec![DestinationVeFaaS::new("function1")]);
    rule.set_destination(dst);

    rule.set_events(vec!["tos:ObjectCreated:*".to_string()]);
    rules.push(rule);
    let mut input = PutBucketNotificationType2Input::new(context.fixed_bucket(), rules);
    input.set_version("");
    match client.put_bucket_notification_type2(&input).await {
        Ok(o) => {
            assert!(o.request_id().len() > 0);

            let o = client.get_bucket_notification_type2(&GetBucketNotificationType2Input::new(context.fixed_bucket())).await.unwrap();
            assert!(o.request_id().len() > 0);
            assert_eq!(o.rules().len(), 1);
            assert_eq!(o.version(), "");
        }
        Err(ex) => {
            println!("{}", ex);
        }
    }
}

async fn test_bucket_encryption(context: &AsyncContext) {
    let client = context.https_client();
    let apply = ApplyServerSideEncryptionByDefault::new("AES256", "");
    let input = PutBucketEncryptionInput::new(context.fixed_bucket(), BucketEncryptionRule::new(apply));
    let o = client.put_bucket_encryption(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_encryption(&GetBucketEncryptionInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rule().apply_server_side_encryption_by_default().sse_algorithm(), "AES256");

    let o = client.delete_bucket_encryption(&DeleteBucketEncryptionInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_encryption(&GetBucketEncryptionInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
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

async fn test_bucket_versioning(context: &AsyncContext) {
    let client = context.client();
    let o = client.get_bucket_versioning(&GetBucketVersioningInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.status(), &VersioningStatusType::VersioningStatusNotSet);

    let o = client.put_bucket_versioning(&PutBucketVersioningInput::new_with_status(context.fixed_bucket(),
                                                                                    VersioningStatusType::VersioningStatusEnabled)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_versioning(&GetBucketVersioningInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.status(), &VersioningStatusType::VersioningStatusEnabled);
}

async fn test_bucket_real_time_log(context: &AsyncContext) {
    let client = context.client();
    let mut conf = AccessLogConfiguration::new();
    conf.set_use_service_topic(true);
    let config = RealTimeLogConfiguration::new("TOSLogArchiveTLSRole", conf);
    let input = PutBucketRealTimeLogInput::new(context.fixed_bucket(), config);
    let o = client.put_bucket_real_time_log(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_real_time_log(&GetBucketRealTimeLogInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.configuration().role(), "TOSLogArchiveTLSRole");
    assert_eq!(o.configuration().configuration().use_service_topic(), true);
    assert!(o.configuration().configuration().tls_project_id().len() > 0);
    assert!(o.configuration().configuration().tls_dashboard_id().len() > 0);
    assert!(o.configuration().configuration().tls_topic_id().len() > 0);
    assert_eq!(o.configuration().configuration().ttl(), 7);

    let o = client.delete_bucket_real_time_log(&DeleteBucketRealTimeLogInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_real_time_log(&GetBucketRealTimeLogInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}

async fn test_bucket_website(context: &AsyncContext) {
    let client = context.client();
    let mut input = PutBucketWebsiteInput::new(context.fixed_bucket());
    input.set_redirect_all_requests_to(RedirectAllRequestsTo::new("www.baidu.com", ProtocolType::ProtocolHttps));
    let o = client.put_bucket_website(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_website(&GetBucketWebsiteInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.redirect_all_requests_to().as_ref().unwrap().host_name(), "www.baidu.com");
    assert_eq!(o.redirect_all_requests_to().as_ref().unwrap().protocol().as_ref().unwrap(), &ProtocolType::ProtocolHttps);

    let mut input = PutBucketWebsiteInput::new(context.fixed_bucket());
    let mut index = IndexDocument::new("suffix1");
    index.set_forbidden_sub_dir(true);
    input.set_index_document(index);
    let doc = ErrorDocument::new("404.html");
    input.set_error_document(doc);
    let mut rules = Vec::with_capacity(2);
    let mut condition = RoutingRuleCondition::new("prefix");
    condition.set_http_error_code_returned_equals(404);
    let mut redirect = RoutingRuleRedirect::new();
    redirect.set_protocol(ProtocolType::ProtocolHttp);
    redirect.set_host_name("www.baidu.com");
    redirect.set_replace_key_with("key");
    rules.push(RoutingRule::new(condition, redirect));
    input.set_routing_rules(rules);
    let o = client.put_bucket_website(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let o = client.get_bucket_website(&GetBucketWebsiteInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.index_document().as_ref().unwrap().suffix(), "suffix1");
    assert_eq!(o.index_document().as_ref().unwrap().forbidden_sub_dir(), true);
    assert_eq!(o.error_document().as_ref().unwrap().key(), "404.html");
    assert_eq!(o.routing_rules().len(), 1);
    assert_eq!(o.routing_rules().get(0).as_ref().unwrap().condition().key_prefix_equals(), "prefix");
    assert_eq!(o.routing_rules().get(0).as_ref().unwrap().condition().http_error_code_returned_equals(), 404);
    assert_eq!(o.routing_rules().get(0).as_ref().unwrap().redirect().host_name(), "www.baidu.com");
    assert_eq!(o.routing_rules().get(0).as_ref().unwrap().redirect().protocol().as_ref().unwrap(), &ProtocolType::ProtocolHttp);
    assert_eq!(o.routing_rules().get(0).as_ref().unwrap().redirect().replace_key_with(), "key");
    assert_eq!(o.routing_rules().get(0).as_ref().unwrap().redirect().http_redirect_code(), 0);

    let o = client.delete_bucket_website(&DeleteBucketWebsiteInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_website(&GetBucketWebsiteInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}
async fn test_bucket_custom_domain(context: &AsyncContext) {
    let client = context.client();
    let domain = gen_random_string(8);
    let rule = CustomDomainRule::new(format!("www.{}.com", domain), AuthProtocolType::AuthProtocolTos);
    let input = PutBucketCustomDomainInput::new(context.fixed_bucket(), rule);
    let o = client.put_bucket_custom_domain(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let domain2 = gen_random_string(8);
    let rule = CustomDomainRule::new(format!("www.{}.com", domain2), AuthProtocolType::AuthProtocolS3);
    let input = PutBucketCustomDomainInput::new(context.fixed_bucket(), rule);
    let o = client.put_bucket_custom_domain(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.list_bucket_custom_domain(&ListBucketCustomDomainInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rules().len(), 2);

    for rule in o.rules() {
        assert!(rule.domain() == format!("www.{}.com", domain) || rule.domain() == format!("www.{}.com", domain2));
        if rule.domain() == format!("www.{}.com", domain) {
            assert_eq!(rule.protocol(), &AuthProtocolType::AuthProtocolTos);
        } else {
            assert_eq!(rule.protocol(), &AuthProtocolType::AuthProtocolS3);
        }
    }

    let o = client.delete_bucket_custom_domain(&DeleteBucketCustomDomainInput::new(context.fixed_bucket(), format!("www.{}.com", domain))).await.unwrap();
    assert!(o.request_id().len() > 0);
    let o = client.delete_bucket_custom_domain(&DeleteBucketCustomDomainInput::new(context.fixed_bucket(), format!("www.{}.com", domain2))).await.unwrap();
    assert!(o.request_id().len() > 0);
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let e = client.list_bucket_custom_domain(&ListBucketCustomDomainInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}

async fn test_bucket_replication(context: &AsyncContext) {
    let client = context.client();
    let bucket1 = gen_random_string(10);
    let bucket1 = bucket1.as_str();
    let o = client.create_bucket(&CreateBucketInput::new(bucket1)).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket1).await;

    let location = client.get_bucket_location(&GetBucketLocationInput::new(bucket1)).await.unwrap();
    println!("{}", location.region());

    let mut input = PutBucketReplicationInput::new(context.fixed_bucket(), "ServiceRoleforReplicationAccessTOS");
    let mut rule = ReplicationRule::new("rule1");
    rule.set_prefix_set(vec!["prefix1".to_string(), "prefix2".to_string()]);
    rule.set_tags(vec![Tag::new("tag1", "value1"), Tag::new("tag2", "value2")]);
    let mut dest = Destination::new(bucket1, location.region());
    dest.set_storage_class(StorageClassType::StorageClassIa);
    dest.set_storage_class_inherit_directive(StorageClassInheritDirectiveType::StorageClassIDSourceObject);
    rule.set_destination(dest);
    rule.set_access_control_translation(AccessControlTranslation::new("BucketOwnerEntrusted"));
    input.add_rule(rule);
    let o = client.put_bucket_replication(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_replication(&GetBucketReplicationInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rules().len(), 1);
    for rule in o.rules() {
        assert!(rule.id() == "rule1" || rule.id() == "rule2");
        if rule.id() == "rule1" {
            assert_eq!(rule.status(), &StatusType::StatusEnabled);
            assert_eq!(rule.historical_object_replication(), &StatusType::StatusDisabled);
            assert_eq!(rule.destination().bucket(), bucket1);
            assert_eq!(rule.destination().location(), location.region());
            assert_eq!(rule.destination().storage_class().as_ref().unwrap(), &StorageClassType::StorageClassIa);
            assert_eq!(rule.destination().storage_class_inherit_directive().as_ref().unwrap(), &StorageClassInheritDirectiveType::StorageClassIDSourceObject);
            assert_eq!(rule.access_control_translation().as_ref().unwrap().owner(), "BucketOwnerEntrusted");
            assert_eq!(rule.prefix_set().len(), 2);
            assert_eq!(rule.prefix_set().get(0).unwrap(), "prefix1");
            assert_eq!(rule.prefix_set().get(1).unwrap(), "prefix2");
            assert_eq!(rule.tags().len(), 2);
            assert_eq!(rule.tags().get(0).unwrap().key(), "tag1");
            assert_eq!(rule.tags().get(0).unwrap().value(), "value1");
            assert_eq!(rule.tags().get(1).unwrap().key(), "tag2");
            assert_eq!(rule.tags().get(1).unwrap().value(), "value2");
        }
    }

    let o = client.delete_bucket_replication(&DeleteBucketReplicationInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_replication(&GetBucketReplicationInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}

async fn test_bucket_mirror_back(context: &AsyncContext) {
    let client = context.https_client();
    let mut input = PutBucketMirrorBackInput::new(context.fixed_bucket());
    let mut rule = MirrorBackRule::new("rule1");
    let mut condition = Condition::new(404);
    condition.set_key_prefix("prefix1");
    condition.set_http_method(vec![String::from("GET"), "HEAD".to_string()]);
    rule.set_condition(condition);
    let mut redirect = Redirect::new(RedirectType::RedirectMirror, false);
    redirect.set_public_source(PublicSource::new(SourceEndpoint::new(vec!["http://www.baidu.com".to_string()])));
    rule.set_redirect(redirect);
    input.add_rule(rule);

    let mut rule = MirrorBackRule::new("rule2");
    let mut condition = Condition::new(404);
    condition.set_key_prefix("prefix2");
    condition.set_http_method(vec![String::from("GET")]);
    rule.set_condition(condition);
    let mut redirect = Redirect::new(RedirectType::RedirectAsync, true);
    redirect.set_follow_redirect(true);
    let mut primary = vec![];
    let mut ep = EndpointCredentialProvider::new("http://www.baidu.com");
    ep.set_bucket_name("bucket1");
    let mut cred = CommonStaticCredential::new("BOS");
    cred.set_sk("sk");
    cred.set_ak("ak");
    ep.set_credential_provider(CommonCredentialProvider::new_with_static_credential(cred, "region1"));
    primary.push(ep);
    let mut ep = EndpointCredentialProvider::new("http://www.163.com");
    ep.set_bucket_name("bucket2");
    let mut cred = CommonStaticCredential::new("S3");
    cred.set_sk("sk");
    cred.set_ak("ak");
    ep.set_credential_provider(CommonCredentialProvider::new_with_static_credential(cred, "region1"));
    primary.push(ep);
    let private_source = PrivateSource::new(CommonSourceEndpoint::new(primary));
    redirect.set_private_source(private_source);
    rule.set_redirect(redirect);
    input.add_rule(rule);
    let o = client.put_bucket_mirror_back(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_mirror_back(&GetBucketMirrorBackInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rules().len(), 2);
    for rule in o.rules() {
        assert!(rule.id() == "rule1" || rule.id() == "rule2");
        if rule.id() == "rule1" {
            assert_eq!(rule.condition().http_code(), 404);
            assert_eq!(rule.condition().key_prefix(), "prefix1");
            assert!(rule.condition().http_method().contains(&"GET".to_string()));
            assert!(rule.condition().http_method().contains(&"HEAD".to_string()));
            assert_eq!(rule.redirect().redirect_type(), &RedirectType::RedirectMirror);
            assert_eq!(rule.redirect().public_source().as_ref().unwrap().source_endpoint().primary().get(0).unwrap(), "http://www.baidu.com");
        } else {
            assert_eq!(rule.condition().http_code(), 404);
            assert_eq!(rule.condition().key_prefix(), "prefix2");
            assert!(rule.condition().http_method().contains(&"GET".to_string()));
            assert_eq!(rule.redirect().redirect_type(), &RedirectType::RedirectAsync);
            assert_eq!(rule.redirect().fetch_source_on_redirect(), true);
            let p = rule.redirect().private_source().as_ref().unwrap().source_endpoint().primary();
            assert_eq!(p.len(), 2);
            assert_eq!(p.get(0).unwrap().endpoint(), "http://www.baidu.com");
            assert_eq!(p.get(0).unwrap().credential_provider().static_credential().as_ref().unwrap().storage_vendor(), "BOS");
            assert_eq!(p.get(1).unwrap().endpoint(), "http://www.163.com");
            assert_eq!(p.get(1).unwrap().credential_provider().static_credential().as_ref().unwrap().storage_vendor(), "S3");
            for e in p.iter() {
                println!("{}", e.credential_provider().static_credential().as_ref().unwrap().ak());
                println!("{}", e.credential_provider().static_credential().as_ref().unwrap().sk());
            }
        }
    }

    let o = client.delete_bucket_mirror_back(&DeleteBucketMirrorBackInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_mirror_back(&GetBucketMirrorBackInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}

async fn test_bucket_acl(context: &AsyncContext) {
    let client = context.client();
    let o = client.get_bucket_acl(&GetBucketACLInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    assert_eq!(o.bucket_acl_delivered(), false);
    assert!(o.owner().id().len() > 0);
    assert_eq!(o.grants().len(), 1);

    let mut input = PutBucketACLInput::new(context.fixed_bucket());
    input.set_bucket_acl_delivered(true);
    input.set_owner(Owner::new(o.owner().id()));
    input.set_grants(o.grants().to_vec());
    let o = client.put_bucket_acl(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_acl(&GetBucketACLInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    assert_eq!(o.bucket_acl_delivered(), true);
    assert!(o.owner().id().len() > 0);
    for grant in o.grants() {
        println!("{:?}", grant);
    }
}

async fn test_bucket_policy(context: &AsyncContext) {
    let client = context.client();
    let mut input = PutBucketPolicyInput::new(context.fixed_bucket());
    let policy = String::from(r#"
        {
                "Statement": [{
                        "Effect": "Allow",
                        "Action": [
                            "tos:GetObject",
                            "tos:PutObject",
                            "tos:DeleteObject",
                            "tos:AbortMultipartUpload",
                            "tos:ListBucket",
                            "tos:HeadBucket"
                        ],
                        "Principal":["*"],
                        "Resource": [
                             "trn:tos:::"#) + context.fixed_bucket() + r#"",
                              "trn:tos:::"# + context.fixed_bucket() + r#"/*"
                        ]
                }]
        }
     "#;

    println!("{}", policy);

    input.set_policy(policy);
    let o = client.put_bucket_policy(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_policy(&GetBucketPolicyInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    println!("{}", o.policy());

    let o = client.delete_bucket_policy(&DeleteBucketPolicyInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_policy(&GetBucketPolicyInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}

async fn test_bucket_lifecycle(context: &AsyncContext) {
    let client = context.client();
    let o = client.put_bucket_access_monitor(&PutBucketAccessMonitorInput::new(context.fixed_bucket(), StatusType::StatusEnabled)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_access_monitor(&GetBucketAccessMonitorInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.status(), &StatusType::StatusEnabled);

    let mut input = PutBucketLifecycleInput::new(context.fixed_bucket());
    input.set_allow_same_action_overlap(true);
    let mut rule = LifecycleRule::new_with_prefix("rule1", "prefix1");
    let exp = Expiration::new_with_days(1isize);
    rule.set_expiration(exp);
    input.add_rule(rule);

    let mut rule = LifecycleRule::new_with_prefix("rule2", "prefix");
    let exp = Expiration::new_with_date(Utc::now());
    rule.set_expiration(exp);
    let mut filter = LifecycleRuleFilter::new();
    filter.set_object_size_greater_than(1000);
    filter.set_greater_than_include_equal(StatusType::StatusEnabled);
    rule.set_filter(filter);
    let mut transitions = Vec::new();
    let mut transition = Transition::new(StorageClassType::StorageClassIa);
    transition.set_days(1isize);
    transitions.push(transition);
    let mut transition = Transition::new(StorageClassType::StorageClassArchiveFr);
    transition.set_days(10isize);
    transitions.push(transition);
    rule.set_transitions(transitions);
    let mut tags = Vec::new();
    tags.push(Tag::new("key1", "value1"));
    tags.push(Tag::new("key2", "value2"));
    rule.set_tags(tags);
    let exp = NoncurrentVersionExpiration::new_with_date(Utc::now());
    rule.set_noncurrent_version_expiration(exp);
    let mut transitions = Vec::new();
    let mut transition = NoncurrentVersionTransition::new(StorageClassType::StorageClassIa);
    transition.set_noncurrent_date(Utc::now());
    transitions.push(transition);
    let mut transition = NoncurrentVersionTransition::new(StorageClassType::StorageClassArchiveFr);
    transition.set_noncurrent_date(Utc::now().add(Duration::days(10)));
    transitions.push(transition);
    rule.set_noncurrent_version_transitions(transitions);
    input.add_rule(rule);

    let mut rule = LifecycleRule::new_with_prefix("rule3", "prefix2");
    rule.set_abort_in_complete_multipart_upload(AbortInCompleteMultipartUpload::new(1));
    let mut transitions = Vec::new();
    let transition = AccessTimeTransition::new(1, StorageClassType::StorageClassIa);
    transitions.push(transition);
    rule.set_access_time_transitions(transitions);
    let mut transitions = Vec::new();
    let transition = NonCurrentVersionAccessTimeTransition::new(1, StorageClassType::StorageClassIa);
    transitions.push(transition);
    rule.set_non_current_version_access_time_transitions(transitions);
    input.add_rule(rule);

    let o = client.put_bucket_lifecycle(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_bucket_lifecycle(&GetBucketLifecycleInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.allow_same_action_overlap(), true);
    assert_eq!(o.rules().len(), 3);
    for (idx, rule) in o.rules().iter().enumerate() {
        let mut _rule = LifecycleRule::default();
        for r in input.rules() {
            if r.id() == rule.id() {
                _rule = r.clone();
            }
        }
        assert_eq!(rule.id(), _rule.id());
        assert_eq!(rule.prefix(), _rule.prefix());
        assert_eq!(rule.tags().len(), _rule.tags().len());
        if rule.expiration().is_some() {
            let exp = rule.expiration().as_ref().unwrap();
            let _exp = _rule.expiration().as_ref().unwrap();
            assert_eq!(exp.days(), _exp.days());
            if exp.date().is_some() {
                assert_eq!(truncate_date_to_midnight(exp.date().unwrap()), truncate_date_to_midnight(_exp.date().unwrap()));
            }
        }

        for (idx, trans) in rule.transitions().iter().enumerate() {
            let _trans = _rule.transitions().get(idx).unwrap();
            assert_eq!(trans.storage_class(), _trans.storage_class());
            assert_eq!(trans.days(), _trans.days());
            if trans.date().is_some() {
                assert_eq!(truncate_date_to_midnight(trans.date().unwrap()), truncate_date_to_midnight(_trans.date().unwrap()));
            }
        }

        if rule.noncurrent_version_expiration().is_some() {
            let exp = rule.noncurrent_version_expiration().as_ref().unwrap();
            let _exp = _rule.noncurrent_version_expiration().as_ref().unwrap();
            assert_eq!(exp.noncurrent_days(), _exp.noncurrent_days());
            if exp.noncurrent_date().is_some() {
                assert_eq!(truncate_date_to_midnight(exp.noncurrent_date().unwrap()), truncate_date_to_midnight(_exp.noncurrent_date().unwrap()));
            }
        }

        for (idx, trans) in rule.noncurrent_version_transitions().iter().enumerate() {
            let _trans = _rule.noncurrent_version_transitions().get(idx).unwrap();
            assert_eq!(trans.storage_class(), _trans.storage_class());
            assert_eq!(trans.noncurrent_days(), _trans.noncurrent_days());
            if trans.noncurrent_date().is_some() {
                assert_eq!(truncate_date_to_midnight(trans.noncurrent_date().unwrap()), truncate_date_to_midnight(_trans.noncurrent_date().unwrap()));
            }
        }
        assert_eq!(rule.abort_in_complete_multipart_upload(), _rule.abort_in_complete_multipart_upload());
        assert_eq!(rule.filter(), _rule.filter());

        if let Some(u) = rule.abort_in_complete_multipart_upload() {
            assert!(!rule.access_time_transitions().is_empty());
            assert!(!rule.non_current_version_access_time_transitions().is_empty());
        }
    }


    let o = client.delete_bucket_lifecycle(&DeleteBucketLifecycleInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_bucket_lifecycle(&GetBucketLifecycleInput::new(context.fixed_bucket())).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}

pub(crate) fn truncate_date_to_midnight(date: DateTime<Utc>) -> String {
    let mut date = date.format("%Y-%m-%d").to_string();
    date.push_str("T00:00:00Z");
    date
}

async fn test_bucket_storage_class_and_cors(context: &AsyncContext) {
    let client = context.client();
    let o = client.head_bucket(&HeadBucketInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    if let Some(sc) = o.storage_class() {
        assert_eq!(sc, &StorageClassType::StorageClassStandard);
    }

    let mut input = PutBucketStorageClassInput::new(context.fixed_bucket());
    input.set_storage_class(StorageClassType::StorageClassIa);
    let o = client.put_bucket_storage_class(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let o = client.head_bucket(&HeadBucketInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    if let Some(sc) = o.storage_class() {
        assert_eq!(sc, &StorageClassType::StorageClassIa);
    }

    let o = client.get_bucket_location(&GetBucketLocationInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.region().len() > 0);
    assert!(o.extranet_endpoint().len() > 0);
    assert!(o.intranet_endpoint().len() > 0);
    println!("{}", o.region());
    println!("{}", o.extranet_endpoint());
    println!("{}", o.intranet_endpoint());

    let mut input = PutBucketCORSInput::new(context.fixed_bucket());
    let mut rule = CORSRule::new();
    rule.set_allowed_origins(vec!["www.abc.com".to_string()]);
    rule.set_allowed_methods(vec!["GET".to_string()]);
    rule.set_max_age_seconds(3600);
    input.add_rule(rule);

    let mut rule = CORSRule::new();
    rule.set_allowed_origins(vec!["*".to_string()]);
    rule.set_allowed_methods(vec!["HEAD".to_string()]);
    rule.set_allowed_headers(vec!["*".to_string()]);
    rule.set_expose_headers(vec!["ETag".to_string(), "RequestId".to_string()]);
    rule.set_max_age_seconds(7200);
    rule.set_response_vary(true);
    input.add_rule(rule);
    let o = client.put_bucket_cors(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let input = GetBucketCORSInput::new(context.fixed_bucket());
    let o = client.get_bucket_cors(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert_eq!(o.rules().len(), 2);
    for (idx, rule) in o.rules().iter().enumerate() {
        if idx == 0 {
            assert_eq!(rule.allowed_origins()[0].as_str(), "www.abc.com");
            assert_eq!(rule.allowed_methods()[0].as_str(), "GET");
            assert_eq!(rule.response_vary(), false);
        } else {
            assert_eq!(rule.allowed_origins()[0].as_str(), "*");
            assert_eq!(rule.allowed_methods()[0].as_str(), "HEAD");
            assert_eq!(rule.response_vary(), true);
        }
    }

    let input = DeleteBucketCORSInput::new(context.fixed_bucket());
    let o = client.delete_bucket_cors(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let input = GetBucketCORSInput::new(context.fixed_bucket());
    let e = client.get_bucket_cors(&input).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
    println!("{}", e.request_url());
}

async fn test_basic(context: &AsyncContext) {
    let client = context.client();
    let o = client.list_buckets(&ListBucketsInput::new()).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.buckets().len() >= 0);

    let bucket1 = gen_random_string(10);
    let bucket1 = bucket1.as_str();
    let o = client.create_bucket(&CreateBucketInput::new(bucket1)).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket1).await;

    let o = client.head_bucket(&HeadBucketInput::new(bucket1)).await.unwrap();
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassType::StorageClassStandard);
    if let Some(az) = o.az_redundancy().as_ref() {
        assert_eq!(az, &AzRedundancyType::AzRedundancySingleAz);
    }

    let o = client.get_bucket_info(&GetBucketInfoInput::new(bucket1)).await.unwrap();
    assert!(o.request_id().len() > 0);
    if let Some(az) = o.bucket().az_redundancy().as_ref() {
        assert_eq!(az, &AzRedundancyType::AzRedundancySingleAz);
    }
    println!("{:?}", o.bucket());

    let bucket2 = gen_random_string(40);
    let bucket2 = bucket2.as_str();
    let mut input = CreateBucketInput::new(bucket2);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_storage_class(StorageClassType::StorageClassIa);
    input.set_az_redundancy(AzRedundancyType::AzRedundancySingleAz);
    let o = client.create_bucket(&input).await.unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket2).await;
    match client.head_bucket(&HeadBucketInput::new(bucket2)).await {
        Ok(o) => {
            assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassType::StorageClassIa);
            if let Some(az) = o.az_redundancy().as_ref() {
                assert_eq!(az, &AzRedundancyType::AzRedundancySingleAz);
            }
        }
        Err(_) => {
            assert!(false);
        }
    }

    let bucket1 = gen_random_string(20);
    let o = client.create_bucket(&CreateBucketInput::new(bucket1.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let bucket2 = gen_random_string(20);
    let o = client.create_bucket(&CreateBucketInput::new(bucket2.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut count = 0;
    let o = client.list_buckets(&ListBucketsInput::new()).await.unwrap();
    for bucket in o.buckets() {
        if bucket.name() == bucket1 || bucket.name() == bucket2 {
            count += 1;
            assert!(bucket.location().len() > 0);
        }
    }

    assert_eq!(count, 2);

    assert_eq!(client.does_bucket_exist(&DoesBucketExistInput::new(bucket1.as_str())).await.unwrap(), true);
    assert_eq!(client.does_bucket_exist(&DoesBucketExistInput::new(bucket2.as_str())).await.unwrap(), true);
    let o = client.delete_bucket(&DeleteBucketInput::new(bucket1.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);
    let o = client.delete_bucket(&DeleteBucketInput::new(bucket2.as_str())).await.unwrap();
    assert!(o.request_id().len() > 0);

    let mut count = 0;
    let o = client.list_buckets(&ListBucketsInput::new()).await.unwrap();
    for bucket in o.buckets() {
        if bucket.name() == bucket1 || bucket.name() == bucket2 {
            count += 1;
            assert!(bucket.location().len() > 0);
        }
    }

    assert_eq!(count, 0);

    assert_eq!(client.does_bucket_exist(&DoesBucketExistInput::new(bucket1.as_str())).await.unwrap(), false);
    assert_eq!(client.does_bucket_exist(&DoesBucketExistInput::new(bucket2.as_str())).await.unwrap(), false);

    let e = client.head_bucket(&HeadBucketInput::new(context.non_exists_bucket())).await.expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}