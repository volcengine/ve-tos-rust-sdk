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

use async_trait::async_trait;

use crate::asynchronous::http::HttpResponse;
use crate::asynchronous::internal::{parse_json, read_response_string, OutputParser};
use crate::bucket::{CreateBucketInput, CreateBucketOutput, DeleteBucketCORSInput, DeleteBucketCORSOutput, DeleteBucketCustomDomainInput, DeleteBucketCustomDomainOutput, DeleteBucketEncryptionInput, DeleteBucketEncryptionOutput, DeleteBucketInput, DeleteBucketInventoryInput, DeleteBucketInventoryOutput, DeleteBucketLifecycleInput, DeleteBucketLifecycleOutput, DeleteBucketMirrorBackInput, DeleteBucketMirrorBackOutput, DeleteBucketOutput, DeleteBucketPolicyInput, DeleteBucketPolicyOutput, DeleteBucketRealTimeLogInput, DeleteBucketRealTimeLogOutput, DeleteBucketRenameInput, DeleteBucketRenameOutput, DeleteBucketReplicationInput, DeleteBucketReplicationOutput, DeleteBucketTaggingInput, DeleteBucketTaggingOutput, DeleteBucketWebsiteInput, DeleteBucketWebsiteOutput, DoesBucketExistInput, GetBucketACLInput, GetBucketACLOutput, GetBucketAccessMonitorInput, GetBucketAccessMonitorOutput, GetBucketCORSInput, GetBucketCORSOutput, GetBucketEncryptionInput, GetBucketEncryptionOutput, GetBucketInfoInput, GetBucketInfoOutput, GetBucketInventoryInput, GetBucketInventoryOutput, GetBucketLifecycleInput, GetBucketLifecycleOutput, GetBucketLocationInput, GetBucketLocationOutput, GetBucketMirrorBackInput, GetBucketMirrorBackOutput, GetBucketNotificationType2Input, GetBucketNotificationType2Output, GetBucketPolicyInput, GetBucketPolicyOutput, GetBucketRealTimeLogInput, GetBucketRealTimeLogOutput, GetBucketRenameInput, GetBucketRenameOutput, GetBucketReplicationInput, GetBucketReplicationOutput, GetBucketTaggingInput, GetBucketTaggingOutput, GetBucketTrashInput, GetBucketTrashOutput, GetBucketTypeInput, GetBucketTypeOutput, GetBucketVersioningInput, GetBucketVersioningOutput, GetBucketWebsiteInput, GetBucketWebsiteOutput, HeadBucketInput, HeadBucketOutput, ListBucketCustomDomainInput, ListBucketCustomDomainOutput, ListBucketInventoryInput, ListBucketInventoryOutput, ListBucketsInput, ListBucketsOutput, PutBucketACLInput, PutBucketACLOutput, PutBucketAccessMonitorInput, PutBucketAccessMonitorOutput, PutBucketCORSInput, PutBucketCORSOutput, PutBucketCustomDomainInput, PutBucketCustomDomainOutput, PutBucketEncryptionInput, PutBucketEncryptionOutput, PutBucketInventoryInput, PutBucketInventoryOutput, PutBucketLifecycleInput, PutBucketLifecycleOutput, PutBucketMirrorBackInput, PutBucketMirrorBackOutput, PutBucketNotificationType2Input, PutBucketNotificationType2Output, PutBucketPolicyInput, PutBucketPolicyOutput, PutBucketRealTimeLogInput, PutBucketRealTimeLogOutput, PutBucketRenameInput, PutBucketRenameOutput, PutBucketReplicationInput, PutBucketReplicationOutput, PutBucketStorageClassInput, PutBucketStorageClassOutput, PutBucketTaggingInput, PutBucketTaggingOutput, PutBucketTrashInput, PutBucketTrashOutput, PutBucketVersioningInput, PutBucketVersioningOutput, PutBucketWebsiteInput, PutBucketWebsiteOutput};
use crate::common::{Meta, RequestInfo};
use crate::constant::{HEADER_ALLOW_SAME_ACTION_OVERLAP, HEADER_AZ_REDUNDANCY, HEADER_BUCKET_REGION, HEADER_BUCKET_TYPE, HEADER_LOCATION, HEADER_PROJECT_NAME, HEADER_STORAGE_CLASS, TRUE};
use crate::enumeration::{AzRedundancyType, BucketType, StorageClassType};
use crate::error::TosError;
use crate::http::HttpRequest;
use crate::internal::{get_header_value, get_header_value_ref, get_header_value_str, parse_date_time_iso8601};

#[async_trait]
pub trait BucketAPI {
    async fn create_bucket(&self, input: &CreateBucketInput) -> Result<CreateBucketOutput, TosError>;
    async fn head_bucket(&self, input: &HeadBucketInput) -> Result<HeadBucketOutput, TosError>;
    async fn delete_bucket(&self, input: &DeleteBucketInput) -> Result<DeleteBucketOutput, TosError>;
    async fn list_buckets(&self, input: &ListBucketsInput) -> Result<ListBucketsOutput, TosError>;
    async fn put_bucket_cors(&self, input: &PutBucketCORSInput) -> Result<PutBucketCORSOutput, TosError>;
    async fn get_bucket_cors(&self, input: &GetBucketCORSInput) -> Result<GetBucketCORSOutput, TosError>;
    async fn delete_bucket_cors(&self, input: &DeleteBucketCORSInput) -> Result<DeleteBucketCORSOutput, TosError>;
    async fn put_bucket_storage_class(&self, input: &PutBucketStorageClassInput) -> Result<PutBucketStorageClassOutput, TosError>;
    async fn get_bucket_location(&self, input: &GetBucketLocationInput) -> Result<GetBucketLocationOutput, TosError>;
    async fn put_bucket_lifecycle(&self, input: &PutBucketLifecycleInput) -> Result<PutBucketLifecycleOutput, TosError>;
    async fn get_bucket_lifecycle(&self, input: &GetBucketLifecycleInput) -> Result<GetBucketLifecycleOutput, TosError>;
    async fn delete_bucket_lifecycle(&self, input: &DeleteBucketLifecycleInput) -> Result<DeleteBucketLifecycleOutput, TosError>;
    async fn put_bucket_policy(&self, input: &PutBucketPolicyInput) -> Result<PutBucketPolicyOutput, TosError>;
    async fn get_bucket_policy(&self, input: &GetBucketPolicyInput) -> Result<GetBucketPolicyOutput, TosError>;
    async fn delete_bucket_policy(&self, input: &DeleteBucketPolicyInput) -> Result<DeleteBucketPolicyOutput, TosError>;
    async fn put_bucket_mirror_back(&self, input: &PutBucketMirrorBackInput) -> Result<PutBucketMirrorBackOutput, TosError>;
    async fn get_bucket_mirror_back(&self, input: &GetBucketMirrorBackInput) -> Result<GetBucketMirrorBackOutput, TosError>;
    async fn delete_bucket_mirror_back(&self, input: &DeleteBucketMirrorBackInput) -> Result<DeleteBucketMirrorBackOutput, TosError>;
    async fn put_bucket_acl(&self, input: &PutBucketACLInput) -> Result<PutBucketACLOutput, TosError>;
    async fn get_bucket_acl(&self, input: &GetBucketACLInput) -> Result<GetBucketACLOutput, TosError>;
    async fn put_bucket_replication(&self, input: &PutBucketReplicationInput) -> Result<PutBucketReplicationOutput, TosError>;
    async fn get_bucket_replication(&self, input: &GetBucketReplicationInput) -> Result<GetBucketReplicationOutput, TosError>;
    async fn delete_bucket_replication(&self, input: &DeleteBucketReplicationInput) -> Result<DeleteBucketReplicationOutput, TosError>;
    async fn put_bucket_versioning(&self, input: &PutBucketVersioningInput) -> Result<PutBucketVersioningOutput, TosError>;
    async fn get_bucket_versioning(&self, input: &GetBucketVersioningInput) -> Result<GetBucketVersioningOutput, TosError>;
    async fn put_bucket_website(&self, input: &PutBucketWebsiteInput) -> Result<PutBucketWebsiteOutput, TosError>;
    async fn get_bucket_website(&self, input: &GetBucketWebsiteInput) -> Result<GetBucketWebsiteOutput, TosError>;
    async fn delete_bucket_website(&self, input: &DeleteBucketWebsiteInput) -> Result<DeleteBucketWebsiteOutput, TosError>;
    async fn put_bucket_custom_domain(&self, input: &PutBucketCustomDomainInput) -> Result<PutBucketCustomDomainOutput, TosError>;
    async fn list_bucket_custom_domain(&self, input: &ListBucketCustomDomainInput) -> Result<ListBucketCustomDomainOutput, TosError>;
    async fn delete_bucket_custom_domain(&self, input: &DeleteBucketCustomDomainInput) -> Result<DeleteBucketCustomDomainOutput, TosError>;
    async fn put_bucket_real_time_log(&self, input: &PutBucketRealTimeLogInput) -> Result<PutBucketRealTimeLogOutput, TosError>;
    async fn get_bucket_real_time_log(&self, input: &GetBucketRealTimeLogInput) -> Result<GetBucketRealTimeLogOutput, TosError>;
    async fn delete_bucket_real_time_log(&self, input: &DeleteBucketRealTimeLogInput) -> Result<DeleteBucketRealTimeLogOutput, TosError>;
    async fn put_bucket_rename(&self, input: &PutBucketRenameInput) -> Result<PutBucketRenameOutput, TosError>;
    async fn get_bucket_rename(&self, input: &GetBucketRenameInput) -> Result<GetBucketRenameOutput, TosError>;
    async fn delete_bucket_rename(&self, input: &DeleteBucketRenameInput) -> Result<DeleteBucketRenameOutput, TosError>;
    async fn put_bucket_encryption(&self, input: &PutBucketEncryptionInput) -> Result<PutBucketEncryptionOutput, TosError>;
    async fn get_bucket_encryption(&self, input: &GetBucketEncryptionInput) -> Result<GetBucketEncryptionOutput, TosError>;
    async fn delete_bucket_encryption(&self, input: &DeleteBucketEncryptionInput) -> Result<DeleteBucketEncryptionOutput, TosError>;
    async fn put_bucket_tagging(&self, input: &PutBucketTaggingInput) -> Result<PutBucketTaggingOutput, TosError>;
    async fn get_bucket_tagging(&self, input: &GetBucketTaggingInput) -> Result<GetBucketTaggingOutput, TosError>;
    async fn delete_bucket_tagging(&self, input: &DeleteBucketTaggingInput) -> Result<DeleteBucketTaggingOutput, TosError>;
    async fn put_bucket_notification_type2(&self, input: &PutBucketNotificationType2Input) -> Result<PutBucketNotificationType2Output, TosError>;
    async fn get_bucket_notification_type2(&self, input: &GetBucketNotificationType2Input) -> Result<GetBucketNotificationType2Output, TosError>;
    async fn put_bucket_inventory(&self, input: &PutBucketInventoryInput) -> Result<PutBucketInventoryOutput, TosError>;
    async fn get_bucket_inventory(&self, input: &GetBucketInventoryInput) -> Result<GetBucketInventoryOutput, TosError>;
    async fn list_bucket_inventory(&self, input: &ListBucketInventoryInput) -> Result<ListBucketInventoryOutput, TosError>;
    async fn delete_bucket_inventory(&self, input: &DeleteBucketInventoryInput) -> Result<DeleteBucketInventoryOutput, TosError>;
    async fn get_bucket_type(&self, input: &GetBucketTypeInput) -> Result<GetBucketTypeOutput, TosError>;
    async fn does_bucket_exist(&self, input: &DoesBucketExistInput) -> Result<bool, TosError>;
    async fn get_bucket_info(&self, input: &GetBucketInfoInput) -> Result<GetBucketInfoOutput, TosError>;
    async fn put_bucket_access_monitor(&self, input: &PutBucketAccessMonitorInput) -> Result<PutBucketAccessMonitorOutput, TosError>;
    async fn get_bucket_access_monitor(&self, input: &GetBucketAccessMonitorInput) -> Result<GetBucketAccessMonitorOutput, TosError>;
    async fn put_bucket_trash(&self, input: &PutBucketTrashInput) -> Result<PutBucketTrashOutput, TosError>;
    async fn get_bucket_trash(&self, input: &GetBucketTrashInput) -> Result<GetBucketTrashOutput, TosError>;
}

#[async_trait]
impl OutputParser for ListBucketsOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for CreateBucketOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let location = get_header_value(response.headers(), HEADER_LOCATION);
        Ok(Self { request_info, location })
    }
}

#[async_trait]
impl OutputParser for HeadBucketOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let region = get_header_value(response.headers(), HEADER_BUCKET_REGION);
        let storage_class = StorageClassType::from(get_header_value_str(response.headers(), HEADER_STORAGE_CLASS));
        let az_redundancy = AzRedundancyType::from(get_header_value_str(response.headers(), HEADER_AZ_REDUNDANCY));
        let project_name = get_header_value(response.headers(), HEADER_PROJECT_NAME);
        let bucket_type = BucketType::from(get_header_value_str(response.headers(), HEADER_BUCKET_TYPE));
        Ok(Self { request_info, region, storage_class, az_redundancy, project_name, bucket_type })
    }
}

#[async_trait]
impl OutputParser for DeleteBucketOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}
#[async_trait]
impl OutputParser for PutBucketCORSOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for GetBucketCORSOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketCORSOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}
#[async_trait]
impl OutputParser for PutBucketStorageClassOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}
#[async_trait]
impl OutputParser for GetBucketLocationOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}
#[async_trait]
impl OutputParser for PutBucketLifecycleOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for GetBucketLifecycleOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let allow_same_action_overlap = get_header_value_ref(response.headers(), HEADER_ALLOW_SAME_ACTION_OVERLAP) == TRUE;
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        result.allow_same_action_overlap = allow_same_action_overlap;
        for rule in result.rules.iter_mut() {
            if let Some(exp) = &mut rule.expiration {
                if let Some(ref date) = exp.date_string {
                    exp.date = parse_date_time_iso8601(&date)?;
                }
            }

            for trans in rule.transitions.iter_mut() {
                if let Some(ref date) = trans.date_string {
                    trans.date = parse_date_time_iso8601(&date)?;
                }
            }

            if let Some(exp) = &mut rule.noncurrent_version_expiration {
                if let Some(ref date) = exp.noncurrent_date_string {
                    exp.noncurrent_date = parse_date_time_iso8601(&date)?;
                }
            }

            for trans in rule.noncurrent_version_transitions.iter_mut() {
                if let Some(ref date) = trans.noncurrent_date_string {
                    trans.noncurrent_date = parse_date_time_iso8601(&date)?;
                }
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketLifecycleOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for PutBucketPolicyOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}


#[async_trait]
impl OutputParser for GetBucketPolicyOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let policy = read_response_string(response).await?;
        Ok(Self {
            request_info,
            policy,
        })
    }
}

#[async_trait]
impl OutputParser for DeleteBucketPolicyOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for PutBucketMirrorBackOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for GetBucketMirrorBackOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}
#[async_trait]
impl OutputParser for DeleteBucketMirrorBackOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for PutBucketACLOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for GetBucketACLOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for PutBucketReplicationOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketReplicationOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketReplicationOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for PutBucketVersioningOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketVersioningOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for PutBucketWebsiteOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}


#[async_trait]
impl OutputParser for GetBucketWebsiteOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketWebsiteOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for PutBucketCustomDomainOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}
#[async_trait]
impl OutputParser for ListBucketCustomDomainOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}
#[async_trait]
impl OutputParser for DeleteBucketCustomDomainOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}
#[async_trait]
impl OutputParser for PutBucketRealTimeLogOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}
#[async_trait]
impl OutputParser for GetBucketRealTimeLogOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketRealTimeLogOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for PutBucketRenameOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketRenameOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketRenameOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}
#[async_trait]
impl OutputParser for PutBucketEncryptionOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketEncryptionOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketEncryptionOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for PutBucketTaggingOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketTaggingOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketTaggingOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}
#[async_trait]
impl OutputParser for PutBucketNotificationType2Output {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketNotificationType2Output {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for PutBucketInventoryOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketInventoryOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for ListBucketInventoryOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteBucketInventoryOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketInfoOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        if let Some(ref date) = result.bucket.creation_date_string {
            result.bucket.creation_date = parse_date_time_iso8601(&date)?;
        }
        result.request_info = request_info;
        Ok(result)
    }
}


#[async_trait]
impl OutputParser for PutBucketAccessMonitorOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketAccessMonitorOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for PutBucketTrashOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetBucketTrashOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}