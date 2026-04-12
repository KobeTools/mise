use eyre::{Result, bail, eyre};
use std::path::PathBuf;
use toml_edit::DocumentMut;

use crate::config::settings::{
    MisercSettings, SETTINGS_META, SettingsFile, SettingsMeta, SettingsType, parse_url_replacements,
};
use crate::toml::dedup_toml_array;
use crate::{config, dirs, duration, file};

/// Add/update a setting
///
/// This modifies the contents of ~/.config/mise/config.toml by default.
/// With `--local`, modifies the local config file instead.
/// See https://mise.jdx.dev/configuration.html#target-file-for-write-operations
#[derive(Debug, clap::Args)]
#[clap(visible_aliases = ["create"], after_long_help = AFTER_LONG_HELP, verbatim_doc_comment)]
pub struct SettingsSet {
    /// The setting to set
    #[clap()]
    pub setting: String,
    /// The value to set (optional if provided as KEY=VALUE)
    pub value: Option<String>,
    /// Use the local config file instead of the global one
    #[clap(long, short)]
    pub local: bool,
}

impl SettingsSet {
    pub fn run(self) -> Result<()> {
        match self.value {
            Some(value) => set(&self.setting, &value, false, self.local),
            None => {
                let (key, value) = self.setting.split_once('=').ok_or_else(|| {
                    eyre!(
                        "Usage: mise settings set <KEY>=<VALUE> or mise settings set <KEY> <VALUE>"
                    )
                })?;
                set(key, value, false, self.local)
            }
        }
    }
}

pub fn set(mut key: &str, value: &str, add: bool, local: bool) -> Result<()> {
    let meta = match SETTINGS_META.get(key) {
        Some(meta) => meta,
        None => {
            bail!("Unknown setting: {}", key);
        }
    };

    let value = match meta.type_ {
        SettingsType::Bool => parse_bool(value)?,
        SettingsType::Integer => parse_i64(value)?,
        SettingsType::Duration => parse_duration(value)?,
        SettingsType::Url | SettingsType::Path | SettingsType::String => value.into(),
        SettingsType::ListString => parse_list_by_comma(value)?,
        SettingsType::ListPath => parse_list_by_colon(value)?,
        SettingsType::SetString => parse_set_by_comma(value)?,
        SettingsType::IndexMap => parse_indexmap_by_json(value)?,
        SettingsType::BoolOrString => parse_bool(value).unwrap_or_else(|_| value.into()),
    };

    let path = settings_path(meta, local)?;
    file::create_dir_all(path.parent().unwrap())?;
    let raw = file::read_to_string(&path).unwrap_or_default();
    let mut config: DocumentMut = raw.parse()?;
    let mut settings = if meta.rc {
        config.as_table_mut()
    } else {
        if !config.contains_key("settings") {
            config["settings"] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        config["settings"].as_table_mut().unwrap()
    };
    if let Some((parent_key, child_key)) = key.split_once('.') {
        key = child_key;
        settings = settings
            .entry(parent_key)
            .or_insert({
                let mut t = toml_edit::Table::new();
                t.set_implicit(true);
                toml_edit::Item::Table(t)
            })
            .as_table_mut()
            .unwrap();
    }

    let value = match settings.get(key).map(|c| c.as_array()) {
        Some(Some(array)) if add => {
            let mut new_array = array.clone();
            new_array.extend(value.as_array().unwrap().iter().cloned());
            match meta.type_ {
                SettingsType::SetString => dedup_toml_array(&new_array).into(),
                _ => new_array.into(),
            }
        }
        _ => value,
    };
    settings.insert(key, value.into());

    validate_settings_document(meta, &config.to_string())?;
    file::write(path, config.to_string())?;
    Ok(())
}

pub fn settings_path(meta: &SettingsMeta, local: bool) -> Result<PathBuf> {
    if meta.rc {
        if local {
            Ok(std::env::current_dir()?.join(".miserc.toml"))
        } else {
            Ok(dirs::CONFIG.join("miserc.toml"))
        }
    } else if local {
        Ok(config::local_toml_config_path())
    } else {
        Ok(config::global_config_path())
    }
}

pub fn validate_settings_document(meta: &SettingsMeta, raw: &str) -> Result<()> {
    if meta.rc {
        let _: MisercSettings = toml::from_str(raw)?;
    } else {
        let _: SettingsFile = toml::from_str(raw)?;
    }
    Ok(())
}

fn parse_list_by_comma(value: &str) -> Result<toml_edit::Value> {
    if value.is_empty() || value == "[]" {
        return Ok(toml_edit::Array::new().into());
    }
    Ok(value.split(',').map(|s| s.trim().to_string()).collect())
}

fn parse_list_by_colon(value: &str) -> Result<toml_edit::Value> {
    if value.is_empty() || value == "[]" {
        return Ok(toml_edit::Array::new().into());
    }
    Ok(value.split(':').map(|s| s.trim().to_string()).collect())
}

fn parse_set_by_comma(value: &str) -> Result<toml_edit::Value> {
    if value.is_empty() || value == "[]" {
        return Ok(toml_edit::Array::new().into());
    }
    let array: toml_edit::Array = value.split(',').map(|s| s.trim().to_string()).collect();
    Ok(dedup_toml_array(&array).into())
}

fn parse_bool(value: &str) -> Result<toml_edit::Value> {
    match value.to_lowercase().as_str() {
        "1" | "true" | "yes" | "y" => Ok(true.into()),
        "0" | "false" | "no" | "n" => Ok(false.into()),
        _ => Err(eyre!("{} must be true or false", value)),
    }
}

fn parse_i64(value: &str) -> Result<toml_edit::Value> {
    match value.parse::<i64>() {
        Ok(value) => Ok(value.into()),
        Err(_) => Err(eyre!("{} must be a number", value)),
    }
}

fn parse_duration(value: &str) -> Result<toml_edit::Value> {
    duration::parse_duration(value)?;
    Ok(value.into())
}

fn parse_indexmap_by_json(value: &str) -> Result<toml_edit::Value> {
    let index_map = parse_url_replacements(value)
        .map_err(|e| eyre!("Failed to parse JSON for IndexMap: {}", e))?;
    Ok(toml_edit::Value::InlineTable({
        let mut table = toml_edit::InlineTable::new();
        for (k, v) in index_map {
            table.insert(&k, toml_edit::Value::String(toml_edit::Formatted::new(v)));
        }
        table
    }))
}

static AFTER_LONG_HELP: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</underline></bold>

    $ <bold>mise settings idiomatic_version_file=true</bold>
"#
);
