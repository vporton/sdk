# Asset Canister Interface

## Overview

The asset canister stores static assets, such as HTML, CSS, JavaScript, images, and other media files. It can store different content encodings of the same asset, such as `identity` and `gzip`.

While the size of any given asset content encoding is limited only by the canister's available memory, the amount of data that can be passed to or returned by a single method call is limited.  For this reason, the interface provides for data upload and retrieval in smaller pieces, called "chunks".

This document is meant to describe the interface, not the idiosyncrasies of the implementation.

## Storing Assets

### Batch Updates

Message ingress limits constrain the size of a single update to the asset canister. To work around this, the asset canister supports batch updates.

To upload data, first create a batch, then upload chunks to the batch, then commit the batch.

#### Batch Expiry

Batches for which `create_chunk()` has not been called within a certain time period may be deleted.

### Batch Updates By Proposal

For canisters controlled by an SNS, the asset canister supports updates by proposal. In this scenario, one principal uploads the proposed changes, which the SNS will commit if the proposal is accepted.

### Individual Updates

## Retrieving Assets

### HTTP Requests

#### Streaming

## Other Methods

## Type Reference

### Asset {#type-asset}

#### Key {#field-asset-key}

The `key` identifies the asset. It is a string that must be unique within the asset canister. By convention, it should begin with a forward slash. For example, `/index.html`.

#### Content Type {#field-asset-content-type}

The `content_type` is a string that identifies the type of the asset, such as `text/plain` or `image/jpeg`. It is used to set the `Content-Type` header when serving the asset over HTTP.

#### Content Encoding {#field-asset-content-encoding}

An asset contains one or more encodings. Each encoding is identified by a `content_encoding` string, such as `identity` or `gzip`. It is used to set the `Content-Encoding` header when serving the asset over HTTP.

#### Content Chunks

Each encoding contains one or more "chunks" of data. The size of each chunk is limited by the message ingress limit.

Content chunks can have any size that fits within the message ingress limit, but for a given asset encoding, all chunks except the last must have the same size.

#### Content Hash {#field-asset-sha256}

The `sha256` field contains the SHA-256 hash of the entire asset encoding. It is used to set the `ETag` header when serving the asset over HTTP.

#### Max Age {#field-asset-max-age}

The `max_age` field is the maximum number of seconds that the asset can be cached by a browser or CDN. It is used to set the `max-age` value of the `Cache-Control` header when serving the asset over HTTP.

#### Headers {#field-asset-headers}

The `headers` field is a list of additional headers to set when serving the asset over HTTP.

#### Aliasing {#field-asset-enable-aliasing}

The `is_enabled` field should be called `enable_aliasing`.  It enables retrieval of an asset with a different key, if the requested key does not match any asset.

The rules are as follows:

- an attempt to retrieve `{some key}/` can instead retrieve `{some key}/index.html`
- an attempt to retrieve `{some key}`, where `{some key}` does not end with `.html`, can instead retrieve either `{some key}.html` or `{some key}/index.html`

#### Raw Access {#field-asset-allow-raw-access}

The `allow_raw_access` field controls whether an asset can be retrieved from `raw.ic0.app` or `raw.icp0.io`.  If false (which is the default), then the asset canister will redirect any such attempts to the non-raw URL.

### Batch

### Chunk

## Method Reference

### HTTP Methods

### Retrieval Methods

#### Method: `get`

### Batch Update Methods

#### Method: `create_batch` {#method-create-batch}

This method creates a new [batch](#batch) and returns its ID.

Preconditions:
- No batch exists for which [propose_commit_batch](#method-propose_commit_batch) has been called.
- Creation of a new batch would not exceed the batch creation limits.

#### Method: `create_chunk` {#method-create-chunk}

This method creates a new chunk.

The `create_chunk` method:
- Verifies that the batch exists.
- Verifies that creating the chunk would not exceed the chunk creation limits.
- Creates the new chunk.
- Extends the batch expiry time.
- Returns the ID of the new chunk.

#### Method: `commit_batch` {#method-commit-batch}

The `commit_batch` method:
- Executes each operation in the method arguments.
- Deletes the batch.

It is not required that the batch ID passed in the method arguments matches any batch ID.

The `commit_batch` method accepts a list of batch operations.

| Operation | Description |
| --------- | ----------- | 
| `CreateAsset` | Creates a new asset. |
| `SetAssetContent` | Adds or changes content for an asset. |
| `SetAssetProperties` | Changes properties for an asset. |
| `UnsetAssetContent` | Removes content for an asset. |
| `DeleteAsset` | Deletes an asset. |
| `Clear` | Deletes all assets. |

#### Method: `delete_batch` {#method-delete-batch}


### Update-By-Proposal Methods

#### Method: `propose_commit_batch`

This method takes the same arguments as `commit_batch`, but does not execute the operations. Instead, it stores the operations in a "proposed batch" for later execution by the `commit_proposed_batch` method.

#### Method: `compute_evidence`

The `compute_evidence` method computes a hash over the proposed commit batch arguments.

Since calculation of this hash may exceed the per-message computation limits, this method computes the hash iteratively, saving its work as it goes. Once it completes the computation, it saves the hash as `evidence` to be checked later.

The method will return `None` if the hash computation has not yet completed, or `Some(evidence)` if the hash computation has been completed.

The returned `evidence` value must be passed to the `commit_proposed_batch` method.

#### Method: `commit_proposed_batch` {#method-commit-proposed-batch}

The `commit_proposed_batch` method:
- Verifies that the batch exists.
- Verifies that the batch has proposed commit batch arguments.
- Verifies that the evidence has been computed by `compute_evidence`.
- Verifies that the evidence passed in the arguments matches the evidence computed by `compute_evidence`.
- Executes each operation in the proposed commit batch arguments.
- Deletes the batch.


### Access Control Methods




## API Versions

### API Version 1



## Candid Definition

```candid
type BatchId = nat;
type ChunkId = nat;
type Key = text;
type Time = int;

type CreateAssetArguments = record {
  key: Key;
  content_type: text;
  max_age: opt nat64;
  headers: opt vec HeaderField;
  enable_aliasing: opt bool;
  allow_raw_access: opt bool;
};

// Add or change content for an asset, by content encoding
type SetAssetContentArguments = record {
  key: Key;
  content_encoding: text;
  chunk_ids: vec ChunkId;
  sha256: opt blob;
};

// Remove content for an asset, by content encoding
type UnsetAssetContentArguments = record {
  key: Key;
  content_encoding: text;
};

// Delete an asset
type DeleteAssetArguments = record {
  key: Key;
};

// Reset everything
type ClearArguments = record {};

type BatchOperationKind = variant {
  CreateAsset: CreateAssetArguments;
  SetAssetContent: SetAssetContentArguments;

  SetAssetProperties: SetAssetPropertiesArguments;

  UnsetAssetContent: UnsetAssetContentArguments;
  DeleteAsset: DeleteAssetArguments;

  Clear: ClearArguments;
};

type CommitBatchArguments = record {
  batch_id: BatchId;
  operations: vec BatchOperationKind
};

type CommitProposedBatchArguments = record {
  batch_id: BatchId;
  evidence: blob;
};

type ComputeEvidenceArguments = record {
  batch_id: BatchId;
  max_iterations: opt nat16
};

type DeleteBatchArguments = record {
  batch_id: BatchId;
};

type HeaderField = record { text; text; };

type HttpRequest = record {
  method: text;
  url: text;
  headers: vec HeaderField;
  body: blob;
  certificate_version: opt nat16;
};

type HttpResponse = record {
  status_code: nat16;
  headers: vec HeaderField;
  body: blob;
  streaming_strategy: opt StreamingStrategy;
};

type StreamingCallbackHttpResponse = record {
  body: blob;
  token: opt StreamingCallbackToken;
};

type StreamingCallbackToken = record {
  key: Key;
  content_encoding: text;
  index: nat;
  sha256: opt blob;
};

type StreamingStrategy = variant {
  Callback: record {
    callback: func (StreamingCallbackToken) -> (opt StreamingCallbackHttpResponse) query;
    token: StreamingCallbackToken;
  };
};

type SetAssetPropertiesArguments = record {
  key: Key;
  max_age: opt opt nat64;
  headers: opt opt vec HeaderField;
  allow_raw_access: opt opt bool;
  is_aliased: opt opt bool;
};

type Permission = variant {
  Commit;
  ManagePermissions;
  Prepare;
};

type GrantPermission = record {
  to_principal: principal;
  permission: Permission;
};
type RevokePermission = record {
  of_principal: principal;
  permission: Permission;
};
type ListPermitted = record { permission: Permission };

type ValidationResult = variant { Ok : text; Err : text };

service: {
  api_version: () -> (nat16) query;

  get: (record {
    key: Key;
    accept_encodings: vec text;
  }) -> (record {
    content: blob; // may be the entirety of the content, or just chunk index 0
    content_type: text;
    content_encoding: text;
    sha256: opt blob; // sha256 of entire asset encoding, calculated by dfx and passed in SetAssetContentArguments
    total_length: nat; // all chunks except last have size == content.size()
  }) query;

  // if get() returned chunks > 1, call this to retrieve them.
  // chunks may or may not be split up at the same boundaries as presented to create_chunk().
  get_chunk: (record {
    key: Key;
    content_encoding: text;
    index: nat;
    sha256: opt blob;  // sha256 of entire asset encoding, calculated by dfx and passed in SetAssetContentArguments
  }) -> (record { content: blob }) query;

  list : (record {}) -> (vec record {
    key: Key;
    content_type: text;
    encodings: vec record {
      content_encoding: text;
      sha256: opt blob; // sha256 of entire asset encoding, calculated by dfx and passed in SetAssetContentArguments
      length: nat; // Size of this encoding's blob. Calculated when uploading assets.
      modified: Time;
    };
  }) query;

  certified_tree : (record {}) -> (record {
    certificate: blob;
    tree: blob;
  }) query;

  create_batch : (record {}) -> (record { batch_id: BatchId });

  create_chunk: (record { batch_id: BatchId; content: blob }) -> (record { chunk_id: ChunkId });

  // Perform all operations successfully, or reject
  commit_batch: (CommitBatchArguments) -> ();

  // Save the batch operations for later commit
  propose_commit_batch: (CommitBatchArguments) -> ();

  // Given a batch already proposed, perform all operations successfully, or reject
  commit_proposed_batch: (CommitProposedBatchArguments) -> ();

  // Compute a hash over the CommitBatchArguments.  Call until it returns Some(evidence).
  compute_evidence: (ComputeEvidenceArguments) -> (opt blob);

  // Delete a batch that has been created, or proposed for commit, but not yet committed
  delete_batch: (DeleteBatchArguments) -> ();

  create_asset: (CreateAssetArguments) -> ();
  set_asset_content: (SetAssetContentArguments) -> ();
  unset_asset_content: (UnsetAssetContentArguments) -> ();

  delete_asset: (DeleteAssetArguments) -> ();

  clear: (ClearArguments) -> ();

  // Single call to create an asset with content for a single content encoding that
  // fits within the message ingress limit.
  store: (record {
    key: Key;
    content_type: text;
    content_encoding: text;
    content: blob;
    sha256: opt blob
  }) -> ();

  http_request: (request: HttpRequest) -> (HttpResponse) query;
  http_request_streaming_callback: (token: StreamingCallbackToken) -> (opt StreamingCallbackHttpResponse) query;

  authorize: (principal) -> ();
  deauthorize: (principal) -> ();
  list_authorized: () -> (vec principal) query;
  grant_permission: (GrantPermission) -> ();
  revoke_permission: (RevokePermission) -> ();
  list_permitted: (ListPermitted) -> (vec principal) query;
  take_ownership: () -> ();

  get_asset_properties : (key: Key) -> (record {
    max_age: opt nat64;
    headers: opt vec HeaderField;
    allow_raw_access: opt bool;
    is_aliased: opt bool; } ) query;
  set_asset_properties: (SetAssetPropertiesArguments) -> ();

  validate_grant_permission: (GrantPermission) -> (ValidationResult);
  validate_revoke_permission: (RevokePermission) -> (ValidationResult);
  validate_take_ownership: () -> (ValidationResult);
  validate_commit_proposed_batch: (CommitProposedBatchArguments) -> (ValidationResult);
}

```