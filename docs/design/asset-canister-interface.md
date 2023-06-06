# Asset Canister Interface

## Overview

## Asset Fields

### Key

The `key` identifies the asset. It is a string that must be unique within the asset canister. By convention, it should begin with a forward slash. For example, `/index.html`.

### Content Type

The `content_type` is a string that identifies the type of the asset, such as `text/plain` or `image/jpeg`. It is used to set the `Content-Type` header when serving the asset over HTTP.

### Content Encoding

An asset contains one or more encodings. Each encoding is identified by a `content_encoding` string, such as `identity` or `gzip`. It is used to set the `Content-Encoding` header when serving the asset over HTTP.

### Content Chunks

Each encoding contains one or more "chunks" of data. The size of each chunk is limited by the message ingress limit.

Content chunks can have any size that fits within the message ingress limit, but all chunks except the last must have the same size.

### Content Hash

The `sha256` field contains the SHA-256 hash of the entire asset encoding. It is used to set the `ETag` header when serving the asset over HTTP.

### Max Age

The `max_age` field is the maximum number of seconds that the asset can be cached by a browser or CDN. It is used to set the `max-age` value of the `Cache-Control` header when serving the asset over HTTP.

### Headers

The `headers` field is a list of additional headers to set when serving the asset over HTTP.

### Aliasing

The `is_aliased` field actually whether aliases are enabled for an asset.

If aliasing is enabled, this means that if lookup for a given `key` 

### Raw Access



## Storing Assets

### Batch Updates

Message ingress limits constrain the size of a single update to the asset canister. To work around this, the asset canister supports batch updates.

To upload data, first create a batch, then upload chunks to the batch, then commit the batch.

- `create_batch()`: creates a batch
- `create_chunk()`: creates a chunk in a batch
- `commit_batch()`: 
- `delete_batch()`: deletes a batch

#### Batch Expiry

Batches for which `create_chunk()` has not been called within a certain time period may be deleted.

### Updates By Proposal

For canisters controlled by an SNS, the asset canister supports updates by proposal. In this scenario, one principal uploads the proposed changes, which the SNS will commit if the proposal is accepted.

- `propose_commit_batch()`: proposes a batch to be committed
- `compute_evidence()`: computes evidence for a proposed batch
- `commit_proposed_batch()`: commits a proposed batch

#### Evidence

### Single-Call Updates

## Retrieving Assets

### HTTP Requests

#### Streaming

## Access Control

## Other Methods

## Types

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