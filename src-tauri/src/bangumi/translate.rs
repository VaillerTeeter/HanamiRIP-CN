use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::os::raw::c_char;
use std::path::PathBuf;

fn contains_kana(text: &str) -> bool {
  text.chars()
    .any(|ch| matches!(ch as u32, 0x3040..=0x30FF | 0x31F0..=0x31FF))
}

fn contains_cjk(text: &str) -> bool {
  text.chars().any(|ch| matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF))
}

pub(crate) fn is_chinese_text(text: &str) -> bool {
  contains_cjk(text) && !contains_kana(text)
}

struct BaiduVerifier {
  _lib: Library,
  get_app_id: unsafe extern "C" fn() -> *const c_char,
  get_api_key: unsafe extern "C" fn() -> *const c_char,
}

static BAIDU_VERIFIER: OnceCell<BaiduVerifier> = OnceCell::new();

fn load_baidu_verifier() -> Result<&'static BaiduVerifier, String> {
  BAIDU_VERIFIER.get_or_try_init(|| {
    #[cfg(target_os = "windows")]
    let lib_name = "baidu_verify.dll";
    #[cfg(target_os = "macos")]
    let lib_name = "libbaidu_verify.dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let lib_name = "libbaidu_verify.so";

    let mut candidates: Vec<PathBuf> = Vec::new();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(root_dir) = manifest_dir.parent() {
      let platform_dir = if cfg!(target_os = "windows") {
        "windows"
      } else if cfg!(target_os = "macos") {
        "macos"
      } else {
        "linux"
      };
      candidates.push(root_dir.join("dist").join("baidu_verify").join(platform_dir).join(lib_name));
    }

    let mut last_error = None;
    for candidate in candidates {
      let path = if candidate.is_absolute() {
        candidate
      } else {
        candidate
          .canonicalize()
          .unwrap_or(candidate)
      };

      if !path.exists() {
        continue;
      }

      match unsafe { Library::new(&path) } {
        Ok(lib) => {
          let get_app_id = {
            let symbol: Symbol<unsafe extern "C" fn() -> *const c_char> =
              unsafe { lib.get(b"baidu_get_app_id") }
                .map_err(|e| format!("解析百度翻译 APP ID 读取函数失败: {e}"))?;
            *symbol
          };
          let get_api_key = {
            let symbol: Symbol<unsafe extern "C" fn() -> *const c_char> =
              unsafe { lib.get(b"baidu_get_api_key") }
                .map_err(|e| format!("解析百度翻译 API Key 读取函数失败: {e}"))?;
            *symbol
          };
          return Ok(BaiduVerifier {
            _lib: lib,
            get_app_id,
            get_api_key,
          });
        }
        Err(err) => {
          last_error = Some(format!("加载百度翻译校验库失败 ({}): {}", path.display(), err));
        }
      }
    }

    if let Some(err) = last_error {
      return Err(err);
    }

    Err(format!(
      "找不到百度翻译动态库文件: {}\n\
       请先构建动态库：\n\
       1. 设置环境变量 BAIDU_TRANSLATE_APP_ID 和 BAIDU_TRANSLATE_API_KEY\n\
       2. 运行命令: yarn run build:baidu-so:windows (Windows) 或 yarn run build:baidu-so:linux (Linux)\n\
       3. 确保生成的 {} 文件位于 dist/baidu_verify/<platform>/ 目录下",
      lib_name, lib_name
    ))
  })
}

fn get_baidu_credentials() -> Result<(String, String), String> {
  let verifier = load_baidu_verifier()?;
  let app_id_ptr = unsafe { (verifier.get_app_id)() };
  let api_key_ptr = unsafe { (verifier.get_api_key)() };
  if app_id_ptr.is_null() || api_key_ptr.is_null() {
    return Err("百度翻译密钥读取失败".to_string());
  }
  let app_id = unsafe { std::ffi::CStr::from_ptr(app_id_ptr) }
    .to_string_lossy()
    .trim()
    .to_string();
  let api_key = unsafe { std::ffi::CStr::from_ptr(api_key_ptr) }
    .to_string_lossy()
    .trim()
    .to_string();
  if app_id.is_empty() || api_key.is_empty() {
    return Err("百度翻译密钥为空".to_string());
  }
  Ok((app_id, api_key))
}

pub(crate) async fn translate_to_cn_baidu(client: &Client, text: &str) -> Result<String, String> {
  let (app_id, api_key) = get_baidu_credentials()?;

  let parse_translation = |body: &str| -> Result<Option<String>, String> {
    let value: Value = serde_json::from_str(body).map_err(|e| format!("响应解析失败: {e}"))?;
    if let Some(code) = value.get("error_code").and_then(|v| v.as_str()) {
      let msg = value
        .get("error_msg")
        .and_then(|v| v.as_str())
        .unwrap_or("");
      return Err(format!("百度翻译错误: {code} {msg}"));
    }
    let extract_list = |list: &Value| {
      list.as_array()
        .map(|items| {
          items
            .iter()
            .filter_map(|item| item.get("dst").and_then(|v| v.as_str()))
            .collect::<String>()
        })
        .filter(|output| !output.trim().is_empty())
    };

    if let Some(output) = value.get("trans_result").and_then(extract_list) {
      return Ok(Some(output));
    }
    if let Some(output) = value.get("data").and_then(extract_list) {
      return Ok(Some(output));
    }
    if let Some(result) = value.get("result") {
      if let Some(output) = result.get("trans_result").and_then(extract_list) {
        return Ok(Some(output));
      }
      if let Some(output) = result.get("data").and_then(extract_list) {
        return Ok(Some(output));
      }
    }
    Ok(None)
  };

  let ai_request = client
    .post("https://fanyi-api.baidu.com/ait/api/aiTextTranslate")
    .json(&serde_json::json!({
      "appid": app_id,
      "q": text,
      "from": "auto",
      "to": "zh",
      "model_type": "llm"
    }))
    .bearer_auth(&api_key);

  let ai_response = ai_request.send().await.map_err(|e| e.to_string())?;

  if ai_response.status().is_success() {
    let body = ai_response.text().await.map_err(|e| e.to_string())?;
    match parse_translation(&body) {
      Ok(Some(output)) => return Ok(output),
      Ok(None) => {}
      Err(err) => {
        if !err.contains("52003") {
          return Err(err);
        }
      }
    }
  }

  Err("翻译结果为空".to_string())
}