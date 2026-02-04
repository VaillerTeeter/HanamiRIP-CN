use serde_json::{Map, Value};

use super::models::{InfoboxItem, SubjectTag};

pub(crate) fn extract_value_from_object(map: &Map<String, Value>) -> Option<String> {
  for key in ["v", "value", "name", "text"] {
    if let Some(Value::String(text)) = map.get(key) {
      if !text.trim().is_empty() {
        return Some(text.clone());
      }
    }
  }
  None
}

pub(crate) fn extract_infobox_value(value: &Value) -> Option<String> {
  match value {
    Value::String(text) => {
      let trimmed = text.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    }
    Value::Object(map) => extract_value_from_object(map),
    Value::Array(items) => {
      let mut parts = Vec::new();
      for item in items {
        let extracted = match item {
          Value::Object(map) => extract_value_from_object(map),
          _ => extract_infobox_value(item),
        };
        if let Some(text) = extracted {
          if !text.trim().is_empty() {
            parts.push(text);
          }
        }
      }
      if parts.is_empty() {
        None
      } else {
        Some(parts.join(" / "))
      }
    }
    _ => None,
  }
}

fn split_infobox_terms(value: &str) -> Vec<String> {
  value
    .split(|ch| matches!(ch, '/' | '／' | '、' | ',' | '，' | '|'))
    .map(|item| item.trim())
    .filter(|item| !item.is_empty())
    .map(|item| item.to_string())
    .collect()
}

pub(crate) fn extract_infobox_values(value: &Value) -> Vec<String> {
  match value {
    Value::String(text) => split_infobox_terms(text),
    Value::Object(map) => extract_value_from_object(map)
      .map(|text| split_infobox_terms(&text))
      .unwrap_or_default(),
    Value::Array(items) => {
      let mut output = Vec::new();
      for item in items {
        let extracted = match item {
          Value::Object(map) => extract_value_from_object(map)
            .map(|text| split_infobox_terms(&text))
            .unwrap_or_default(),
          _ => extract_infobox_values(item),
        };
        output.extend(extracted);
      }
      output
    }
    _ => Vec::new(),
  }
}

pub(crate) fn collect_subject_tags(
  tags: Option<Vec<SubjectTag>>,
  meta_tags: Option<Vec<String>>,
) -> Vec<String> {
  let mut output = Vec::new();
  if let Some(items) = tags {
    for item in items {
      let name = item.name.trim().to_string();
      if !name.is_empty() {
        output.push(name);
      }
    }
  }
  if let Some(items) = meta_tags {
    for name in items {
      let name = name.trim().to_string();
      if !name.is_empty() {
        output.push(name);
      }
    }
  }
  dedupe_terms(output)
}

fn normalize_tag(value: &str) -> String {
  value.trim().to_lowercase()
}

pub(crate) fn map_tags_to_official(
  tags: &[String],
  official: &[&str],
  aliases: &[(&str, &str)],
) -> Vec<String> {
  let mut official_map = std::collections::HashMap::new();
  for name in official {
    official_map.insert(normalize_tag(name), (*name).to_string());
  }
  let mut alias_map = std::collections::HashMap::new();
  for (from, to) in aliases {
    alias_map.insert(normalize_tag(from), (*to).to_string());
  }
  let mut output = Vec::new();
  let mut seen = std::collections::HashSet::new();
  for tag in tags {
    let normalized = normalize_tag(tag);
    if let Some(value) = official_map.get(&normalized) {
      if seen.insert(value.clone()) {
        output.push(value.clone());
      }
      continue;
    }
    if let Some(value) = alias_map.get(&normalized) {
      if seen.insert(value.clone()) {
        output.push(value.clone());
      }
    }
  }
  output
}

pub(crate) fn dedupe_terms(values: Vec<String>) -> Vec<String> {
  let mut seen = std::collections::HashSet::new();
  let mut output = Vec::new();
  for value in values {
    if seen.insert(value.clone()) {
      output.push(value);
    }
  }
  output
}

pub(crate) fn extract_filter_groups(
  infobox: Option<Vec<InfoboxItem>>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
  let mut types = Vec::new();
  let mut regions = Vec::new();
  let mut audiences = Vec::new();
  let items = match infobox {
    Some(items) => items,
    None => return (types, regions, audiences),
  };
  for item in items {
    let values = extract_infobox_values(&item.value);
    if values.is_empty() {
      continue;
    }
    match item.key.as_str() {
      "类型" | "题材" | "动画类型" | "分类" | "类别" => types.extend(values),
      "地区" | "国家/地区" | "国家地区" | "国家" | "发行地区" => regions.extend(values),
      "受众" | "对象" | "读者对象" => audiences.extend(values),
      _ => {}
    }
  }
  (dedupe_terms(types), dedupe_terms(regions), dedupe_terms(audiences))
}

pub(crate) fn extract_origin(infobox: Option<Vec<InfoboxItem>>) -> Option<String> {
  let items = infobox?;
  for item in items {
    if item.key == "原作" {
      if let Some(value) = extract_infobox_value(&item.value) {
        return Some(value);
      }
    }
  }
  None
}

pub(crate) fn extract_aliases(infobox: Option<Vec<InfoboxItem>>) -> Vec<String> {
  let mut output = Vec::new();
  let items = match infobox {
    Some(items) => items,
    None => return output,
  };
  for item in items {
    let key = item.key.as_str();
    let is_alias_key = key.contains("别名")
      || key.contains("又名")
      || key.contains("英文")
      || key.contains("罗马")
      || key.eq_ignore_ascii_case("romaji")
      || key.eq_ignore_ascii_case("english");
    if !is_alias_key {
      continue;
    }
    let values = extract_infobox_values(&item.value);
    for v in values {
      output.push(v);
    }
  }
  dedupe_terms(output)
}
