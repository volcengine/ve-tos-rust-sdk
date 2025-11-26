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
use common::gen_random_string;
use ve_tos_rust_sdk::bucket::ListBucketsInput;
use ve_tos_rust_sdk::bucket::{BucketAPI, CreateBucketInput, DeleteBucketInput, HeadBucketInput};
use ve_tos_rust_sdk::enumeration::{ACLType, AzRedundancyType, StorageClassType};

use crate::common::{create_context, Context};

mod common;

#[test]
fn test_main() {
    let context = &create_context();
    test_list_buckets(context);
    test_create_bucket(context);
    test_invalid_argument(context);
}

fn test_list_buckets(context: &Context) {
    let client = context.client();
    let o = client.list_buckets(&ListBucketsInput::new()).unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.buckets().len() >= 0);
}

fn test_create_bucket(context: &Context) {
    let client = context.client();
    let bucket1 = gen_random_string(40);
    let bucket1 = bucket1.as_str();
    let o = client.create_bucket(&CreateBucketInput::new(bucket1)).unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket1);

    let o = client.head_bucket(&HeadBucketInput::new(bucket1)).unwrap();
    assert_eq!(o.storage_class().as_ref().unwrap(), &StorageClassType::StorageClassStandard);
    if let Some(az) = o.az_redundancy().as_ref() {
        assert_eq!(az, &AzRedundancyType::AzRedundancySingleAz);
    }

    let bucket2 = gen_random_string(40);
    let bucket2 = bucket2.as_str();
    let mut input = CreateBucketInput::new(bucket2);
    input.set_acl(ACLType::ACLPublicRead);
    input.set_storage_class(StorageClassType::StorageClassIa);
    input.set_az_redundancy(AzRedundancyType::AzRedundancySingleAz);
    let o = client.create_bucket(&input).unwrap();
    assert!(o.request_id().len() > 0);
    context.add_bucket(bucket2);
    match client.head_bucket(&HeadBucketInput::new(bucket2)) {
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

    let bucket1 = self::gen_random_string(20);
    let o = client.create_bucket(&CreateBucketInput::new(bucket1.as_str())).unwrap();
    assert!(o.request_id().len() > 0);

    let bucket2 = self::gen_random_string(20);
    let o = client.create_bucket(&CreateBucketInput::new(bucket2.as_str())).unwrap();
    assert!(o.request_id().len() > 0);

    let mut count = 0;
    let o = client.list_buckets(&ListBucketsInput::new()).unwrap();
    for bucket in o.buckets() {
        if bucket.name() == bucket1 || bucket.name() == bucket2 {
            count += 1;
            assert!(bucket.location().len() > 0);
        }
    }

    assert_eq!(count, 2);
    let o = client.delete_bucket(&DeleteBucketInput::new(bucket1.as_str())).unwrap();
    assert!(o.request_id().len() > 0);
    let o = client.delete_bucket(&DeleteBucketInput::new(bucket2.as_str())).unwrap();
    assert!(o.request_id().len() > 0);

    let mut count = 0;
    let o = client.list_buckets(&ListBucketsInput::new()).unwrap();
    for bucket in o.buckets() {
        if bucket.name() == bucket1 || bucket.name() == bucket2 {
            count += 1;
            assert!(bucket.location().len() > 0);
        }
    }

    assert_eq!(count, 0);
}

fn test_invalid_argument(context: &Context) {
    let client = context.client();

    let e = client.head_bucket(&HeadBucketInput::new(context.non_exists_bucket())).expect_err("");
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);

    for bn in ["a#b#c", "a", "-abc", "abc-"] {
        let e = client.create_bucket(&CreateBucketInput::new(bn)).expect_err("");
        assert!(!e.is_server_error());
    }
}