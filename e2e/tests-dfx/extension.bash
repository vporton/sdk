#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

}

teardown() {
    dfx_stop

    standard_teardown
}

@test "manually create extension" {
    assert_command dfx extension list
    assert_match 'No extensions installed'

    CACHE_DIR=$(dfx cache show)
    mkdir -p "$CACHE_DIR"/extensions/test_extension
    echo '#!/usr/bin/env bash

echo testoutput' > "$CACHE_DIR"/extensions/test_extension/test_extension
    chmod +x "$CACHE_DIR"/extensions/test_extension/test_extension

    echo '{
  "name": "test_extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "canister_types": {
    "test": {
      
    }
  }

}' > "$CACHE_DIR"/extensions/test_extension/extension.json

    assert_command dfx --help
    assert_match "test_extension.*Test extension for e2e purposes."

    assert_command dfx test_extension --help
    assert_match "Test extension for e2e purposes..*Usage: dfx test_extension"

    assert_command dfx extension list
    assert_match "test_extension"

    assert_command dfx extension run test_extension
    assert_match "testoutput"

    assert_command dfx test_extension
    assert_match "testoutput"

    assert_command dfx extension uninstall test_extension
    # TODO: how to capture spinner message?
    # assert_match 'Successfully uninstalled extension'

    assert_command dfx extension list
    assert_match 'No extensions installed'
}
