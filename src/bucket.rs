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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use ve_tos_generic::{AclHeader, BucketSetter, GenericInput, RequestInfo};

use super::enumeration::{ACLType, AuthProtocolType, AzRedundancyType, BucketType, CertStatusType, HttpMethodType::HttpMethodPut, InventoryFormatType, InventoryFrequencyType, InventoryIncludedObjType, ProtocolType, RedirectType, StatusType, StorageClassInheritDirectiveType, StorageClassType, VersioningStatusType};
use super::error::{GenericError, TosError};
use super::http::{HttpRequest, HttpResponse};
use super::internal::{base64_md5, get_header_value, parse_json, truncate_date_to_midnight, InputTranslator, OutputParser};
use crate::common::{GenericInput, Grant, Meta, Owner, RequestInfo, Tag, TagSet};
use crate::config::ConfigHolder;
use crate::constant::{HEADER_ALLOW_SAME_ACTION_OVERLAP, HEADER_AZ_REDUNDANCY, HEADER_BUCKET_REGION, HEADER_BUCKET_TYPE, HEADER_CONTENT_LENGTH, HEADER_CONTENT_MD5, HEADER_LOCATION, HEADER_PROJECT_NAME, HEADER_STORAGE_CLASS, HEADER_TAGGING, ISO8601_DATE_FORMAT, TRUE};
use crate::enumeration::HttpMethodType::{HttpMethodDelete, HttpMethodGet, HttpMethodHead};
use crate::internal::{get_header_value_str, map_insert, set_acl_header, InputDescriptor};
use crate::reader::BuildBufferReader;

pub trait BucketAPI {
    fn create_bucket(&self, input: &CreateBucketInput) -> Result<CreateBucketOutput, TosError>;
    fn head_bucket(&self, input: &HeadBucketInput) -> Result<HeadBucketOutput, TosError>;
    fn delete_bucket(&self, input: &DeleteBucketInput) -> Result<DeleteBucketOutput, TosError>;
    fn list_buckets(&self, input: &ListBucketsInput) -> Result<ListBucketsOutput, TosError>;
}


#[derive(Debug, Clone, PartialEq, Default, AclHeader, GenericInput)]
#[enable_grant_write]
pub struct CreateBucketInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) acl: Option<ACLType>,
    pub(crate) grant_full_control: String,
    pub(crate) grant_read: String,
    pub(crate) grant_read_acp: String,
    pub(crate) grant_write: String,
    pub(crate) grant_write_acp: String,
    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) az_redundancy: Option<AzRedundancyType>,
    pub(crate) project_name: String,
    pub(crate) tagging: String,
    pub(crate) bucket_type: Option<BucketType>,
}

impl CreateBucketInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }
    pub fn az_redundancy(&self) -> &Option<AzRedundancyType> {
        &self.az_redundancy
    }
    pub fn project_name(&self) -> &str {
        &self.project_name
    }
    pub fn tagging(&self) -> &str {
        &self.tagging
    }
    pub fn bucket_type(&self) -> &Option<BucketType> {
        &self.bucket_type
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = Some(storage_class.into());
    }
    pub fn set_az_redundancy(&mut self, az_redundancy: impl Into<AzRedundancyType>) {
        self.az_redundancy = Some(az_redundancy.into());
    }
    pub fn set_project_name(&mut self, project_name: impl Into<String>) {
        self.project_name = project_name.into();
    }
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.tagging = tagging.into();
    }

    pub fn set_bucket_type(&mut self, bucket_type: impl Into<BucketType>) {
        self.bucket_type = Some(bucket_type.into());
    }
}


impl InputDescriptor for CreateBucketInput {
    fn operation(&self) -> &str {
        "CreateBucket"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for CreateBucketInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodPut;
        let header = &mut request.header;
        set_acl_header(header, self);
        if let Some(x) = &self.storage_class {
            header.insert(HEADER_STORAGE_CLASS, x.as_str().to_string());
        }

        if let Some(x) = &self.az_redundancy {
            header.insert(HEADER_AZ_REDUNDANCY, x.as_str().to_string());
        }
        map_insert(header, HEADER_PROJECT_NAME, self.project_name());
        map_insert(header, HEADER_TAGGING, self.tagging());
        if let Some(x) = &self.bucket_type {
            header.insert(HEADER_BUCKET_TYPE, x.as_str().to_string());
        }
        Ok(request)
    }
}


#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct CreateBucketOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) location: String,
}

impl CreateBucketOutput {
    pub fn location(&self) -> &str {
        &self.location
    }
}

impl OutputParser for CreateBucketOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let location = get_header_value(response.headers(), HEADER_LOCATION);
        Ok(Self { request_info, location })
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct HeadBucketInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}


impl InputDescriptor for HeadBucketInput {
    fn operation(&self) -> &str {
        "HeadBucket"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for HeadBucketInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodHead;
        Ok(request)
    }
}


#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct HeadBucketOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) region: String,
    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) az_redundancy: Option<AzRedundancyType>,
    pub(crate) project_name: String,
    pub(crate) bucket_type: Option<BucketType>,
}


impl HeadBucketOutput {
    pub fn region(&self) -> &str {
        &self.region
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }
    pub fn az_redundancy(&self) -> &Option<AzRedundancyType> {
        &self.az_redundancy
    }
    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    pub fn bucket_type(&self) -> &Option<BucketType> {
        &self.bucket_type
    }
}

impl OutputParser for HeadBucketOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let region = get_header_value(response.headers(), HEADER_BUCKET_REGION);
        let storage_class = StorageClassType::from(get_header_value_str(response.headers(), HEADER_STORAGE_CLASS));
        let az_redundancy = AzRedundancyType::from(get_header_value_str(response.headers(), HEADER_AZ_REDUNDANCY));
        let project_name = get_header_value(response.headers(), HEADER_PROJECT_NAME);
        let bucket_type = BucketType::from(get_header_value_str(response.headers(), HEADER_BUCKET_TYPE));

        Ok(Self { request_info, region, storage_class, az_redundancy, project_name, bucket_type })
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}


impl InputDescriptor for DeleteBucketInput {
    fn operation(&self) -> &str {
        "DeleteBucket"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodDelete;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketOutput {
    pub(crate) request_info: RequestInfo,
}

impl OutputParser for DeleteBucketOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, _: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        Ok(Self { request_info })
    }
}

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct ListBucketsInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) project_name: String,
    pub(crate) bucket_type: Option<BucketType>,
}

impl ListBucketsInput {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn project_name(&self) -> &str {
        &self.project_name
    }
    pub fn bucket_type(&self) -> &Option<BucketType> {
        &self.bucket_type
    }
    pub fn set_project_name(&mut self, project_name: impl Into<String>) {
        self.project_name = project_name.into();
    }
    pub fn set_bucket_type(&mut self, bucket_type: impl Into<BucketType>) {
        self.bucket_type = Some(bucket_type.into());
    }
}

impl InputDescriptor for ListBucketsInput {
    fn operation(&self) -> &str {
        "ListBuckets"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Err(TosError::client_error("unsupported"))
    }
}

impl<B> InputTranslator<B> for ListBucketsInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = HttpRequest::default();
        request.operation = self.operation();
        request.method = HttpMethodGet;
        map_insert(&mut request.header, HEADER_PROJECT_NAME, &self.project_name);
        if let Some(x) = &self.bucket_type {
            map_insert(&mut request.header, HEADER_BUCKET_TYPE, x.as_str());
        }
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListBucketsOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Buckets")]
    pub(crate) buckets: Vec<ListedBucket>,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ListedBucket {
    #[serde(default)]
    #[serde(rename = "CreationDate")]
    pub(crate) creation_date: String,
    #[serde(default)]
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(default)]
    #[serde(rename = "Location")]
    pub(crate) location: String,
    #[serde(default)]
    #[serde(rename = "ExtranetEndpoint")]
    pub(crate) extranet_endpoint: String,
    #[serde(default)]
    #[serde(rename = "IntranetEndpoint")]
    pub(crate) intranet_endpoint: String,
    #[serde(default)]
    #[serde(rename = "ProjectName")]
    pub(crate) project_name: String,
    #[serde(default)]
    #[serde(rename = "BucketType")]
    pub(crate) bucket_type: BucketType,
}


impl ListBucketsOutput {
    pub fn buckets(&self) -> &Vec<ListedBucket> {
        &self.buckets
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
}

impl OutputParser for ListBucketsOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        result.request_info = request_info;
        Ok(result)
    }
}

impl ListedBucket {
    pub fn creation_date(&self) -> &str {
        &self.creation_date
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn location(&self) -> &str {
        &self.location
    }
    pub fn extranet_endpoint(&self) -> &str {
        &self.extranet_endpoint
    }
    pub fn intranet_endpoint(&self) -> &str {
        &self.intranet_endpoint
    }
    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    pub fn bucket_type(&self) -> &BucketType {
        &self.bucket_type
    }
}


#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketCORSInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "CORSRules")]
    pub(crate) rules: Vec<CORSRule>,
}

impl PutBucketCORSInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rules: vec![],
        }
    }

    pub fn new_with_rules(bucket: impl Into<String>, rules: impl Into<Vec<CORSRule>>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rules: rules.into(),
        }
    }

    pub fn add_rule(&mut self, rule: impl Into<CORSRule>) {
        self.rules.push(rule.into());
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn rules(&self) -> &Vec<CORSRule> {
        &self.rules
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_rules(&mut self, rules: impl Into<Vec<CORSRule>>) {
        self.rules = rules.into();
    }
}

impl InputDescriptor for PutBucketCORSInput {
    fn operation(&self) -> &str {
        "PutBucketCORS"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketCORSInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.rules.len() == 0 {
            return Err(TosError::client_error("empty cors rules"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("cors", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CORSRule {
    #[serde(rename = "AllowedOrigins")]
    #[serde(default)]
    pub(crate) allowed_origins: Vec<String>,
    #[serde(rename = "AllowedMethods")]
    #[serde(default)]
    pub(crate) allowed_methods: Vec<String>,
    #[serde(rename = "AllowedHeaders")]
    #[serde(default)]
    pub(crate) allowed_headers: Vec<String>,
    #[serde(rename = "ExposeHeaders")]
    #[serde(default)]
    pub(crate) expose_headers: Vec<String>,
    #[serde(rename = "MaxAgeSeconds")]
    #[serde(default)]
    pub(crate) max_age_seconds: isize,
    #[serde(rename = "ResponseVary")]
    #[serde(default)]
    pub(crate) response_vary: bool,
}

impl CORSRule {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn allowed_origins(&self) -> &Vec<String> {
        &self.allowed_origins
    }

    pub fn allowed_methods(&self) -> &Vec<String> {
        &self.allowed_methods
    }

    pub fn allowed_headers(&self) -> &Vec<String> {
        &self.allowed_headers
    }

    pub fn expose_headers(&self) -> &Vec<String> {
        &self.expose_headers
    }

    pub fn max_age_seconds(&self) -> isize {
        self.max_age_seconds
    }

    pub fn response_vary(&self) -> bool {
        self.response_vary
    }

    pub fn set_allowed_origins(&mut self, allowed_origins: impl Into<Vec<String>>) {
        self.allowed_origins = allowed_origins.into();
    }

    pub fn set_allowed_methods(&mut self, allowed_methods: impl Into<Vec<String>>) {
        self.allowed_methods = allowed_methods.into();
    }

    pub fn set_allowed_headers(&mut self, allowed_headers: impl Into<Vec<String>>) {
        self.allowed_headers = allowed_headers.into();
    }

    pub fn set_expose_headers(&mut self, expose_headers: impl Into<Vec<String>>) {
        self.expose_headers = expose_headers.into();
    }

    pub fn set_max_age_seconds(&mut self, max_age_seconds: isize) {
        self.max_age_seconds = max_age_seconds;
    }

    pub fn set_response_vary(&mut self, response_vary: bool) {
        self.response_vary = response_vary;
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketCORSOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketCORSInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketCORSInput {
    fn operation(&self) -> &str {
        "GetBucketCORS"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketCORSInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("cors", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketCORSOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "CORSRules")]
    pub(crate) rules: Vec<CORSRule>,
}

impl GetBucketCORSOutput {
    pub fn rules(&self) -> &Vec<CORSRule> {
        &self.rules
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketCORSInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for DeleteBucketCORSInput {
    fn operation(&self) -> &str {
        "DeleteBucketCORS"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketCORSInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("cors", "".to_string())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketCORSOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct PutBucketStorageClassInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) storage_class: StorageClassType,
}
impl PutBucketStorageClassInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            storage_class: StorageClassType::default(),
        }
    }

    pub fn new_with_storage_class(bucket: impl Into<String>, storage_class: impl Into<StorageClassType>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            storage_class: storage_class.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn storage_class(&self) -> &StorageClassType {
        &self.storage_class
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = storage_class.into();
    }
}

impl InputDescriptor for PutBucketStorageClassInput {
    fn operation(&self) -> &str {
        "PutBucketStorageClass"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketStorageClassInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.header.insert(HEADER_STORAGE_CLASS, self.storage_class.as_str().to_string());
        request.query = Some(HashMap::from([("storageClass", "".to_string())]));
        request.method = HttpMethodPut;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketStorageClassOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketLocationInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketLocationInput {
    fn operation(&self) -> &str {
        "GetBucketLocation"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketLocationInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("location", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketLocationOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Region")]
    pub(crate) region: String,
    #[serde(default)]
    #[serde(rename = "ExtranetEndpoint")]
    pub(crate) extranet_endpoint: String,
    #[serde(default)]
    #[serde(rename = "IntranetEndpoint")]
    pub(crate) intranet_endpoint: String,
}

impl GetBucketLocationOutput {
    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn extranet_endpoint(&self) -> &str {
        &self.extranet_endpoint
    }

    pub fn intranet_endpoint(&self) -> &str {
        &self.intranet_endpoint
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketLifecycleInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Rules")]
    pub(crate) rules: Vec<LifecycleRule>,
    #[serde(skip)]
    pub(crate) allow_same_action_overlap: bool,
}

impl PutBucketLifecycleInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rules: vec![],
            allow_same_action_overlap: false,
        }
    }

    pub fn new_with_rules(bucket: impl Into<String>, rules: impl Into<Vec<LifecycleRule>>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rules: rules.into(),
            allow_same_action_overlap: false,
        }
    }

    pub fn add_rule(&mut self, rule: impl Into<LifecycleRule>) {
        self.rules.push(rule.into());
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn rules(&self) -> &Vec<LifecycleRule> {
        &self.rules
    }

    pub fn allow_same_action_overlap(&self) -> bool {
        self.allow_same_action_overlap
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_rules(&mut self, rules: impl Into<Vec<LifecycleRule>>) {
        self.rules = rules.into();
    }

    pub fn set_allow_same_action_overlap(&mut self, allow_same_action_overlap: bool) {
        self.allow_same_action_overlap = allow_same_action_overlap;
    }
}

impl InputDescriptor for PutBucketLifecycleInput {
    fn operation(&self) -> &str {
        "PutBucketLifecycle"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketLifecycleInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.rules.len() == 0 {
            return Err(TosError::client_error("empty lifecycle rules"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("lifecycle", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                if self.allow_same_action_overlap {
                    request.header.insert(HEADER_ALLOW_SAME_ACTION_OVERLAP, TRUE.to_string());
                }
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LifecycleRule {
    #[serde(rename = "ID")]
    pub(crate) id: String,
    #[serde(rename = "Prefix")]
    #[serde(default)]
    pub(crate) prefix: String,
    #[serde(rename = "Status")]
    pub(crate) status: StatusType,
    #[serde(rename = "Tags")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) tags: Vec<Tag>,
    #[serde(rename = "Filter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) filter: Option<LifecycleRuleFilter>,
    #[serde(rename = "Expiration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) expiration: Option<Expiration>,
    #[serde(rename = "NoncurrentVersionExpiration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) noncurrent_version_expiration: Option<NoncurrentVersionExpiration>,
    #[serde(rename = "AbortIncompleteMultipartUpload")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) abort_in_complete_multipart_upload: Option<AbortInCompleteMultipartUpload>,
    #[serde(rename = "Transitions")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) transitions: Vec<Transition>,
    #[serde(rename = "NoncurrentVersionTransitions")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) noncurrent_version_transitions: Vec<NoncurrentVersionTransition>,
    #[serde(rename = "AccessTimeTransitions")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) access_time_transitions: Vec<AccessTimeTransition>,
    #[serde(rename = "NoncurrentVersionAccessTimeTransitions")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) non_current_version_access_time_transitions: Vec<NonCurrentVersionAccessTimeTransition>,
}

impl LifecycleRule {
    pub fn new(id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.id = id.into();
        input
    }

    pub fn new_with_prefix(id: impl Into<String>, prefix: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.id = id.into();
        input.prefix = prefix.into();
        input
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn status(&self) -> &StatusType {
        &self.status
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn filter(&self) -> &Option<LifecycleRuleFilter> {
        &self.filter
    }

    pub fn expiration(&self) -> &Option<Expiration> {
        &self.expiration
    }

    pub fn noncurrent_version_expiration(&self) -> &Option<NoncurrentVersionExpiration> {
        &self.noncurrent_version_expiration
    }

    pub fn abort_in_complete_multipart_upload(&self) -> &Option<AbortInCompleteMultipartUpload> {
        &self.abort_in_complete_multipart_upload
    }

    pub fn transitions(&self) -> &Vec<Transition> {
        &self.transitions
    }

    pub fn noncurrent_version_transitions(&self) -> &Vec<NoncurrentVersionTransition> {
        &self.noncurrent_version_transitions
    }

    pub fn access_time_transitions(&self) -> &Vec<AccessTimeTransition> {
        &self.access_time_transitions
    }

    pub fn non_current_version_access_time_transitions(&self) -> &Vec<NonCurrentVersionAccessTimeTransition> {
        &self.non_current_version_access_time_transitions
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    pub fn set_prefix(&mut self, prefix: impl Into<String>) {
        self.prefix = prefix.into();
    }

    pub fn set_status(&mut self, status: impl Into<StatusType>) {
        self.status = status.into();
    }

    pub fn set_tags(&mut self, tags: impl Into<Vec<Tag>>) {
        self.tags = tags.into();
    }

    pub fn set_filter(&mut self, filter: impl Into<LifecycleRuleFilter>) {
        self.filter = Some(filter.into());
    }

    pub fn set_expiration(&mut self, expiration: impl Into<Expiration>) {
        self.expiration = Some(expiration.into());
    }

    pub fn set_noncurrent_version_expiration(&mut self, noncurrent_version_expiration: impl Into<NoncurrentVersionExpiration>) {
        self.noncurrent_version_expiration = Some(noncurrent_version_expiration.into());
    }

    pub fn set_abort_in_complete_multipart_upload(&mut self, abort_in_complete_multipart_upload: impl Into<AbortInCompleteMultipartUpload>) {
        self.abort_in_complete_multipart_upload = Some(abort_in_complete_multipart_upload.into());
    }

    pub fn set_transitions(&mut self, transitions: impl Into<Vec<Transition>>) {
        self.transitions = transitions.into();
    }

    pub fn set_noncurrent_version_transitions(&mut self, noncurrent_version_transitions: impl Into<Vec<NoncurrentVersionTransition>>) {
        self.noncurrent_version_transitions = noncurrent_version_transitions.into();
    }

    pub fn set_access_time_transitions(&mut self, access_time_transitions: impl Into<Vec<AccessTimeTransition>>) {
        self.access_time_transitions = access_time_transitions.into();
    }

    pub fn set_non_current_version_access_time_transitions(&mut self, non_current_version_access_time_transitions: impl Into<Vec<NonCurrentVersionAccessTimeTransition>>) {
        self.non_current_version_access_time_transitions = non_current_version_access_time_transitions.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Transition {
    #[serde(rename = "Date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) date_string: Option<String>,
    #[serde(skip)]
    pub(crate) date: Option<DateTime<Utc>>,
    #[serde(rename = "Days")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) days: Option<isize>,
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: StorageClassType,
}

impl Transition {
    pub fn new(storage_class: impl Into<StorageClassType>) -> Self {
        Self {
            date_string: None,
            date: None,
            days: None,
            storage_class: storage_class.into(),
        }
    }

    pub fn date(&self) -> Option<DateTime<Utc>> {
        self.date
    }

    pub fn days(&self) -> Option<isize> {
        self.days
    }

    pub fn storage_class(&self) -> &StorageClassType {
        &self.storage_class
    }

    pub fn set_date(&mut self, date: impl Into<DateTime<Utc>>) {
        let date = date.into();
        self.date_string = Some(date.format(ISO8601_DATE_FORMAT).to_string());
        self.date = Some(date);
    }

    pub fn set_days(&mut self, days: impl Into<isize>) {
        self.days = Some(days.into());
    }

    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = storage_class.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Expiration {
    #[serde(rename = "Date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) date_string: Option<String>,
    #[serde(skip)]
    pub(crate) date: Option<DateTime<Utc>>,
    #[serde(rename = "Days")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) days: Option<isize>,
}

impl Expiration {
    pub fn new_with_date(date: impl Into<DateTime<Utc>>) -> Self {
        let mut exp = Self::default();
        exp.set_date(date);
        exp
    }
    pub fn new_with_days(days: impl Into<isize>) -> Self {
        let mut exp = Self::default();
        exp.set_days(days);
        exp
    }

    pub fn date(&self) -> Option<DateTime<Utc>> {
        self.date
    }

    pub fn days(&self) -> Option<isize> {
        self.days
    }

    pub fn set_date(&mut self, date: impl Into<DateTime<Utc>>) {
        let date = date.into();
        self.date_string = Some(truncate_date_to_midnight(date));
        self.date = Some(date.into());
    }

    pub fn set_days(&mut self, days: impl Into<isize>) {
        self.days = Some(days.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NoncurrentVersionTransition {
    #[serde(rename = "NoncurrentDate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) noncurrent_date_string: Option<String>,
    #[serde(skip)]
    pub(crate) noncurrent_date: Option<DateTime<Utc>>,
    #[serde(rename = "NoncurrentDays")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) noncurrent_days: Option<isize>,
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: StorageClassType,
}

impl NoncurrentVersionTransition {
    pub fn new(storage_class: impl Into<StorageClassType>) -> Self {
        Self {
            noncurrent_date_string: None,
            noncurrent_date: None,
            noncurrent_days: None,
            storage_class: storage_class.into(),
        }
    }

    pub fn noncurrent_date(&self) -> Option<DateTime<Utc>> {
        self.noncurrent_date
    }

    pub fn noncurrent_days(&self) -> Option<isize> {
        self.noncurrent_days
    }

    pub fn storage_class(&self) -> &StorageClassType {
        &self.storage_class
    }

    pub fn set_noncurrent_date(&mut self, noncurrent_date: impl Into<DateTime<Utc>>) {
        let noncurrent_date = noncurrent_date.into();
        self.noncurrent_date_string = Some(truncate_date_to_midnight(noncurrent_date));
        self.noncurrent_date = Some(noncurrent_date);
    }

    pub fn set_noncurrent_days(&mut self, noncurrent_days: impl Into<isize>) {
        self.noncurrent_days = Some(noncurrent_days.into());
    }

    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = storage_class.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NoncurrentVersionExpiration {
    #[serde(rename = "NoncurrentDate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) noncurrent_date_string: Option<String>,
    #[serde(skip)]
    pub(crate) noncurrent_date: Option<DateTime<Utc>>,
    #[serde(rename = "NoncurrentDays")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) noncurrent_days: Option<isize>,
}
impl NoncurrentVersionExpiration {
    pub fn new_with_date(noncurrent_date: impl Into<DateTime<Utc>>) -> Self {
        let mut exp = Self::default();
        exp.set_noncurrent_date(noncurrent_date);
        exp
    }

    pub fn new_with_days(noncurrent_days: impl Into<isize>) -> Self {
        let mut exp = Self::default();
        exp.set_noncurrent_days(noncurrent_days);
        exp
    }

    pub fn noncurrent_date(&self) -> Option<DateTime<Utc>> {
        self.noncurrent_date
    }

    pub fn noncurrent_days(&self) -> Option<isize> {
        self.noncurrent_days
    }

    pub fn set_noncurrent_date(&mut self, noncurrent_date: impl Into<DateTime<Utc>>) {
        let noncurrent_date = noncurrent_date.into();
        self.noncurrent_date_string = Some(truncate_date_to_midnight(noncurrent_date));
        self.noncurrent_date = Some(noncurrent_date);
    }

    pub fn set_noncurrent_days(&mut self, noncurrent_days: impl Into<isize>) {
        self.noncurrent_days = Some(noncurrent_days.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AbortInCompleteMultipartUpload {
    #[serde(rename = "DaysAfterInitiation")]
    pub(crate) days_after_initiation: isize,
}
impl AbortInCompleteMultipartUpload {
    pub fn new(days_after_initiation: isize) -> Self {
        Self {
            days_after_initiation
        }
    }

    pub fn days_after_initiation(&self) -> isize {
        self.days_after_initiation
    }

    pub fn set_days_after_initiation(&mut self, days_after_initiation: isize) {
        self.days_after_initiation = days_after_initiation;
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LifecycleRuleFilter {
    #[serde(rename = "ObjectSizeGreaterThan")]
    #[serde(default)]
    pub(crate) object_size_greater_than: isize,
    #[serde(rename = "GreaterThanIncludeEqual")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) greater_than_include_equal: Option<StatusType>,
    #[serde(rename = "ObjectSizeLessThan")]
    #[serde(default)]
    pub(crate) object_size_less_than: isize,
    #[serde(rename = "LessThanIncludeEqual")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) less_than_include_equal: Option<StatusType>,
}

impl LifecycleRuleFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn object_size_greater_than(&self) -> isize {
        self.object_size_greater_than
    }

    pub fn greater_than_include_equal(&self) -> &Option<StatusType> {
        &self.greater_than_include_equal
    }

    pub fn object_size_less_than(&self) -> isize {
        self.object_size_less_than
    }

    pub fn less_than_include_equal(&self) -> &Option<StatusType> {
        &self.less_than_include_equal
    }

    pub fn set_object_size_greater_than(&mut self, object_size_greater_than: isize) {
        self.object_size_greater_than = object_size_greater_than;
    }

    pub fn set_greater_than_include_equal(&mut self, greater_than_include_equal: impl Into<StatusType>) {
        self.greater_than_include_equal = Some(greater_than_include_equal.into());
    }

    pub fn set_object_size_less_than(&mut self, object_size_less_than: isize) {
        self.object_size_less_than = object_size_less_than;
    }

    pub fn set_less_than_include_equal(&mut self, less_than_include_equal: impl Into<StatusType>) {
        self.less_than_include_equal = Some(less_than_include_equal.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AccessTimeTransition {
    #[serde(rename = "Days")]
    pub(crate) days: isize,
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: StorageClassType,
}
impl AccessTimeTransition {
    pub fn new(days: isize, storage_class: impl Into<StorageClassType>) -> Self {
        Self {
            days,
            storage_class: storage_class.into(),
        }
    }

    pub fn days(&self) -> isize {
        self.days
    }

    pub fn storage_class(&self) -> &StorageClassType {
        &self.storage_class
    }

    pub fn set_days(&mut self, days: isize) {
        self.days = days;
    }

    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = storage_class.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NonCurrentVersionAccessTimeTransition {
    #[serde(rename = "NoncurrentDays")]
    pub(crate) non_current_days: isize,
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: StorageClassType,
}

impl NonCurrentVersionAccessTimeTransition {
    pub fn new(non_current_days: isize, storage_class: impl Into<StorageClassType>) -> Self {
        Self {
            non_current_days,
            storage_class: storage_class.into(),
        }
    }

    pub fn non_current_days(&self) -> isize {
        self.non_current_days
    }

    pub fn storage_class(&self) -> &StorageClassType {
        &self.storage_class
    }

    pub fn set_non_current_days(&mut self, non_current_days: isize) {
        self.non_current_days = non_current_days;
    }

    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = storage_class.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketLifecycleOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketLifecycleInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketLifecycleInput {
    fn operation(&self) -> &str {
        "GetBucketLifecycle"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}


impl<B> InputTranslator<B> for GetBucketLifecycleInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("lifecycle", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketLifecycleOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Rules")]
    pub(crate) rules: Vec<LifecycleRule>,
    #[serde(skip)]
    pub(crate) allow_same_action_overlap: bool,
}

impl GetBucketLifecycleOutput {
    pub fn request_info(&self) -> &RequestInfo {
        &self.request_info
    }

    pub fn rules(&self) -> &Vec<LifecycleRule> {
        &self.rules
    }

    pub fn allow_same_action_overlap(&self) -> bool {
        self.allow_same_action_overlap
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketLifecycleInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}


impl InputDescriptor for DeleteBucketLifecycleInput {
    fn operation(&self) -> &str {
        "DeleteBucketLifecycle"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketLifecycleInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("lifecycle", "".to_string())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketLifecycleOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct PutBucketPolicyInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) policy: String,
}
impl PutBucketPolicyInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            policy: "".to_string(),
        }
    }

    pub fn new_with_policy(&self, bucket: impl Into<String>, policy: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            policy: policy.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn policy(&self) -> &str {
        &self.policy
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_policy(&mut self, policy: impl Into<String>) {
        self.policy = policy.into();
    }
}

impl InputDescriptor for PutBucketPolicyInput {
    fn operation(&self) -> &str {
        "PutBucketPolicy"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketPolicyInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.policy == "" {
            return Err(TosError::client_error("empty policy"));
        }

        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("policy", "".to_string())]));
        request.method = HttpMethodPut;
        request.header.insert(HEADER_CONTENT_MD5, base64_md5(&self.policy));
        let (body, len) = B::new(self.policy.clone().into_bytes())?;
        request.body = Some(body);
        request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketPolicyOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketPolicyInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketPolicyInput {
    fn operation(&self) -> &str {
        "GetBucketPolicy"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketPolicyInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("policy", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct GetBucketPolicyOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) policy: String,
}

impl GetBucketPolicyOutput {
    pub fn policy(&self) -> &str {
        &self.policy
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketPolicyInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for DeleteBucketPolicyInput {
    fn operation(&self) -> &str {
        "DeleteBucketPolicy"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketPolicyInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("policy", "".to_string())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketPolicyOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketMirrorBackInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Rules")]
    pub(crate) rules: Vec<MirrorBackRule>,
}

impl PutBucketMirrorBackInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rules: vec![],
        }
    }
    pub fn new_with_rules(bucket: impl Into<String>, rules: impl Into<Vec<MirrorBackRule>>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rules: rules.into(),
        }
    }

    pub fn add_rule(&mut self, rule: impl Into<MirrorBackRule>) {
        self.rules.push(rule.into());
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn rules(&self) -> &Vec<MirrorBackRule> {
        &self.rules
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_rules(&mut self, rules: impl Into<Vec<MirrorBackRule>>) {
        self.rules = rules.into();
    }
}

impl InputDescriptor for PutBucketMirrorBackInput {
    fn operation(&self) -> &str {
        "PutBucketMirrorBack"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketMirrorBackInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.rules.len() == 0 {
            return Err(TosError::client_error("empty mirror back rules"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("mirror", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MirrorBackRule {
    #[serde(rename = "ID")]
    pub(crate) id: String,
    #[serde(rename = "Condition")]
    pub(crate) condition: Condition,
    #[serde(rename = "Redirect")]
    pub(crate) redirect: Redirect,
}

impl MirrorBackRule {
    pub fn new(id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.id = id.into();
        input
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn condition(&self) -> &Condition {
        &self.condition
    }

    pub fn redirect(&self) -> &Redirect {
        &self.redirect
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    pub fn set_condition(&mut self, condition: impl Into<Condition>) {
        self.condition = condition.into();
    }

    pub fn set_redirect(&mut self, redirect: impl Into<Redirect>) {
        self.redirect = redirect.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Condition {
    #[serde(rename = "HttpCode")]
    pub(crate) http_code: isize,
    #[serde(rename = "KeyPrefix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) key_prefix: String,
    #[serde(rename = "KeySuffix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) key_suffix: String,
    #[serde(rename = "AllowHost")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) allow_host: Vec<String>,
    #[serde(rename = "HttpMethod")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) http_method: Vec<String>,
}
impl Condition {
    pub fn new(http_code: isize) -> Self {
        Self {
            http_code,
            key_prefix: "".to_string(),
            key_suffix: "".to_string(),
            allow_host: vec![],
            http_method: vec![],
        }
    }

    pub fn http_code(&self) -> isize {
        self.http_code
    }

    pub fn key_prefix(&self) -> &str {
        &self.key_prefix
    }

    pub fn key_suffix(&self) -> &str {
        &self.key_suffix
    }

    pub fn allow_host(&self) -> &Vec<String> {
        &self.allow_host
    }

    pub fn http_method(&self) -> &Vec<String> {
        &self.http_method
    }

    pub fn set_http_code(&mut self, http_code: isize) {
        self.http_code = http_code;
    }

    pub fn set_key_prefix(&mut self, key_prefix: impl Into<String>) {
        self.key_prefix = key_prefix.into();
    }

    pub fn set_key_suffix(&mut self, key_suffix: impl Into<String>) {
        self.key_suffix = key_suffix.into();
    }

    pub fn set_allow_host(&mut self, allow_host: impl Into<Vec<String>>) {
        self.allow_host = allow_host.into();
    }

    pub fn set_http_method(&mut self, http_method: impl Into<Vec<String>>) {
        self.http_method = http_method.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Redirect {
    #[serde(rename = "RedirectType")]
    pub(crate) redirect_type: RedirectType,
    #[serde(rename = "DisableUploadSourceForNoneRangeMirror")]
    #[serde(default)]
    pub(crate) fetch_source_on_redirect: bool,
    #[serde(rename = "PassQuery")]
    #[serde(default)]
    pub(crate) pass_query: bool,
    #[serde(rename = "FollowRedirect")]
    #[serde(default)]
    pub(crate) follow_redirect: bool,
    #[serde(rename = "MirrorHeader")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mirror_header: Option<MirrorHeader>,
    #[serde(rename = "PublicSource")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) public_source: Option<PublicSource>,
    #[serde(rename = "PrivateSource")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) private_source: Option<PrivateSource>,
    #[serde(rename = "Transform")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) transform: Option<Transform>,
    #[serde(rename = "FetchHeaderToMetaDataRules")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) fetch_header_to_meta_data_rules: Vec<FetchHeaderToMetaDataRule>,
    #[serde(rename = "FetchSourceOnRedirectWithQuery")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fetch_source_on_redirect_with_query: Option<bool>,
    #[serde(rename = "PassStatusCodeFromSource")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) pass_status_code_from_source: Vec<isize>,
    #[serde(rename = "PassHeaderFromSource")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) pass_header_from_source: Vec<String>,
}
impl Redirect {
    pub fn new(redirect_type: impl Into<RedirectType>, fetch_source_on_redirect: bool) -> Self {
        let mut default = Self::default();
        default.redirect_type = redirect_type.into();
        default.fetch_source_on_redirect = fetch_source_on_redirect;
        default
    }

    pub fn redirect_type(&self) -> &RedirectType {
        &self.redirect_type
    }

    pub fn fetch_source_on_redirect(&self) -> bool {
        self.fetch_source_on_redirect
    }

    pub fn pass_query(&self) -> bool {
        self.pass_query
    }

    pub fn follow_redirect(&self) -> bool {
        self.follow_redirect
    }

    pub fn mirror_header(&self) -> &Option<MirrorHeader> {
        &self.mirror_header
    }

    pub fn public_source(&self) -> &Option<PublicSource> {
        &self.public_source
    }

    pub fn private_source(&self) -> &Option<PrivateSource> {
        &self.private_source
    }

    pub fn transform(&self) -> &Option<Transform> {
        &self.transform
    }

    pub fn fetch_header_to_meta_data_rules(&self) -> &Vec<FetchHeaderToMetaDataRule> {
        &self.fetch_header_to_meta_data_rules
    }

    pub fn fetch_source_on_redirect_with_query(&self) -> Option<bool> {
        self.fetch_source_on_redirect_with_query
    }

    pub fn pass_status_code_from_source(&self) -> &Vec<isize> {
        &self.pass_status_code_from_source
    }

    pub fn pass_header_from_source(&self) -> &Vec<String> {
        &self.pass_header_from_source
    }

    pub fn set_redirect_type(&mut self, redirect_type: impl Into<RedirectType>) {
        self.redirect_type = redirect_type.into();
    }

    pub fn set_fetch_source_on_redirect(&mut self, fetch_source_on_redirect: bool) {
        self.fetch_source_on_redirect = fetch_source_on_redirect;
    }

    pub fn set_pass_query(&mut self, pass_query: bool) {
        self.pass_query = pass_query;
    }

    pub fn set_follow_redirect(&mut self, follow_redirect: bool) {
        self.follow_redirect = follow_redirect;
    }

    pub fn set_mirror_header(&mut self, mirror_header: impl Into<MirrorHeader>) {
        self.mirror_header = Some(mirror_header.into());
    }

    pub fn set_public_source(&mut self, public_source: impl Into<PublicSource>) {
        self.public_source = Some(public_source.into());
    }

    pub fn set_private_source(&mut self, private_source: impl Into<PrivateSource>) {
        self.private_source = Some(private_source.into());
    }

    pub fn set_transform(&mut self, transform: impl Into<Transform>) {
        self.transform = Some(transform.into());
    }

    pub fn set_fetch_header_to_meta_data_rules(&mut self, fetch_header_to_meta_data_rules: impl Into<Vec<FetchHeaderToMetaDataRule>>) {
        self.fetch_header_to_meta_data_rules = fetch_header_to_meta_data_rules.into();
    }

    pub fn set_fetch_source_on_redirect_with_query(&mut self, fetch_source_on_redirect_with_query: impl Into<bool>) {
        self.fetch_source_on_redirect_with_query = Some(fetch_source_on_redirect_with_query.into());
    }

    pub fn set_pass_status_code_from_source(&mut self, pass_status_code_from_source: impl Into<Vec<isize>>) {
        self.pass_status_code_from_source = pass_status_code_from_source.into();
    }

    pub fn set_pass_header_from_source(&mut self, pass_header_from_source: impl Into<Vec<String>>) {
        self.pass_header_from_source = pass_header_from_source.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MirrorHeader {
    #[serde(rename = "PassAll")]
    #[serde(default)]
    pub(crate) pass_all: bool,
    #[serde(rename = "Pass")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) pass: Vec<String>,
    #[serde(rename = "Remove")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) remove: Vec<String>,
    #[serde(rename = "Set")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) set: Vec<MirrorHeaderKeyValue>,
}
impl MirrorHeader {
    pub fn new() -> Self {
        Self {
            pass_all: false,
            pass: vec![],
            remove: vec![],
            set: vec![],
        }
    }

    pub fn pass_all(&self) -> bool {
        self.pass_all
    }

    pub fn pass(&self) -> &Vec<String> {
        &self.pass
    }

    pub fn remove(&self) -> &Vec<String> {
        &self.remove
    }

    pub fn set(&self) -> &Vec<MirrorHeaderKeyValue> {
        &self.set
    }

    pub fn set_pass_all(&mut self, pass_all: bool) {
        self.pass_all = pass_all;
    }

    pub fn set_pass(&mut self, pass: impl Into<Vec<String>>) {
        self.pass = pass.into();
    }

    pub fn set_remove(&mut self, remove: impl Into<Vec<String>>) {
        self.remove = remove.into();
    }

    pub fn set_set(&mut self, set: impl Into<Vec<MirrorHeaderKeyValue>>) {
        self.set = set.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MirrorHeaderKeyValue {
    #[serde(rename = "Key")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) key: String,
    #[serde(rename = "Value")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) value: String,
}
impl MirrorHeaderKeyValue {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PublicSource {
    #[serde(rename = "SourceEndpoint")]
    #[serde(default)]
    pub(crate) source_endpoint: SourceEndpoint,
    #[serde(rename = "FixedEndpoint")]
    #[serde(default)]
    pub(crate) fixed_endpoint: bool,
}
impl PublicSource {
    pub fn new(source_endpoint: impl Into<SourceEndpoint>) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            fixed_endpoint: false,
        }
    }

    pub fn source_endpoint(&self) -> &SourceEndpoint {
        &self.source_endpoint
    }

    pub fn fixed_endpoint(&self) -> bool {
        self.fixed_endpoint
    }

    pub fn set_source_endpoint(&mut self, source_endpoint: impl Into<SourceEndpoint>) {
        self.source_endpoint = source_endpoint.into();
    }

    pub fn set_fixed_endpoint(&mut self, fixed_endpoint: bool) {
        self.fixed_endpoint = fixed_endpoint;
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SourceEndpoint {
    #[serde(rename = "Primary")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) primary: Vec<String>,
    #[serde(rename = "Follower")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) follower: Vec<String>,
}
impl SourceEndpoint {
    pub fn new(primary: impl Into<Vec<String>>) -> Self {
        Self {
            primary: primary.into(),
            follower: vec![],
        }
    }

    pub fn primary(&self) -> &Vec<String> {
        &self.primary
    }

    pub fn follower(&self) -> &Vec<String> {
        &self.follower
    }

    pub fn set_primary(&mut self, primary: impl Into<Vec<String>>) {
        self.primary = primary.into();
    }

    pub fn set_follower(&mut self, follower: impl Into<Vec<String>>) {
        self.follower = follower.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PrivateSource {
    #[serde(rename = "SourceEndpoint")]
    pub(crate) source_endpoint: CommonSourceEndpoint,
}
impl PrivateSource {
    pub fn new(source_endpoint: impl Into<CommonSourceEndpoint>) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
        }
    }

    pub fn source_endpoint(&self) -> &CommonSourceEndpoint {
        &self.source_endpoint
    }

    pub fn set_source_endpoint(&mut self, source_endpoint: impl Into<CommonSourceEndpoint>) {
        self.source_endpoint = source_endpoint.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommonSourceEndpoint {
    #[serde(rename = "Primary")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) primary: Vec<EndpointCredentialProvider>,
    #[serde(rename = "Follower")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) follower: Vec<EndpointCredentialProvider>,
}
impl CommonSourceEndpoint {
    pub fn new(primary: impl Into<Vec<EndpointCredentialProvider>>) -> Self {
        Self {
            primary: primary.into(),
            follower: vec![],
        }
    }

    pub fn primary(&self) -> &Vec<EndpointCredentialProvider> {
        &self.primary
    }

    pub fn follower(&self) -> &Vec<EndpointCredentialProvider> {
        &self.follower
    }

    pub fn set_primary(&mut self, primary: impl Into<Vec<EndpointCredentialProvider>>) {
        self.primary = primary.into();
    }

    pub fn set_follower(&mut self, follower: impl Into<Vec<EndpointCredentialProvider>>) {
        self.follower = follower.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EndpointCredentialProvider {
    #[serde(rename = "Endpoint")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) endpoint: String,
    #[serde(rename = "BucketName")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) bucket_name: String,
    #[serde(rename = "CredentialProvider")]
    pub(crate) credential_provider: CommonCredentialProvider,
}
impl EndpointCredentialProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            bucket_name: "".to_string(),
            credential_provider: Default::default(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn credential_provider(&self) -> &CommonCredentialProvider {
        &self.credential_provider
    }

    pub fn set_endpoint(&mut self, endpoint: impl Into<String>) {
        self.endpoint = endpoint.into();
    }

    pub fn set_bucket_name(&mut self, bucket_name: impl Into<String>) {
        self.bucket_name = bucket_name.into();
    }

    pub fn set_credential_provider(&mut self, credential_provider: impl Into<CommonCredentialProvider>) {
        self.credential_provider = credential_provider.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommonCredentialProvider {
    #[serde(rename = "Role")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) role: String,
    #[serde(rename = "StaticCredential")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) static_credential: Option<CommonStaticCredential>,
    #[serde(rename = "Region")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) region: String,
}
impl CommonCredentialProvider {
    pub fn new_with_role(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            static_credential: None,
            region: "".to_string(),
        }
    }
    pub fn new_with_static_credential(static_credential: impl Into<CommonStaticCredential>, region: impl Into<String>) -> Self {
        Self {
            role: "".to_string(),
            static_credential: Some(static_credential.into()),
            region: region.into(),
        }
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn static_credential(&self) -> &Option<CommonStaticCredential> {
        &self.static_credential
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn set_role(&mut self, role: impl Into<String>) {
        self.role = role.into();
    }

    pub fn set_static_credential(&mut self, static_credential: impl Into<CommonStaticCredential>) {
        self.static_credential = Some(static_credential.into());
    }

    pub fn set_region(&mut self, region: impl Into<String>) {
        self.region = region.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CommonStaticCredential {
    #[serde(rename = "StorageVendor")]
    pub(crate) storage_vendor: String,
    #[serde(rename = "AK")]
    pub(crate) ak: String,
    #[serde(rename = "SK")]
    pub(crate) sk: String,
    #[serde(rename = "SKEncryptType")]
    #[serde(skip_serializing)]
    pub(crate) sk_encrypt_type: String,
}
impl CommonStaticCredential {
    pub fn new(storage_vendor: impl Into<String>) -> Self {
        Self {
            storage_vendor: storage_vendor.into(),
            ak: "".to_string(),
            sk: "".to_string(),
            sk_encrypt_type: "".to_string(),
        }
    }

    pub fn storage_vendor(&self) -> &str {
        &self.storage_vendor
    }

    pub fn ak(&self) -> &str {
        &self.ak
    }

    pub fn sk(&self) -> &str {
        &self.sk
    }

    pub fn sk_encrypt_type(&self) -> &str {
        &self.sk_encrypt_type
    }

    pub fn set_storage_vendor(&mut self, storage_vendor: impl Into<String>) {
        self.storage_vendor = storage_vendor.into();
    }

    pub fn set_ak(&mut self, ak: impl Into<String>) {
        self.ak = ak.into();
    }

    pub fn set_sk(&mut self, sk: impl Into<String>) {
        self.sk = sk.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Transform {
    #[serde(rename = "WithKeyPrefix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) with_key_prefix: String,
    #[serde(rename = "WithKeySuffix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) with_key_suffix: String,
    #[serde(rename = "ReplaceKeyPrefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) replace_key_prefix: Option<ReplaceKeyPrefix>,
}
impl Transform {
    pub fn new() -> Self {
        Self {
            with_key_prefix: "".to_string(),
            with_key_suffix: "".to_string(),
            replace_key_prefix: Default::default(),
        }
    }

    pub fn with_key_prefix(&self) -> &str {
        &self.with_key_prefix
    }

    pub fn with_key_suffix(&self) -> &str {
        &self.with_key_suffix
    }

    pub fn replace_key_prefix(&self) -> &Option<ReplaceKeyPrefix> {
        &self.replace_key_prefix
    }

    pub fn set_with_key_prefix(&mut self, with_key_prefix: impl Into<String>) {
        self.with_key_prefix = with_key_prefix.into();
    }

    pub fn set_with_key_suffix(&mut self, with_key_suffix: impl Into<String>) {
        self.with_key_suffix = with_key_suffix.into();
    }

    pub fn set_replace_key_prefix(&mut self, replace_key_prefix: impl Into<ReplaceKeyPrefix>) {
        self.replace_key_prefix = Some(replace_key_prefix.into());
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ReplaceKeyPrefix {
    #[serde(rename = "KeyPrefix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) key_prefix: String,
    #[serde(rename = "ReplaceWith")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) replace_with: String,
}
impl ReplaceKeyPrefix {
    pub fn new(key_prefix: impl Into<String>, replace_with: impl Into<String>) -> Self {
        Self {
            key_prefix: key_prefix.into(),
            replace_with: replace_with.into(),
        }
    }

    pub fn key_prefix(&self) -> &str {
        &self.key_prefix
    }

    pub fn replace_with(&self) -> &str {
        &self.replace_with
    }

    pub fn set_key_prefix(&mut self, key_prefix: impl Into<String>) {
        self.key_prefix = key_prefix.into();
    }

    pub fn set_replace_with(&mut self, replace_with: impl Into<String>) {
        self.replace_with = replace_with.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FetchHeaderToMetaDataRule {
    #[serde(rename = "SourceHeader")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) source_header: String,
    #[serde(rename = "MetaDataSuffix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) meta_data_suffix: String,
}
impl FetchHeaderToMetaDataRule {
    pub fn new(source_header: impl Into<String>, meta_data_suffix: impl Into<String>) -> Self {
        Self {
            source_header: source_header.into(),
            meta_data_suffix: meta_data_suffix.into(),
        }
    }

    pub fn source_header(&self) -> &str {
        &self.source_header
    }

    pub fn meta_data_suffix(&self) -> &str {
        &self.meta_data_suffix
    }

    pub fn set_source_header(&mut self, source_header: impl Into<String>) {
        self.source_header = source_header.into();
    }

    pub fn set_meta_data_suffix(&mut self, meta_data_suffix: impl Into<String>) {
        self.meta_data_suffix = meta_data_suffix.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketMirrorBackOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketMirrorBackInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for GetBucketMirrorBackInput {
    fn operation(&self) -> &str {
        "GetBucketMirrorBack"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketMirrorBackInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("mirror", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketMirrorBackOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Rules")]
    pub(crate) rules: Vec<MirrorBackRule>,
}

impl GetBucketMirrorBackOutput {
    pub fn rules(&self) -> &Vec<MirrorBackRule> {
        &self.rules
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketMirrorBackInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for DeleteBucketMirrorBackInput {
    fn operation(&self) -> &str {
        "DeleteBucketMirrorBack"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketMirrorBackInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("mirror", "".to_string())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketMirrorBackOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, AclHeader, Serialize, GenericInput)]
#[enable_grant_write]
pub struct PutBucketACLInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(skip)]
    pub(crate) acl: Option<ACLType>,
    #[serde(skip)]
    pub(crate) grant_full_control: String,
    #[serde(skip)]
    pub(crate) grant_read: String,
    #[serde(skip)]
    pub(crate) grant_read_acp: String,
    #[serde(skip)]
    pub(crate) grant_write: String,
    #[serde(skip)]
    pub(crate) grant_write_acp: String,
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(rename = "Grants")]
    pub(crate) grants: Vec<Grant>,
    #[serde(rename = "BucketAclDelivered")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub(crate) bucket_acl_delivered: bool,
}

impl PutBucketACLInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input
    }

    pub fn new_with_acl(bucket: impl Into<String>, acl: impl Into<ACLType>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.acl = Some(acl.into());
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn grants(&self) -> &Vec<Grant> {
        &self.grants
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_owner(&mut self, owner: impl Into<Owner>) {
        self.owner = owner.into();
    }
    pub fn set_grants(&mut self, grants: impl Into<Vec<Grant>>) {
        self.grants = grants.into();
    }

    pub fn bucket_acl_delivered(&self) -> bool {
        self.bucket_acl_delivered
    }
    pub fn set_bucket_acl_delivered(&mut self, bucket_acl_delivered: bool) {
        self.bucket_acl_delivered = bucket_acl_delivered;
    }
}

impl InputDescriptor for PutBucketACLInput {
    fn operation(&self) -> &str {
        "PutBucketACL"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketACLInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodPut;
        if self.acl.is_some() && self.grants.len() > 0 {
            return Err(TosError::client_error("both acl and grants are set for put bucket acl"));
        }

        if self.acl.is_some() {
            set_acl_header(&mut request.header, self);
        } else if self.grants.len() == 0 {
            return Err(TosError::client_error("neither acl nor grants is set for put bucket acl"));
        } else if self.owner.id == "" {
            return Err(TosError::client_error("empty owner id for put bucket acl"));
        } else {
            match serde_json::to_string(self) {
                Err(e) => return Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
                Ok(json) => {
                    let (body, len) = B::new(json.into_bytes())?;
                    request.body = Some(body);
                    request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                }
            }
        }
        request.query = Some(HashMap::from([("acl", "".to_string())]));
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketACLOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketACLInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for GetBucketACLInput {
    fn operation(&self) -> &str {
        "GetBucketACL"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketACLInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("acl", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketACLOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "Grants")]
    pub(crate) grants: Vec<Grant>,
    #[serde(default)]
    #[serde(rename = "BucketAclDelivered")]
    pub(crate) bucket_acl_delivered: bool,
}

impl GetBucketACLOutput {
    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn grants(&self) -> &Vec<Grant> {
        &self.grants
    }

    pub fn bucket_acl_delivered(&self) -> bool {
        self.bucket_acl_delivered
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketReplicationInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Role")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) role: String,
    #[serde(rename = "Rules")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) rules: Vec<ReplicationRule>,
}

impl PutBucketReplicationInput {
    pub fn new(bucket: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            role: role.into(),
            rules: vec![],
        }
    }

    pub fn new_with_rules(bucket: impl Into<String>, role: impl Into<String>, rules: impl Into<Vec<ReplicationRule>>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            role: role.into(),
            rules: rules.into(),
        }
    }

    pub fn add_rule(&mut self, rule: impl Into<ReplicationRule>) {
        self.rules.push(rule.into());
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn rules(&self) -> &Vec<ReplicationRule> {
        &self.rules
    }

    pub fn role(&self) -> &str {
        &self.role
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_rules(&mut self, rules: impl Into<Vec<ReplicationRule>>) {
        self.rules = rules.into();
    }
    pub fn set_role(&mut self, role: impl Into<String>) {
        self.role = role.into();
    }
}

impl InputDescriptor for PutBucketReplicationInput {
    fn operation(&self) -> &str {
        "PutBucketReplication"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketReplicationInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.rules.len() == 0 {
            return Err(TosError::client_error("empty replication rules"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("replication", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

fn default_historical_object_replication() -> StatusType {
    StatusType::StatusDisabled
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ReplicationRule {
    #[serde(rename = "ID")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) id: String,
    #[serde(rename = "Status")]
    #[serde(default)]
    pub(crate) status: StatusType,
    #[serde(rename = "PrefixSet")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) prefix_set: Vec<String>,
    #[serde(rename = "Tags")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) tags: Vec<Tag>,
    #[serde(rename = "Destination")]
    #[serde(default)]
    pub(crate) destination: Destination,
    #[serde(rename = "HistoricalObjectReplication")]
    #[serde(default = "default_historical_object_replication")]
    pub(crate) historical_object_replication: StatusType,
    #[serde(rename = "AccessControlTranslation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub(crate) access_control_translation: Option<AccessControlTranslation>,
    #[serde(rename = "Progress")]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub(crate) progress: Option<Progress>,
}

impl ReplicationRule {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: Default::default(),
            prefix_set: vec![],
            tags: vec![],
            destination: Default::default(),
            historical_object_replication: StatusType::StatusDisabled,
            access_control_translation: None,
            progress: None,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn status(&self) -> &StatusType {
        &self.status
    }

    pub fn prefix_set(&self) -> &Vec<String> {
        &self.prefix_set
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    pub fn historical_object_replication(&self) -> &StatusType {
        &self.historical_object_replication
    }

    pub fn access_control_translation(&self) -> &Option<AccessControlTranslation> {
        &self.access_control_translation
    }
    pub fn progress(&self) -> &Option<Progress> {
        &self.progress
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    pub fn set_status(&mut self, status: impl Into<StatusType>) {
        self.status = status.into();
    }

    pub fn set_prefix_set(&mut self, prefix_set: impl Into<Vec<String>>) {
        self.prefix_set = prefix_set.into();
    }

    pub fn set_tags(&mut self, tags: impl Into<Vec<Tag>>) {
        self.tags = tags.into();
    }

    pub fn set_destination(&mut self, destination: impl Into<Destination>) {
        self.destination = destination.into();
    }

    pub fn set_historical_object_replication(&mut self, historical_object_replication: impl Into<StatusType>) {
        self.historical_object_replication = historical_object_replication.into();
    }

    pub fn set_access_control_translation(&mut self, access_control_translation: impl Into<AccessControlTranslation>) {
        self.access_control_translation = Some(access_control_translation.into());
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Destination {
    #[serde(rename = "Bucket")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) bucket: String,
    #[serde(rename = "Location")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) location: String,
    #[serde(rename = "StorageClass")]
    #[serde(default)]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(rename = "StorageClassInheritDirective")]
    #[serde(default)]
    pub(crate) storage_class_inherit_directive: Option<StorageClassInheritDirectiveType>,
}

impl Destination {
    pub fn new(bucket: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            location: location.into(),
            storage_class: None,
            storage_class_inherit_directive: None,
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }

    pub fn storage_class_inherit_directive(&self) -> &Option<StorageClassInheritDirectiveType> {
        &self.storage_class_inherit_directive
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_location(&mut self, location: impl Into<String>) {
        self.location = location.into();
    }

    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = Some(storage_class.into());
    }

    pub fn set_storage_class_inherit_directive(&mut self, storage_class_inherit_directive: impl Into<StorageClassInheritDirectiveType>) {
        self.storage_class_inherit_directive = Some(storage_class_inherit_directive.into());
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AccessControlTranslation {
    #[serde(rename = "Owner")]
    #[serde(default)]
    pub(crate) owner: String,
}
impl AccessControlTranslation {
    pub fn new(owner: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn set_owner(&mut self, owner: impl Into<String>) {
        self.owner = owner.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Progress {
    #[serde(rename = "HistoricalObject")]
    #[serde(default)]
    pub(crate) historical_object: f64,
    #[serde(rename = "NewObject")]
    #[serde(default)]
    pub(crate) new_object: String,
}
impl Progress {
    pub fn historical_object(&self) -> f64 {
        self.historical_object
    }

    pub fn new_object(&self) -> &str {
        &self.new_object
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketReplicationOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketReplicationInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) rule_id: String,
}

impl GetBucketReplicationInput {
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    pub fn set_rule_id(&mut self, rule_id: impl Into<String>) {
        self.rule_id = rule_id.into();
    }
}

impl InputDescriptor for GetBucketReplicationInput {
    fn operation(&self) -> &str {
        "GetBucketReplication"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketReplicationInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        let mut query = HashMap::with_capacity(2);
        query.insert("replication", "".to_string());
        if self.rule_id != "" {
            query.insert("rule-id", self.rule_id.to_string());
        }
        request.query = Some(query);
        request.method = HttpMethodGet;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketReplicationOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Role")]
    #[serde(default)]
    pub(crate) role: String,
    #[serde(rename = "Rules")]
    #[serde(default)]
    pub(crate) rules: Vec<ReplicationRule>,
}

impl GetBucketReplicationOutput {
    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn rules(&self) -> &Vec<ReplicationRule> {
        &self.rules
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketReplicationInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for DeleteBucketReplicationInput {
    fn operation(&self) -> &str {
        "DeleteBucketReplication"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketReplicationInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("replication", "".to_string())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketReplicationOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, Serialize, GenericInput)]
pub struct PutBucketVersioningInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Status")]
    pub(crate) status: VersioningStatusType,
}

impl PutBucketVersioningInput {
    pub fn new_with_status(bucket: impl Into<String>, status: impl Into<VersioningStatusType>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            status: status.into(),
        }
    }

    pub fn status(&self) -> &VersioningStatusType {
        &self.status
    }

    pub fn set_status(&mut self, status: impl Into<VersioningStatusType>) {
        self.status = status.into();
    }
}

impl InputDescriptor for PutBucketVersioningInput {
    fn operation(&self) -> &str {
        "PutBucketVersioning"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketVersioningInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("versioning", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketVersioningOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketVersioningInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketVersioningInput {
    fn operation(&self) -> &str {
        "GetBucketVersioning"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketVersioningInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("versioning", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketVersioningOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Status")]
    #[serde(default)]
    pub(crate) status: VersioningStatusType,
}
impl GetBucketVersioningOutput {
    pub fn status(&self) -> &VersioningStatusType {
        &self.status
    }
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, Serialize, GenericInput)]
pub struct PutBucketWebsiteInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "RedirectAllRequestsTo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) redirect_all_requests_to: Option<RedirectAllRequestsTo>,
    #[serde(rename = "IndexDocument")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) index_document: Option<IndexDocument>,
    #[serde(rename = "ErrorDocument")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_document: Option<ErrorDocument>,
    #[serde(rename = "RoutingRules")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) routing_rules: Vec<RoutingRule>,
}

impl PutBucketWebsiteInput {
    pub fn add_rule(&mut self, routing_rule: impl Into<RoutingRule>) {
        self.routing_rules.push(routing_rule.into());
    }
    pub fn redirect_all_requests_to(&self) -> &Option<RedirectAllRequestsTo> {
        &self.redirect_all_requests_to
    }

    pub fn index_document(&self) -> &Option<IndexDocument> {
        &self.index_document
    }

    pub fn error_document(&self) -> &Option<ErrorDocument> {
        &self.error_document
    }

    pub fn routing_rules(&self) -> &Vec<RoutingRule> {
        &self.routing_rules
    }

    pub fn set_redirect_all_requests_to(&mut self, redirect_all_requests_to: impl Into<RedirectAllRequestsTo>) {
        self.redirect_all_requests_to = Some(redirect_all_requests_to.into());
    }

    pub fn set_index_document(&mut self, index_document: impl Into<IndexDocument>) {
        self.index_document = Some(index_document.into());
    }

    pub fn set_error_document(&mut self, error_document: impl Into<ErrorDocument>) {
        self.error_document = Some(error_document.into());
    }

    pub fn set_routing_rules(&mut self, routing_rules: impl Into<Vec<RoutingRule>>) {
        self.routing_rules = routing_rules.into();
    }
}

impl InputDescriptor for PutBucketWebsiteInput {
    fn operation(&self) -> &str {
        "PutBucketWebsite"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketWebsiteInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("website", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RedirectAllRequestsTo {
    #[serde(rename = "HostName")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) host_name: String,
    #[serde(rename = "Protocol")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub(crate) protocol: Option<ProtocolType>,
}
impl RedirectAllRequestsTo {
    pub fn new(host_name: impl Into<String>, protocol: impl Into<ProtocolType>) -> Self {
        Self {
            host_name: host_name.into(),
            protocol: Some(protocol.into()),
        }
    }

    pub fn host_name(&self) -> &str {
        &self.host_name
    }

    pub fn protocol(&self) -> &Option<ProtocolType> {
        &self.protocol
    }

    pub fn set_host_name(&mut self, host_name: impl Into<String>) {
        self.host_name = host_name.into();
    }

    pub fn set_protocol(&mut self, protocol: impl Into<ProtocolType>) {
        self.protocol = Some(protocol.into());
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IndexDocument {
    #[serde(rename = "Suffix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) suffix: String,
    #[serde(rename = "ForbiddenSubDir")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    #[serde(default)]
    pub(crate) forbidden_sub_dir: bool,
}
impl IndexDocument {
    pub fn new(suffix: impl Into<String>) -> Self {
        Self {
            suffix: suffix.into(),
            forbidden_sub_dir: false,
        }
    }

    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    pub fn forbidden_sub_dir(&self) -> bool {
        self.forbidden_sub_dir
    }

    pub fn set_suffix(&mut self, suffix: impl Into<String>) {
        self.suffix = suffix.into();
    }

    pub fn set_forbidden_sub_dir(&mut self, forbidden_sub_dir: bool) {
        self.forbidden_sub_dir = forbidden_sub_dir;
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ErrorDocument {
    #[serde(rename = "Key")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) key: String,
}
impl ErrorDocument {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoutingRule {
    #[serde(rename = "Condition")]
    #[serde(default)]
    pub(crate) condition: RoutingRuleCondition,
    #[serde(rename = "Redirect")]
    #[serde(default)]
    pub(crate) redirect: RoutingRuleRedirect,
}
impl RoutingRule {
    pub fn new(condition: impl Into<RoutingRuleCondition>, redirect: impl Into<RoutingRuleRedirect>) -> Self {
        Self {
            condition: condition.into(),
            redirect: redirect.into(),
        }
    }

    pub fn condition(&self) -> &RoutingRuleCondition {
        &self.condition
    }

    pub fn redirect(&self) -> &RoutingRuleRedirect {
        &self.redirect
    }
    pub fn set_condition(&mut self, condition: impl Into<RoutingRuleCondition>) {
        self.condition = condition.into();
    }

    pub fn set_redirect(&mut self, redirect: impl Into<RoutingRuleRedirect>) {
        self.redirect = redirect.into();
    }
}

pub(crate) fn is_non_positive(x: &isize) -> bool {
    *x <= 0
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoutingRuleCondition {
    #[serde(rename = "KeyPrefixEquals")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) key_prefix_equals: String,
    #[serde(rename = "HttpErrorCodeReturnedEquals")]
    #[serde(skip_serializing_if = "is_non_positive")]
    #[serde(default)]
    pub(crate) http_error_code_returned_equals: isize,
}
impl RoutingRuleCondition {
    pub fn new(key_prefix_equals: impl Into<String>) -> Self {
        Self {
            key_prefix_equals: key_prefix_equals.into(),
            http_error_code_returned_equals: -1,
        }
    }

    pub fn key_prefix_equals(&self) -> &str {
        &self.key_prefix_equals
    }

    pub fn http_error_code_returned_equals(&self) -> isize {
        self.http_error_code_returned_equals
    }

    pub fn set_key_prefix_equals(&mut self, key_prefix_equals: impl Into<String>) {
        self.key_prefix_equals = key_prefix_equals.into();
    }

    pub fn set_http_error_code_returned_equals(&mut self, http_error_code_returned_equals: isize) {
        self.http_error_code_returned_equals = http_error_code_returned_equals;
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoutingRuleRedirect {
    #[serde(rename = "Protocol")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub(crate) protocol: Option<ProtocolType>,
    #[serde(rename = "HostName")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) host_name: String,
    #[serde(rename = "ReplaceKeyPrefixWith")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) replace_key_prefix_with: String,
    #[serde(rename = "ReplaceKeyWith")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) replace_key_with: String,
    #[serde(rename = "HttpRedirectCode")]
    #[serde(skip_serializing_if = "is_non_positive")]
    #[serde(default)]
    pub(crate) http_redirect_code: isize,
}
impl RoutingRuleRedirect {
    pub fn new() -> Self {
        Self {
            protocol: None,
            host_name: "".to_string(),
            replace_key_prefix_with: "".to_string(),
            replace_key_with: "".to_string(),
            http_redirect_code: -1,
        }
    }

    pub fn protocol(&self) -> &Option<ProtocolType> {
        &self.protocol
    }

    pub fn host_name(&self) -> &str {
        &self.host_name
    }

    pub fn replace_key_prefix_with(&self) -> &str {
        &self.replace_key_prefix_with
    }

    pub fn replace_key_with(&self) -> &str {
        &self.replace_key_with
    }

    pub fn http_redirect_code(&self) -> isize {
        self.http_redirect_code
    }

    pub fn set_protocol(&mut self, protocol: impl Into<ProtocolType>) {
        self.protocol = Some(protocol.into());
    }

    pub fn set_host_name(&mut self, host_name: impl Into<String>) {
        self.host_name = host_name.into();
    }

    pub fn set_replace_key_prefix_with(&mut self, replace_key_prefix_with: impl Into<String>) {
        self.replace_key_prefix_with = replace_key_prefix_with.into();
    }

    pub fn set_replace_key_with(&mut self, replace_key_with: impl Into<String>) {
        self.replace_key_with = replace_key_with.into();
    }

    pub fn set_http_redirect_code(&mut self, http_redirect_code: isize) {
        self.http_redirect_code = http_redirect_code;
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketWebsiteOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketWebsiteInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketWebsiteInput {
    fn operation(&self) -> &str {
        "GetBucketWebsite"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketWebsiteInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("website", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketWebsiteOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "RedirectAllRequestsTo")]
    #[serde(default)]
    pub(crate) redirect_all_requests_to: Option<RedirectAllRequestsTo>,
    #[serde(rename = "IndexDocument")]
    #[serde(default)]
    pub(crate) index_document: Option<IndexDocument>,
    #[serde(rename = "ErrorDocument")]
    #[serde(default)]
    pub(crate) error_document: Option<ErrorDocument>,
    #[serde(rename = "RoutingRules")]
    #[serde(default)]
    pub(crate) routing_rules: Vec<RoutingRule>,
}
impl GetBucketWebsiteOutput {
    pub fn redirect_all_requests_to(&self) -> &Option<RedirectAllRequestsTo> {
        &self.redirect_all_requests_to
    }

    pub fn index_document(&self) -> &Option<IndexDocument> {
        &self.index_document
    }

    pub fn error_document(&self) -> &Option<ErrorDocument> {
        &self.error_document
    }

    pub fn routing_rules(&self) -> &Vec<RoutingRule> {
        &self.routing_rules
    }
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketWebsiteInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for DeleteBucketWebsiteInput {
    fn operation(&self) -> &str {
        "DeleteBucketWebsite"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketWebsiteInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("website", "".to_string())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketWebsiteOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketCustomDomainInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "CustomDomainRule")]
    pub(crate) rule: CustomDomainRule,
}

impl PutBucketCustomDomainInput {
    pub fn new(bucket: impl Into<String>, rule: impl Into<CustomDomainRule>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rule: rule.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn rule(&self) -> &CustomDomainRule {
        &self.rule
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_rule(&mut self, rule: impl Into<CustomDomainRule>) {
        self.rule = rule.into();
    }
}

impl InputDescriptor for PutBucketCustomDomainInput {
    fn operation(&self) -> &str {
        "PutBucketCustomDomain"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketCustomDomainInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("customdomain", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomDomainRule {
    #[serde(rename = "CertId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) cert_id: String,
    #[serde(rename = "CertStatus")]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub(crate) cert_status: CertStatusType,
    #[serde(rename = "Domain")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) domain: String,
    #[serde(rename = "Forbidden")]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub(crate) forbidden: bool,
    #[serde(rename = "ForbiddenReason")]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub(crate) forbidden_reason: String,
    #[serde(rename = "Protocol")]
    #[serde(default)]
    pub(crate) protocol: AuthProtocolType,
}

impl CustomDomainRule {
    pub fn new(domain: impl Into<String>, protocol: impl Into<AuthProtocolType>) -> Self {
        Self {
            cert_id: "".to_string(),
            cert_status: Default::default(),
            domain: domain.into(),
            forbidden: false,
            forbidden_reason: "".to_string(),
            protocol: protocol.into(),
        }
    }

    pub fn cert_id(&self) -> &str {
        &self.cert_id
    }

    pub fn cert_status(&self) -> &CertStatusType {
        &self.cert_status
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn forbidden(&self) -> bool {
        self.forbidden
    }

    pub fn forbidden_reason(&self) -> &str {
        &self.forbidden_reason
    }

    pub fn protocol(&self) -> &AuthProtocolType {
        &self.protocol
    }

    pub fn set_cert_id(&mut self, cert_id: impl Into<String>) {
        self.cert_id = cert_id.into();
    }

    pub fn set_domain(&mut self, domain: impl Into<String>) {
        self.domain = domain.into();
    }

    pub fn set_protocol(&mut self, protocol: impl Into<AuthProtocolType>) {
        self.protocol = protocol.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketCustomDomainOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct ListBucketCustomDomainInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for ListBucketCustomDomainInput {
    fn operation(&self) -> &str {
        "ListBucketCustomDomain"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for ListBucketCustomDomainInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("customdomain", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListBucketCustomDomainOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "CustomDomainRules")]
    pub(crate) rules: Vec<CustomDomainRule>,
}
impl ListBucketCustomDomainOutput {
    pub fn rules(&self) -> &Vec<CustomDomainRule> {
        &self.rules
    }
}

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct DeleteBucketCustomDomainInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) domain: String,
}

impl DeleteBucketCustomDomainInput {
    pub fn new(bucket: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            domain: domain.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_domain(&mut self, domain: impl Into<String>) {
        self.domain = domain.into();
    }
}

impl InputDescriptor for DeleteBucketCustomDomainInput {
    fn operation(&self) -> &str {
        "DeleteBucketCustomDomain"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketCustomDomainInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("customdomain", self.domain.clone())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketCustomDomainOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketRealTimeLogInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "RealTimeLogConfiguration")]
    pub(crate) configuration: RealTimeLogConfiguration,
}

impl PutBucketRealTimeLogInput {
    pub fn new(bucket: impl Into<String>, configuration: impl Into<RealTimeLogConfiguration>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            configuration: configuration.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn configuration(&self) -> &RealTimeLogConfiguration {
        &self.configuration
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_configuration(&mut self, configuration: impl Into<RealTimeLogConfiguration>) {
        self.configuration = configuration.into();
    }
}

impl InputDescriptor for PutBucketRealTimeLogInput {
    fn operation(&self) -> &str {
        "PutBucketRealTimeLog"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketRealTimeLogInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("realtimeLog", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RealTimeLogConfiguration {
    #[serde(rename = "Role")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) role: String,
    #[serde(rename = "AccessLogConfiguration")]
    #[serde(default)]
    pub(crate) configuration: AccessLogConfiguration,
}
impl RealTimeLogConfiguration {
    pub fn new(role: impl Into<String>, configuration: impl Into<AccessLogConfiguration>) -> Self {
        Self {
            role: role.into(),
            configuration: configuration.into(),
        }
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn configuration(&self) -> &AccessLogConfiguration {
        &self.configuration
    }

    pub fn set_role(&mut self, role: impl Into<String>) {
        self.role = role.into();
    }

    pub fn set_configuration(&mut self, configuration: impl Into<AccessLogConfiguration>) {
        self.configuration = configuration.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AccessLogConfiguration {
    #[serde(rename = "UseServiceTopic")]
    #[serde(default)]
    pub(crate) use_service_topic: bool,
    #[serde(rename = "TLSProjectID")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) tls_project_id: String,
    #[serde(rename = "TLSTopicID")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) tls_topic_id: String,
    #[serde(rename = "TLSDashboardID")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) tls_dashboard_id: String,
    #[serde(rename = "TTL")]
    #[serde(skip_serializing_if = "is_non_positive")]
    #[serde(default)]
    pub(crate) ttl: isize,
}

impl AccessLogConfiguration {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn use_service_topic(&self) -> bool {
        self.use_service_topic
    }

    pub fn tls_project_id(&self) -> &str {
        &self.tls_project_id
    }

    pub fn tls_topic_id(&self) -> &str {
        &self.tls_topic_id
    }

    pub fn tls_dashboard_id(&self) -> &str {
        &self.tls_dashboard_id
    }

    pub fn ttl(&self) -> isize {
        self.ttl
    }

    pub fn set_use_service_topic(&mut self, use_service_topic: bool) {
        self.use_service_topic = use_service_topic;
    }

    pub fn set_tls_project_id(&mut self, tls_project_id: impl Into<String>) {
        self.tls_project_id = tls_project_id.into();
    }

    pub fn set_tls_topic_id(&mut self, tls_topic_id: impl Into<String>) {
        self.tls_topic_id = tls_topic_id.into();
    }

    pub fn set_tls_dashboard_id(&mut self, tls_dashboard_id: impl Into<String>) {
        self.tls_dashboard_id = tls_dashboard_id.into();
    }

    pub fn set_ttl(&mut self, ttl: isize) {
        self.ttl = ttl;
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketRealTimeLogOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketRealTimeLogInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketRealTimeLogInput {
    fn operation(&self) -> &str {
        "GetBucketRealTimeLog"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketRealTimeLogInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("realtimeLog", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketRealTimeLogOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "RealTimeLogConfiguration")]
    pub(crate) configuration: RealTimeLogConfiguration,
}

impl GetBucketRealTimeLogOutput {
    pub fn configuration(&self) -> &RealTimeLogConfiguration {
        &self.configuration
    }
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketRealTimeLogInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for DeleteBucketRealTimeLogInput {
    fn operation(&self) -> &str {
        "DeleteBucketRealTimeLog"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketRealTimeLogInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("realtimeLog", "".to_string())]));
        request.method = HttpMethodDelete;
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketRealTimeLogOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketRenameInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "RenameEnable")]
    pub(crate) rename_enable: bool,
}
impl PutBucketRenameInput {
    pub fn new(bucket: impl Into<String>, rename_enable: bool) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rename_enable,
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn rename_enable(&self) -> bool {
        self.rename_enable
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_rename_enable(&mut self, rename_enable: bool) {
        self.rename_enable = rename_enable;
    }
}

impl InputDescriptor for PutBucketRenameInput {
    fn operation(&self) -> &str {
        "PutBucketRename"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketRenameInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("rename", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketRenameOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketRenameInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for GetBucketRenameInput {
    fn operation(&self) -> &str {
        "GetBucketRename"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketRenameInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        request.query = Some(HashMap::from([("rename", "".to_string())]));
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketRenameOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "RenameEnable")]
    #[serde(default)]
    pub(crate) rename_enable: bool,
}
impl GetBucketRenameOutput {
    pub fn rename_enable(&self) -> bool {
        self.rename_enable
    }
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketRenameInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for DeleteBucketRenameInput {
    fn operation(&self) -> &str {
        "DeleteBucketRename"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketRenameInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodDelete;
        request.query = Some(HashMap::from([("rename", "".to_string())]));
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketRenameOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketEncryptionInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Rule")]
    pub(crate) rule: BucketEncryptionRule,
}
impl PutBucketEncryptionInput {
    pub fn new(bucket: impl Into<String>, rule: impl Into<BucketEncryptionRule>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rule: rule.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn rule(&self) -> &BucketEncryptionRule {
        &self.rule
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_rule(&mut self, rule: impl Into<BucketEncryptionRule>) {
        self.rule = rule.into();
    }
}

impl InputDescriptor for PutBucketEncryptionInput {
    fn operation(&self) -> &str {
        "PutBucketEncryption"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketEncryptionInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("encryption", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BucketEncryptionRule {
    #[serde(rename = "ApplyServerSideEncryptionByDefault")]
    pub(crate) apply_server_side_encryption_by_default: ApplyServerSideEncryptionByDefault,
}
impl BucketEncryptionRule {
    pub fn new(apply_server_side_encryption_by_default: impl Into<ApplyServerSideEncryptionByDefault>) -> Self {
        Self {
            apply_server_side_encryption_by_default: apply_server_side_encryption_by_default.into(),
        }
    }

    pub fn apply_server_side_encryption_by_default(&self) -> &ApplyServerSideEncryptionByDefault {
        &self.apply_server_side_encryption_by_default
    }

    pub fn set_apply_server_side_encryption_by_default(&mut self, apply_server_side_encryption_by_default: impl Into<ApplyServerSideEncryptionByDefault>) {
        self.apply_server_side_encryption_by_default = apply_server_side_encryption_by_default.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ApplyServerSideEncryptionByDefault {
    #[serde(rename = "SSEAlgorithm")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) sse_algorithm: String,
    #[serde(rename = "KMSMasterKeyID")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) kms_master_key_id: String,
}
impl ApplyServerSideEncryptionByDefault {
    pub fn new(sse_algorithm: impl Into<String>, kms_master_key_id: impl Into<String>) -> Self {
        Self {
            sse_algorithm: sse_algorithm.into(),
            kms_master_key_id: kms_master_key_id.into(),
        }
    }

    pub fn sse_algorithm(&self) -> &str {
        &self.sse_algorithm
    }

    pub fn kms_master_key_id(&self) -> &str {
        &self.kms_master_key_id
    }

    pub fn set_kms_master_key_id(&mut self, kms_master_key_id: impl Into<String>) {
        self.kms_master_key_id = kms_master_key_id.into();
    }

    pub fn set_sse_algorithm(&mut self, sse_algorithm: impl Into<String>) {
        self.sse_algorithm = sse_algorithm.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketEncryptionOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketEncryptionInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketEncryptionInput {
    fn operation(&self) -> &str {
        "GetBucketEncryption"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketEncryptionInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        request.query = Some(HashMap::from([("encryption", "".to_string())]));
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketEncryptionOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Rule")]
    pub(crate) rule: BucketEncryptionRule,
}
impl GetBucketEncryptionOutput {
    pub fn rule(&self) -> &BucketEncryptionRule {
        &self.rule
    }
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketEncryptionInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for DeleteBucketEncryptionInput {
    fn operation(&self) -> &str {
        "DeleteBucketEncryption"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketEncryptionInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodDelete;
        request.query = Some(HashMap::from([("encryption", "".to_string())]));
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketEncryptionOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketTaggingInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "TagSet")]
    pub(crate) tag_set: TagSet,
}

impl PutBucketTaggingInput {
    pub fn new(bucket: impl Into<String>, tag_set: impl Into<TagSet>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            tag_set: tag_set.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn tag_set(&self) -> &TagSet {
        &self.tag_set
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_tag_set(&mut self, tag_set: impl Into<TagSet>) {
        self.tag_set = tag_set.into();
    }
}

impl InputDescriptor for PutBucketTaggingInput {
    fn operation(&self) -> &str {
        "PutBucketTagging"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketTaggingInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("tagging", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketTaggingOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketTaggingInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}
impl InputDescriptor for GetBucketTaggingInput {
    fn operation(&self) -> &str {
        "GetBucketTagging"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketTaggingInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        request.query = Some(HashMap::from([("tagging", "".to_string())]));
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketTaggingOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "TagSet")]
    pub(crate) tag_set: TagSet,
}
impl GetBucketTaggingOutput {
    pub fn tag_set(&self) -> &TagSet {
        &self.tag_set
    }
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DeleteBucketTaggingInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for DeleteBucketTaggingInput {
    fn operation(&self) -> &str {
        "DeleteBucketTagging"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketTaggingInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodDelete;
        request.query = Some(HashMap::from([("tagging", "".to_string())]));
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketTaggingOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketNotificationType2Input {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Rules")]
    pub(crate) rules: Vec<NotificationRule>,
    #[serde(rename = "Version")]
    pub(crate) version: String,
}
impl PutBucketNotificationType2Input {
    pub fn new(bucket: impl Into<String>, rules: impl Into<Vec<NotificationRule>>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            rules: rules.into(),
            version: "".to_string(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn rules(&self) -> &Vec<NotificationRule> {
        &self.rules
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn add_rule(&mut self, rules: impl Into<NotificationRule>) {
        self.rules.push(rules.into());
    }
    pub fn set_rules(&mut self, rules: impl Into<Vec<NotificationRule>>) {
        self.rules = rules.into();
    }

    pub fn set_version(&mut self, version: impl Into<String>) {
        self.version = version.into();
    }
}

impl InputDescriptor for PutBucketNotificationType2Input {
    fn operation(&self) -> &str {
        "PutBucketNotificationType2"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketNotificationType2Input
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                request.query = Some(HashMap::from([("notification_v2", "".to_string())]));
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationRule {
    #[serde(rename = "RuleId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) rule_id: String,
    #[serde(rename = "Events")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) events: Vec<String>,
    #[serde(rename = "Filter")]
    pub(crate) filter: NotificationFilter,
    #[serde(rename = "Destination")]
    pub(crate) destination: NotificationDestination,
}
impl NotificationRule {
    pub fn new(rule_id: impl Into<String>) -> Self {
        Self {
            rule_id: rule_id.into(),
            events: vec![],
            filter: Default::default(),
            destination: Default::default(),
        }
    }

    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    pub fn events(&self) -> &Vec<String> {
        &self.events
    }

    pub fn filter(&self) -> &NotificationFilter {
        &self.filter
    }

    pub fn destination(&self) -> &NotificationDestination {
        &self.destination
    }

    pub fn set_rule_id(&mut self, rule_id: impl Into<String>) {
        self.rule_id = rule_id.into();
    }

    pub fn set_events(&mut self, events: impl Into<Vec<String>>) {
        self.events = events.into();
    }

    pub fn set_filter(&mut self, filter: impl Into<NotificationFilter>) {
        self.filter = filter.into();
    }

    pub fn set_destination(&mut self, destination: impl Into<NotificationDestination>) {
        self.destination = destination.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationFilter {
    #[serde(rename = "TOSKey")]
    pub(crate) tos_key: NotificationFilterKey,
}
impl NotificationFilter {
    pub fn new(tos_key: impl Into<NotificationFilterKey>) -> Self {
        Self {
            tos_key: tos_key.into(),
        }
    }

    pub fn tos_key(&self) -> &NotificationFilterKey {
        &self.tos_key
    }

    pub fn set_tos_key(&mut self, tos_key: impl Into<NotificationFilterKey>) {
        self.tos_key = tos_key.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationFilterKey {
    #[serde(rename = "FilterRules")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) filter_rules: Vec<NotificationFilterRule>,
}
impl NotificationFilterKey {
    pub fn new(filter_rules: impl Into<Vec<NotificationFilterRule>>) -> Self {
        Self {
            filter_rules: filter_rules.into(),
        }
    }

    pub fn filter_rules(&self) -> &Vec<NotificationFilterRule> {
        &self.filter_rules
    }

    pub fn set_filter_rules(&mut self, filter_rules: impl Into<Vec<NotificationFilterRule>>) {
        self.filter_rules = filter_rules.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationFilterRule {
    #[serde(rename = "Name")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) name: String,
    #[serde(rename = "Value")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) value: String,
}
impl NotificationFilterRule {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationDestination {
    #[serde(rename = "RocketMQ")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) rocket_mq: Vec<DestinationRocketMQ>,
    #[serde(rename = "VeFaaS")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) ve_faas: Vec<DestinationVeFaaS>,
    #[serde(rename = "Kafka")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) kafka: Vec<DestinationKafka>,
    #[serde(rename = "HttpServer")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) http_server: Vec<DestinationHttpServer>,
}
impl NotificationDestination {
    pub fn new() -> Self {
        Self {
            rocket_mq: vec![],
            ve_faas: vec![],
            kafka: vec![],
            http_server: vec![],
        }
    }

    pub fn rocket_mq(&self) -> &Vec<DestinationRocketMQ> {
        &self.rocket_mq
    }

    pub fn ve_faas(&self) -> &Vec<DestinationVeFaaS> {
        &self.ve_faas
    }

    pub fn kafka(&self) -> &Vec<DestinationKafka> {
        &self.kafka
    }

    pub fn http_server(&self) -> &Vec<DestinationHttpServer> {
        &self.http_server
    }

    pub fn set_rocket_mq(&mut self, rocket_mq: impl Into<Vec<DestinationRocketMQ>>) {
        self.rocket_mq = rocket_mq.into();
    }

    pub fn set_ve_faas(&mut self, ve_faas: impl Into<Vec<DestinationVeFaaS>>) {
        self.ve_faas = ve_faas.into();
    }

    pub fn set_kafka(&mut self, kafka: impl Into<Vec<DestinationKafka>>) {
        self.kafka = kafka.into();
    }

    pub fn set_http_server(&mut self, http_server: impl Into<Vec<DestinationHttpServer>>) {
        self.http_server = http_server.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DestinationRocketMQ {
    #[serde(rename = "Role")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) role: String,
    #[serde(rename = "InstanceId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) instance_id: String,
    #[serde(rename = "Topic")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) topic: String,
    #[serde(rename = "AccessKeyId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) access_key_id: String,
    #[serde(rename = "Region")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) region: String,
}
impl DestinationRocketMQ {
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            instance_id: "".to_string(),
            topic: "".to_string(),
            access_key_id: "".to_string(),
            region: "".to_string(),
        }
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn set_role(&mut self, role: impl Into<String>) {
        self.role = role.into();
    }

    pub fn set_instance_id(&mut self, instance_id: impl Into<String>) {
        self.instance_id = instance_id.into();
    }

    pub fn set_topic(&mut self, topic: impl Into<String>) {
        self.topic = topic.into();
    }

    pub fn set_access_key_id(&mut self, access_key_id: impl Into<String>) {
        self.access_key_id = access_key_id.into();
    }

    pub fn set_region(&mut self, region: impl Into<String>) {
        self.region = region.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DestinationVeFaaS {
    #[serde(rename = "FunctionId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) function_id: String,
}
impl DestinationVeFaaS {
    pub fn new(function_id: impl Into<String>) -> Self {
        Self {
            function_id: function_id.into(),
        }
    }

    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    pub fn set_function_id(&mut self, function_id: impl Into<String>) {
        self.function_id = function_id.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DestinationKafka {
    #[serde(rename = "Role")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) role: String,
    #[serde(rename = "InstanceId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) instance_id: String,
    #[serde(rename = "Topic")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) topic: String,
    #[serde(rename = "User")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) user: String,
    #[serde(rename = "Region")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) region: String,
}
impl DestinationKafka {
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            instance_id: "".to_string(),
            topic: "".to_string(),
            user: "".to_string(),
            region: "".to_string(),
        }
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn set_role(&mut self, role: impl Into<String>) {
        self.role = role.into();
    }

    pub fn set_instance_id(&mut self, instance_id: impl Into<String>) {
        self.instance_id = instance_id.into();
    }

    pub fn set_topic(&mut self, topic: impl Into<String>) {
        self.topic = topic.into();
    }

    pub fn set_user(&mut self, user: impl Into<String>) {
        self.user = user.into();
    }

    pub fn set_region(&mut self, region: impl Into<String>) {
        self.region = region.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DestinationHttpServer {
    #[serde(rename = "Url")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) url: String,
}
impl DestinationHttpServer {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = url.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketNotificationType2Output {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketNotificationType2Input {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketNotificationType2Input {
    fn operation(&self) -> &str {
        "GetBucketNotificationType2"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketNotificationType2Input {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        request.query = Some(HashMap::from([("notification_v2", "".to_string())]));
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketNotificationType2Output {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Rules")]
    pub(crate) rules: Vec<NotificationRule>,
    #[serde(rename = "Version")]
    pub(crate) version: String,
}
impl GetBucketNotificationType2Output {
    pub fn rules(&self) -> &Vec<NotificationRule> {
        &self.rules
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketInventoryInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Id")]
    pub(crate) id: String,
    #[serde(rename = "IsEnabled")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub(crate) is_enabled: bool,
    #[serde(rename = "Filter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) filter: Option<InventoryFilter>,
    #[serde(rename = "Destination")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) destination: Option<InventoryDestination>,
    #[serde(rename = "Schedule")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) schedule: Option<InventorySchedule>,
    #[serde(rename = "IncludedObjectVersions")]
    pub(crate) included_object_versions: InventoryIncludedObjType,
    #[serde(rename = "OptionalFields")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) optional_fields: Option<InventoryOptionalFields>,
    #[serde(rename = "IsUnCompressed")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub(crate) is_un_compressed: bool,
}

impl PutBucketInventoryInput {
    pub fn new(bucket: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            id: id.into(),
            is_enabled: true,
            filter: None,
            destination: None,
            schedule: None,
            included_object_versions: Default::default(),
            optional_fields: None,
            is_un_compressed: false,
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn filter(&self) -> &Option<InventoryFilter> {
        &self.filter
    }

    pub fn destination(&self) -> &Option<InventoryDestination> {
        &self.destination
    }

    pub fn schedule(&self) -> &Option<InventorySchedule> {
        &self.schedule
    }

    pub fn included_object_versions(&self) -> &InventoryIncludedObjType {
        &self.included_object_versions
    }

    pub fn optional_fields(&self) -> &Option<InventoryOptionalFields> {
        &self.optional_fields
    }

    pub fn is_un_compressed(&self) -> bool {
        self.is_un_compressed
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    pub fn set_is_enabled(&mut self, is_enabled: bool) {
        self.is_enabled = is_enabled;
    }

    pub fn set_filter(&mut self, filter: impl Into<InventoryFilter>) {
        self.filter = Some(filter.into());
    }

    pub fn set_destination(&mut self, destination: impl Into<InventoryDestination>) {
        self.destination = Some(destination.into());
    }

    pub fn set_schedule(&mut self, schedule: impl Into<InventorySchedule>) {
        self.schedule = Some(schedule.into());
    }

    pub fn set_included_object_versions(&mut self, included_object_versions: impl Into<InventoryIncludedObjType>) {
        self.included_object_versions = included_object_versions.into();
    }

    pub fn set_optional_fields(&mut self, optional_fields: impl Into<InventoryOptionalFields>) {
        self.optional_fields = Some(optional_fields.into());
    }

    pub fn set_is_un_compressed(&mut self, is_un_compressed: bool) {
        self.is_un_compressed = is_un_compressed;
    }
}

impl InputDescriptor for PutBucketInventoryInput {
    fn operation(&self) -> &str {
        "PutBucketInventory"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketInventoryInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.id == "" {
            return Err(TosError::client_error("empty id"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                let mut query = HashMap::with_capacity(2);
                query.insert("inventory", "".to_string());
                query.insert("id", self.id.to_string());
                request.query = Some(query);
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct InventoryFilter {
    #[serde(rename = "Prefix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) prefix: String,
}
impl InventoryFilter {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn set_prefix(&mut self, prefix: impl Into<String>) {
        self.prefix = prefix.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct InventoryDestination {
    #[serde(rename = "TOSBucketDestination")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub(crate) tos_bucket_destination: Option<TOSBucketDestination>,
}
impl InventoryDestination {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_tos_bucket_destination(tos_bucket_destination: impl Into<TOSBucketDestination>) -> Self {
        Self {
            tos_bucket_destination: Some(tos_bucket_destination.into()),
        }
    }

    pub fn tos_bucket_destination(&self) -> &Option<TOSBucketDestination> {
        &self.tos_bucket_destination
    }

    pub fn set_tos_bucket_destination(&mut self, tos_bucket_destination: impl Into<TOSBucketDestination>) {
        self.tos_bucket_destination = Some(tos_bucket_destination.into());
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TOSBucketDestination {
    #[serde(rename = "Format")]
    #[serde(default)]
    pub(crate) format: InventoryFormatType,
    #[serde(rename = "AccountId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) account_id: String,
    #[serde(rename = "Role")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) role: String,
    #[serde(rename = "Bucket")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) bucket: String,
    #[serde(rename = "Prefix")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) prefix: String,
}

impl TOSBucketDestination {
    pub fn new(format: impl Into<InventoryFormatType>) -> Self {
        Self {
            format: format.into(),
            account_id: "".to_string(),
            role: "".to_string(),
            bucket: "".to_string(),
            prefix: "".to_string(),
        }
    }

    pub fn format(&self) -> &InventoryFormatType {
        &self.format
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn set_format(&mut self, format: impl Into<InventoryFormatType>) {
        self.format = format.into();
    }

    pub fn set_account_id(&mut self, account_id: impl Into<String>) {
        self.account_id = account_id.into();
    }

    pub fn set_role(&mut self, role: impl Into<String>) {
        self.role = role.into();
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_prefix(&mut self, prefix: impl Into<String>) {
        self.prefix = prefix.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct InventorySchedule {
    #[serde(rename = "Frequency")]
    #[serde(default)]
    pub(crate) frequency: InventoryFrequencyType,
}
impl InventorySchedule {
    pub fn new(frequency: impl Into<InventoryFrequencyType>) -> Self {
        Self {
            frequency: frequency.into()
        }
    }

    pub fn frequency(&self) -> &InventoryFrequencyType {
        &self.frequency
    }

    pub fn set_frequency(&mut self, frequency: impl Into<InventoryFrequencyType>) {
        self.frequency = frequency.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct InventoryOptionalFields {
    #[serde(rename = "Field")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) field: Vec<String>,
}
impl InventoryOptionalFields {
    pub fn new(field: impl Into<Vec<String>>) -> Self {
        Self {
            field: field.into(),
        }
    }

    pub fn field(&self) -> &Vec<String> {
        &self.field
    }

    pub fn set_field(&mut self, field: impl Into<Vec<String>>) {
        self.field = field.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketInventoryOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct GetBucketInventoryInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) id: String,
}
impl GetBucketInventoryInput {
    pub fn new(bucket: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            id: id.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }
}

impl InputDescriptor for GetBucketInventoryInput {
    fn operation(&self) -> &str {
        "GetBucketInventory"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketInventoryInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.id == "" {
            return Err(TosError::client_error("empty id"));
        }

        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(2);
        query.insert("inventory", "".to_string());
        query.insert("id", self.id.to_string());
        request.query = Some(query);
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketInventoryOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Id")]
    #[serde(default)]
    pub(crate) id: String,
    #[serde(rename = "IsEnabled")]
    #[serde(default)]
    pub(crate) is_enabled: bool,
    #[serde(rename = "Filter")]
    #[serde(default)]
    pub(crate) filter: Option<InventoryFilter>,
    #[serde(rename = "Destination")]
    #[serde(default)]
    pub(crate) destination: Option<InventoryDestination>,
    #[serde(rename = "Schedule")]
    #[serde(default)]
    pub(crate) schedule: Option<InventorySchedule>,
    #[serde(rename = "IncludedObjectVersions")]
    #[serde(default)]
    pub(crate) included_object_versions: InventoryIncludedObjType,
    #[serde(rename = "OptionalFields")]
    #[serde(default)]
    pub(crate) optional_fields: Option<InventoryOptionalFields>,
    #[serde(rename = "IsUnCompressed")]
    #[serde(default)]
    pub(crate) is_un_compressed: bool,
}
impl GetBucketInventoryOutput {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn filter(&self) -> &Option<InventoryFilter> {
        &self.filter
    }

    pub fn destination(&self) -> &Option<InventoryDestination> {
        &self.destination
    }

    pub fn schedule(&self) -> &Option<InventorySchedule> {
        &self.schedule
    }

    pub fn included_object_versions(&self) -> &InventoryIncludedObjType {
        &self.included_object_versions
    }

    pub fn optional_fields(&self) -> &Option<InventoryOptionalFields> {
        &self.optional_fields
    }

    pub fn is_un_compressed(&self) -> bool {
        self.is_un_compressed
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct ListBucketInventoryInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) continuation_token: String,
}
impl ListBucketInventoryInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            continuation_token: "".to_string(),
        }
    }
    pub fn new_with_continuation_token(bucket: impl Into<String>, continuation_token: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            continuation_token: continuation_token.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn continuation_token(&self) -> &str {
        &self.continuation_token
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_continuation_token(&mut self, continuation_token: impl Into<String>) {
        self.continuation_token = continuation_token.into();
    }
}

impl InputDescriptor for ListBucketInventoryInput {
    fn operation(&self) -> &str {
        "ListBucketInventory"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for ListBucketInventoryInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(2);
        query.insert("inventory", "".to_string());
        if self.continuation_token != "" {
            query.insert("continuation-token", self.continuation_token.to_string());
        }
        request.query = Some(query);
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListBucketInventoryOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "IsTruncated")]
    #[serde(default)]
    pub(crate) is_truncated: bool,
    #[serde(rename = "InventoryConfigurations")]
    #[serde(default)]
    pub(crate) configurations: Vec<BucketInventoryConfiguration>,
    #[serde(rename = "NextContinuationToken")]
    #[serde(default)]
    pub(crate) next_continuation_token: String,
}

impl ListBucketInventoryOutput {
    pub fn request_info(&self) -> &RequestInfo {
        &self.request_info
    }

    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn configurations(&self) -> &Vec<BucketInventoryConfiguration> {
        &self.configurations
    }

    pub fn next_continuation_token(&self) -> &str {
        &self.next_continuation_token
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct BucketInventoryConfiguration {
    #[serde(rename = "Id")]
    #[serde(default)]
    pub(crate) id: String,
    #[serde(rename = "IsEnabled")]
    #[serde(default)]
    pub(crate) is_enabled: bool,
    #[serde(rename = "Filter")]
    #[serde(default)]
    pub(crate) filter: Option<InventoryFilter>,
    #[serde(rename = "Destination")]
    #[serde(default)]
    pub(crate) destination: Option<InventoryDestination>,
    #[serde(rename = "Schedule")]
    #[serde(default)]
    pub(crate) schedule: Option<InventorySchedule>,
    #[serde(rename = "IncludedObjectVersions")]
    #[serde(default)]
    pub(crate) included_object_versions: InventoryIncludedObjType,
    #[serde(rename = "OptionalFields")]
    #[serde(default)]
    pub(crate) optional_fields: Option<InventoryOptionalFields>,
    #[serde(rename = "IsUnCompressed")]
    #[serde(default)]
    pub(crate) is_un_compressed: bool,
}

impl BucketInventoryConfiguration {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn filter(&self) -> &Option<InventoryFilter> {
        &self.filter
    }

    pub fn destination(&self) -> &Option<InventoryDestination> {
        &self.destination
    }

    pub fn schedule(&self) -> &Option<InventorySchedule> {
        &self.schedule
    }

    pub fn included_object_versions(&self) -> &InventoryIncludedObjType {
        &self.included_object_versions
    }

    pub fn optional_fields(&self) -> &Option<InventoryOptionalFields> {
        &self.optional_fields
    }

    pub fn is_un_compressed(&self) -> bool {
        self.is_un_compressed
    }
}

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct DeleteBucketInventoryInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) id: String,
}

impl DeleteBucketInventoryInput {
    pub fn new(bucket: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            id: id.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }
}

impl InputDescriptor for DeleteBucketInventoryInput {
    fn operation(&self) -> &str {
        "DeleteBucketInventory"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteBucketInventoryInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.id == "" {
            return Err(TosError::client_error("empty id"));
        }

        let mut request = self.trans_bucket()?;
        request.method = HttpMethodDelete;
        let mut query = HashMap::with_capacity(2);
        query.insert("inventory", "".to_string());
        query.insert("id", self.id.to_string());
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteBucketInventoryOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketTypeInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct GetBucketTypeOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) region: String,
    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) az_redundancy: Option<AzRedundancyType>,
    pub(crate) project_name: String,
    pub(crate) bucket_type: Option<BucketType>,
    pub(crate) expire_at: DateTime<Utc>,
}

impl GetBucketTypeOutput {
    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }

    pub fn az_redundancy(&self) -> &Option<AzRedundancyType> {
        &self.az_redundancy
    }

    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    pub fn bucket_type(&self) -> &Option<BucketType> {
        &self.bucket_type
    }

    #[cfg(feature = "tokio-runtime")]
    pub fn expire_at(&self) -> DateTime<Utc> {
        self.expire_at
    }
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct DoesBucketExistInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketInfoInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketInfoInput {
    fn operation(&self) -> &str {
        "GetBucketInfo"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketInfoInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(1);
        query.insert("bucketInfo", "".to_string());
        request.query = Some(query);
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketInfoOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Bucket")]
    pub(crate) bucket: BucketInfo,
}

impl GetBucketInfoOutput {
    pub fn bucket(&self) -> &BucketInfo {
        &self.bucket
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct BucketInfo {
    #[serde(default)]
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "CreationDate")]
    pub(crate) creation_date_string: Option<String>,
    #[serde(skip)]
    pub(crate) creation_date: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(default)]
    #[serde(rename = "ProjectName")]
    pub(crate) project_name: String,
    #[serde(default)]
    #[serde(rename = "Type")]
    pub(crate) bucket_type: Option<BucketType>,
    #[serde(default)]
    #[serde(rename = "Location")]
    pub(crate) location: String,
    #[serde(default)]
    #[serde(rename = "AzRedundancy")]
    pub(crate) az_redundancy: Option<AzRedundancyType>,
    #[serde(default)]
    #[serde(rename = "ExtranetEndpoint")]
    pub(crate) extranet_endpoint: String,
    #[serde(default)]
    #[serde(rename = "IntranetEndpoint")]
    pub(crate) intranet_endpoint: String,
    #[serde(default)]
    #[serde(rename = "ExtranetS3Endpoint")]
    pub(crate) extranet_s3_endpoint: String,
    #[serde(default)]
    #[serde(rename = "IntranetS3Endpoint")]
    pub(crate) intranet_s3_endpoint: String,
    #[serde(default)]
    #[serde(rename = "Versioning")]
    pub(crate) versioning: Option<VersioningStatusType>,
    #[serde(default)]
    #[serde(rename = "CrossRegionReplication")]
    pub(crate) cross_region_replication: Option<StatusType>,
    #[serde(default)]
    #[serde(rename = "TransferAcceleration")]
    pub(crate) transfer_acceleration: Option<StatusType>,
    #[serde(default)]
    #[serde(rename = "AccessMonitor")]
    pub(crate) access_monitor: Option<StatusType>,
    #[serde(default)]
    #[serde(rename = "ServerSideEncryptionConfiguration")]
    pub(crate) server_side_encryption_configuration: ServerSideEncryptionConfiguration,
}

impl BucketInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn creation_date(&self) -> Option<DateTime<Utc>> {
        self.creation_date
    }

    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }

    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    pub fn bucket_type(&self) -> &Option<BucketType> {
        &self.bucket_type
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn az_redundancy(&self) -> &Option<AzRedundancyType> {
        &self.az_redundancy
    }

    pub fn extranet_endpoint(&self) -> &str {
        &self.extranet_endpoint
    }

    pub fn intranet_endpoint(&self) -> &str {
        &self.intranet_endpoint
    }

    pub fn extranet_s3_endpoint(&self) -> &str {
        &self.extranet_s3_endpoint
    }

    pub fn intranet_s3_endpoint(&self) -> &str {
        &self.intranet_s3_endpoint
    }

    pub fn versioning(&self) -> &Option<VersioningStatusType> {
        &self.versioning
    }

    pub fn cross_region_replication(&self) -> &Option<StatusType> {
        &self.cross_region_replication
    }

    pub fn transfer_acceleration(&self) -> &Option<StatusType> {
        &self.transfer_acceleration
    }

    pub fn access_monitor(&self) -> &Option<StatusType> {
        &self.access_monitor
    }

    pub fn server_side_encryption_configuration(&self) -> &ServerSideEncryptionConfiguration {
        &self.server_side_encryption_configuration
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ServerSideEncryptionConfiguration {
    #[serde(default)]
    #[serde(rename = "Rule")]
    pub(crate) rule: BucketEncryptionRule,
}

impl ServerSideEncryptionConfiguration {
    pub fn rule(&self) -> &BucketEncryptionRule {
        &self.rule
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketAccessMonitorInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Status")]
    pub(crate) status: StatusType,
}

impl PutBucketAccessMonitorInput {
    pub fn new(bucket: impl Into<String>, status: impl Into<StatusType>) -> PutBucketAccessMonitorInput {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            status: status.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn status(&self) -> &StatusType {
        &self.status
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_status(&mut self, status: impl Into<StatusType>) {
        self.status = status.into();
    }
}

impl InputDescriptor for PutBucketAccessMonitorInput {
    fn operation(&self) -> &str {
        "PutBucketAccessMonitor"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketAccessMonitorInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                let mut query = HashMap::with_capacity(1);
                query.insert("accessmonitor", "".to_string());
                request.query = Some(query);
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketAccessMonitorOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketAccessMonitorInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketAccessMonitorInput {
    fn operation(&self) -> &str {
        "GetBucketAccessMonitor"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketAccessMonitorInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("accessmonitor", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketAccessMonitorOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Status")]
    #[serde(default)]
    pub(crate) status: StatusType,
}

impl GetBucketAccessMonitorOutput {
    pub fn status(&self) -> &StatusType {
        &self.status
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutBucketTrashInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Trash")]
    pub(crate) trash: BucketTrash,
}

impl PutBucketTrashInput {
    pub fn new(bucket: impl Into<String>, trash: impl Into<BucketTrash>) -> PutBucketTrashInput {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            trash: trash.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn trash(&self) -> &BucketTrash {
        &self.trash
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_trash(&mut self, trash: impl Into<BucketTrash>) {
        self.trash = trash.into();
    }
}

impl InputDescriptor for PutBucketTrashInput {
    fn operation(&self) -> &str {
        "PutBucketTrash"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for PutBucketTrashInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPut;
                let mut query = HashMap::with_capacity(1);
                query.insert("trash", "".to_string());
                request.query = Some(query);
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BucketTrash {
    #[serde(rename = "TrashPath")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) trash_path: String,
    #[serde(rename = "CleanInterval")]
    #[serde(default)]
    pub(crate) clean_interval: isize,
    #[serde(rename = "Status")]
    #[serde(default)]
    pub(crate) status: StatusType,
    #[serde(rename = "PrefixMatchRules")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) prefix_match_rules: Vec<BucketTrashPrefixRule>,
}

impl BucketTrash {
    pub fn new(trash_path: impl Into<String>, clean_interval: isize, status: impl Into<StatusType>) -> Self {
        Self {
            trash_path: trash_path.into(),
            clean_interval,
            status: status.into(),
            prefix_match_rules: vec![],
        }
    }

    pub fn trash_path(&self) -> &str {
        &self.trash_path
    }

    pub fn clean_interval(&self) -> isize {
        self.clean_interval
    }

    pub fn status(&self) -> &StatusType {
        &self.status
    }

    pub fn prefix_match_rules(&self) -> &Vec<BucketTrashPrefixRule> {
        &self.prefix_match_rules
    }

    pub fn set_trash_path(&mut self, trash_path: impl Into<String>) {
        self.trash_path = trash_path.into();
    }

    pub fn set_clean_interval(&mut self, clean_interval: isize) {
        self.clean_interval = clean_interval;
    }

    pub fn set_status(&mut self, status: impl Into<StatusType>) {
        self.status = status.into();
    }

    pub fn set_prefix_match_rules(&mut self, prefix_match_rules: impl Into<Vec<BucketTrashPrefixRule>>) {
        self.prefix_match_rules = prefix_match_rules.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BucketTrashPrefixRule {
    #[serde(rename = "PrefixList")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub(crate) prefix_list: Vec<String>,
    #[serde(rename = "TrashPath")]
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub(crate) trash_path: String,
    #[serde(rename = "CleanInterval")]
    #[serde(default)]
    pub(crate) clean_interval: isize,
}

impl BucketTrashPrefixRule {
    pub fn new(trash_path: impl Into<String>, clean_interval: isize, prefix_list: impl Into<Vec<String>>) -> Self {
        Self {
            prefix_list: prefix_list.into(),
            trash_path: trash_path.into(),
            clean_interval,
        }
    }

    pub fn prefix_list(&self) -> &Vec<String> {
        &self.prefix_list
    }

    pub fn trash_path(&self) -> &str {
        &self.trash_path
    }

    pub fn clean_interval(&self) -> isize {
        self.clean_interval
    }

    pub fn set_prefix_list(&mut self, prefix_list: impl Into<Vec<String>>) {
        self.prefix_list = prefix_list.into();
    }

    pub fn set_trash_path(&mut self, trash_path: impl Into<String>) {
        self.trash_path = trash_path.into();
    }

    pub fn set_clean_interval(&mut self, clean_interval: isize) {
        self.clean_interval = clean_interval;
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutBucketTrashOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, BucketSetter, GenericInput)]
pub struct GetBucketTrashInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
}

impl InputDescriptor for GetBucketTrashInput {
    fn operation(&self) -> &str {
        "GetBucketTrash"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetBucketTrashInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.query = Some(HashMap::from([("trash", "".to_string())]));
        request.method = HttpMethodGet;
        Ok(request)
    }
}


#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetBucketTrashOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Trash")]
    pub(crate) trash: BucketTrash,
}


impl GetBucketTrashOutput {
    pub fn trash(&self) -> &BucketTrash {
        &self.trash
    }
}