/*
  这个文件负责“翻译相关功能”，核心目标：
  1) 判断文本是否已经是中文；
  2) 通过百度翻译接口把非中文内容翻成中文。
  其中百度翻译的密钥不是明文写在代码里，而是通过动态库读取，
  这样可以避免密钥直接暴露在仓库中。
*/

use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::os::raw::c_char;
use std::path::PathBuf;

// 判断文本是否包含日文假名。
// 用于区分“日文”和“中文”。
fn contains_kana(text: &str) -> bool {
  text.chars()
    .any(|ch| matches!(ch as u32, 0x3040..=0x30FF | 0x31F0..=0x31FF))
}

// 判断文本是否包含中日韩统一表意文字（汉字）。
// 注意：这里只是检测字符范围，并不保证“完整中文”。
fn contains_cjk(text: &str) -> bool {
  text.chars().any(|ch| matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF))
}

// 判断一段文本是否“看起来是中文”：
// - 有汉字
// - 但没有日文假名
pub(crate) fn is_chinese_text(text: &str) -> bool {
  contains_cjk(text) && !contains_kana(text)
}

// 通过动态库读取百度翻译密钥的结构体。
// _lib 用来持有动态库句柄，防止被提前释放。
struct BaiduVerifier {
  _lib: Library,
  get_app_id: unsafe extern "C" fn() -> *const c_char,
  get_api_key: unsafe extern "C" fn() -> *const c_char,
}

// OnceCell 确保动态库只加载一次（线程安全）。
static BAIDU_VERIFIER: OnceCell<BaiduVerifier> = OnceCell::new();

// 加载百度翻译动态库，并解析出读取密钥的函数地址。
// 返回静态引用，后续重复调用不会再次加载。
fn load_baidu_verifier() -> Result<&'static BaiduVerifier, String> {
  BAIDU_VERIFIER.get_or_try_init(|| {
    #[cfg(target_os = "windows")]
    let lib_name = "baidu_verify.dll";
    #[cfg(target_os = "macos")]
    let lib_name = "libbaidu_verify.dylib";
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let lib_name = "libbaidu_verify.so";

    // 动态库可能存在的路径候选列表。
    let mut candidates: Vec<PathBuf> = Vec::new();

    // CARGO_MANIFEST_DIR 是编译期环境变量：当前 crate 的根目录。
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir
      .parent()
      .and_then(|p| p.parent())
      .and_then(|p| p.parent())
      .unwrap_or(&manifest_dir);
    {
      // 根据不同系统拼接动态库文件名。
      let platform_dir = if cfg!(target_os = "windows") {
        "windows"
      } else if cfg!(target_os = "macos") {
        "macos"
      } else {
        "linux"
      };
      candidates.push(root_dir.join("dist").join("baidu_verify").join(platform_dir).join(lib_name));
    }

    // 用来保存“最后一次失败原因”，方便最终报错。
    let mut last_error = None;
    for candidate in candidates {
      let path = if candidate.is_absolute() {
        candidate
      } else {
        candidate
          .canonicalize()
          .unwrap_or(candidate)
      };

      // 路径不存在就跳过。
      if !path.exists() {
        continue;
      }

      // 这里是 unsafe：动态库加载可能失败，且需要手动管理。
      match unsafe { Library::new(&path) } {
        Ok(lib) => {
          // 解析动态库里的函数指针：读取 APP ID。
          let get_app_id = {
            let symbol: Symbol<unsafe extern "C" fn() -> *const c_char> =
              unsafe { lib.get(b"baidu_get_app_id") }
                .map_err(|e| format!("解析百度翻译 APP ID 读取函数失败: {e}"))?;
            *symbol
          };
          // 解析动态库里的函数指针：读取 API Key。
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
          // 记录失败原因，继续尝试下一个候选路径。
          last_error = Some(format!("加载百度翻译校验库失败 ({}): {}", path.display(), err));
        }
      }
    }

    // 所有路径都失败时，返回最接近原因的错误。
    if let Some(err) = last_error {
      return Err(err);
    }

    // 彻底找不到动态库时给出构建提示。
    Err(format!(
      "找不到百度翻译动态库文件: {}\n\
       请先构建动态库：\n\
       1. 设置环境变量 BAIDU_TRANSLATE_APP_ID 和 BAIDU_TRANSLATE_API_KEY\n\
       2. 运行命令: yarn build:baidu-dll (Windows)\n\
       3. 确保生成的 {} 文件位于 dist/baidu_verify/<platform>/ 目录下",
      lib_name, lib_name
    ))
  })
}

// 从动态库中读取百度翻译密钥。
// 返回 (app_id, api_key)。
fn get_baidu_credentials() -> Result<(String, String), String> {
  let verifier = load_baidu_verifier()?;
  // 调用动态库函数得到 C 字符串指针。
  let app_id_ptr = unsafe { (verifier.get_app_id)() };
  let api_key_ptr = unsafe { (verifier.get_api_key)() };
  if app_id_ptr.is_null() || api_key_ptr.is_null() {
    return Err("百度翻译密钥读取失败".to_string());
  }
  // 把 C 字符串转换成 Rust String。
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

// 调用百度翻译 API，把文本翻译成中文。
// 失败时返回错误字符串。
pub(crate) async fn translate_to_cn_baidu(client: &Client, text: &str) -> Result<String, String> {
  let (app_id, api_key) = get_baidu_credentials()?;

  // 用闭包解析翻译结果，兼容多种返回结构。
  let parse_translation = |body: &str| -> Result<Option<String>, String> {
    let value: Value = serde_json::from_str(body).map_err(|e| format!("响应解析失败: {e}"))?;
    // 如果响应里有 error_code，说明请求失败。
    if let Some(code) = value.get("error_code").and_then(|v| v.as_str()) {
      let msg = value
        .get("error_msg")
        .and_then(|v| v.as_str())
        .unwrap_or("");
      return Err(format!("百度翻译错误: {code} {msg}"));
    }
    // 尝试从列表里提取翻译结果。
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

    // 兼容不同字段名：trans_result / data / result.*
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

  // 构造请求体。
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

  // 发送请求。
  let ai_response = ai_request.send().await.map_err(|e| e.to_string())?;

  // 状态码成功才解析响应体。
  if ai_response.status().is_success() {
    let body = ai_response.text().await.map_err(|e| e.to_string())?;
    match parse_translation(&body) {
      Ok(Some(output)) => return Ok(output),
      Ok(None) => {}
      Err(err) => {
        // 特定错误码允许继续尝试（比如配额等临时问题）。
        if !err.contains("52003") {
          return Err(err);
        }
      }
    }
  }

  // 走到这里说明没拿到有效翻译结果。
  Err("翻译结果为空".to_string())
}
