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

use reqwest::Method;
use serde::{Deserialize, Serialize};

use ve_tos_generic::FromRefAndDisplay;

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Deserialize)]
pub enum ACLType {
    #[default]
    #[serde(rename = "private")]
    ACLPrivate,
    #[serde(rename = "public-read")]
    ACLPublicRead,
    #[serde(rename = "public-read-write")]
    ACLPublicReadWrite,
    #[serde(rename = "authenticated-read")]
    ACLAuthenticatedRead,
    #[serde(rename = "bucket-owner-read")]
    ACLBucketOwnerRead,
    #[serde(rename = "bucket-owner-full-control")]
    ACLBucketOwnerFullControl,
    #[serde(rename = "bucket-owner-entrusted")]
    ACLBucketOwnerEntrusted,
}


impl ACLType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ACLPrivate => "private",
            Self::ACLPublicRead => "public-read",
            Self::ACLPublicReadWrite => "public-read-write",
            Self::ACLAuthenticatedRead => "authenticated-read",
            Self::ACLBucketOwnerRead => "bucket-owner-read",
            Self::ACLBucketOwnerFullControl => "bucket-owner-full-control",
            Self::ACLBucketOwnerEntrusted => "bucket-owner-entrusted",
        }
    }
    pub(crate) fn from(value: String) -> Option<Self> {
        match value.as_str() {
            "private" => Some(Self::ACLPrivate),
            "public-read" => Some(Self::ACLPublicRead),
            "public-read-write" => Some(Self::ACLPublicReadWrite),
            "authenticated-read" => Some(Self::ACLAuthenticatedRead),
            "bucket-owner-read" => Some(Self::ACLBucketOwnerRead),
            "bucket-owner-full-control" => Some(Self::ACLBucketOwnerFullControl),
            "bucket-owner-entrusted" => Some(Self::ACLBucketOwnerEntrusted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum StatusType {
    #[default]
    #[serde(rename = "Enabled")]
    StatusEnabled,
    #[serde(rename = "Disabled")]
    StatusDisabled,
}

impl StatusType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::StatusEnabled => "Enabled",
            Self::StatusDisabled => "Disabled",
        }
    }

    pub(crate) fn from(value: impl AsRef<str>) -> Option<Self> {
        match value.as_ref() {
            "Enabled" => Some(Self::StatusEnabled),
            "Disabled" => Some(Self::StatusDisabled),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum VersioningStatusType {
    #[default]
    #[serde(rename = "")]
    VersioningStatusNotSet,
    #[serde(rename = "Enabled")]
    VersioningStatusEnabled,
    #[serde(rename = "Suspended")]
    VersioningStatusSuspended,
}

impl VersioningStatusType {
    pub fn as_str(&self) -> &str {
        match self {
            VersioningStatusType::VersioningStatusNotSet => "",
            VersioningStatusType::VersioningStatusEnabled => "Enabled",
            VersioningStatusType::VersioningStatusSuspended => "Suspended",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum ProtocolType {
    #[default]
    #[serde(rename = "http")]
    ProtocolHttp,
    #[serde(rename = "https")]
    ProtocolHttps,
}
impl ProtocolType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ProtocolHttp => "http",
            Self::ProtocolHttps => "https",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum CertStatusType {
    #[default]
    #[serde(rename = "CertBound")]
    CertStatusBound,
    #[serde(rename = "CertUnbound")]
    CertStatusUnbound,
    #[serde(rename = "CertExpired")]
    CertStatusExpired,
}

impl CertStatusType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::CertStatusBound => "CertBound",
            Self::CertStatusUnbound => "CertUnbound",
            Self::CertStatusExpired => "CertExpired",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum AuthProtocolType {
    #[default]
    #[serde(rename = "tos")]
    AuthProtocolTos,
    #[serde(rename = "s3")]
    AuthProtocolS3,
}

impl AuthProtocolType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::AuthProtocolTos => "tos",
            Self::AuthProtocolS3 => "s3",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum RedirectType {
    #[default]
    #[serde(rename = "Mirror")]
    RedirectMirror,
    #[serde(rename = "Async")]
    RedirectAsync,
}

impl RedirectType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::RedirectMirror => "Mirror",
            Self::RedirectAsync => "Async",
        }
    }

    pub(crate) fn from(value: impl AsRef<str>) -> Option<Self> {
        match value.as_ref() {
            "Mirror" => Some(Self::RedirectMirror),
            "Async" => Some(Self::RedirectAsync),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum StorageClassType {
    #[default]
    #[serde(rename = "STANDARD")]
    StorageClassStandard,
    #[serde(rename = "IA")]
    StorageClassIa,
    #[serde(rename = "ARCHIVE_FR")]
    StorageClassArchiveFr,
    #[serde(rename = "INTELLIGENT_TIERING")]
    StorageClassIntelligentTiering,
    #[serde(rename = "COLD_ARCHIVE")]
    StorageClassColdArchive,
    #[serde(rename = "ARCHIVE")]
    StorageClassArchive,
    #[serde(rename = "DEEP_COLD_ARCHIVE")]
    StorageClassDeepColdArchive,
}


impl StorageClassType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::StorageClassStandard => "STANDARD",
            Self::StorageClassIa => "IA",
            Self::StorageClassArchiveFr => "ARCHIVE_FR",
            Self::StorageClassIntelligentTiering => "INTELLIGENT_TIERING",
            Self::StorageClassColdArchive => "COLD_ARCHIVE",
            Self::StorageClassArchive => "ARCHIVE",
            Self::StorageClassDeepColdArchive => "DEEP_COLD_ARCHIVE",
        }
    }

    pub(crate) fn from(value: impl AsRef<str>) -> Option<Self> {
        match value.as_ref() {
            "STANDARD" => Some(Self::StorageClassStandard),
            "IA" => Some(Self::StorageClassIa),
            "ARCHIVE_FR" => Some(Self::StorageClassArchiveFr),
            "INTELLIGENT_TIERING" => Some(Self::StorageClassIntelligentTiering),
            "COLD_ARCHIVE" => Some(Self::StorageClassColdArchive),
            "ARCHIVE" => Some(Self::StorageClassArchive),
            "DEEP_COLD_ARCHIVE" => Some(Self::StorageClassDeepColdArchive),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum StorageClassInheritDirectiveType {
    #[default]
    #[serde(rename = "DESTINATION_BUCKET")]
    StorageClassIDDestinationBucket,
    #[serde(rename = "SOURCE_OBJECT")]
    StorageClassIDSourceObject,
}

impl StorageClassInheritDirectiveType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::StorageClassIDDestinationBucket => "DESTINATION_BUCKET",
            Self::StorageClassIDSourceObject => "SOURCE_OBJECT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Deserialize)]
pub enum AzRedundancyType {
    #[default]
    #[serde(rename = "single-az")]
    AzRedundancySingleAz,
    #[serde(rename = "multi-az")]
    AzRedundancyMultiAz,
}

impl AzRedundancyType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::AzRedundancySingleAz => "single-az",
            Self::AzRedundancyMultiAz => "multi-az",
        }
    }
    pub(crate) fn from(value: impl AsRef<str>) -> Option<Self> {
        match value.as_ref() {
            "single-az" => Some(Self::AzRedundancySingleAz),
            "multi-az" => Some(Self::AzRedundancyMultiAz),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Deserialize)]
pub enum BucketType {
    #[default]
    #[serde(rename = "fns")]
    BucketTypeFns,
    #[serde(rename = "hns")]
    BucketTypeHns,
}

impl BucketType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::BucketTypeFns => "fns",
            Self::BucketTypeHns => "hns",
        }
    }
    pub(crate) fn from(value: impl AsRef<str>) -> Option<Self> {
        match value.as_ref() {
            "fns" => Some(Self::BucketTypeFns),
            "hns" => Some(Self::BucketTypeHns),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay)]
pub enum MetadataDirectiveType {
    #[default]
    MetadataDirectiveCopy,
    MetadataDirectiveReplace,
    MetadataDirectiveReplaceNew,
}

impl MetadataDirectiveType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::MetadataDirectiveCopy => "COPY",
            Self::MetadataDirectiveReplace => "REPLACE",
            Self::MetadataDirectiveReplaceNew => "REPLACE_NEW",
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay)]
pub enum TaggingDirectiveType {
    #[default]
    TaggingDirectiveCopy,
    TaggingDirectiveReplace,
}

impl TaggingDirectiveType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TaggingDirectiveCopy => "COPY",
            Self::TaggingDirectiveReplace => "REPLACE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Deserialize, Serialize)]
pub enum GranteeType {
    #[default]
    #[serde(rename = "Group")]
    GranteeGroup,
    #[serde(rename = "CanonicalUser")]
    GranteeUser,
}

impl GranteeType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::GranteeGroup => "Group",
            Self::GranteeUser => "CanonicalUser",
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Deserialize, Serialize)]
pub enum CannedType {
    #[default]
    #[serde(rename = "AllUsers")]
    CannedAllUsers,
    #[serde(rename = "AuthenticatedUsers")]
    CannedAuthenticatedUsers,
}

impl CannedType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::CannedAllUsers => "AllUsers",
            Self::CannedAuthenticatedUsers => "AuthenticatedUsers",
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Deserialize, Serialize)]
pub enum PermissionType {
    #[default]
    #[serde(rename = "READ")]
    PermissionRead,
    #[serde(rename = "READ_NON_LIST")]
    PermissionReadNonList,
    #[serde(rename = "WRITE")]
    PermissionWrite,
    #[serde(rename = "READ_ACP")]
    PermissionReadAcp,
    #[serde(rename = "WRITE_ACP")]
    PermissionWriteAcp,
    #[serde(rename = "FULL_CONTROL")]
    PermissionFullControl,
}

impl PermissionType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PermissionRead => "READ",
            Self::PermissionReadNonList => "READ_NON_LIST",
            Self::PermissionWrite => "WRITE",
            Self::PermissionReadAcp => "READ_ACP",
            Self::PermissionWriteAcp => "WRITE_ACP",
            Self::PermissionFullControl => "FULL_CONTROL",
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay)]
pub enum ReplicationStatusType {
    #[default]
    ReplicationStatusPending,
    ReplicationStatusComplete,
    ReplicationStatusFailed,
    ReplicationStatusReplica,
}

impl ReplicationStatusType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ReplicationStatusPending => "PENDING",
            Self::ReplicationStatusComplete => "COMPLETE",
            Self::ReplicationStatusFailed => "FAILED",
            Self::ReplicationStatusReplica => "REPLICA",
        }
    }

    pub(crate) fn from(value: impl AsRef<str>) -> Option<Self> {
        match value.as_ref() {
            "PENDING" => Some(Self::ReplicationStatusPending),
            "COMPLETE" => Some(Self::ReplicationStatusComplete),
            "FAILED" => Some(Self::ReplicationStatusFailed),
            "REPLICA" => Some(Self::ReplicationStatusReplica),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum TierType {
    #[default]
    #[serde(rename = "Standard")]
    TierStandard,
    #[serde(rename = "Expedited")]
    TierExpedited,
    #[serde(rename = "Bulk")]
    TierBulk,
}

impl TierType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TierStandard => "Standard",
            Self::TierExpedited => "Expedited",
            Self::TierBulk => "Bulk",
        }
    }
    pub(crate) fn from(value: impl AsRef<str>) -> Option<Self> {
        match value.as_ref() {
            "Standard" => Some(Self::TierStandard),
            "Expedited" => Some(Self::TierExpedited),
            "Bulk" => Some(Self::TierBulk),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay)]
pub enum HttpMethodType {
    #[default]
    HttpMethodGet,
    HttpMethodPut,
    HttpMethodPost,
    HttpMethodDelete,
    HttpMethodHead,
}


impl HttpMethodType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::HttpMethodGet => "GET",
            Self::HttpMethodPut => "PUT",
            Self::HttpMethodPost => "POST",
            Self::HttpMethodDelete => "DELETE",
            Self::HttpMethodHead => "HEAD",
        }
    }

    pub fn as_http_method(&self) -> Method {
        match self {
            Self::HttpMethodGet => Method::GET,
            Self::HttpMethodPut => Method::PUT,
            Self::HttpMethodPost => Method::POST,
            Self::HttpMethodDelete => Method::DELETE,
            Self::HttpMethodHead => Method::HEAD,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay)]
pub enum DocPreviewSrcType {
    #[default]
    DocPreviewSrcTypeDoc,
    DocPreviewSrcTypeDocx,
    DocPreviewSrcTypePpt,
    DocPreviewSrcTypePptx,
    DocPreviewSrcTypeXls,
    DocPreviewSrcTypeXlsx,
}

impl DocPreviewSrcType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DocPreviewSrcTypeDoc => "doc",
            Self::DocPreviewSrcTypeDocx => "docx",
            Self::DocPreviewSrcTypePpt => "ppt",
            Self::DocPreviewSrcTypePptx => "pptx",
            Self::DocPreviewSrcTypeXls => "xls",
            Self::DocPreviewSrcTypeXlsx => "xlsx",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay)]
pub enum DocPreviewDstType {
    #[default]
    DocPreviewDstTypePdf,
    DocPreviewDstTypeHtml,
    DocPreviewDstTypePng,
    DocPreviewDstTypeJpg,
}

impl DocPreviewDstType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DocPreviewDstTypePdf => "pdf",
            Self::DocPreviewDstTypeHtml => "html",
            Self::DocPreviewDstTypePng => "png",
            Self::DocPreviewDstTypeJpg => "jpg",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay)]
pub enum ObjectLockModeType {
    #[default]
    ObjectLockModeCompliance
}

impl ObjectLockModeType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ObjectLockModeCompliance => "COMPLIANCE"
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum InventoryIncludedObjType {
    #[default]
    #[serde(rename = "All")]
    InventoryIncludedObjAll,
    #[serde(rename = "Current")]
    InventoryIncludedObjCurrent,
}
impl InventoryIncludedObjType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InventoryIncludedObjAll => "All",
            Self::InventoryIncludedObjCurrent => "Current",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum InventoryFormatType {
    #[default]
    #[serde(rename = "CSV")]
    InventoryFormatCsv
}

impl InventoryFormatType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InventoryFormatCsv => "CSV",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, FromRefAndDisplay, Serialize, Deserialize)]
pub enum InventoryFrequencyType {
    #[default]
    #[serde(rename = "Daily")]
    InventoryFrequencyDaily,
    #[serde(rename = "Weekly")]
    InventoryFrequencyWeekly,
}

impl InventoryFrequencyType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InventoryFrequencyDaily => "Daily",
            Self::InventoryFrequencyWeekly => "Weekly",
        }
    }
}