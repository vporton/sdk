use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, fmt::Display, path::Path};

use crate::error::extension::ExtensionError;

use super::custom_canister_type::CustomCanisterTypeDeclaration;

pub static MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub homepage: String,
    pub authors: Option<String>,
    pub summary: String,
    pub categories: Vec<String>,
    pub keywords: Option<Vec<String>>,
    pub description: Option<String>,
    pub commands: JsonValue,
    pub dependencies: Option<HashMap<String, String>>,
    pub canister_types: Option<HashMap<String, CustomCanisterTypeDeclaration>>,
}

impl Display for ExtensionManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Ok(json) = serde_json::to_string_pretty(self) else {
            return Err(std::fmt::Error)
        };
        write!(f, "{}", json)
    }
}
impl ExtensionManifest {
    pub fn new(name: &str, extensions_root_dir: &Path) -> Result<Self, ExtensionError> {
        let manifest_path = extensions_root_dir.join(name).join(MANIFEST_FILE_NAME);
        if !manifest_path.exists() {
            return Err(ExtensionError::ExtensionManifestMissing(
                name.to_owned(),
                manifest_path,
            ));
        }
        let mut m: ExtensionManifest = crate::json::load_json_file(&manifest_path)
            .map_err(ExtensionError::ExtensionManifestIsNotValid)?;
        m.name = name.to_string();
        Ok(m)
    }
}

#[test]
fn parse_test_file() {
    let f = r#"
{
  "name": "sns",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Toolkit for simulating decentralizing a dapp via SNS.",
  "categories": [
    "sns",
    "nns"
  ],
  "keywords": [
    "sns",
    "nns",
    "deployment"
  ],
  "commands": {
  },
  "canister_types": {
    "azyl": {
        "type": "custom",
        "main": "fff",
        "wasm": ".azle/{{canister_name}}/{{canister_name}}.wasm.gz",
        "candid": { "replace": { "input": "{{main}}", "search": "(.*).ts", "output": "$1.did" }},
        "build": "npx azle {{canister_name}}",

        "root": { "replace": { "input": "{{main}}", "search": "(.*)/[^/]*", "output": "$1"}},
        "ts": "{{main}}",

        "main": { "remove": true }
    }
  }
}
"#;

    let m: Result<ExtensionManifest, serde_json::Error> = serde_json::from_str(f);
    dbg!(&m);
    assert!(m.is_ok());

    // let subcmds = m.unwrap().into_clap_commands().unwrap();
    // dbg!(&subcmds);
    // for s in &subcmds {
    //     if s.get_name() == "download" {
    //         let matches = s
    //             .clone()
    //             .get_matches_from(vec!["download", "--ic-commit", "value"]);
    //         assert_eq!(
    //             Some(&"value".to_string()),
    //             matches.get_one::<String>("ic_commit")
    //         );
    //     }
    // }

    // let cli = clap::Command::new("sns").subcommands(subcmds);
    // cli.debug_assert();
}
