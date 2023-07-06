#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

}

teardown() {
    dfx_stop

    standard_teardown
}

@test "custom canister types" {
    use_test_specific_cache_root
    dfx cache install

    CACHE_DIR=$(dfx cache show)
    mkdir -p "$CACHE_DIR"/extensions/azle
    echo '#!/usr/bin/env bash

echo testoutput' > "$CACHE_DIR"/extensions/azle/azle
    chmod +x "$CACHE_DIR"/extensions/azle/azle

    echo '{
  "name": "azle",
  "version": "0.1.0",
  "homepage": "https://github.com/demergent-labs/azle",
  "authors": "Demergent Labs",
  "summary": "TypeScript CDK for the Internet Computer",
  "categories": [],
  "keywords": [],
  "commands": {},
  "canister_types": {
    "azle": {
      "type": "custom",
      "build": "npx azle {{canister_name}}",
      "root": "src",
      "ts": { "replace": { "input": "{{main}}", "search": "(.*).ts", "output": "$1.ts" }},
      "candid": { "replace": { "input": "{{main}}", "search": "(.*).ts", "output": "$1.did" }},
      "wasm": ".azle/{{canister_name}}/{{canister_name}}.wasm.gz",
      "main": { "remove": true }
    }
  }
}' > "$CACHE_DIR"/extensions/azle/manifest.json

    assert_command dfx extension list
    assert_match "azle"

    npx --yes azle new hello
    cd hello

    echo '{
  "canisters": {
      "hello_world": {
          "type": "azle",
          "main": "src/index.ts"
      }
  },
  "defaults": {
    "build": {
      "args": "",
      "packtool": ""
    }
  },
  "output_env_file": ".env",
  "version": 1
}' > dfx.json

    dfx_start
    assert_command dfx deploy -v
    assert_match 'No extensions installed'
}
