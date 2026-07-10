//! The real store (feature `s3`, on by default) — `aws-sdk-s3` behind the
//! seam, thin enough to read whole (T3). This module is the ONLY place in
//! the project that names the SDK (S7); everything above the trait is
//! SDK-free by construction.
//!
//! A translation layer this thin has nowhere for logic bugs to live —
//! list-map plus put-passthrough — EXCEPT the one line where a thin layer
//! can still lie: the etag quote-strip below. Read that comment.
//!
//! ## Where "etag = md5" holds, and where it breaks (the honest paragraph)
//!
//! The idempotence compare (lake-sync's plan) treats the etag as the md5
//! of the body. That is guaranteed for objects written by a **single-part
//! PUT under SSE-S3** — exactly what this project does: bodies are
//! KB-scale (single PUT, no multipart), and the S10 stack mandates SSE-S3
//! on the bucket. It BREAKS under SSE-KMS (S3 returns an etag that is not
//! the body md5) and for multipart uploads (the etag becomes
//! `md5-of-part-md5s` + `-<part count>`). If either ever arrives here, the
//! compare degrades safely — etags stop matching, so files re-upload
//! (wasteful, never lossy) — and the fix is a different compare, not a
//! different law.

use crate::{ObjectMeta, ObjectStore, StoreError};

/// Re-exported so consumers can build the client without adding
/// `aws-sdk-s3` to their own manifest — the SDK stays named in exactly one
/// crate's dependency list (S7's "only in the real impl module", extended
/// to the dependency graph).
pub use aws_sdk_s3::Client as S3Client;

/// [`ObjectStore`] over one real bucket. The client is built by the caller
/// (async config loading happens once, in `main`, before any traffic —
/// the cold-start discipline hello-lambda's main.rs teaches) and cheap to
/// clone/share: it is an `Arc` around connection pools internally.
#[derive(Debug, Clone)]
pub struct S3Store {
    client: S3Client,
    bucket: String,
}

impl S3Store {
    /// Wrap a configured SDK client and a bucket name.
    pub fn new(client: S3Client, bucket: impl Into<String>) -> Self {
        S3Store {
            client,
            bucket: bucket.into(),
        }
    }
}

// teach: sugar in the IMPL, desugared in the TRAIT — see fake.rs's note on
// this split; the compiler checks these concrete futures (built from the
// SDK's own Send futures) against the trait's written `+ Send` promise.
impl ObjectStore for S3Store {
    async fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>, StoreError> {
        // ListObjectsV2 answers at most 1000 keys per page; the
        // paginator turns continuation tokens into a stream of pages
        // so callers of the seam never see pagination at all.
        let mut pages = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .into_paginator()
            .send();

        let mut found = Vec::new();
        while let Some(page) = pages.next().await {
            let page = page.map_err(|e| StoreError::list(prefix, e))?;
            for object in page.contents.unwrap_or_default() {
                found.push(ObjectMeta {
                    key: object.key.unwrap_or_default(),
                    // The SDK models Size as Option<i64> because the
                    // wire allows absence; a negative size cannot
                    // happen, so the fallback is a formality, not a
                    // guess.
                    size: object.size.and_then(|s| u64::try_from(s).ok()).unwrap_or(0),
                    // THE QUOTE-STRIP (design etag fact #1): S3 returns
                    // the ETag WRAPPED IN LITERAL DOUBLE QUOTES —
                    // `"9bb58f26192e4ba00f01e2e7b136bbd8"` — because
                    // HTTP's ETag header is a quoted-string. Compared
                    // raw against a computed bare-hex md5, every etag
                    // check fails, every file re-uploads, and the
                    // `uploaded 0` idempotence demo silently dies.
                    // Strip here, once, so the seam's contract
                    // (ObjectMeta.etag = bare hex) holds for both impls.
                    etag: object
                        .e_tag
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_owned(),
                });
            }
        }
        Ok(found)
    }

    async fn put(&self, key: String, body: Vec<u8>) -> Result<(), StoreError> {
        // Single-part PUT, whole body — files here are KB-scale, so
        // multipart (and its etag caveat, module docs) never enters.
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(aws_sdk_s3::primitives::ByteStream::from(body))
            .send()
            .await
            .map_err(|e| StoreError::put(&key, e))?;
        Ok(())
    }
}
