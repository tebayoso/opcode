//! Configuration file parsers for various formats
//!
//! This module provides parsers for JSON, JSONC, TOML, YAML, and Markdown
//! configuration files, normalizing them to a common JSON representation.

use super::traits::{ConfigFileContent, ConfigFileType};
use serde_json::Value;
use std::path::Path;

/// Error type for parsing operations
#[derive(Debug, Clone)]
pub enum ParseError {
    /// JSON parse error
    Json(String),
    /// TOML parse error
    Toml(String),
    /// YAML parse error
    Yaml(String),
    /// Markdown parse error
    Markdown(String),
    /// Unknown file type
    UnknownFormat(String),
    /// I/O error
    Io(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Json(msg) => write!(f, "JSON parse error: {}", msg),
            ParseError::Toml(msg) => write!(f, "TOML parse error: {}", msg),
            ParseError::Yaml(msg) => write!(f, "YAML parse error: {}", msg),
            ParseError::Markdown(msg) => write!(f, "Markdown parse error: {}", msg),
            ParseError::UnknownFormat(ext) => write!(f, "Unknown format: {}", ext),
            ParseError::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Configuration file parser
pub struct ConfigParser;

impl ConfigParser {
    /// Parse content based on file type
    pub fn parse(content: &str, file_type: ConfigFileType) -> Result<Value, ParseError> {
        match file_type {
            ConfigFileType::Json => Self::parse_json(content),
            ConfigFileType::Jsonc => Self::parse_jsonc(content),
            ConfigFileType::Toml => Self::parse_toml(content),
            ConfigFileType::Yaml => Self::parse_yaml(content),
            ConfigFileType::Markdown => Self::parse_markdown(content),
        }
    }

    /// Parse content from file path (auto-detect format)
    pub fn parse_from_path(content: &str, path: &Path) -> Result<Value, ParseError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let file_type = ConfigFileType::from_extension(ext)
            .ok_or_else(|| ParseError::UnknownFormat(ext.to_string()))?;

        Self::parse(content, file_type)
    }

    /// Parse JSON content
    pub fn parse_json(content: &str) -> Result<Value, ParseError> {
        serde_json::from_str(content).map_err(|e| ParseError::Json(e.to_string()))
    }

    /// Parse JSONC (JSON with comments) content
    pub fn parse_jsonc(content: &str) -> Result<Value, ParseError> {
        // Remove single-line comments
        let mut result = String::new();
        let mut in_string = false;
        let mut escape_next = false;
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            if escape_next {
                result.push(c);
                escape_next = false;
                continue;
            }

            if c == '\\' && in_string {
                result.push(c);
                escape_next = true;
                continue;
            }

            if c == '"' {
                in_string = !in_string;
                result.push(c);
                continue;
            }

            if !in_string && c == '/' {
                if let Some(&next) = chars.peek() {
                    if next == '/' {
                        // Single-line comment - skip to end of line
                        while let Some(nc) = chars.next() {
                            if nc == '\n' {
                                result.push('\n');
                                break;
                            }
                        }
                        continue;
                    } else if next == '*' {
                        // Multi-line comment - skip to */
                        chars.next(); // consume *
                        while let Some(nc) = chars.next() {
                            if nc == '*' {
                                if let Some(&'/') = chars.peek() {
                                    chars.next();
                                    break;
                                }
                            }
                        }
                        continue;
                    }
                }
            }

            result.push(c);
        }

        // Remove trailing commas before ] or }
        let result = Self::remove_trailing_commas(&result);

        serde_json::from_str(&result).map_err(|e| ParseError::Json(e.to_string()))
    }

    /// Remove trailing commas in JSON (common in JSONC)
    fn remove_trailing_commas(content: &str) -> String {
        let mut result = String::new();
        let mut chars = content.chars().peekable();
        let mut in_string = false;
        let mut escape_next = false;

        while let Some(c) = chars.next() {
            if escape_next {
                result.push(c);
                escape_next = false;
                continue;
            }

            if c == '\\' && in_string {
                result.push(c);
                escape_next = true;
                continue;
            }

            if c == '"' {
                in_string = !in_string;
                result.push(c);
                continue;
            }

            if !in_string && c == ',' {
                // Look ahead for ] or } (with optional whitespace)
                let remaining: String = chars.clone().collect();
                let trimmed = remaining.trim_start();
                if trimmed.starts_with(']') || trimmed.starts_with('}') {
                    // Skip this comma
                    continue;
                }
            }

            result.push(c);
        }

        result
    }

    /// Parse TOML content
    pub fn parse_toml(content: &str) -> Result<Value, ParseError> {
        let toml_value: toml::Value =
            toml::from_str(content).map_err(|e| ParseError::Toml(e.to_string()))?;

        // Convert TOML value to JSON value
        Self::toml_to_json(toml_value)
    }

    /// Convert TOML value to JSON value
    fn toml_to_json(toml: toml::Value) -> Result<Value, ParseError> {
        match toml {
            toml::Value::String(s) => Ok(Value::String(s)),
            toml::Value::Integer(i) => Ok(Value::Number(i.into())),
            toml::Value::Float(f) => {
                let n = serde_json::Number::from_f64(f)
                    .ok_or_else(|| ParseError::Toml("Invalid float value".to_string()))?;
                Ok(Value::Number(n))
            }
            toml::Value::Boolean(b) => Ok(Value::Bool(b)),
            toml::Value::Datetime(dt) => Ok(Value::String(dt.to_string())),
            toml::Value::Array(arr) => {
                let json_arr: Result<Vec<Value>, ParseError> =
                    arr.into_iter().map(Self::toml_to_json).collect();
                Ok(Value::Array(json_arr?))
            }
            toml::Value::Table(table) => {
                let mut map = serde_json::Map::new();
                for (k, v) in table {
                    map.insert(k, Self::toml_to_json(v)?);
                }
                Ok(Value::Object(map))
            }
        }
    }

    /// Parse YAML content
    pub fn parse_yaml(content: &str) -> Result<Value, ParseError> {
        let yaml_value: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| ParseError::Yaml(e.to_string()))?;

        // Convert YAML value to JSON value
        Self::yaml_to_json(yaml_value)
    }

    /// Convert YAML value to JSON value
    fn yaml_to_json(yaml: serde_yaml::Value) -> Result<Value, ParseError> {
        match yaml {
            serde_yaml::Value::Null => Ok(Value::Null),
            serde_yaml::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Number(i.into()))
                } else if let Some(f) = n.as_f64() {
                    let num = serde_json::Number::from_f64(f)
                        .ok_or_else(|| ParseError::Yaml("Invalid number".to_string()))?;
                    Ok(Value::Number(num))
                } else {
                    Err(ParseError::Yaml("Invalid number".to_string()))
                }
            }
            serde_yaml::Value::String(s) => Ok(Value::String(s)),
            serde_yaml::Value::Sequence(seq) => {
                let json_arr: Result<Vec<Value>, ParseError> =
                    seq.into_iter().map(Self::yaml_to_json).collect();
                Ok(Value::Array(json_arr?))
            }
            serde_yaml::Value::Mapping(map) => {
                let mut json_map = serde_json::Map::new();
                for (k, v) in map {
                    let key = match k {
                        serde_yaml::Value::String(s) => s,
                        other => other.as_str().unwrap_or("").to_string(),
                    };
                    json_map.insert(key, Self::yaml_to_json(v)?);
                }
                Ok(Value::Object(json_map))
            }
            serde_yaml::Value::Tagged(tagged) => Self::yaml_to_json(tagged.value),
        }
    }

    /// Parse Markdown content (extract YAML frontmatter and content)
    pub fn parse_markdown(content: &str) -> Result<Value, ParseError> {
        let mut result = serde_json::Map::new();

        // Check for YAML frontmatter
        if content.starts_with("---") {
            let parts: Vec<&str> = content.splitn(3, "---").collect();
            if parts.len() >= 3 {
                // Parse frontmatter as YAML
                let frontmatter = parts[1].trim();
                if !frontmatter.is_empty() {
                    let fm_value = Self::parse_yaml(frontmatter)?;
                    result.insert("frontmatter".to_string(), fm_value);
                }
                // Store the body content
                result.insert("body".to_string(), Value::String(parts[2].trim().to_string()));
            } else {
                result.insert("body".to_string(), Value::String(content.to_string()));
            }
        } else {
            result.insert("body".to_string(), Value::String(content.to_string()));
        }

        Ok(Value::Object(result))
    }

    /// Serialize value back to the original format
    pub fn serialize(value: &Value, file_type: ConfigFileType) -> Result<String, ParseError> {
        match file_type {
            ConfigFileType::Json | ConfigFileType::Jsonc => {
                serde_json::to_string_pretty(value).map_err(|e| ParseError::Json(e.to_string()))
            }
            ConfigFileType::Toml => {
                let toml_value = Self::json_to_toml(value)?;
                toml::to_string_pretty(&toml_value).map_err(|e| ParseError::Toml(e.to_string()))
            }
            ConfigFileType::Yaml => {
                serde_yaml::to_string(value).map_err(|e| ParseError::Yaml(e.to_string()))
            }
            ConfigFileType::Markdown => {
                // Reconstruct markdown with frontmatter
                if let Some(obj) = value.as_object() {
                    let mut result = String::new();
                    if let Some(fm) = obj.get("frontmatter") {
                        result.push_str("---\n");
                        let yaml =
                            serde_yaml::to_string(fm).map_err(|e| ParseError::Yaml(e.to_string()))?;
                        result.push_str(&yaml);
                        result.push_str("---\n\n");
                    }
                    if let Some(body) = obj.get("body").and_then(|b| b.as_str()) {
                        result.push_str(body);
                    }
                    Ok(result)
                } else {
                    Err(ParseError::Markdown(
                        "Invalid markdown structure".to_string(),
                    ))
                }
            }
        }
    }

    /// Convert JSON value to TOML value
    fn json_to_toml(json: &Value) -> Result<toml::Value, ParseError> {
        match json {
            Value::Null => Ok(toml::Value::String("null".to_string())),
            Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(toml::Value::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(toml::Value::Float(f))
                } else {
                    Err(ParseError::Toml("Invalid number".to_string()))
                }
            }
            Value::String(s) => Ok(toml::Value::String(s.clone())),
            Value::Array(arr) => {
                let toml_arr: Result<Vec<toml::Value>, ParseError> =
                    arr.iter().map(Self::json_to_toml).collect();
                Ok(toml::Value::Array(toml_arr?))
            }
            Value::Object(map) => {
                let mut table = toml::map::Map::new();
                for (k, v) in map {
                    table.insert(k.clone(), Self::json_to_toml(v)?);
                }
                Ok(toml::Value::Table(table))
            }
        }
    }

    /// Create a ConfigFileContent from raw content
    pub fn create_file_content(
        path: &str,
        content: &str,
        file_type: ConfigFileType,
    ) -> ConfigFileContent {
        let parsed = Self::parse(content, file_type).ok();
        ConfigFileContent {
            path: path.to_string(),
            content: content.to_string(),
            parsed,
            file_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json = r#"{"key": "value", "number": 42}"#;
        let result = ConfigParser::parse_json(json).unwrap();
        assert_eq!(result["key"], "value");
        assert_eq!(result["number"], 42);
    }

    #[test]
    fn test_parse_jsonc_with_comments() {
        let jsonc = r#"{
            // This is a comment
            "key": "value",
            /* Multi-line
               comment */
            "number": 42,
        }"#;
        let result = ConfigParser::parse_jsonc(jsonc).unwrap();
        assert_eq!(result["key"], "value");
        assert_eq!(result["number"], 42);
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[section]
key = "value"
number = 42
"#;
        let result = ConfigParser::parse_toml(toml_str).unwrap();
        assert_eq!(result["section"]["key"], "value");
        assert_eq!(result["section"]["number"], 42);
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
section:
  key: value
  number: 42
"#;
        let result = ConfigParser::parse_yaml(yaml).unwrap();
        assert_eq!(result["section"]["key"], "value");
        assert_eq!(result["section"]["number"], 42);
    }

    #[test]
    fn test_parse_markdown_with_frontmatter() {
        let markdown = r#"---
title: Test
tags:
  - one
  - two
---

# Hello World

This is the body content.
"#;
        let result = ConfigParser::parse_markdown(markdown).unwrap();
        assert_eq!(result["frontmatter"]["title"], "Test");
        assert!(result["body"].as_str().unwrap().contains("Hello World"));
    }
}
