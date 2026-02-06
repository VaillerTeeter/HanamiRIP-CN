/*
  这里是“信息抽取与标签归一化”的工具函数集合。
  主要用途：
  - 把 Bangumi 的 infobox（结构复杂、字段不统一）整理成可用数据；
  - 把标签去重、标准化；
  - 让上层逻辑不用关心底层 JSON 的各种形式。
*/

use serde_json::{Map, Value};

// 只使用到少量模型：InfoboxItem 与 SubjectTag。
use super::models::{InfoboxItem, SubjectTag};

// 从 JSON 对象里提取“最可能是文本的字段”。
// Bangumi infobox 的字段不统一，所以我们尝试多个 key。
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

// 从任意 JSON 值中提取一个“可读文本”。
// 可能是字符串、对象、数组，都会尽量转换成文字。
pub(crate) fn extract_infobox_value(value: &Value) -> Option<String> {
  match value {
    // 直接是字符串，就去掉空白并返回。
    Value::String(text) => {
      let trimmed = text.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    }
    // 对象：尝试从常见字段中提取。
    Value::Object(map) => extract_value_from_object(map),
    // 数组：把每个元素提取成文字，再拼起来。
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

// 把一个字符串按常见分隔符拆成多个词条。
// 例如："动作/冒险" -> ["动作","冒险"]。
fn split_infobox_terms(value: &str) -> Vec<String> {
  value
    .split(|ch| matches!(ch, '/' | '／' | '、' | ',' | '，' | '|'))
    .map(|item| item.trim())
    .filter(|item| !item.is_empty())
    .map(|item| item.to_string())
    .collect()
}

// 从 JSON 值中提取“多个词条”。
// 这比 extract_infobox_value 更适合“类型/地区/别名”等多值场景。
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

// 收集条目的标签（tags + meta_tags），并做去重。
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

// 统一标签格式：去空白 + 小写化，便于比较。
fn normalize_tag(value: &str) -> String {
  value.trim().to_lowercase()
}

// 把一组标签映射到“官方规范列表”。
// 支持别名转换（如“机甲” -> “机战”）。
pub(crate) fn map_tags_to_official(
  tags: &[String],
  official: &[&str],
  aliases: &[(&str, &str)],
) -> Vec<String> {
  // official_map：规范标签的快速查找表。
  let mut official_map = std::collections::HashMap::new();
  for name in official {
    official_map.insert(normalize_tag(name), (*name).to_string());
  }
  // alias_map：别名 -> 规范标签。
  let mut alias_map = std::collections::HashMap::new();
  for (from, to) in aliases {
    alias_map.insert(normalize_tag(from), (*to).to_string());
  }
  // output 保存最终结果；seen 用来去重。
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

// 去重工具：保持插入顺序，只去掉重复项。
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

// 从 infobox 中提取“类型 / 地区 / 受众”。
// 返回 (types, regions, audiences) 三个列表。
pub(crate) fn extract_filter_groups(
  infobox: Option<Vec<InfoboxItem>>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
  let mut types = Vec::new();
  let mut regions = Vec::new();
  let mut audiences = Vec::new();
  // infobox 为空时直接返回空列表。
  let items = match infobox {
    Some(items) => items,
    None => return (types, regions, audiences),
  };
  for item in items {
    // item.value 里可能是字符串、对象或数组，所以用统一抽取函数处理。
    let values = extract_infobox_values(&item.value);
    if values.is_empty() {
      continue;
    }
    // 根据 key 判断它属于哪个分组。
    match item.key.as_str() {
      "类型" | "题材" | "动画类型" | "分类" | "类别" => types.extend(values),
      "地区" | "国家/地区" | "国家地区" | "国家" | "发行地区" => regions.extend(values),
      "受众" | "对象" | "读者对象" => audiences.extend(values),
      _ => {}
    }
  }
  (dedupe_terms(types), dedupe_terms(regions), dedupe_terms(audiences))
}

// 提取“原作”字段（如小说、漫画、原创）。
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

// 提取“别名/英文名/罗马音”等字段。
// 这些字段名称并不统一，所以需要做关键词匹配。
pub(crate) fn extract_aliases(infobox: Option<Vec<InfoboxItem>>) -> Vec<String> {
  let mut output = Vec::new();
  let items = match infobox {
    Some(items) => items,
    None => return output,
  };
  for item in items {
    let key = item.key.as_str();
    // 判断当前 key 是否代表“别名含义”。
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
