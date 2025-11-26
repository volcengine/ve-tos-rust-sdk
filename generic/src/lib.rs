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
extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;
#[proc_macro_derive(BucketSetter)]
pub fn derive_bucket_getter_setter(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl #ident{
            pub fn new(bucket: impl Into<String>) -> Self {
                let mut input = Self::default();
                input.bucket = bucket.into();
                input
            }

            pub fn bucket(&self) -> &str {
                &self.bucket
            }
            pub fn set_bucket(&mut self, bucket: impl Into<String>) {
                self.bucket = bucket.into();
            }
        }
    };
    output.into()
}
#[proc_macro_derive(ObjectLock, attributes(
    handle_stream,
    handle_async,
    handle_content,
    use_inner,
))]
pub fn derive_object_lock(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }

    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn object_lock_mode(&self) -> &Option<crate::enumeration::ObjectLockModeType>{
                &#self_ptr.object_lock_mode
            }
            pub fn object_lock_retain_util_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.object_lock_retain_util_date
            }
            pub fn set_object_lock_mode(&mut self, object_lock_mode: impl Into<crate::enumeration::ObjectLockModeType>) {
                self.object_lock_mode = Some(object_lock_mode.into());
            }
            pub fn set_object_lock_retain_util_date(&mut self, object_lock_retain_util_date: impl Into<chrono::DateTime<chrono::Utc>>) {
                self.object_lock_retain_util_date = Some(object_lock_retain_util_date.into());
            }
        }

        #impl_stream crate::internal::ObjectLock for #ident #stream_bound{
            fn object_lock_mode(&self) -> &Option<crate::enumeration::ObjectLockModeType>{
                &#self_ptr.object_lock_mode
            }
            fn object_lock_retain_util_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.object_lock_retain_util_date
            }
        }
    };
    output.into()
}

#[proc_macro_derive(IfMatch, attributes(
    handle_stream,
    handle_async,
    handle_content,
    use_inner,
))]
pub fn derive_if_match(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }

    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn if_match(&self) -> &str{
                &#self_ptr.if_match
            }
            pub fn if_none_match(&self) -> &str{
                &#self_ptr.if_none_match
            }
            pub fn set_if_match(&mut self, if_match: impl Into<String>) {
                self.if_match = if_match.into();
            }
            pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
                self.if_none_match = if_none_match.into();
            }
        }

        #impl_stream crate::internal::IfMatch for #ident #stream_bound{
            fn if_match(&self) -> &str{
                &#self_ptr.if_match
            }
            fn if_none_match(&self) -> &str{
                &#self_ptr.if_none_match
            }
        }
    };
    output.into()
}

#[proc_macro_derive(RequestInfo)]
pub fn derive_request_info(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let output = quote! {
        impl #ident{
            pub fn request_id(&self) -> &str {
                &self.request_info.request_id
            }

            pub fn id2(&self) -> &str {
                &self.request_info.id2
            }

            pub fn status_code(&self) -> isize {
                self.request_info.status_code
            }

            pub fn header(&self) -> &std::collections::HashMap<String, String> {
                &self.request_info.header
            }
        }

        impl crate::common::RequestInfoTrait for #ident{
            fn request_id(&self) -> &str {
                &self.request_info.request_id
            }

            fn id2(&self) -> &str {
                &self.request_info.id2
            }

            fn status_code(&self) -> isize {
                self.request_info.status_code
            }

            fn header(&self) -> &std::collections::HashMap<String, String> {
                &self.request_info.header
            }
        }
    };
    output.into()
}

#[proc_macro_derive(GenericInput, attributes(
    handle_stream,
    use_inner,
    handle_async,
    handle_content,
))]
pub fn derive_generic_input(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self.generic_input};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner.generic_input};
        }
    }

    let output = quote! {
         #impl_stream #ident #stream_bound{
            pub fn request_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.request_date
            }

            pub fn request_host(&self) -> &str {
                &#self_ptr.request_host
            }

            pub fn request_header(&self) -> &Option<std::collections::HashMap<String, String>>{
                &#self_ptr.request_header
            }

            pub fn request_query(&self) -> &Option<std::collections::HashMap<String, String>>{
                 &#self_ptr.request_query
            }

            pub fn set_request_date(&mut self, request_date: impl Into<chrono::DateTime<chrono::Utc>>) {
                #self_ptr.request_date = Some(request_date.into());
            }

            pub fn set_request_host(&mut self, request_host: impl Into<String>) {
                #self_ptr.request_host = request_host.into();
            }

            pub fn set_request_header(&mut self, request_header: impl Into<std::collections::HashMap<String, String>>) {
                #self_ptr.request_header = Some(request_header.into());
            }

            pub fn set_request_query(&mut self, request_query: impl Into<std::collections::HashMap<String, String>>) {
                #self_ptr.request_query = Some(request_query.into());
            }
        }

        #impl_stream crate::common::GenericInputTrait for #ident #stream_bound{
            fn request_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.request_date
            }

            fn request_host(&self) -> &str {
                &#self_ptr.request_host
            }

            fn request_header(&self) -> &Option<std::collections::HashMap<String, String>>{
                &#self_ptr.request_header
            }

            fn request_query(&self) -> &Option<std::collections::HashMap<String, String>>{
                 &#self_ptr.request_query
            }
        }
    };
    output.into()
}

#[proc_macro_derive(FromRefAndDisplay)]
pub fn derive_from_ref(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
         impl std::fmt::Display for #ident{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
         }

         impl From<&#ident> for #ident{
            fn from(value: &#ident) -> Self {
                value.to_owned()
            }
         }

    };
    output.into()
}


#[proc_macro_derive(AclHeader, attributes(
    enable_grant_write,
    handle_stream,
    use_inner,
    handle_async,
    handle_content,
))]
pub fn derive_acl_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut visible = quote! {pub(crate) };
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("enable_grant_write") {
            visible = quote! {pub };
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }

    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn acl(&self) -> &Option<crate::enumeration::ACLType> {
                &#self_ptr.acl
            }
            pub fn grant_full_control(&self) -> &str {
                &#self_ptr.grant_full_control
            }
            pub fn grant_read(&self) -> &str {
                &#self_ptr.grant_read
            }
            pub fn grant_read_acp(&self) -> &str {
                &#self_ptr.grant_read_acp
            }
            #visible fn grant_write(&self) -> &str {
                &#self_ptr.grant_write
            }
            pub fn grant_write_acp(&self) -> &str {
                &#self_ptr.grant_write_acp
            }
            pub fn set_acl(&mut self, acl: impl Into<crate::enumeration::ACLType>) {
                #self_ptr.acl = Some(acl.into());
            }
            pub fn set_grant_full_control(&mut self, grant_full_control: impl Into<String>) {
                #self_ptr.grant_full_control = grant_full_control.into();
            }
            pub fn set_grant_read(&mut self, grant_read: impl Into<String>) {
                #self_ptr.grant_read = grant_read.into();
            }
            pub fn set_grant_read_acp(&mut self, grant_read_acp: impl Into<String>) {
                #self_ptr.grant_read_acp = grant_read_acp.into();
            }
            #visible fn set_grant_write(&mut self, grant_write: impl Into<String>) {
                #self_ptr.grant_write = grant_write.into();
            }
            pub fn set_grant_write_acp(&mut self, grant_write_acp: impl Into<String>) {
                #self_ptr.grant_write_acp = grant_write_acp.into();
            }
        }

        #impl_stream crate::internal::AclHeader for #ident #stream_bound{
            fn acl(&self) -> &Option<crate::enumeration::ACLType> {
                &#self_ptr.acl
            }
            fn grant_full_control(&self) -> &str {
                &#self_ptr.grant_full_control
            }
            fn grant_read(&self) -> &str {
                &#self_ptr.grant_read
            }
            fn grant_read_acp(&self) -> &str {
                &#self_ptr.grant_read_acp
            }
            fn grant_write(&self) -> &str {
                &#self_ptr.grant_write
            }
            fn grant_write_acp(&self) -> &str {
                &#self_ptr.grant_write_acp
            }
        }
    };
    output.into()
}

#[proc_macro_derive(HttpBasicHeader, attributes(
    enable_content_length,
    handle_stream,
    use_inner,
    handle_async,
    handle_content,
))]
pub fn derive_http_basic_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut visable = quote! {pub(crate) };
    let mut content_length = quote! {-1};
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("enable_content_length") {
            visable = quote! {pub };
            content_length = quote! { #self_ptr.content_length };
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
            content_length = quote! { #self_ptr.content_length };
        }
    }

    let output = quote! {
        #impl_stream #ident #stream_bound{
            #visable fn content_length(&self) -> i64{
                #content_length
            }
            pub fn cache_control(&self) -> &str {
                &#self_ptr.cache_control
            }
            pub fn content_disposition(&self) -> &str {
                &#self_ptr.content_disposition
            }
            pub fn content_encoding(&self) -> &str {
                &#self_ptr.content_encoding
            }
            pub fn content_language(&self) -> &str {
                &#self_ptr.content_language
            }
            pub fn content_type(&self) -> &str {
                &#self_ptr.content_type
            }
            pub fn expires(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.expires
            }
            #visable fn set_content_length(&mut self, content_length: i64){
                #self_ptr.content_length = content_length;
            }
            pub fn set_cache_control(&mut self, cache_control: impl Into<String>) {
                #self_ptr.cache_control = cache_control.into();
            }
            pub fn set_content_disposition(&mut self, content_disposition: impl Into<String>) {
                #self_ptr.content_disposition = content_disposition.into();
            }
            pub fn set_content_encoding(&mut self, content_encoding: impl Into<String>) {
                #self_ptr.content_encoding = content_encoding.into();
            }
            pub fn set_content_language(&mut self, content_language: impl Into<String>) {
                #self_ptr.content_language = content_language.into();
            }
            pub fn set_content_type(&mut self, content_type: impl Into<String>) {
                #self_ptr.content_type = content_type.into();
            }
            pub fn set_expires(&mut self, expires: impl Into<chrono::DateTime<chrono::Utc>>) {
                #self_ptr.expires = Some(expires.into());
            }
        }

        #impl_stream crate::internal::HttpBasicHeader for #ident #stream_bound{
            fn content_length(&self) -> i64{
                #content_length
            }
            fn cache_control(&self) -> &str {
                 &#self_ptr.cache_control
            }
            fn content_disposition(&self) -> &str {
                &#self_ptr.content_disposition
            }
            fn content_encoding(&self) -> &str {
                &#self_ptr.content_encoding
            }
            fn content_language(&self) -> &str {
                &#self_ptr.content_language
            }
            fn content_type(&self) -> &str {
                &#self_ptr.content_type
            }
            fn expires(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.expires
            }
        }
    };
    output.into()
}

#[proc_macro_derive(MiscHeader, attributes(handle_stream, use_inner, handle_async, handle_content))]
pub fn derive_misc_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }
    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn website_redirect_location(&self) -> &str{
                &#self_ptr.website_redirect_location
            }
            pub fn storage_class(&self) -> &Option<crate::enumeration::StorageClassType>{
                &#self_ptr.storage_class
            }
            pub fn set_website_redirect_location(&mut self, website_redirect_location: impl Into<String>){
                #self_ptr.website_redirect_location = website_redirect_location.into();
            }
            pub fn set_storage_class(&mut self, storage_class: impl Into<crate::enumeration::StorageClassType>){
                #self_ptr.storage_class = Some(storage_class.into());
            }
        }

        #impl_stream crate::internal::MiscHeader for #ident #stream_bound{
            fn website_redirect_location(&self) -> &str{
                &#self_ptr.website_redirect_location
            }
            fn storage_class(&self) -> &Option<crate::enumeration::StorageClassType>{
                &#self_ptr.storage_class
            }
        }
    };
    output.into()
}

#[proc_macro_derive(SseHeader, attributes(handle_stream, use_inner, handle_async, handle_content))]
pub fn derive_sse_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }
    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn server_side_encryption(&self) -> &str{
                &#self_ptr.server_side_encryption
            }
            pub fn server_side_encryption_key_id(&self) -> &str{
                &#self_ptr.server_side_encryption_key_id
            }
            pub fn set_server_side_encryption(&mut self, server_side_encryption: impl Into<String>){
                #self_ptr.server_side_encryption = server_side_encryption.into();
            }
            pub fn set_server_side_encryption_key_id(&mut self, server_side_encryption_key_id: impl Into<String>){
                #self_ptr.server_side_encryption_key_id = server_side_encryption_key_id.into();
            }
        }

        #impl_stream crate::internal::SseHeader for #ident #stream_bound{
            fn server_side_encryption(&self) -> &str{
                &#self_ptr.server_side_encryption
            }
            fn server_side_encryption_key_id(&self) -> &str{
                &#self_ptr.server_side_encryption_key_id
            }
        }

    };
    output.into()
}

#[proc_macro_derive(SsecHeader, attributes(handle_stream, use_inner, handle_async, handle_content))]
pub fn derive_ssec_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }
    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn ssec_algorithm(&self) -> &str{
                &#self_ptr.ssec_algorithm
            }
            pub fn ssec_key(&self) -> &str{
                &#self_ptr.ssec_key
            }
            pub fn ssec_key_md5(&self) -> &str{
                &#self_ptr.ssec_key_md5
            }
            pub fn set_ssec_algorithm(&mut self, ssec_algorithm: impl Into<String>){
                #self_ptr.ssec_algorithm = ssec_algorithm.into();
            }
            pub fn set_ssec_key(&mut self, ssec_key: impl Into<String>){
                #self_ptr.ssec_key = ssec_key.into();
            }
            pub fn set_ssec_key_md5(&mut self, ssec_key_md5: impl Into<String>){
                #self_ptr.ssec_key_md5 = ssec_key_md5.into();
            }
        }

        #impl_stream crate::internal::SsecHeader for #ident #stream_bound{
            fn ssec_algorithm(&self) -> &str{
                &#self_ptr.ssec_algorithm
            }
            fn ssec_key(&self) -> &str{
                &#self_ptr.ssec_key
            }
            fn ssec_key_md5(&self) -> &str{
                &#self_ptr.ssec_key_md5
            }
        }

    };
    output.into()
}


#[proc_macro_derive(CopySourceHeader)]
pub fn derive_copy_source_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl #ident{
            pub fn src_bucket(&self) -> &str{
                &self.src_bucket
            }
            pub fn src_key(&self) -> &str{
                &self.src_key
            }
            pub fn src_version_id(&self) -> &str{
                &self.src_version_id
            }
            pub fn set_src_bucket(&mut self, src_bucket: impl Into<String>) {
                self.src_bucket = src_bucket.into();
            }
            pub fn set_src_key(&mut self, src_key: impl Into<String>) {
                self.src_key = src_key.into();
            }
            pub fn set_src_version_id(&mut self, src_version_id: impl Into<String>) {
                self.src_version_id = src_version_id.into();
            }
        }

        impl crate::internal::CopySourceHeader for #ident{
            fn src_bucket(&self) -> &str{
                &self.src_bucket
            }
            fn src_key(&self) -> &str{
                &self.src_key
            }
            fn src_version_id(&self) -> &str{
                &self.src_version_id
            }
        }

    };
    output.into()
}

#[proc_macro_derive(CopySourceSSecHeader)]
pub fn derive_copy_source_ssec_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl #ident{
            pub fn copy_source_ssec_algorithm(&self) -> &str{
                &self.copy_source_ssec_algorithm
            }
            pub fn copy_source_ssec_key(&self) -> &str{
                &self.copy_source_ssec_key
            }
            pub fn copy_source_ssec_key_md5(&self) -> &str{
                &self.copy_source_ssec_key_md5
            }
            pub fn set_copy_source_ssec_algorithm(&mut self, copy_source_ssec_algorithm: impl Into<String>){
                self.copy_source_ssec_algorithm = copy_source_ssec_algorithm.into();
            }
            pub fn set_copy_source_ssec_key(&mut self, copy_source_ssec_key: impl Into<String>){
                self.copy_source_ssec_key = copy_source_ssec_key.into();
            }
            pub fn set_copy_source_ssec_key_md5(&mut self, copy_source_ssec_key_md5: impl Into<String>){
                self.copy_source_ssec_key_md5 = copy_source_ssec_key_md5.into();
            }
        }

        impl crate::internal::CopySourceSSecHeader for #ident{
            fn copy_source_ssec_algorithm(&self) -> &str{
                &self.copy_source_ssec_algorithm
            }
            fn copy_source_ssec_key(&self) -> &str{
                &self.copy_source_ssec_key
            }
            fn copy_source_ssec_key_md5(&self) -> &str{
                &self.copy_source_ssec_key_md5
            }
        }
    };
    output.into()
}

#[proc_macro_derive(IfConditionHeader, attributes(use_inner))]
pub fn derive_if_condition_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }
    let output = quote! {
        impl #ident{
            pub fn if_match(&self) -> &str{
                &#self_ptr.if_match
            }
            pub fn if_modified_since(&self) -> Option<chrono::DateTime<chrono::Utc>>{
                #self_ptr.if_modified_since
            }
            pub fn if_none_match(&self) -> &str{
                &#self_ptr.if_none_match
            }
            pub fn if_unmodified_since(&self) -> Option<chrono::DateTime<chrono::Utc>>{
                #self_ptr.if_unmodified_since
            }
            pub fn set_if_match(&mut self, if_match: impl Into<String>){
                #self_ptr.if_match = if_match.into();
            }
            pub fn set_if_modified_since(&mut self, if_modified_since: impl Into<chrono::DateTime<chrono::Utc>>){
                #self_ptr.if_modified_since = Some(if_modified_since.into());
            }
            pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>){
                #self_ptr.if_none_match = if_none_match.into();
            }
            pub fn set_if_unmodified_since(&mut self, if_unmodified_since: impl Into<chrono::DateTime<chrono::Utc>>){
                #self_ptr.if_unmodified_since = Some(if_unmodified_since.into())
            }
        }

        impl crate::internal::IfConditionHeader for #ident{
            fn if_match(&self) -> &str{
                &#self_ptr.if_match
            }
            fn if_modified_since(&self) -> Option<chrono::DateTime<chrono::Utc>>{
                #self_ptr.if_modified_since
            }
            fn if_none_match(&self) -> &str{
                &#self_ptr.if_none_match
            }
            fn if_unmodified_since(&self) -> Option<chrono::DateTime<chrono::Utc>>{
                #self_ptr.if_unmodified_since
            }
        }
    };
    output.into()
}

#[proc_macro_derive(CopySourceIfConditionHeader)]
pub fn derive_copy_source_if_condition_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl #ident{
            pub fn copy_source_if_match(&self) -> &str{
                &self.copy_source_if_match
            }
            pub fn copy_source_if_modified_since(&self) -> Option< chrono::DateTime<chrono::Utc>>{
                self.copy_source_if_modified_since
            }
            pub fn copy_source_if_none_match(&self) -> &str{
                &self.copy_source_if_none_match
            }
            pub fn copy_source_if_unmodified_since(&self) -> Option<chrono::DateTime<chrono::Utc>>{
                self.copy_source_if_unmodified_since
            }
            pub fn set_copy_source_if_match(&mut self, copy_source_if_match: impl Into<String>){
                self.copy_source_if_match = copy_source_if_match.into();
            }
            pub fn set_copy_source_if_modified_since(&mut self, copy_source_if_modified_since: impl Into<chrono::DateTime<chrono::Utc>>){
                self.copy_source_if_modified_since = Some(copy_source_if_modified_since.into());
            }
            pub fn set_copy_source_if_none_match(&mut self, copy_source_if_none_match: impl Into<String>){
                self.copy_source_if_none_match = copy_source_if_none_match.into();
            }
            pub fn set_copy_source_if_unmodified_since(&mut self, copy_source_if_unmodified_since: impl Into<chrono::DateTime<chrono::Utc>>){
                self.copy_source_if_unmodified_since = Some(copy_source_if_unmodified_since.into())
            }
        }

        impl crate::internal::CopySourceIfConditionHeader for #ident{
            fn copy_source_if_match(&self) -> &str{
                &self.copy_source_if_match
            }
            fn copy_source_if_modified_since(&self) -> Option<chrono::DateTime<chrono::Utc>>{
                self.copy_source_if_modified_since
            }
            fn copy_source_if_none_match(&self) -> &str{
                &self.copy_source_if_none_match
            }
            fn copy_source_if_unmodified_since(&self) -> Option<chrono::DateTime<chrono::Utc>>{
                self.copy_source_if_unmodified_since
            }
        }
    };
    output.into()
}

#[proc_macro_derive(CallbackHeader, attributes(
    handle_stream,
    use_inner,
    handle_async,
    handle_content
))]
pub fn derive_callback_header(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::CommonError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }
    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn callback(&self) -> &str {
                &#self_ptr.callback
            }
            pub fn callback_var(&self) -> &str {
                &#self_ptr.callback_var
            }
            pub fn set_callback(&mut self, callback: impl Into<String>) {
                #self_ptr.callback = callback.into();
            }
            pub fn set_callback_var(&mut self, callback_var: impl Into<String>) {
                #self_ptr.callback_var = callback_var.into();
            }
        }

        #impl_stream crate::internal::CallbackHeader for #ident #stream_bound{
            fn callback(&self) -> &str {
                &#self_ptr.callback
            }
            fn callback_var(&self) -> &str {
                &#self_ptr.callback_var
            }
        }
    };
    output.into()
}

#[proc_macro_derive(ListCommonQuery)]
pub fn derive_list_common_query(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl #ident{
            pub fn prefix(&self) -> &str {
                &self.prefix
            }
            pub fn delimiter(&self) -> &str {
                &self.delimiter
            }
            pub fn encoding_type(&self) -> &str {
                &self.encoding_type
            }
            pub fn set_prefix(&mut self, prefix: impl Into<String>) {
                self.prefix = prefix.into();
            }
            pub fn set_delimiter(&mut self, delimiter: impl Into<String>) {
                self.delimiter = delimiter.into();
            }
            pub fn set_encoding_type(&mut self, encoding_type: impl Into<String>) {
                self.encoding_type = encoding_type.into();
            }
        }

        impl crate::internal::ListCommonQuery for #ident{
            fn prefix(&self) -> &str {
                &self.prefix
            }
            fn delimiter(&self) -> &str {
                &self.delimiter
            }
            fn encoding_type(&self) -> &str {
                &self.encoding_type
            }
        }
    };
    output.into()
}

#[proc_macro_derive(MultipartUploadQuery, attributes(
    handle_stream,
    use_inner,
    handle_async,
    handle_content
))]
pub fn derive_multipart_upload_query(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut impl_stream = quote! {impl};
    let mut stream_bound = quote! {};
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("handle_stream") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: std::io::Read + Send + 'static};
        } else if attr.path().is_ident("handle_async") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B> where B: Stream<Item=Result<bytes::Bytes, crate::error::TosError>> + Send};
        } else if attr.path().is_ident("handle_content") {
            impl_stream = quote! {impl<B>};
            stream_bound = quote! {<B>};
        } else if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }
    let output = quote! {
        #impl_stream #ident #stream_bound{
            pub fn upload_id(&self) -> &str{
                &#self_ptr.upload_id
            }
            pub fn part_number(&self) -> isize{
                #self_ptr.part_number
            }
            pub fn set_upload_id(&mut self, upload_id: impl Into<String>){
                #self_ptr.upload_id = upload_id.into();
            }
            pub fn set_part_number(&mut self, part_number: isize) {
                #self_ptr.part_number = part_number;
            }
        }

         #impl_stream crate::internal::MultipartUploadQuery for #ident #stream_bound{
            fn upload_id(&self) -> &str{
                &#self_ptr.upload_id
            }
            fn part_number(&self) -> isize{
                #self_ptr.part_number
            }
        }
    };
    output.into()
}

#[proc_macro_derive(RewriteResponseQuery, attributes(
    use_inner,
))]
pub fn derive_rewrite_response_query(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }
    let output = quote! {
        impl #ident {
            pub fn response_cache_control(&self) -> &str {
                &#self_ptr.response_cache_control
            }
            pub fn response_content_disposition(&self) -> &str {
                &#self_ptr.response_content_disposition
            }
            pub fn response_content_encoding(&self) -> &str {
                &#self_ptr.response_content_encoding
            }
            pub fn response_content_language(&self) -> &str {
                &#self_ptr.response_content_language
            }
            pub fn response_content_type(&self) -> &str {
                &#self_ptr.response_content_type
            }
            pub fn response_expires(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.response_expires
            }

            pub fn set_response_cache_control(&mut self, response_cache_control: impl Into<String>) {
                #self_ptr.response_cache_control = response_cache_control.into();
            }
            pub fn set_response_content_disposition(&mut self, response_content_disposition: impl Into<String>) {
                #self_ptr.response_content_disposition = response_content_disposition.into();
            }
            pub fn set_response_content_encoding(&mut self, response_content_encoding: impl Into<String>) {
                #self_ptr.response_content_encoding = response_content_encoding.into();
            }
            pub fn set_response_content_language(&mut self, response_content_language: impl Into<String>) {
                #self_ptr.response_content_language = response_content_language.into();
            }
            pub fn set_response_content_type(&mut self, response_content_type: impl Into<String>) {
                #self_ptr.response_content_type = response_content_type.into();
            }
            pub fn set_response_expires(&mut self, response_expires: impl Into<chrono::DateTime<chrono::Utc>>) {
                #self_ptr.response_expires = Some(response_expires.into());
            }
        }

         impl crate::internal::RewriteResponseQuery for #ident{
             fn response_cache_control(&self) -> &str {
                &#self_ptr.response_cache_control
            }
             fn response_content_disposition(&self) -> &str {
                &#self_ptr.response_content_disposition
            }
             fn response_content_encoding(&self) -> &str {
                &#self_ptr.response_content_encoding
            }
             fn response_content_language(&self) -> &str {
                &#self_ptr.response_content_language
            }
             fn response_content_type(&self) -> &str {
                &#self_ptr.response_content_type
            }
             fn response_expires(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                #self_ptr.response_expires
            }
        }
    };
    output.into()
}

#[proc_macro_derive(DataProcessQuery, attributes(
    use_inner,
))]
pub fn derive_data_process_query(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let mut self_ptr = quote! {self};
    for attr in attrs.iter() {
        if attr.path().is_ident("use_inner") {
            self_ptr = quote! {self.inner};
        }
    }

    let output = quote! {
        impl #ident {
            pub fn process(&self) -> &str {
                &#self_ptr.process
            }
            pub fn doc_page(&self) -> isize {
                #self_ptr.doc_page
            }
            pub fn src_type(&self) -> &Option<crate::enumeration::DocPreviewSrcType> {
                &#self_ptr.src_type
            }
            pub fn dst_type(&self) -> &Option<crate::enumeration::DocPreviewDstType> {
                &#self_ptr.dst_type
            }
            pub fn save_bucket(&self) -> &str {
                &#self_ptr.save_bucket
            }
            pub fn save_object(&self) -> &str {
                &#self_ptr.save_object
            }

            pub fn set_process(&mut self, process: impl Into<String>) {
                #self_ptr.process = process.into();
            }
            pub fn set_doc_page(&mut self, doc_page: isize) {
                #self_ptr.doc_page = doc_page;
            }
            pub fn set_src_type(&mut self, src_type: impl Into<crate::enumeration::DocPreviewSrcType>) {
                #self_ptr.src_type = Some(src_type.into());
            }
            pub fn set_dst_type(&mut self, dst_type: impl Into<crate::enumeration::DocPreviewDstType>) {
                #self_ptr.dst_type = Some(dst_type.into());
            }
            pub fn set_save_bucket(&mut self, save_bucket: impl Into<String>) {
                #self_ptr.save_bucket = save_bucket.into();
            }
            pub fn set_save_object(&mut self, save_object: impl Into<String>) {
                #self_ptr.save_object = save_object.into();
            }
        }

        impl crate::internal::DataProcessQuery for #ident{
            fn process(&self) -> &str {
                &#self_ptr.process
            }
            fn doc_page(&self) -> isize {
                #self_ptr.doc_page
            }
            fn src_type(&self) -> &Option<crate::enumeration::DocPreviewSrcType> {
                &#self_ptr.src_type
            }
            fn dst_type(&self) -> &Option<crate::enumeration::DocPreviewDstType> {
                &#self_ptr.dst_type
            }
            fn save_bucket(&self) -> &str {
                &#self_ptr.save_bucket
            }
            fn save_object(&self) -> &str {
                &#self_ptr.save_object
            }
        }
    };
    output.into()
}
