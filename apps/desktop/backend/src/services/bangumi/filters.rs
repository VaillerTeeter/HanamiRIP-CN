/* 文件：filters.rs | 用途：解析 Bangumi infobox 与标签为筛选项 | 关键对象：extract_* 系列函数、dedupe_terms */
// 本行目的：引入 infobox 条目模型。
use super::models::InfoboxItem;
// 本行目的：引入条目标签模型。
use super::models::SubjectTag;
// 本行目的：引入 JSON 对象映射类型。
use serde_json::Map;
// 本行目的：引入 JSON 值类型。
use serde_json::Value;

/// 函数：extract_value_from_object | 输入：JSON 对象引用 | 输出：可选文本 | 可能失败：无，使用 Option 表示缺失
// 本行目的：从 infobox 对象中提取优先级最高的文本字段。
pub(crate) fn extract_value_from_object(map: &Map<String, Value>) -> Option<String> {
    // 本行目的：按优先级遍历可能的键名。
    for key in ["v", "value", "name", "text"] {
        // 本行目的：读取键对应的字符串值。
        if let Some(Value::String(text)) = map.get(key) {
            // 本行目的：过滤掉仅包含空白的值。
            if !text.trim().is_empty() {
                // 本行目的：返回提取到的文本副本。
                return Some(text.clone());
            // 本行目的：结束空白过滤分支。
            }
        // 本行目的：结束字段匹配分支。
        }
    // 本行目的：结束键名遍历。
    }

    // 本行目的：未找到有效文本时返回 None。
    None
// 本行目的：结束对象值提取函数。
}

/// 函数：extract_infobox_value | 输入：JSON 值引用 | 输出：可选文本 | 可能失败：无，使用 Option 表示缺失
// 本行目的：从 infobox 的多种值结构中提取统一文本。
pub(crate) fn extract_infobox_value(value: &Value) -> Option<String> {
    // 本行目的：根据 JSON 类型分支处理。
    match value {
        // 本行目的：处理直接字符串值。
        Value::String(text) => {
            // 变量：trimmed | 含义：去除首尾空白后的文本 | 类型：&str | 作用域：extract_infobox_value
            // 本行目的：规范化字符串以判断有效性。
            let trimmed = text.trim();

            // 本行目的：如果为空字符串则返回 None。
            if trimmed.is_empty() {
                // 本行目的：显式返回 None 表示无有效文本。
                None
            // 本行目的：处理非空字符串分支。
            } else {
                // 本行目的：返回规范化后的字符串副本。
                Some(trimmed.to_string())
            // 本行目的：结束空字符串判断。
            }
        }
        // 本行目的：处理对象值并复用对象提取逻辑。
        Value::Object(map) => extract_value_from_object(map),
        // 本行目的：处理数组值并逐项提取文本。
        Value::Array(items) => {
            // 变量：parts | 含义：累计的文本片段列表 | 类型：Vec<String> | 作用域：extract_infobox_value
            // 本行目的：初始化拼接片段的容器。
            let mut parts = Vec::new();

            // 本行目的：遍历数组中的每一项。
            for item in items {
                // 变量：extracted | 含义：当前项提取到的文本 | 类型：Option<String> | 作用域：extract_infobox_value
                // 本行目的：根据项类型选择对应的提取方式。
                let extracted = match item {
                    // 本行目的：对象项使用对象提取逻辑。
                    Value::Object(map) => extract_value_from_object(map),
                    // 本行目的：其他类型递归调用自身。
                    _ => extract_infobox_value(item),
                // 本行目的：结束项类型分支。
                };

                // 本行目的：仅在提取到有效文本时加入列表。
                if let Some(text) = extracted {
                    // 本行目的：过滤掉空白文本。
                    if !text.trim().is_empty() {
                        // 本行目的：追加有效文本片段。
                        parts.push(text);
                    // 本行目的：结束空白过滤判断。
                    }
                // 本行目的：结束提取结果判断。
                }
            // 本行目的：结束数组遍历。
            }

            // 本行目的：当没有片段时返回 None。
            if parts.is_empty() {
                // 本行目的：显式返回 None 表示无有效文本。
                None
            // 本行目的：处理有片段的情况。
            } else {
                // 本行目的：用分隔符拼接多个片段并返回。
                Some(parts.join(" / "))
            // 本行目的：结束片段为空判断。
            }
        }
        // 本行目的：对其他类型返回 None。
        _ => None,
    // 本行目的：结束类型分支。
    }
// 本行目的：结束 infobox 值提取函数。
}

/// 函数：split_infobox_terms | 输入：原始文本 | 输出：拆分后的词条列表 | 可能失败：无
// 本行目的：按常见分隔符拆分 infobox 的复合文本。
fn split_infobox_terms(value: &str) -> Vec<String> {
    // 本行目的：对分隔符进行拆分并清理空白。
    value
        // 本行目的：按多种分隔符切分字符串。
        .split(|ch| matches!(ch, '/' | '／' | '、' | ',' | '，' | '|'))
        // 本行目的：裁剪每个词条的首尾空白。
        .map(|item| item.trim())
        // 本行目的：过滤掉空字符串。
        .filter(|item| !item.is_empty())
        // 本行目的：将 &str 转为 String。
        .map(|item| item.to_string())
        // 本行目的：收集为向量。
        .collect()
// 本行目的：结束拆分函数。
}

/// 函数：extract_infobox_values | 输入：JSON 值引用 | 输出：词条列表 | 可能失败：无
// 本行目的：提取 infobox 值并拆分成多个词条。
pub(crate) fn extract_infobox_values(value: &Value) -> Vec<String> {
    // 本行目的：根据 JSON 类型分支处理。
    match value {
        // 本行目的：直接拆分字符串值。
        Value::String(text) => split_infobox_terms(text),
        // 本行目的：从对象中提取文本并拆分，缺失则返回空列表。
        Value::Object(map) => extract_value_from_object(map).map(|text| split_infobox_terms(&text)).unwrap_or_default(),
        // 本行目的：对数组逐项提取并合并结果。
        Value::Array(items) => {
            // 变量：output | 含义：累计的词条列表 | 类型：Vec<String> | 作用域：extract_infobox_values
            // 本行目的：初始化输出列表。
            let mut output = Vec::new();

            // 本行目的：遍历数组每个元素。
            for item in items {
                // 变量：extracted | 含义：当前项提取到的词条列表 | 类型：Vec<String> | 作用域：extract_infobox_values
                // 本行目的：根据项类型提取词条。
                let extracted = match item {
                    // 本行目的：对象项提取文本后拆分。
                    Value::Object(map) => extract_value_from_object(map).map(|text| split_infobox_terms(&text)).unwrap_or_default(),
                    // 本行目的：其他类型递归处理。
                    _ => extract_infobox_values(item),
                // 本行目的：结束项类型分支。
                };

                // 本行目的：合并当前项的词条到输出列表。
                output.extend(extracted);
            // 本行目的：结束数组遍历。
            }

            // 本行目的：返回汇总后的词条列表。
            output
        }
        // 本行目的：对其他类型返回空列表。
        _ => Vec::new(),
    // 本行目的：结束类型分支。
    }
// 本行目的：结束 infobox 值列表提取函数。
}

/// 函数：collect_subject_tags | 输入：官方标签与 meta 标签 | 输出：合并去重后的标签列表 | 可能失败：无
// 本行目的：收集条目标签并去重。
pub(crate) fn collect_subject_tags(tags: Option<Vec<SubjectTag>>, meta_tags: Option<Vec<String>>) -> Vec<String> {
    // 变量：output | 含义：收集到的标签列表 | 类型：Vec<String> | 作用域：collect_subject_tags
    // 本行目的：初始化输出容器。
    let mut output = Vec::new();

    // 本行目的：处理官方 tags 列表。
    if let Some(items) = tags {
        // 本行目的：遍历每个标签项。
        for item in items {
            // 变量：name | 含义：去除空白后的标签名 | 类型：String | 作用域：collect_subject_tags
            // 本行目的：规范化标签名。
            let name = item.name.trim().to_string();

            // 本行目的：仅在标签非空时加入输出。
            if !name.is_empty() {
                // 本行目的：追加标签名到列表。
                output.push(name);
            // 本行目的：结束非空检查。
            }
        // 本行目的：结束标签遍历。
        }
    // 本行目的：结束官方标签处理。
    }

    // 本行目的：处理 meta_tags 列表。
    if let Some(items) = meta_tags {
        // 本行目的：遍历每个 meta 标签名。
        for name in items {
            // 变量：name | 含义：去除空白后的标签名 | 类型：String | 作用域：collect_subject_tags
            // 本行目的：规范化标签名。
            let name = name.trim().to_string();

            // 本行目的：仅在标签非空时加入输出。
            if !name.is_empty() {
                // 本行目的：追加标签名到列表。
                output.push(name);
            // 本行目的：结束非空检查。
            }
        // 本行目的：结束 meta 标签遍历。
        }
    // 本行目的：结束 meta 标签处理。
    }

    // 本行目的：对标签列表进行去重并返回。
    dedupe_terms(output)
// 本行目的：结束标签收集函数。
}

/// 函数：normalize_tag | 输入：原始标签 | 输出：规范化标签 | 可能失败：无
// 本行目的：将标签统一为小写并去除空白。
fn normalize_tag(value: &str) -> String {
    // 本行目的：裁剪空白并转换为小写。
    value.trim().to_lowercase()
// 本行目的：结束标签规范化函数。
}

/// 函数：map_tags_to_official | 输入：标签列表、官方列表、别名映射 | 输出：官方标签列表 | 可能失败：无
// 本行目的：将标签与别名映射到官方标签集合。
pub(crate) fn map_tags_to_official(tags: &[String], official: &[&str], aliases: &[(&str, &str)]) -> Vec<String> {
    // 变量：official_map | 含义：规范化标签到官方名称的映射 | 类型：HashMap<String, String> | 作用域：map_tags_to_official
    // 本行目的：创建官方标签映射。
    let mut official_map = std::collections::HashMap::new();

    // 本行目的：遍历官方标签并填充映射。
    for name in official {
        // 本行目的：插入规范化后的官方标签映射。
        official_map.insert(normalize_tag(name), (*name).to_string());
    // 本行目的：结束官方标签遍历。
    }

    // 变量：alias_map | 含义：规范化别名到官方名称的映射 | 类型：HashMap<String, String> | 作用域：map_tags_to_official
    // 本行目的：创建别名映射。
    let mut alias_map = std::collections::HashMap::new();

    // 本行目的：遍历别名映射并填充。
    for (from, to) in aliases {
        // 本行目的：插入规范化后的别名映射。
        alias_map.insert(normalize_tag(from), (*to).to_string());
    // 本行目的：结束别名遍历。
    }

    // 变量：output | 含义：最终的官方标签列表 | 类型：Vec<String> | 作用域：map_tags_to_official
    // 本行目的：初始化输出列表。
    let mut output = Vec::new();
    // 变量：seen | 含义：去重集合 | 类型：HashSet<String> | 作用域：map_tags_to_official
    // 本行目的：初始化去重集合。
    let mut seen = std::collections::HashSet::new();

    // 本行目的：遍历所有标签。
    for tag in tags {
        // 变量：normalized | 含义：规范化后的标签 | 类型：String | 作用域：map_tags_to_official
        // 本行目的：对标签进行规范化。
        let normalized = normalize_tag(tag);

        // 本行目的：如果标签是官方名称则直接收录。
        if let Some(value) = official_map.get(&normalized) {
            // 本行目的：在未出现过时加入输出列表。
            if seen.insert(value.clone()) {
                // 本行目的：追加官方标签名称。
                output.push(value.clone());
            // 本行目的：结束去重判断。
            }
            // 本行目的：跳过别名映射逻辑。
            continue;
        // 本行目的：结束官方映射判断。
        }

        // 本行目的：如果标签匹配别名则映射为官方名称。
        if let Some(value) = alias_map.get(&normalized) {
            // 本行目的：在未出现过时加入输出列表。
            if seen.insert(value.clone()) {
                // 本行目的：追加官方标签名称。
                output.push(value.clone());
            // 本行目的：结束去重判断。
            }
        // 本行目的：结束别名映射判断。
        }
    // 本行目的：结束标签遍历。
    }

    // 本行目的：返回映射后的官方标签列表。
    output
// 本行目的：结束标签映射函数。
}

/// 函数：dedupe_terms | 输入：字符串列表 | 输出：去重后的列表 | 可能失败：无
// 本行目的：按出现顺序去重字符串列表。
pub(crate) fn dedupe_terms(values: Vec<String>) -> Vec<String> {
    // 变量：seen | 含义：去重集合 | 类型：HashSet<String> | 作用域：dedupe_terms
    // 本行目的：初始化去重集合。
    let mut seen = std::collections::HashSet::new();
    // 变量：output | 含义：去重后的输出列表 | 类型：Vec<String> | 作用域：dedupe_terms
    // 本行目的：初始化输出列表。
    let mut output = Vec::new();

    // 本行目的：遍历输入列表。
    for value in values {
        // 本行目的：仅在首次出现时加入输出。
        if seen.insert(value.clone()) {
            // 本行目的：追加去重后的值。
            output.push(value);
        // 本行目的：结束去重判断。
        }
    // 本行目的：结束遍历。
    }

    // 本行目的：返回去重后的列表。
    output
// 本行目的：结束去重函数。
}

/// 函数：extract_filter_groups | 输入：infobox 列表 | 输出：类型/地区/受众列表 | 可能失败：无
// 本行目的：从 infobox 中提取筛选分组。
pub(crate) fn extract_filter_groups(infobox: Option<Vec<InfoboxItem>>) -> (Vec<String>, Vec<String>, Vec<String>) {
    // 变量：types | 含义：类型列表 | 类型：Vec<String> | 作用域：extract_filter_groups
    // 本行目的：初始化类型列表。
    let mut types = Vec::new();
    // 变量：regions | 含义：地区列表 | 类型：Vec<String> | 作用域：extract_filter_groups
    // 本行目的：初始化地区列表。
    let mut regions = Vec::new();
    // 变量：audiences | 含义：受众列表 | 类型：Vec<String> | 作用域：extract_filter_groups
    // 本行目的：初始化受众列表。
    let mut audiences = Vec::new();
    // 变量：items | 含义：infobox 条目列表 | 类型：Vec<InfoboxItem> | 作用域：extract_filter_groups
    // 本行目的：处理 infobox 缺失时直接返回空结果。
    let items = match infobox {
        // 本行目的：提取实际条目列表。
        Some(items) => items,
        // 本行目的：缺失时返回空列表集合。
        None => return (types, regions, audiences),
    // 本行目的：结束 match 表达式。
    };

    // 本行目的：遍历 infobox 条目。
    for item in items {
        // 变量：values | 含义：当前条目拆分后的值列表 | 类型：Vec<String> | 作用域：extract_filter_groups
        // 本行目的：提取并拆分条目值。
        let values = extract_infobox_values(&item.value);

        // 本行目的：若当前条目无值则跳过。
        if values.is_empty() {
            // 本行目的：继续处理下一条目。
            continue;
        // 本行目的：结束空值判断。
        }

        // 本行目的：根据 infobox key 分类到不同列表。
        match item.key.as_str() {
            // 本行目的：将类型相关字段加入类型列表。
            "类型" | "题材" | "动画类型" | "分类" | "类别" => types.extend(values),
            // 本行目的：将地区相关字段加入地区列表。
            "地区" | "国家/地区" | "国家地区" | "国家" | "发行地区" => regions.extend(values),
            // 本行目的：将受众相关字段加入受众列表。
            "受众" | "对象" | "读者对象" => audiences.extend(values),
            // 本行目的：忽略其他未关注字段。
            _ => {}
        // 本行目的：结束 key 分类匹配。
        }
    // 本行目的：结束条目遍历。
    }

    // 本行目的：返回去重后的三类筛选列表。
    (dedupe_terms(types), dedupe_terms(regions), dedupe_terms(audiences))
// 本行目的：结束筛选分组提取函数。
}

/// 函数：extract_origin | 输入：infobox 列表 | 输出：原作字段 | 可能失败：无
// 本行目的：从 infobox 中提取原作信息。
pub(crate) fn extract_origin(infobox: Option<Vec<InfoboxItem>>) -> Option<String> {
    // 变量：items | 含义：infobox 条目列表 | 类型：Vec<InfoboxItem> | 作用域：extract_origin
    // 本行目的：解包 infobox，缺失则返回 None。
    let items = infobox?;

    // 本行目的：遍历 infobox 条目。
    for item in items {
        // 本行目的：仅处理 key 为原作的条目。
        if item.key == "原作" {
            // 本行目的：尝试提取条目值并返回。
            if let Some(value) = extract_infobox_value(&item.value) {
                // 本行目的：返回原作字段文本。
                return Some(value);
            // 本行目的：结束值存在判断。
            }
        // 本行目的：结束 key 判断。
        }
    // 本行目的：结束条目遍历。
    }

    // 本行目的：未找到原作字段时返回 None。
    None
// 本行目的：结束原作提取函数。
}

/// 函数：extract_aliases | 输入：infobox 列表 | 输出：别名列表 | 可能失败：无
// 本行目的：从 infobox 中提取别名信息。
pub(crate) fn extract_aliases(infobox: Option<Vec<InfoboxItem>>) -> Vec<String> {
    // 变量：output | 含义：别名输出列表 | 类型：Vec<String> | 作用域：extract_aliases
    // 本行目的：初始化输出列表。
    let mut output = Vec::new();
    // 变量：items | 含义：infobox 条目列表 | 类型：Vec<InfoboxItem> | 作用域：extract_aliases
    // 本行目的：处理 infobox 缺失时直接返回空列表。
    let items = match infobox {
        // 本行目的：取出条目列表。
        Some(items) => items,
        // 本行目的：缺失时返回空输出。
        None => return output,
    // 本行目的：结束 match 表达式。
    };

    // 本行目的：遍历 infobox 条目。
    for item in items {
        // 变量：key | 含义：条目的 key 字段 | 类型：&str | 作用域：extract_aliases
        // 本行目的：获取条目 key 的字符串视图。
        let key = item.key.as_str();
        // 变量：is_alias_key | 含义：是否为别名相关字段 | 类型：bool | 作用域：extract_aliases
        // 本行目的：判断当前 key 是否包含别名语义。
        let is_alias_key = key.contains("别名")
            // 本行目的：匹配“又名”关键词。
            || key.contains("又名")
            // 本行目的：匹配“英文”关键词。
            || key.contains("英文")
            // 本行目的：匹配“罗马”关键词。
            || key.contains("罗马")
            // 本行目的：匹配 romaji 英文关键词。
            || key.eq_ignore_ascii_case("romaji")
            // 本行目的：匹配 english 英文关键词。
            || key.eq_ignore_ascii_case("english");

        // 本行目的：如果不是别名字段则跳过。
        if !is_alias_key {
            // 本行目的：继续处理下一条目。
            continue;
        // 本行目的：结束别名字段判断。
        }

        // 变量：values | 含义：当前条目的别名值列表 | 类型：Vec<String> | 作用域：extract_aliases
        // 本行目的：提取并拆分条目值。
        let values = extract_infobox_values(&item.value);

        // 本行目的：追加所有别名值到输出列表。
        for v in values {
            // 本行目的：将别名加入输出列表。
            output.push(v);
        // 本行目的：结束别名遍历。
        }
    // 本行目的：结束条目遍历。
    }

    // 本行目的：去重后返回别名列表。
    dedupe_terms(output)
// 本行目的：结束别名提取函数。
}
