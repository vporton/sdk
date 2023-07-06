use crate::{
    error::extension::ExtensionError,
    extension::{manager::ExtensionManager, manifest::ExtensionManifest},
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) fn transform(
    extension_manager: &ExtensionManager,
    canister_name: &str,
    canister_type: &str,
    canister_declaration: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<serde_json::Map<String, serde_json::Value>, ExtensionError> {
    let mut split = canister_type.split(':');
    let extension_name = split.next().unwrap_or_default(); // TODO
    let canister_type = split.next().unwrap_or_else(|| extension_name);
    if extension_manager.is_extension_installed(extension_name) {
        let manifest = ExtensionManifest::new(extension_name, &extension_manager.dir)?;
        if let Some(extension_custom_canister_declaration) =
            manifest.canister_types.unwrap().get(canister_type)
        {
            let mut values = extract_values_from_canister_declaration(canister_declaration);
            values.insert("canister_name".into(), canister_name.into());
            return Ok(extension_custom_canister_declaration.apply_template(values)?);
        }
    }
    Err(ExtensionError::CommandAlreadyExists(extension_name.into())) // TODO
}

fn extract_values_from_canister_declaration(
    canister_declaration: &serde_json::Map<String, serde_json::Value>,
) -> HashMap<String, serde_json::Value> {
    canister_declaration
        .into_iter()
        .filter_map(|(k, v)| {
            if v.is_array() || v.is_object() {
                None
            } else {
                Some((k.clone(), v.clone()))
            }
        })
        .collect()
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
enum Op {
    Replace { replace: Replace },
    Remove { remove: bool },
    Template(String),
    BoolValue(bool),
    NumberValue(serde_json::Number),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
struct Replace {
    input: String,
    search: String,
    output: String,
}

impl Replace {
    fn apply(
        &self,
        values: &HashMap<FieldName, serde_json::Value>,
    ) -> Result<String, regex::Error> {
        let re = Regex::new(&self.search)?;
        let input = handlebars::Handlebars::new()
            .render_template(&self.input, &values)
            .unwrap();
        dbg!(&input);
        Ok(re.replace_all(&input, &self.output).to_string())
    }
}

type FieldName = String;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomCanisterTypeDeclaration(HashMap<FieldName, Op>);

impl CustomCanisterTypeDeclaration {
    fn apply_template(
        &self,
        values: HashMap<FieldName, serde_json::Value>,
    ) -> Result<serde_json::Map<FieldName, serde_json::Value>, ExtensionError> {
        let mut remove_fields = vec![];
        let mut final_fields = serde_json::Map::new();
        for (field_name, op) in self
            .0
            .clone()
            .into_iter()
            .collect::<Vec<_>>()
            .clone()
            .into_iter()
        {
            match op {
                Op::NumberValue(x) => {
                    final_fields.insert(field_name, x.into());
                }
                Op::BoolValue(x) => {
                    final_fields.insert(field_name, x.into());
                }

                Op::Template(template) => {
                    let x = handlebars::Handlebars::new()
                        .render_template(&template, &values)
                        .unwrap();
                    final_fields.insert(field_name, x.into());
                }
                Op::Replace { replace } => {
                    let x = replace.apply(&values).unwrap();
                    final_fields.insert(field_name, x.into());
                }
                Op::Remove { remove } if remove => {
                    remove_fields.push(field_name);
                }
                _ => {}
            }
        }
        // Removing fields should be done last because of the order of the fields in the map.
        // It's easier to do in second for loop than to sort Ops beforehand, bacause Op would need to implement PartialOrd,
        // which is not possible, because serde_json::Number does not implement it.
        for field_name in remove_fields {
            final_fields.remove(&field_name);
        }
        Ok(final_fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct DummyExtensionManifest {
        name: String,
        canister_types: HashMap<String, CustomCanisterTypeDeclaration>,
    }

    const EXTENSION_CUSTOM_CANISTER_TYPE_DECLARATION: &str = r#"
        {
            "name": "azyl",
            "canister_types": {
                "azyl": {
                    "type": "custom",
                    "main": "src/main.ts",
                    "ts": { "replace": { "input": "{{main}}", "search": "(.*).ts", "output": "$1.ts" }},
                    "wasm": ".azyl/{{canister_name}}/{{canister_name}}.wasm.gz",
                    "build": "npx azyl {{canister_name}}",
                    "candid": { "replace": { "input": "{{main}}", "search": "(.*).ts", "output": "$1.did" }},
                    "main": { "remove": true },
                    "gzip": true
                }
            }
        }
        "#;

    const DFX_JSON_WITH_CUSTOM_CANISTER_TYPE: &str = r#"
        {
            "canisters": {
                "azyl": {
                    "type": "azyl",
                    "main": "src/main.ts"
                }
            }
        }
        "#;

    #[test]
    fn deserializing_json() {
        let data: DummyExtensionManifest =
            serde_json::from_str(EXTENSION_CUSTOM_CANISTER_TYPE_DECLARATION).unwrap();
        dbg!(&data);
        assert_eq!(
            data.canister_types
                .get("azyl")
                .unwrap()
                .0
                .get("type")
                .unwrap(),
            &Op::Template("custom".into())
        );
    }

    #[test]
    fn applying_replace() {
        let replace = Replace {
            input: "Hello, world!".to_string(),
            search: "world".to_string(),
            output: "regex".to_string(),
        };
        assert_eq!(replace.apply(&HashMap::new()).unwrap(), "Hello, regex!");
    }

    #[test]
    fn applying_replace_with_handlebars() {
        let replace = Replace {
            input: "{{hello}}, world!".to_string(),
            search: "world".to_string(),
            output: "regex".to_string(),
        };
        let values = [("hello".into(), "Salut".into())].iter().cloned().collect();
        assert_eq!(replace.apply(&values).unwrap(), "Salut, regex!");
    }

    #[test]
    fn applying_transformations() {
        let canister = DFX_JSON_WITH_CUSTOM_CANISTER_TYPE
            .parse::<serde_json::Value>()
            .unwrap()
            .get("canisters")
            .unwrap()
            .get("azyl")
            .unwrap()
            .clone()
            .as_object()
            .unwrap()
            .clone();
        let values = {
            let mut hm = extract_values_from_canister_declaration(&canister);
            hm.insert("canister_name".into(), "azyl_frontend".into());
            hm
        };
        let data: DummyExtensionManifest =
            serde_json::from_str(EXTENSION_CUSTOM_CANISTER_TYPE_DECLARATION).unwrap();
        let custom_canister_type_declaration = data.canister_types.get("azyl").unwrap();
        let transformed = custom_canister_type_declaration
            .apply_template(values)
            .unwrap();
        assert_eq!(
            serde_json::to_string_pretty(&transformed).unwrap(),
            r#"{
  "build": "npx azyl azyl_frontend",
  "candid": "src/main.did",
  "gzip": true,
  "ts": "src/main.ts",
  "type": "custom",
  "wasm": ".azyl/azyl_frontend/azyl_frontend.wasm.gz"
}"#
        );
    }
}
