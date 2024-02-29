use crate::util::{asset_str, ResultExt};
use anyhow::{anyhow, Context, Result};
use gpui::{Action, AppContext, KeyBinding};
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
pub use std::collections::*;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "settings/*"]
#[include = "keymaps/*"]
#[exclude = "*.DS_Store"]
pub struct SettingsAssets;

#[derive(Debug, Deserialize, Default, Clone, JsonSchema)]
#[serde(transparent)]
pub struct KeymapFile(Vec<KeymapBlock>);

#[derive(Debug, Deserialize, Default, Clone, JsonSchema)]
pub struct KeymapBlock {
    #[serde(default)]
    context: Option<String>,
    bindings: BTreeMap<String, KeymapAction>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(transparent)]
pub struct KeymapAction(Value);

impl JsonSchema for KeymapAction {
    fn schema_name() -> String {
        "KeymapAction".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        Schema::Bool(true)
    }
}

#[derive(Deserialize)]
struct ActionWithData(Box<str>, Value);

impl KeymapFile {
    pub fn load_asset(asset_path: &str, cx: &mut AppContext) -> Result<()> {
        let content = asset_str::<SettingsAssets>(asset_path);

        Self::parse(content.as_ref())?.add_to_cx(cx)
    }

    pub fn parse(content: &str) -> Result<Self> {
        parse_json_with_comments::<Self>(content)
    }

    pub fn add_to_cx(self, cx: &mut AppContext) -> Result<()> {
        for KeymapBlock { context, bindings } in self.0 {
            let bindings = bindings
                .into_iter()
                .filter_map(|(keystroke, action)| {
                    let action = action.0;

                    // This is a workaround for a limitation in serde: serde-rs/json#497
                    // We want to deserialize the action data as a `RawValue` so that we can
                    // deserialize the action itself dynamically directly from the JSON
                    // string. But `RawValue` currently does not work inside of an untagged enum.
                    match action {
                        Value::Array(items) => {
                            let Ok([name, data]): Result<[serde_json::Value; 2], _> =
                                items.try_into()
                            else {
                                return Some(Err(anyhow!("Expected array of length 2")));
                            };
                            let serde_json::Value::String(name) = name else {
                                return Some(Err(anyhow!(
                                    "Expected first item in array to be a string."
                                )));
                            };
                            cx.build_action(&name, Some(data))
                        }
                        Value::String(name) => cx.build_action(&name, None),
                        Value::Null => Ok(no_action()),
                        _ => {
                            return Some(Err(anyhow!("Expected two-element array, got {action:?}")))
                        }
                    }
                    .with_context(|| {
                        format!(
                            "invalid binding value for keystroke {keystroke}, context {context:?}"
                        )
                    })
                    .log_err()
                    .map(|action| KeyBinding::load(&keystroke, action, context.as_deref()))
                })
                .collect::<Result<Vec<_>>>()?;

            cx.bind_keys(bindings);
        }
        Ok(())
    }
}

pub fn parse_json_with_comments<T: DeserializeOwned>(content: &str) -> Result<T> {
    Ok(serde_json_lenient::from_str(content)?)
}

fn no_action() -> Box<dyn gpui::Action> {
    gpui::NoAction.boxed_clone()
}
