use crate::config::ConfigHolder;
use crate::enumeration::HttpMethodType::HttpMethodDelete;
use crate::http::HttpRequest;
use crate::internal::InputTranslator;
use crate::multipart::{CompleteMultipartUploadInput, CreateMultipartUploadInput};
use crate::object::{
    AppendObjectFromBufferInput, AppendObjectFromFileInput, AppendObjectInput, CopyObjectInput,
    DeleteObjectInput, ModifyObjectFromBufferInput, ModifyObjectFromFileInput, ModifyObjectInput,
    PutObjectFromBufferInput, PutObjectFromFileInput, PutObjectInput,
};
use crate::reader::{InternalReader, MultiBytes};
use bytes::Bytes;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;

// 覆盖调用方 setter 到真实请求转换；使用协议字面量，避免错误常量让测试同时放行。
macro_rules! check_if_match {
    ($name:ident, $input:expr, $body:ty) => {
        #[test]
        fn $name() {
            let mut input = $input;
            for condition in [
                None,
                Some("current-etag"),
                Some("\"current-etag\""),
                Some(""),
            ] {
                if let Some(value) = condition {
                    input.set_if_match(value);
                }
                let request: HttpRequest<$body> =
                    input.trans(Arc::new(ConfigHolder::default())).unwrap();
                assert_eq!(
                    request.header.get("If-Match").map(String::as_str),
                    condition.filter(|value| !value.is_empty())
                );
                assert!(!request.header.contains_key("x-tos-if-match"));
            }
        }
    };
}

check_if_match!(delete, DeleteObjectInput::new("bucket", "key"), ());
check_if_match!(
    copy,
    CopyObjectInput::new("bucket", "key", "source-bucket", "source-key"),
    ()
);
check_if_match!(put_stream, PutObjectInput::<()>::new("bucket", "key"), ());
check_if_match!(
    put_buffer,
    PutObjectFromBufferInput::new("bucket", "key"),
    InternalReader<MultiBytes>
);
check_if_match!(
    put_file,
    PutObjectFromFileInput::new("bucket", "key"),
    InternalReader<File>
);
check_if_match!(
    append_stream,
    AppendObjectInput::<()>::new("bucket", "key"),
    ()
);
check_if_match!(
    append_buffer,
    AppendObjectFromBufferInput::new("bucket", "key"),
    InternalReader<MultiBytes>
);
check_if_match!(
    append_file,
    AppendObjectFromFileInput::new("bucket", "key"),
    InternalReader<File>
);
check_if_match!(
    modify_stream,
    ModifyObjectInput::<()>::new("bucket", "key"),
    ()
);
check_if_match!(
    modify_buffer,
    ModifyObjectFromBufferInput::new("bucket", "key"),
    InternalReader<MultiBytes>
);
check_if_match!(
    modify_file,
    ModifyObjectFromFileInput::new("bucket", "key"),
    InternalReader<File>
);
check_if_match!(
    create_multipart,
    CreateMultipartUploadInput::new("bucket", "key"),
    ()
);
check_if_match!(
    complete_multipart,
    CompleteMultipartUploadInput::new_with_complete_all("bucket", "key", "upload-id", true),
    InternalReader<Cursor<Bytes>>
);

#[test]
fn delete_preserves_version_and_recursive_options() {
    let mut input = DeleteObjectInput::new_with_version_id("bucket", "dir/key", "version");
    input.set_if_match("etag");
    input.set_recursive(true);
    input.set_skip_trash(true);
    let request: HttpRequest<()> = input.trans(Arc::new(ConfigHolder::default())).unwrap();
    assert_eq!(request.method, HttpMethodDelete);
    assert_eq!(request.bucket, "bucket");
    assert_eq!(request.key, "dir/key");
    assert_eq!(
        request.header.get("If-Match").map(String::as_str),
        Some("etag")
    );
    assert_eq!(
        request.query,
        Some(HashMap::from([
            ("versionId", "version".to_string()),
            ("recursive", "true".to_string()),
            ("skipTrash", "true".to_string()),
        ]))
    );
}

#[test]
fn copy_keeps_source_condition_separate_from_destination() {
    let mut input = CopyObjectInput::new("bucket", "key", "source-bucket", "source-key");
    input.set_if_match("destination-etag");
    input.set_copy_source_if_match("source-etag");
    let request: HttpRequest<()> = input.trans(Arc::new(ConfigHolder::default())).unwrap();
    assert_eq!(
        request.header.get("If-Match").map(String::as_str),
        Some("destination-etag")
    );
    assert_eq!(
        request
            .header
            .get("x-tos-copy-source-if-match")
            .map(String::as_str),
        Some("source-etag")
    );
    assert!(!request.header.contains_key("x-tos-if-match"));
}

#[test]
fn multipart_if_none_match_remains_independent() {
    let mut create = CreateMultipartUploadInput::new("bucket", "key");
    create.set_if_none_match("*");
    let request: HttpRequest<()> = create.trans(Arc::new(ConfigHolder::default())).unwrap();
    assert_eq!(
        request.header.get("If-None-Match").map(String::as_str),
        Some("*")
    );
    assert!(!request.header.contains_key("If-Match"));

    let mut complete =
        CompleteMultipartUploadInput::new_with_complete_all("bucket", "key", "upload-id", true);
    complete.set_if_none_match("*");
    let request: HttpRequest<InternalReader<Cursor<Bytes>>> =
        complete.trans(Arc::new(ConfigHolder::default())).unwrap();
    assert_eq!(
        request.header.get("If-None-Match").map(String::as_str),
        Some("*")
    );
    assert!(!request.header.contains_key("If-Match"));
}
