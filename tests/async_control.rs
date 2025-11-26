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
use crate::common::asynchronous::{create_async_context, AsyncContext};
use scopeguard::defer;
use tokio::runtime;
use ve_tos_rust_sdk::asynchronous::bucket::BucketAPI;
use ve_tos_rust_sdk::asynchronous::control::ControlAPI;
use ve_tos_rust_sdk::bucket::GetBucketACLInput;
use ve_tos_rust_sdk::control::{DeleteQosPolicyInput, GetQosPolicyInput, PutQosPolicyInput};

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
        test_qos_policy(&context).await;
    })
}

async fn test_qos_policy(context: &AsyncContext) {
    let client = context.client();
    let o = client.get_bucket_acl(&GetBucketACLInput::new(context.fixed_bucket())).await.unwrap();
    assert!(o.request_id().len() > 0);
    let account_id = o.owner().id();

    let policy;
    if let Ok(o) = client.get_qos_policy(&GetQosPolicyInput::new(account_id)).await {
        assert!(o.request_id().len() > 0);
        assert!(o.policy().len() > 0);
        println!("{}", o.policy());
        policy = o.policy().to_string();
    } else {
        policy = "{\"Version\":\"2012-10-17\",\"Statement\":[{\"Sid\":\"statement1\",\"Quota\":{\"WritesQps\":\"100\",\"ReadsQps\":\"100\",\"ListQps\":\"100\",\"WritesRate\":\"10\",\"ReadsRate\":\"10\"},\"Principal\":[\"trn:iam::AccountId1:role/tos_role\",\"trn:iam::AccountId2:user/tos_user\",\"*\"],\"Resource\":\"trn:tos:::examplebucket1/*\"}]}".to_string();
    }

    let input = PutQosPolicyInput::new(account_id, policy);
    let o = client.put_qos_policy(&input).await.unwrap();
    assert!(o.request_id().len() > 0);

    let o = client.get_qos_policy(&GetQosPolicyInput::new(account_id)).await.unwrap();
    assert!(o.request_id().len() > 0);
    assert!(o.policy().len() > 0);

    let o = client.delete_qos_policy(&DeleteQosPolicyInput::new(account_id)).await.unwrap();
    assert!(o.request_id().len() > 0);

    let e = client.get_qos_policy(&GetQosPolicyInput::new(account_id)).await.unwrap_err();
    assert!(e.is_server_error());
    let ex = e.as_server_error().unwrap();
    assert_eq!(ex.status_code(), 404);
}