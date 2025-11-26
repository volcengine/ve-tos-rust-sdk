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
use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

pub(crate) const DEFAULT_MAX_RETRY_COUNT: isize = 3;
pub(crate) const DEFAULT_CONNECTION_TIMEOUT: isize = 10000;
pub(crate) const DEFAULT_SOCKET_TIMEOUT: isize = 30000;
pub(crate) const DEFAULT_REQUEST_TIMEOUT: isize = 120000;
pub(crate) const DEFAULT_MAX_CONNECTIONS: isize = 1024;
pub(crate) const DEFAULT_IDLE_CONNECTION_TIME: isize = 60000;
pub(crate) const DEFAULT_DNS_CACHE_TIME: isize = 15;
pub(crate) const DEFAULT_EXPECT_100_CONTINUE_THRESHOLD: isize = 65536;
pub(crate) const DEFAULT_HIGH_LATENCY_LOG_THRESHOLD: isize = 100;
pub(crate) const BASE_DELAY_MS: u64 = 100;
pub(crate) const MAX_DELAY_MS: u64 = 10000;
pub(crate) const TRUE: &str = "true";
pub(crate) const DEFAULT_MAX_KEYS: isize = 1000;
pub(crate) const DEFAULT_READ_BUFFER_SIZE: usize = if cfg!(target_os = "espidf") { 512 } else { 8 * 1024 };
pub(crate) const MAX_READ_BUFFER_SIZE_FOR_JSON: usize = 50 * 1024 * 1024;
pub(crate) const DEFAULT_DIR_MODE: u32 = 0755;
pub(crate) const DEFAULT_FILE_MODE: u32 = 0644;
pub(crate) const DEFAULT_UPLOAD_PART_SIZE: i64 = 8 * 1024 * 1024;
pub(crate) const MAX_UPLOAD_PART_SIZE: i64 = 5 * 1024 * 1024 * 1024;
pub(crate) const MAX_UPLOAD_PART_NUMBER: i64 = 10000;
pub(crate) const DEFAULT_UPLOAD_TASK_NUM: isize = 3;
pub(crate) const DEFAULT_REGION: &str = REGION_BEIJING;
pub(crate) const REGION_BEIJING: &str = "cn-beijing";
pub(crate) const REGION_GUANGZHOU: &str = "cn-guangzhou";
pub(crate) const REGION_SHANGHAI: &str = "cn-shanghai";
pub(crate) const REGION_HONGKONG: &str = "cn-hongkong";
pub(crate) const REGION_ROUFO: &str = "ap-southeast-1";
pub(crate) const REGION_YAJIADA: &str = "ap-southeast-3";
pub(crate) const SCHEMA_HTTP: &str = "http://";
pub(crate) const SCHEMA_HTTPS: &str = "https://";
pub(crate) const ALLOWED_IN_URL: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
pub(crate) const ALGORITHM_AES256: &str = "AES256";
pub(crate) const ALGORITHM_KMS: &str = "kms";

// header key
pub(crate) const HEADER_PREFIX: &str = "x-tos-";
pub(crate) const HEADER_PREFIX_META: &str = "x-tos-meta-";
pub(crate) const HEADER_ACL: &str = "x-tos-acl";
pub(crate) const HEADER_GRANT_FULL_CONTROL: &str = "x-tos-grant-full-control";
pub(crate) const HEADER_GRANT_READ: &str = "x-tos-grant-read";
pub(crate) const HEADER_GRANT_READ_ACP: &str = "x-tos-grant-read-acp";
pub(crate) const HEADER_GRANT_WRITE: &str = "x-tos-grant-write";
pub(crate) const HEADER_GRANT_WRITE_ACP: &str = "x-tos-grant-write-acp";
pub(crate) const HEADER_STORAGE_CLASS: &str = "x-tos-storage-class";
pub(crate) const HEADER_REPLICATION_STATUS: &str = "x-tos-replication-status";
pub(crate) const HEADER_TAGGING_COUNT: &str = "x-tos-tagging-count";
pub(crate) const HEADER_WEBSITE_REDIRECT_LOCATION: &str = "x-tos-website-redirect-location";
pub(crate) const HEADER_FETCH_WEBSITE_REDIRECT_LOCATION: &str = "x-tos-fetch-website-redirect-location";
pub(crate) const HEADER_AZ_REDUNDANCY: &str = "x-tos-az-redundancy";
pub(crate) const HEADER_BUCKET_TYPE: &str = "x-tos-bucket-type";
pub(crate) const HEADER_REQUEST_DATE: &str = "x-tos-date";
pub(crate) const HEADER_REQUEST_ID: &str = "x-tos-request-id";
pub(crate) const HEADER_ID2: &str = "x-tos-id-2";
pub(crate) const HEADER_EC: &str = "x-tos-ec";
pub(crate) const HEADER_SECURITY_TOKEN: &str = "x-tos-security-token";
pub(crate) const HEADER_BUCKET_REGION: &str = "x-tos-bucket-region";
pub(crate) const HEADER_PROJECT_NAME: &str = "x-tos-project-name";
pub(crate) const HEADER_COPY_SOURCE: &str = "x-tos-copy-source";
pub(crate) const HEADER_COPY_SOURCE_RANGE: &str = "x-tos-copy-source-range";
pub(crate) const HEADER_COPY_SOURCE_IF_MATCH: &str = "x-tos-copy-source-if-match";
pub(crate) const HEADER_COPY_SOURCE_IF_MODIFIED_SINCE: &str = "x-tos-copy-source-if-modified-since";
pub(crate) const HEADER_COPY_SOURCE_IF_NONE_MATCH: &str = "x-tos-copy-source-if-none-match";
pub(crate) const HEADER_COPY_SOURCE_IF_UNMODIFIED_SINCE: &str = "x-tos-copy-source-if-unmodified-since";
pub(crate) const HEADER_SSEC_ALGORITHM: &str = "x-tos-server-side-encryption-customer-algorithm";
pub(crate) const HEADER_SSEC_KEY: &str = "x-tos-server-side-encryption-customer-key";
pub(crate) const HEADER_SSEC_KEY_MD5: &str = "x-tos-server-side-encryption-customer-key-md5";
pub(crate) const HEADER_SERVER_SIDE_ENCRYPTION: &str = "x-tos-server-side-encryption";
pub(crate) const HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID: &str = "x-tos-server-side-encryption-kms-key-id";
pub(crate) const HEADER_COPY_SOURCE_SSEC_ALGORITHM: &str = "x-tos-copy-source-server-side-encryption-customer-algorithm";
pub(crate) const HEADER_COPY_SOURCE_SSEC_KEY: &str = "x-tos-copy-source-server-side-encryption-customer-key";
pub(crate) const HEADER_COPY_SOURCE_SSEC_KEY_MD5: &str = "x-tos-copy-source-server-side-encryption-customer-key-md5";
pub(crate) const HEADER_METADATA_DIRECTIVE: &str = "x-tos-metadata-directive";
pub(crate) const HEADER_TRAFFIC_LIMIT: &str = "x-tos-traffic-limit";
pub(crate) const HEADER_FORBID_OVERWRITE: &str = "x-tos-forbid-overwrite";
pub(crate) const HEADER_RECURSIVE_MKDIR: &str = "x-tos-recursive-mkdir";
pub(crate) const HEADER_COMPLETE_ALL: &str = "x-tos-complete-all";
pub(crate) const HEADER_CALLBACK: &str = "x-tos-callback";
pub(crate) const HEADER_CALLBACK_VAR: &str = "x-tos-callback-var";
pub(crate) const HEADER_TAGGING: &str = "x-tos-tagging";
pub(crate) const HEADER_TAGGING_DIRECTIVE: &str = "x-tos-tagging-directive";
pub(crate) const HEADER_VERSION_ID: &str = "x-tos-version-id";
pub(crate) const HEADER_COPY_SOURCE_VERSION_ID: &str = "x-tos-copy-source-version-id";
pub(crate) const HEADER_DELETE_MARKER: &str = "x-tos-delete-marker";
pub(crate) const HEADER_OBJECT_TYPE: &str = "x-tos-object-type";
pub(crate) const HEADER_HASH_CRC64ECMA: &str = "x-tos-hash-crc64ecma";
pub(crate) const HEADER_RESTORE: &str = "x-tos-restore";
pub(crate) const HEADER_RESTORE_REQUEST_DATE: &str = "x-tos-restore-request-date";
pub(crate) const HEADER_RESTORE_EXPIRY_DAYS: &str = "x-tos-restore-expiry-days";
pub(crate) const HEADER_RESTORE_TIER: &str = "x-tos-restore-tier";
pub(crate) const HEADER_SDK_RETRY_COUNT: &str = "x-sdk-retry-count";
pub(crate) const HEADER_CONTENT_SHA256: &str = "x-tos-content-sha256";
pub(crate) const HEADER_NEXT_APPEND_OFFSET: &str = "x-tos-next-append-offset";
pub(crate) const HEADER_NEXT_MODIFY_OFFSET: &str = "x-tos-next-modify-offset";
pub(crate) const HEADER_X_IF_MATCH: &str = "x-tos-if-match";
pub(crate) const HEADER_DIRECTORY: &str = "x-tos-directory";
pub(crate) const HEADER_ALLOW_SAME_ACTION_OVERLAP: &str = "x-tos-allow-same-action-overlap";
pub(crate) const HEADER_OBJECT_EXPIRES: &str = "x-tos-object-expires";
pub(crate) const HEADER_OBJECT_LOCK_MODE: &str = "x-tos-object-lock-mode";
pub(crate) const HEADER_OBJECT_LOCK_RETAIN_UNTIL_DATE: &str = "x-tos-object-lock-retain-until-date";
pub(crate) const HEADER_FETCH_DETECT_MIME_TYPE: &str = "x-tos-fetch-detect-mime-type";
pub(crate) const HEADER_NOTIFICATION_CUSTOM_PARAMETERS: &str = "x-tos-notification-custom-parameters";
pub(crate) const HEADER_SYMLINK_TARGET: &str = "x-tos-symlink-target";
pub(crate) const HEADER_SYMLINK_BUCKET: &str = "x-tos-symlink-bucket";
pub(crate) const HEADER_SYMLINK_TARGET_SIZE: &str = "x-tos-symlink-target-size";
pub(crate) const HEADER_EXPIRATION: &str = "x-tos-expiration";
pub(crate) const HEADER_LAST_MODIFIED_NS: &str = "x-tos-last-modified-ns";
// http basic header key
pub(crate) const HEADER_HOST: &str = "Host";
pub(crate) const HEADER_RANGE: &str = "Range";
pub(crate) const HEADER_HOST_LOWER: &str = "host";
pub(crate) const HEADER_CONTENT_RANGE: &str = "Content-Range";
pub(crate) const HEADER_EXPECT: &str = "Expect";
pub(crate) const HEADER_ETAG: &str = "ETag";
pub(crate) const HEADER_IF_MATCH: &str = "If-Match";
pub(crate) const HEADER_IF_MODIFIED_SINCE: &str = "If-Modified-Since";
pub(crate) const HEADER_IF_NONE_MATCH: &str = "If-None-Match";
pub(crate) const HEADER_IF_UNMODIFIED_SINCE: &str = "If-Unmodified-Since";
pub(crate) const HEADER_LAST_MODIFIED: &str = "Last-Modified";
pub(crate) const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub(crate) const HEADER_CONTENT_TYPE_LOWER: &str = "content-type";
pub(crate) const HEADER_CONTENT_MD5: &str = "Content-MD5";
pub(crate) const HEADER_CONTENT_LENGTH: &str = "Content-Length";
pub(crate) const HEADER_CONTENT_LENGTH_LOWER: &str = "content-length";
pub(crate) const HEADER_CACHE_CONTROL: &str = "Cache-Control";
pub(crate) const HEADER_CONTENT_LANGUAGE: &str = "Content-Language";
pub(crate) const HEADER_CONTENT_ENCODING: &str = "Content-Encoding";
pub(crate) const HEADER_CONTENT_DISPOSITION: &str = "Content-Disposition";
pub(crate) const HEADER_EXPIRES: &str = "Expires";
pub(crate) const HEADER_FETCH_CONTENT_TYPE: &str = "x-tos-fetch-content-type";
pub(crate) const HEADER_FETCH_CACHE_CONTROL: &str = "x-tos-fetch-cache-control";
pub(crate) const HEADER_FETCH_CONTENT_LANGUAGE: &str = "x-tos-fetch-content-language";
pub(crate) const HEADER_FETCH_CONTENT_ENCODING: &str = "x-tos-fetch-content-encoding";
pub(crate) const HEADER_FETCH_CONTENT_DISPOSITION: &str = "x-tos-fetch-content-disposition";
pub(crate) const HEADER_FETCH_EXPIRES: &str = "x-tos-fetch-expires";
pub(crate) const HEADER_MODIFY_TIMESTAMP: &str = "x-tos-modify-timestamp";
pub(crate) const HEADER_MODIFY_TIMESTAMP_NS: &str = "x-tos-modify-timestamp-ns";
pub(crate) const HEADER_ACCOUNT_ID: &str = "x-tos-account-id";
pub(crate) const HEADER_AUTHORIZATION: &str = "Authorization";
pub(crate) const HEADER_LOCATION: &str = "Location";
pub(crate) const HEADER_RETRY_AFTER: &str = "Retry-After";
pub(crate) const HEADER_RETRY_AFTER_LOWER: &str = "retry-after";


// query key
pub(crate) const QUERY_VERSION_ID: &str = "versionId";
pub(crate) const QUERY_RECURSIVE: &str = "recursive";
pub(crate) const QUERY_SKIP_TRASH: &str = "skipTrash";
pub(crate) const QUERY_PART_NUMBER: &str = "partNumber";
pub(crate) const QUERY_RESPONSE_CACHE_CONTROL: &str = "response-cache-control";
pub(crate) const QUERY_RESPONSE_CONTENT_DISPOSITION: &str = "response-content-disposition";
pub(crate) const QUERY_RESPONSE_CONTENT_ENCODING: &str = "response-content-encoding";
pub(crate) const QUERY_RESPONSE_CONTENT_LANGUAGE: &str = "response-content-language";
pub(crate) const QUERY_RESPONSE_CONTENT_TYPE: &str = "response-content-type";
pub(crate) const QUERY_RESPONSE_EXPIRES: &str = "response-expires";
pub(crate) const QUERY_PREFIX: &str = "prefix";
pub(crate) const QUERY_DELIMITER: &str = "delimiter";
pub(crate) const QUERY_ENCODING_TYPE: &str = "encoding-type";
pub(crate) const QUERY_MARKER: &str = "marker";
pub(crate) const QUERY_START_AFTER: &str = "start-after";
pub(crate) const QUERY_CONTINUATION_TOKEN: &str = "continuation-token";
pub(crate) const QUERY_FETCH_META: &str = "fetch-meta";
pub(crate) const QUERY_FETCH_OWNER: &str = "fetch-owner";
pub(crate) const QUERY_KEY_MARKER: &str = "key-marker";
pub(crate) const QUERY_VERSION_ID_MARKER: &str = "version-id-marker";
pub(crate) const QUERY_UPLOAD_ID_MARKER: &str = "upload-id-marker";
pub(crate) const QUERY_MAX_KEYS: &str = "max-keys";
pub(crate) const QUERY_MAX_UPLOADS: &str = "max-uploads";
pub(crate) const QUERY_MAX_PARTS: &str = "max-parts";
pub(crate) const QUERY_UPLOAD_ID: &str = "uploadId";
pub(crate) const QUERY_PART_NUMBER_MARKER: &str = "part-number-marker";
pub(crate) const QUERY_OFFSET: &str = "offset";
pub(crate) const QUERY_PROCESS: &str = "x-tos-process";
pub(crate) const QUERY_SAVE_BUCKET: &str = "x-tos-save-bucket";
pub(crate) const QUERY_SAVE_OBJECT: &str = "x-tos-save-object";
pub(crate) const QUERY_DOC_DST_TYPE: &str = "x-tos-doc-dst-type";
pub(crate) const QUERY_DOC_SRC_TYPE: &str = "x-tos-doc-src-type";
pub(crate) const QUERY_DOC_PAGE: &str = "x-tos-doc-page";
pub(crate) const QUERY_TASK_ID: &str = "taskId";
pub(crate) const QUERY_ALGORITHM: &str = "X-Tos-Algorithm";
pub(crate) const QUERY_CREDENTIAL: &str = "X-Tos-Credential";
pub(crate) const QUERY_DATE: &str = "X-Tos-Date";
pub(crate) const QUERY_EXPIRES: &str = "X-Tos-Expires";
pub(crate) const QUERY_SIGNED_HEADERS: &str = "X-Tos-SignedHeaders";
pub(crate) const QUERY_SECURITY_TOKEN: &str = "X-Tos-Security-Token";
pub(crate) const QUERY_SIGNATURE: &str = "X-Tos-Signature";
pub(crate) const QUERY_POLICY: &str = "X-Tos-Policy";
pub(crate) const UUID_NODE: [u8; 6] = [1, 2, 3, 4, 5, 6];
pub(crate) const GET_OBJECT_TO_FILE_OPERATION: &str = "GetObjectToFile";

pub(crate) static CHINESE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\x{4e00}-\x{9fa5}]").unwrap());

pub(crate) static BUCKET_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9-]+$").unwrap());
pub(crate) static ACCOUNT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9]+$").unwrap());

pub(crate) static TOS_S3_ENDPOINTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    ("tos-s3-cn-beijing.volces.com", ""),
    ("tos-s3-cn-beijing.ivolces.com", ""),
    ("tos-s3-cn-guangzhou.volces.com", ""),
    ("tos-s3-cn-guangzhou.ivolces.com", ""),
    ("tos-s3-cn-shanghai.ivolces.com", ""),
    ("tos-s3-cn-shanghai.volces.com", ""),
    ("tos-s3-cn-beijing2.volces.com", ""),
    ("tos-s3-cn-beijing2.ivolces.com", ""),
    ("tos-s3-cn-hongkong.volces.com", ""),
    ("tos-s3-cn-hongkong.ivolces.com", ""),
    ("tos-s3-ap-southeast-1.volces.com", ""),
    ("tos-s3-ap-southeast-1.ivolces.com", ""),
    ("tos-s3-ap-southeast-3.volces.com", ""),
    ("tos-s3-ap-southeast-3.ivolces.com", ""),
]));

pub(crate) static REGION_ENDPOINTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    (REGION_BEIJING, "https://tos-cn-beijing.volces.com"),
    (REGION_SHANGHAI, "https://tos-cn-shanghai.volces.com"),
    (REGION_GUANGZHOU, "https://tos-cn-guangzhou.volces.com"),
    (REGION_HONGKONG, "https://tos-cn-hongkong.volces.com"),
    (REGION_ROUFO, "https://tos-ap-southeast-1.volces.com"),
    (REGION_YAJIADA, "https://tos-ap-southeast-3.volces.com")
]));

pub(crate) static REGION_CONTROL_ENDPOINTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    (REGION_BEIJING, "https://tos-control-cn-beijing.volces.com"),
    (REGION_SHANGHAI, "https://tos-control-cn-shanghai.volces.com"),
    (REGION_GUANGZHOU, "https://tos-control-cn-guangzhou.volces.com"),
    (REGION_HONGKONG, "https://tos-control-cn-hongkong.volces.com"),
    (REGION_ROUFO, "https://tos-control-ap-southeast-1.volces.com"),
    (REGION_YAJIADA, "https://tos-control-ap-southeast-3.volces.com")
]));

pub(crate) static NOT_ALLOWED_REQUEST_HEADER: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    ("content-length", ""),
    ("host", ""),
    ("connection", ""),
    ("x-tos-date", ""),
    ("range", ""),
    ("transfer-encoding", ""),
    ("authorization", ""),
    ("date", ""),
]));

pub(crate) static ALL_UPLOAD_OPERATIONS: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    ("AppendObject", ""),
    ("AppendObjectFromBuffer", ""),
    ("PutObject", ""),
    ("PutObjectFromFile", ""),
    ("PutObjectFromBuffer", ""),
    ("UploadPart", ""),
    ("UploadPartFromFile", ""),
    ("UploadPartFromBuffer", ""),
    ("ModifyObject", ""),
    ("ModifyObjectFromBuffer", ""),
]));

pub(crate) static NO_IDEMPOTENT_OPERATIONS: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    ("CreateBucket", ""),
    ("DeleteBucket", ""),
    ("CreateMultipartUpload", ""),
    ("CompleteMultipartUpload", ""),
    ("AbortMultipartUpload", ""),
    ("AppendObject", ""),
    ("ModifyObject", ""),
    ("RenameObject", ""),
    ("PutFetchTask", ""),
    ("RestoreObject", ""),
]));

pub(crate) static STREAM_UPLOAD_OPERATIONS: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    ("AppendObject", ""),
    ("PutObject", ""),
    ("UploadPart", ""),
    ("ModifyObject", ""),
]));

pub(crate) static AUTO_RECOGNIZE_CONTENT_TYPE_OPERATIONS: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    ("CreateMultipartUpload", ""),
    ("AppendObject", ""),
    ("AppendObjectFromBuffer", ""),
    ("PutObject", ""),
    ("PutObjectFromFile", ""),
    ("PutObjectFromBuffer", ""),
    ("FetchObject", ""),
    ("PutFetchTask", ""),
    ("PutSymlink", ""),
]));

pub(crate) static MIME_TYPES: Lazy<HashMap<&str, &str>> = Lazy::new(|| HashMap::from([
    ("3gp", "video/3gpp"),
    ("7z", "application/x-7z-compressed"),
    ("abw", "application/x-abiword"),
    ("ai", "application/postscript"),
    ("aif", "audio/x-aiff"),
    ("aifc", "audio/x-aiff"),
    ("aiff", "audio/x-aiff"),
    ("alc", "chemical/x-alchemy"),
    ("amr", "audio/amr"),
    ("anx", "application/annodex"),
    ("apk", "application/vnd.android.package-archive"),
    ("appcache", "text/cache-manifest"),
    ("art", "image/x-jg"),
    ("asc", "text/plain"),
    ("asf", "video/x-ms-asf"),
    ("aso", "chemical/x-ncbi-asn1-binary"),
    ("asx", "video/x-ms-asf"),
    ("atom", "application/atom+xml"),
    ("atomcat", "application/atomcat+xml"),
    ("atomsrv", "application/atomserv+xml"),
    ("au", "audio/basic"),
    ("avi", "video/x-msvideo"),
    ("awb", "audio/amr-wb"),
    ("axa", "audio/annodex"),
    ("axv", "video/annodex"),
    ("b", "chemical/x-molconn-Z"),
    ("bak", "application/x-trash"),
    ("bat", "application/x-msdos-program"),
    ("bcpio", "application/x-bcpio"),
    ("bib", "text/x-bibtex"),
    ("bin", "application/octet-stream"),
    ("bmp", "image/x-ms-bmp"),
    ("boo", "text/x-boo"),
    ("book", "application/x-maker"),
    ("brf", "text/plain"),
    ("bsd", "chemical/x-crossfire"),
    ("c", "text/x-csrc"),
    ("c++", "text/x-c++src"),
    ("c3d", "chemical/x-chem3d"),
    ("cab", "application/x-cab"),
    ("cac", "chemical/x-cache"),
    ("cache", "chemical/x-cache"),
    ("cap", "application/vnd.tcpdump.pcap"),
    ("cascii", "chemical/x-cactvs-binary"),
    ("cat", "application/vnd.ms-pki.seccat"),
    ("cbin", "chemical/x-cactvs-binary"),
    ("cbr", "application/x-cbr"),
    ("cbz", "application/x-cbz"),
    ("cc", "text/x-c++src"),
    ("cda", "application/x-cdf"),
    ("cdf", "application/x-cdf"),
    ("cdr", "image/x-coreldraw"),
    ("cdt", "image/x-coreldrawtemplate"),
    ("cdx", "chemical/x-cdx"),
    ("cdy", "application/vnd.cinderella"),
    ("cef", "chemical/x-cxf"),
    ("cer", "chemical/x-cerius"),
    ("chm", "chemical/x-chemdraw"),
    ("chrt", "application/x-kchart"),
    ("cif", "chemical/x-cif"),
    ("class", "application/java-vm"),
    ("cls", "text/x-tex"),
    ("cmdf", "chemical/x-cmdf"),
    ("cml", "chemical/x-cml"),
    ("cod", "application/vnd.rim.cod"),
    ("com", "application/x-msdos-program"),
    ("cpa", "chemical/x-compass"),
    ("cpio", "application/x-cpio"),
    ("cpp", "text/x-c++src"),
    ("cpt", "application/mac-compactpro"),
    ("cr2", "image/x-canon-cr2"),
    ("crl", "application/x-pkcs7-crl"),
    ("crt", "application/x-x509-ca-cert"),
    ("crw", "image/x-canon-crw"),
    ("csd", "audio/csound"),
    ("csf", "chemical/x-cache-csf"),
    ("csh", "application/x-csh"),
    ("csm", "chemical/x-csml"),
    ("csml", "chemical/x-csml"),
    ("css", "text/css"),
    ("csv", "text/csv"),
    ("ctab", "chemical/x-cactvs-binary"),
    ("ctx", "chemical/x-ctx"),
    ("cu", "application/cu-seeme"),
    ("cub", "chemical/x-gaussian-cube"),
    ("cxf", "chemical/x-cxf"),
    ("cxx", "text/x-c++src"),
    ("d", "text/x-dsrc"),
    ("davmount", "application/davmount+xml"),
    ("dcm", "application/dicom"),
    ("dcr", "application/x-director"),
    ("ddeb", "application/vnd.debian.binary-package"),
    ("dif", "video/dv"),
    ("diff", "text/x-diff"),
    ("dir", "application/x-director"),
    ("djv", "image/vnd.djvu"),
    ("djvu", "image/vnd.djvu"),
    ("dl", "video/dl"),
    ("dll", "application/x-msdos-program"),
    ("dmg", "application/x-apple-diskimage"),
    ("dms", "application/x-dms"),
    ("doc", "application/msword"),
    ("docm", "application/vnd.ms-word.document.macroEnabled.12"),
    ("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
    ("dot", "application/msword"),
    ("dotm", "application/vnd.ms-word.template.macroEnabled.12"),
    ("dotx", "application/vnd.openxmlformats-officedocument.wordprocessingml.template"),
    ("dv", "video/dv"),
    ("dvi", "application/x-dvi"),
    ("dx", "chemical/x-jcamp-dx"),
    ("dxr", "application/x-director"),
    ("emb", "chemical/x-embl-dl-nucleotide"),
    ("embl", "chemical/x-embl-dl-nucleotide"),
    ("eml", "message/rfc822"),
    ("eot", "application/vnd.ms-fontobject"),
    ("eps", "application/postscript"),
    ("eps2", "application/postscript"),
    ("eps3", "application/postscript"),
    ("epsf", "application/postscript"),
    ("epsi", "application/postscript"),
    ("erf", "image/x-epson-erf"),
    ("es", "application/ecmascript"),
    ("etx", "text/x-setext"),
    ("exe", "application/x-msdos-program"),
    ("ez", "application/andrew-inset"),
    ("fb", "application/x-maker"),
    ("fbdoc", "application/x-maker"),
    ("fch", "chemical/x-gaussian-checkpoint"),
    ("fchk", "chemical/x-gaussian-checkpoint"),
    ("fig", "application/x-xfig"),
    ("flac", "audio/flac"),
    ("fli", "video/fli"),
    ("flv", "video/x-flv"),
    ("fm", "application/x-maker"),
    ("frame", "application/x-maker"),
    ("frm", "application/x-maker"),
    ("gal", "chemical/x-gaussian-log"),
    ("gam", "chemical/x-gamess-input"),
    ("gamin", "chemical/x-gamess-input"),
    ("gan", "application/x-ganttproject"),
    ("gau", "chemical/x-gaussian-input"),
    ("gcd", "text/x-pcs-gcd"),
    ("gcf", "application/x-graphing-calculator"),
    ("gcg", "chemical/x-gcg8-sequence"),
    ("gen", "chemical/x-genbank"),
    ("gf", "application/x-tex-gf"),
    ("gif", "image/gif"),
    ("gjc", "chemical/x-gaussian-input"),
    ("gjf", "chemical/x-gaussian-input"),
    ("gl", "video/gl"),
    ("gnumeric", "application/x-gnumeric"),
    ("gpt", "chemical/x-mopac-graph"),
    ("gsf", "application/x-font"),
    ("gsm", "audio/x-gsm"),
    ("gtar", "application/x-gtar"),
    ("gz", "application/gzip"),
    ("h", "text/x-chdr"),
    ("h++", "text/x-c++hdr"),
    ("hdf", "application/x-hdf"),
    ("hh", "text/x-c++hdr"),
    ("hin", "chemical/x-hin"),
    ("hpp", "text/x-c++hdr"),
    ("hqx", "application/mac-binhex40"),
    ("hs", "text/x-haskell"),
    ("hta", "application/hta"),
    ("htc", "text/x-component"),
    ("htm", "text/html"),
    ("html", "text/html"),
    ("hwp", "application/x-hwp"),
    ("hxx", "text/x-c++hdr"),
    ("ica", "application/x-ica"),
    ("ice", "x-conference/x-cooltalk"),
    ("ico", "image/vnd.microsoft.icon"),
    ("ics", "text/calendar"),
    ("icz", "text/calendar"),
    ("ief", "image/ief"),
    ("iges", "model/iges"),
    ("igs", "model/iges"),
    ("iii", "application/x-iphone"),
    ("info", "application/x-info"),
    ("inp", "chemical/x-gamess-input"),
    ("ins", "application/x-internet-signup"),
    ("iso", "application/x-iso9660-image"),
    ("isp", "application/x-internet-signup"),
    ("ist", "chemical/x-isostar"),
    ("istr", "chemical/x-isostar"),
    ("jad", "text/vnd.sun.j2me.app-descriptor"),
    ("jam", "application/x-jam"),
    ("jar", "application/java-archive"),
    ("java", "text/x-java"),
    ("jdx", "chemical/x-jcamp-dx"),
    ("jmz", "application/x-jmol"),
    ("jng", "image/x-jng"),
    ("jnlp", "application/x-java-jnlp-file"),
    ("jp2", "image/jp2"),
    ("jpe", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("jpf", "image/jpx"),
    ("jpg", "image/jpeg"),
    ("jpg2", "image/jp2"),
    ("jpm", "image/jpm"),
    ("jpx", "image/jpx"),
    ("js", "application/javascript"),
    ("json", "application/json"),
    ("kar", "audio/midi"),
    ("key", "application/pgp-keys"),
    ("kil", "application/x-killustrator"),
    ("kin", "chemical/x-kinemage"),
    ("kml", "application/vnd.google-earth.kml+xml"),
    ("kmz", "application/vnd.google-earth.kmz"),
    ("kpr", "application/x-kpresenter"),
    ("kpt", "application/x-kpresenter"),
    ("ksp", "application/x-kspread"),
    ("kwd", "application/x-kword"),
    ("kwt", "application/x-kword"),
    ("latex", "application/x-latex"),
    ("lha", "application/x-lha"),
    ("lhs", "text/x-literate-haskell"),
    ("lin", "application/bbolin"),
    ("lsf", "video/x-la-asf"),
    ("lsx", "video/x-la-asf"),
    ("ltx", "text/x-tex"),
    ("ly", "text/x-lilypond"),
    ("lyx", "application/x-lyx"),
    ("lzh", "application/x-lzh"),
    ("lzx", "application/x-lzx"),
    ("m3g", "application/m3g"),
    ("m3u", "audio/x-mpegurl"),
    ("m3u8", "application/x-mpegURL"),
    ("m4a", "audio/mpeg"),
    ("maker", "application/x-maker"),
    ("man", "application/x-troff-man"),
    ("mbox", "application/mbox"),
    ("mcif", "chemical/x-mmcif"),
    ("mcm", "chemical/x-macmolecule"),
    ("mdb", "application/msaccess"),
    ("me", "application/x-troff-me"),
    ("mesh", "model/mesh"),
    ("mid", "audio/midi"),
    ("midi", "audio/midi"),
    ("mif", "application/x-mif"),
    ("mkv", "video/x-matroska"),
    ("mm", "application/x-freemind"),
    ("mmd", "chemical/x-macromodel-input"),
    ("mmf", "application/vnd.smaf"),
    ("mml", "text/mathml"),
    ("mmod", "chemical/x-macromodel-input"),
    ("mng", "video/x-mng"),
    ("moc", "text/x-moc"),
    ("mol", "chemical/x-mdl-molfile"),
    ("mol2", "chemical/x-mol2"),
    ("moo", "chemical/x-mopac-out"),
    ("mop", "chemical/x-mopac-input"),
    ("mopcrt", "chemical/x-mopac-input"),
    ("mov", "video/quicktime"),
    ("movie", "video/x-sgi-movie"),
    ("mp2", "audio/mpeg"),
    ("mp3", "audio/mpeg"),
    ("mp4", "video/mp4"),
    ("mpc", "chemical/x-mopac-input"),
    ("mpe", "video/mpeg"),
    ("mpeg", "video/mpeg"),
    ("mpega", "audio/mpeg"),
    ("mpg", "video/mpeg"),
    ("mpga", "audio/mpeg"),
    ("mph", "application/x-comsol"),
    ("mpv", "video/x-matroska"),
    ("ms", "application/x-troff-ms"),
    ("msh", "model/mesh"),
    ("msi", "application/x-msi"),
    ("mvb", "chemical/x-mopac-vib"),
    ("mxf", "application/mxf"),
    ("mxu", "video/vnd.mpegurl"),
    ("nb", "application/mathematica"),
    ("nbp", "application/mathematica"),
    ("nc", "application/x-netcdf"),
    ("nef", "image/x-nikon-nef"),
    ("nwc", "application/x-nwc"),
    ("o", "application/x-object"),
    ("oda", "application/oda"),
    ("odb", "application/vnd.oasis.opendocument.database"),
    ("odc", "application/vnd.oasis.opendocument.chart"),
    ("odf", "application/vnd.oasis.opendocument.formula"),
    ("odg", "application/vnd.oasis.opendocument.graphics"),
    ("odi", "application/vnd.oasis.opendocument.image"),
    ("odm", "application/vnd.oasis.opendocument.text-master"),
    ("odp", "application/vnd.oasis.opendocument.presentation"),
    ("ods", "application/vnd.oasis.opendocument.spreadsheet"),
    ("odt", "application/vnd.oasis.opendocument.text"),
    ("oga", "audio/ogg"),
    ("ogg", "audio/ogg"),
    ("ogv", "video/ogg"),
    ("ogx", "application/ogg"),
    ("old", "application/x-trash"),
    ("one", "application/onenote"),
    ("onepkg", "application/onenote"),
    ("onetmp", "application/onenote"),
    ("onetoc2", "application/onenote"),
    ("opf", "application/oebps-package+xml"),
    ("opus", "audio/ogg"),
    ("orc", "audio/csound"),
    ("orf", "image/x-olympus-orf"),
    ("otf", "application/font-sfnt"),
    ("otg", "application/vnd.oasis.opendocument.graphics-template"),
    ("oth", "application/vnd.oasis.opendocument.text-web"),
    ("otp", "application/vnd.oasis.opendocument.presentation-template"),
    ("ots", "application/vnd.oasis.opendocument.spreadsheet-template"),
    ("ott", "application/vnd.oasis.opendocument.text-template"),
    ("oza", "application/x-oz-application"),
    ("p", "text/x-pascal"),
    ("p7r", "application/x-pkcs7-certreqresp"),
    ("pac", "application/x-ns-proxy-autoconfig"),
    ("pas", "text/x-pascal"),
    ("pat", "image/x-coreldrawpattern"),
    ("patch", "text/x-diff"),
    ("pbm", "image/x-portable-bitmap"),
    ("pcap", "application/vnd.tcpdump.pcap"),
    ("pcf", "application/x-font-pcf"),
    ("pcf.Z", "application/x-font-pcf"),
    ("pcx", "image/pcx"),
    ("pdb", "chemical/x-pdb"),
    ("pdf", "application/pdf"),
    ("pfa", "application/x-font"),
    ("pfb", "application/x-font"),
    ("pfr", "application/font-tdpfr"),
    ("pgm", "image/x-portable-graymap"),
    ("pgn", "application/x-chess-pgn"),
    ("pgp", "application/pgp-encrypted"),
    ("php", "#application/x-httpd-php"),
    ("php3", "#application/x-httpd-php3"),
    ("php3p", "#application/x-httpd-php3-preprocessed"),
    ("php4", "#application/x-httpd-php4"),
    ("php5", "#application/x-httpd-php5"),
    ("phps", "#application/x-httpd-php-source"),
    ("pht", "#application/x-httpd-php"),
    ("phtml", "#application/x-httpd-php"),
    ("pk", "application/x-tex-pk"),
    ("pl", "text/x-perl"),
    ("pls", "audio/x-scpls"),
    ("pm", "text/x-perl"),
    ("png", "image/png"),
    ("pnm", "image/x-portable-anymap"),
    ("pot", "text/plain"),
    ("potm", "application/vnd.ms-powerpoint.template.macroEnabled.12"),
    ("potx", "application/vnd.openxmlformats-officedocument.presentationml.template"),
    ("ppam", "application/vnd.ms-powerpoint.addin.macroEnabled.12"),
    ("ppm", "image/x-portable-pixmap"),
    ("pps", "application/vnd.ms-powerpoint"),
    ("ppsm", "application/vnd.ms-powerpoint.slideshow.macroEnabled.12"),
    ("ppsx", "application/vnd.openxmlformats-officedocument.presentationml.slideshow"),
    ("ppt", "application/vnd.ms-powerpoint"),
    ("pptm", "application/vnd.ms-powerpoint.presentation.macroEnabled.12"),
    ("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
    ("prf", "application/pics-rules"),
    ("prt", "chemical/x-ncbi-asn1-ascii"),
    ("ps", "application/postscript"),
    ("psd", "image/x-photoshop"),
    ("py", "text/x-python"),
    ("pyc", "application/x-python-code"),
    ("pyo", "application/x-python-code"),
    ("qgs", "application/x-qgis"),
    ("qt", "video/quicktime"),
    ("qtl", "application/x-quicktimeplayer"),
    ("ra", "audio/x-pn-realaudio"),
    ("ram", "audio/x-pn-realaudio"),
    ("rar", "application/rar"),
    ("ras", "image/x-cmu-raster"),
    ("rb", "application/x-ruby"),
    ("rd", "chemical/x-mdl-rdfile"),
    ("rdf", "application/rdf+xml"),
    ("rdp", "application/x-rdp"),
    ("rgb", "image/x-rgb"),
    ("rhtml", "#application/x-httpd-eruby"),
    ("rm", "audio/x-pn-realaudio"),
    ("roff", "application/x-troff"),
    ("ros", "chemical/x-rosdal"),
    ("rpm", "application/x-redhat-package-manager"),
    ("rss", "application/x-rss+xml"),
    ("rtf", "application/rtf"),
    ("rtx", "text/richtext"),
    ("rxn", "chemical/x-mdl-rxnfile"),
    ("scala", "text/x-scala"),
    ("sce", "application/x-scilab"),
    ("sci", "application/x-scilab"),
    ("sco", "audio/csound"),
    ("scr", "application/x-silverlight"),
    ("sct", "text/scriptlet"),
    ("sd", "chemical/x-mdl-sdfile"),
    ("sd2", "audio/x-sd2"),
    ("sda", "application/vnd.stardivision.draw"),
    ("sdc", "application/vnd.stardivision.calc"),
    ("sdd", "application/vnd.stardivision.impress"),
    ("sds", "application/vnd.stardivision.chart"),
    ("sdw", "application/vnd.stardivision.writer"),
    ("ser", "application/java-serialized-object"),
    ("sfd", "application/vnd.font-fontforge-sfd"),
    ("sfv", "text/x-sfv"),
    ("sgf", "application/x-go-sgf"),
    ("sgl", "application/vnd.stardivision.writer-global"),
    ("sh", "application/x-sh"),
    ("shar", "application/x-shar"),
    ("shp", "application/x-qgis"),
    ("shtml", "text/html"),
    ("shx", "application/x-qgis"),
    ("sid", "audio/prs.sid"),
    ("sig", "application/pgp-signature"),
    ("sik", "application/x-trash"),
    ("silo", "model/mesh"),
    ("sis", "application/vnd.symbian.install"),
    ("sisx", "x-epoc/x-sisx-app"),
    ("sit", "application/x-stuffit"),
    ("sitx", "application/x-stuffit"),
    ("skd", "application/x-koan"),
    ("skm", "application/x-koan"),
    ("skp", "application/x-koan"),
    ("skt", "application/x-koan"),
    ("sldm", "application/vnd.ms-powerpoint.slide.macroEnabled.12"),
    ("sldx", "application/vnd.openxmlformats-officedocument.presentationml.slide"),
    ("smi", "application/smil+xml"),
    ("smil", "application/smil+xml"),
    ("snd", "audio/basic"),
    ("spc", "chemical/x-galactic-spc"),
    ("spl", "application/x-futuresplash"),
    ("spx", "audio/ogg"),
    ("sql", "application/x-sql"),
    ("src", "application/x-wais-source"),
    ("srt", "text/plain"),
    ("stc", "application/vnd.sun.xml.calc.template"),
    ("std", "application/vnd.sun.xml.draw.template"),
    ("sti", "application/vnd.sun.xml.impress.template"),
    ("stw", "application/vnd.sun.xml.writer.template"),
    ("sty", "text/x-tex"),
    ("sv4cpio", "application/x-sv4cpio"),
    ("sv4crc", "application/x-sv4crc"),
    ("svg", "image/svg+xml"),
    ("svgz", "image/svg+xml"),
    ("sw", "chemical/x-swissprot"),
    ("swf", "application/x-shockwave-flash"),
    ("swfl", "application/x-shockwave-flash"),
    ("sxc", "application/vnd.sun.xml.calc"),
    ("sxd", "application/vnd.sun.xml.draw"),
    ("sxg", "application/vnd.sun.xml.writer.global"),
    ("sxi", "application/vnd.sun.xml.impress"),
    ("sxm", "application/vnd.sun.xml.math"),
    ("sxw", "application/vnd.sun.xml.writer"),
    ("t", "application/x-troff"),
    ("tar", "application/x-tar"),
    ("taz", "application/x-gtar-compressed"),
    ("tcl", "application/x-tcl"),
    ("tex", "text/x-tex"),
    ("texi", "application/x-texinfo"),
    ("texinfo", "application/x-texinfo"),
    ("text", "text/plain"),
    ("tgf", "chemical/x-mdl-tgf"),
    ("tgz", "application/x-gtar-compressed"),
    ("thmx", "application/vnd.ms-officetheme"),
    ("tif", "image/tiff"),
    ("tiff", "image/tiff"),
    ("tk", "text/x-tcl"),
    ("tm", "text/texmacs"),
    ("torrent", "application/x-bittorrent"),
    ("tr", "application/x-troff"),
    ("ts", "video/MP2T"),
    ("tsp", "application/dsptype"),
    ("tsv", "text/tab-separated-values"),
    ("ttf", "application/font-sfnt"),
    ("ttl", "text/turtle"),
    ("txt", "text/plain"),
    ("uls", "text/iuls"),
    ("ustar", "application/x-ustar"),
    ("val", "chemical/x-ncbi-asn1-binary"),
    ("vcard", "text/vcard"),
    ("vcd", "application/x-cdlink"),
    ("vcf", "text/vcard"),
    ("vcs", "text/x-vcalendar"),
    ("vmd", "chemical/x-vmd"),
    ("vms", "chemical/x-vamas-iso14976"),
    ("vrm", "x-world/x-vrml"),
    ("vrml", "model/vrml"),
    ("vsd", "application/vnd.visio"),
    ("vss", "application/vnd.visio"),
    ("vst", "application/vnd.visio"),
    ("vsw", "application/vnd.visio"),
    ("wad", "application/x-doom"),
    ("wasm", "application/wasm"),
    ("wav", "audio/x-wav"),
    ("wax", "audio/x-ms-wax"),
    ("wbmp", "image/vnd.wap.wbmp"),
    ("wbxml", "application/vnd.wap.wbxml"),
    ("webm", "video/webm"),
    ("wk", "application/x-123"),
    ("wm", "video/x-ms-wm"),
    ("wma", "audio/x-ms-wma"),
    ("wmd", "application/x-ms-wmd"),
    ("wml", "text/vnd.wap.wml"),
    ("wmlc", "application/vnd.wap.wmlc"),
    ("wmls", "text/vnd.wap.wmlscript"),
    ("wmlsc", "application/vnd.wap.wmlscriptc"),
    ("wmv", "video/x-ms-wmv"),
    ("wmx", "video/x-ms-wmx"),
    ("wmz", "application/x-ms-wmz"),
    ("woff", "application/font-woff"),
    ("wp5", "application/vnd.wordperfect5.1"),
    ("wpd", "application/vnd.wordperfect"),
    ("wrl", "model/vrml"),
    ("wsc", "text/scriptlet"),
    ("wvx", "video/x-ms-wvx"),
    ("wz", "application/x-wingz"),
    ("x3d", "model/x3d+xml"),
    ("x3db", "model/x3d+binary"),
    ("x3dv", "model/x3d+vrml"),
    ("xbm", "image/x-xbitmap"),
    ("xcf", "application/x-xcf"),
    ("xcos", "application/x-scilab-xcos"),
    ("xht", "application/xhtml+xml"),
    ("xhtml", "application/xhtml+xml"),
    ("xlam", "application/vnd.ms-excel.addin.macroEnabled.12"),
    ("xlb", "application/vnd.ms-excel"),
    ("xls", "application/vnd.ms-excel"),
    ("xlsb", "application/vnd.ms-excel.sheet.binary.macroEnabled.12"),
    ("xlsm", "application/vnd.ms-excel.sheet.macroEnabled.12"),
    ("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
    ("xlt", "application/vnd.ms-excel"),
    ("xltm", "application/vnd.ms-excel.template.macroEnabled.12"),
    ("xltx", "application/vnd.openxmlformats-officedocument.spreadsheetml.template"),
    ("xml", "application/xml"),
    ("xpi", "application/x-xpinstall"),
    ("xpm", "image/x-xpixmap"),
    ("xsd", "application/xml"),
    ("xsl", "application/xslt+xml"),
    ("xslt", "application/xslt+xml"),
    ("xspf", "application/xspf+xml"),
    ("xtel", "chemical/x-xtel"),
    ("xul", "application/vnd.mozilla.xul+xml"),
    ("xwd", "image/x-xwindowdump"),
    ("xyz", "chemical/x-xyz"),
    ("xz", "application/x-xz"),
    ("zip", "application/zip"),
]));


pub(crate) const LONG_DATE_FORMAT: &str = "%Y%m%dT%H%M%SZ";
pub(crate) const RFC1123_DATE_FORMAT: &str = "%a, %d %b %Y %H:%M:%S GMT";
pub(crate) const ISO8601_DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";
pub(crate) const ISO8601_DATE_FORMAT_TRUNCATED: &str = "%Y-%m-%d";
pub(crate) const UNDEFINED: &str = "undefined";
pub(crate) const CREDENTIALS_EXPIRES: i64 = 3600 * 10;
pub(crate) const CREDENTIALS_REFRESH_INTERVAL: u64 = 600;