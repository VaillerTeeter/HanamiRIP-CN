/* 文件：translate.rs | 用途：检测中文并通过百度翻译接口生成中文文本 | 关键对象：BaiduVerifier、translate_to_cn_baidu */
// 本行目的：引入动态库加载类型。
use libloading::Library;
// 本行目的：引入动态库符号类型。
use libloading::Symbol;
// 本行目的：引入一次性初始化容器。
use once_cell::sync::OnceCell;
// 本行目的：引入 HTTP 客户端类型。
use reqwest::Client;
// 本行目的：引入 JSON 值类型。
use serde_json::Value;
// 本行目的：引入环境变量读取模块。
use std::env;
// 本行目的：引入 C 字符类型。
use std::os::raw::c_char;
// 本行目的：引入路径缓冲类型。
use std::path::PathBuf;

/// 函数：contains_kana | 输入：文本引用 | 输出：是否包含假名 | 可能失败：无
// 本行目的：判断文本是否包含日文假名字符。
fn contains_kana(text: &str) -> bool {
    // 本行目的：遍历字符并检查 Unicode 范围。
    text.chars().any(|ch| matches!(ch as u32, 0x3040..=0x30FF | 0x31F0..=0x31FF))
// 本行目的：结束假名检测函数。
}

/// 函数：contains_cjk | 输入：文本引用 | 输出：是否包含 CJK 字符 | 可能失败：无
// 本行目的：判断文本是否包含中日韩汉字范围字符。
fn contains_cjk(text: &str) -> bool {
    // 本行目的：遍历字符并检查 Unicode 范围。
    text.chars().any(|ch| matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF))
// 本行目的：结束 CJK 检测函数。
}

/// 函数：is_chinese_text | 输入：文本引用 | 输出：是否为中文文本 | 可能失败：无
// 本行目的：判断文本包含中文且不包含假名。
pub(crate) fn is_chinese_text(text: &str) -> bool {
    // 本行目的：用 CJK 且非假名的规则判断是否中文。
    contains_cjk(text) && !contains_kana(text)
// 本行目的：结束中文检测函数。
}

// 本行目的：定义百度翻译密钥读取器结构体。
struct BaiduVerifier {
    // 变量：_lib | 含义：已加载的动态库实例 | 类型：Library | 作用域：BaiduVerifier
    // 本行目的：保存动态库以保持符号有效。
    _lib: Library,
    // 变量：get_app_id | 含义：读取 APP ID 的函数指针 | 类型：unsafe extern "C" fn() -> *const c_char | 作用域：BaiduVerifier
    // 本行目的：保存 APP ID 读取函数指针。
    get_app_id: unsafe extern "C" fn() -> *const c_char,
    // 变量：get_api_key | 含义：读取 API Key 的函数指针 | 类型：unsafe extern "C" fn() -> *const c_char | 作用域：BaiduVerifier
    // 本行目的：保存 API Key 读取函数指针。
    get_api_key: unsafe extern "C" fn() -> *const c_char,
// 本行目的：结束验证器结构体定义。
}

// 变量：BAIDU_VERIFIER | 含义：全局单例验证器 | 类型：OnceCell<BaiduVerifier> | 作用域：全局
// 本行目的：定义懒加载的百度验证器单例。
static BAIDU_VERIFIER: OnceCell<BaiduVerifier> = OnceCell::new();

/// 函数：load_baidu_verifier | 输入：无 | 输出：验证器引用 | 可能失败：找不到动态库或符号解析失败
// 本行目的：加载百度翻译动态库并解析符号。
fn load_baidu_verifier() -> Result<&'static BaiduVerifier, String> {
    // 本行目的：初始化单例或返回已初始化实例。
    BAIDU_VERIFIER.get_or_try_init(|| {
        // 本行目的：在 Windows 平台选择 DLL 名称。
        #[cfg(target_os = "windows")]
        // 变量：lib_name | 含义：动态库文件名 | 类型：&str | 作用域：load_baidu_verifier
        // 本行目的：设置 Windows 动态库名称。
        let lib_name = "baidu_verify.dll";

        // 本行目的：在 macOS 平台选择 dylib 名称。
        #[cfg(target_os = "macos")]
        // 变量：lib_name | 含义：动态库文件名 | 类型：&str | 作用域：load_baidu_verifier
        // 本行目的：设置 macOS 动态库名称。
        let lib_name = "libbaidu_verify.dylib";

        // 本行目的：在其他平台选择 so 名称。
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        // 变量：lib_name | 含义：动态库文件名 | 类型：&str | 作用域：load_baidu_verifier
        // 本行目的：设置 Linux 动态库名称。
        let lib_name = "libbaidu_verify.so";

        // 变量：candidates | 含义：候选动态库路径列表 | 类型：Vec<PathBuf> | 作用域：load_baidu_verifier
        // 本行目的：初始化候选路径列表。
        let mut candidates: Vec<PathBuf> = Vec::new();
        // 变量：manifest_dir | 含义：当前 crate 的清单目录 | 类型：PathBuf | 作用域：load_baidu_verifier
        // 本行目的：读取编译时的清单目录路径。
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // 变量：root_dir | 含义：项目根目录 | 类型：PathBuf | 作用域：load_baidu_verifier
        // 本行目的：回溯到项目根目录以定位 dist。
        let root_dir = manifest_dir.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).unwrap_or(&manifest_dir);
        // 本行目的：创建平台目录并加入候选路径。
        {
            // 变量：platform_dir | 含义：平台目录名称 | 类型：&str | 作用域：load_baidu_verifier
            // 本行目的：根据平台选择目录名称。
            let platform_dir = if cfg!(target_os = "windows") {
                // 本行目的：Windows 平台目录名。
                "windows"
            // 本行目的：处理 macOS 平台目录名。
            } else if cfg!(target_os = "macos") {
                // 本行目的：macOS 平台目录名。
                "macos"
            // 本行目的：处理其他平台目录名。
            } else {
                // 本行目的：Linux 平台目录名。
                "linux"
            // 本行目的：结束平台目录选择。
            };

            // 本行目的：将候选路径加入列表。
            candidates.push(root_dir.join("dist").join("baidu_verify").join(platform_dir).join(lib_name));
        // 本行目的：结束候选路径构建块。
        }

        // 变量：last_error | 含义：最后一次错误信息 | 类型：Option<String> | 作用域：load_baidu_verifier
        // 本行目的：初始化最后错误信息。
        let mut last_error = None;

        // 本行目的：遍历所有候选路径并尝试加载。
        for candidate in candidates {
            // 变量：path | 含义：规范化后的候选路径 | 类型：PathBuf | 作用域：load_baidu_verifier
            // 本行目的：将相对路径规范化为绝对路径。
            let path = if candidate.is_absolute() {
                // 本行目的：直接使用绝对路径。
                candidate
            // 本行目的：处理相对路径。
            } else {
                // 本行目的：尝试规范化路径，失败则使用原路径。
                candidate.canonicalize().unwrap_or(candidate)
            // 本行目的：结束路径规范化分支。
            };

            // 本行目的：如果文件不存在则跳过。
            if !path.exists() {
                // 本行目的：继续检查下一个候选路径。
                continue;
            // 本行目的：结束存在性判断。
            }

            // 本行目的：尝试加载动态库。
            match unsafe { Library::new(&path) } {
                // 本行目的：处理加载成功的情况。
                Ok(lib) => {
                    // 变量：get_app_id | 含义：APP ID 读取函数指针 | 类型：unsafe extern "C" fn() -> *const c_char | 作用域：load_baidu_verifier
                    // 本行目的：解析 APP ID 读取函数符号。
                    let get_app_id = {
                        // 变量：symbol | 含义：动态库符号 | 类型：Symbol<unsafe extern "C" fn() -> *const c_char> | 作用域：load_baidu_verifier
                        // 本行目的：加载 APP ID 读取函数符号。
                        let symbol: Symbol<unsafe extern "C" fn() -> *const c_char> =
                            // 本行目的：从动态库获取符号并转换错误。
                            unsafe { lib.get(b"baidu_get_app_id") }.map_err(|e| format!("解析百度翻译 APP ID 读取函数失败: {e}"))?;
                        // 本行目的：解引用符号获取函数指针。
                        *symbol
                    // 本行目的：结束 APP ID 符号解析块。
                    };

                    // 变量：get_api_key | 含义：API Key 读取函数指针 | 类型：unsafe extern "C" fn() -> *const c_char | 作用域：load_baidu_verifier
                    // 本行目的：解析 API Key 读取函数符号。
                    let get_api_key = {
                        // 变量：symbol | 含义：动态库符号 | 类型：Symbol<unsafe extern "C" fn() -> *const c_char> | 作用域：load_baidu_verifier
                        // 本行目的：加载 API Key 读取函数符号。
                        let symbol: Symbol<unsafe extern "C" fn() -> *const c_char> =
                            // 本行目的：从动态库获取符号并转换错误。
                            unsafe { lib.get(b"baidu_get_api_key") }.map_err(|e| format!("解析百度翻译 API Key 读取函数失败: {e}"))?;
                        // 本行目的：解引用符号获取函数指针。
                        *symbol
                    // 本行目的：结束 API Key 符号解析块。
                    };

                    // 本行目的：返回构建完成的验证器。
                    return Ok(BaiduVerifier {
                        // 本行目的：保存动态库句柄。
                        _lib: lib,
                        // 本行目的：保存 APP ID 读取函数指针。
                        get_app_id,
                        // 本行目的：保存 API Key 读取函数指针。
                        get_api_key,
                    // 本行目的：结束验证器结构体构建。
                    });
                // 本行目的：结束加载成功分支。
                }
                // 本行目的：处理加载失败的情况。
                Err(err) => {
                    // 本行目的：记录最后一次错误信息。
                    last_error = Some(format!("加载百度翻译校验库失败 ({}): {}", path.display(), err));
                // 本行目的：结束加载失败分支。
                }
            // 本行目的：结束动态库加载匹配。
            }
        // 本行目的：结束候选路径遍历。
        }

        // 本行目的：如果存在错误则优先返回最后错误。
        if let Some(err) = last_error {
            // 本行目的：返回最后一次错误信息。
            return Err(err);
        // 本行目的：结束错误存在判断。
        }

        // 本行目的：返回找不到动态库的错误提示。
        Err(format!(
            // 本行目的：构造完整错误提示文本。
            "找不到百度翻译动态库文件: {}\n请先构建动态库：\n1. 设置环境变量 BAIDU_TRANSLATE_APP_ID 和 BAIDU_TRANSLATE_API_KEY\n2. 运行命令: yarn \
             build:baidu-dll (Windows)\n3. 确保生成的 {} 文件位于 dist/baidu_verify/<platform>/ 目录下",
            // 本行目的：填充动态库文件名。
            lib_name, lib_name
        // 本行目的：结束格式化字符串构建。
        ))
    // 本行目的：结束单例初始化闭包。
    })
// 本行目的：结束验证器加载函数。
}

/// 函数：get_baidu_credentials | 输入：无 | 输出：APP ID 与 API Key | 可能失败：动态库缺失或密钥为空
// 本行目的：从动态库获取百度翻译密钥。
fn get_baidu_credentials() -> Result<(String, String), String> {
    // 变量：verifier | 含义：已加载的验证器 | 类型：&BaiduVerifier | 作用域：get_baidu_credentials
    // 本行目的：加载验证器单例。
    let verifier = load_baidu_verifier()?;
    // 变量：app_id_ptr | 含义：APP ID C 字符串指针 | 类型：*const c_char | 作用域：get_baidu_credentials
    // 本行目的：调用动态库函数获取 APP ID 指针。
    let app_id_ptr = unsafe { (verifier.get_app_id)() };
    // 变量：api_key_ptr | 含义：API Key C 字符串指针 | 类型：*const c_char | 作用域：get_baidu_credentials
    // 本行目的：调用动态库函数获取 API Key 指针。
    let api_key_ptr = unsafe { (verifier.get_api_key)() };

    // 本行目的：检查指针是否为空。
    if app_id_ptr.is_null() || api_key_ptr.is_null() {
        // 本行目的：返回密钥读取失败的错误。
        return Err("百度翻译密钥读取失败".to_string());
    // 本行目的：结束空指针检查。
    }

    // 变量：app_id | 含义：APP ID 字符串 | 类型：String | 作用域：get_baidu_credentials
    // 本行目的：将 C 字符串转换为 Rust 字符串并去空白。
    let app_id = unsafe { std::ffi::CStr::from_ptr(app_id_ptr) }.to_string_lossy().trim().to_string();
    // 变量：api_key | 含义：API Key 字符串 | 类型：String | 作用域：get_baidu_credentials
    // 本行目的：将 C 字符串转换为 Rust 字符串并去空白。
    let api_key = unsafe { std::ffi::CStr::from_ptr(api_key_ptr) }.to_string_lossy().trim().to_string();

    // 本行目的：检查密钥是否为空。
    if app_id.is_empty() || api_key.is_empty() {
        // 本行目的：返回密钥为空的错误。
        return Err("百度翻译密钥为空".to_string());
    // 本行目的：结束空字符串检查。
    }

    // 本行目的：返回 APP ID 与 API Key。
    Ok((app_id, api_key))
// 本行目的：结束密钥获取函数。
}

/// 函数：translate_to_cn_baidu | 输入：HTTP 客户端与原文 | 输出：翻译后的中文文本 | 可能失败：网络请求或翻译解析失败
// 本行目的：调用百度翻译接口并解析中文结果。
pub(crate) async fn translate_to_cn_baidu(client: &Client, text: &str) -> Result<String, String> {
    // 变量：(app_id, api_key) | 含义：百度翻译鉴权信息 | 类型：(String, String) | 作用域：translate_to_cn_baidu
    // 本行目的：获取百度翻译的鉴权信息。
    let (app_id, api_key) = get_baidu_credentials()?;

    // 变量：parse_translation | 含义：解析翻译响应的闭包 | 类型：impl Fn(&str) -> Result<Option<String>, String> | 作用域：translate_to_cn_baidu
    // 本行目的：定义解析翻译响应的辅助闭包。
    let parse_translation = |body: &str| -> Result<Option<String>, String> {
        // 变量：value | 含义：解析后的 JSON 值 | 类型：Value | 作用域：parse_translation
        // 本行目的：解析响应体为 JSON。
        let value: Value = serde_json::from_str(body).map_err(|e| format!("响应解析失败: {e}"))?;

        // 本行目的：检查是否存在错误码并返回错误信息。
        if let Some(code) = value.get("error_code").and_then(|v| v.as_str()) {
            // 变量：msg | 含义：错误消息文本 | 类型：&str | 作用域：parse_translation
            // 本行目的：读取错误消息字段。
            let msg = value.get("error_msg").and_then(|v| v.as_str()).unwrap_or("");
            // 本行目的：返回包含错误码的错误信息。
            return Err(format!("百度翻译错误: {code} {msg}"));
        // 本行目的：结束错误码检查。
        }

        // 变量：extract_list | 含义：从列表中提取 dst 文本的闭包 | 类型：impl Fn(&Value) -> Option<String> | 作用域：parse_translation
        // 本行目的：定义提取翻译结果的辅助闭包。
        let extract_list = |list: &Value| {
            // 本行目的：从数组中提取 dst 字段并拼接。
            list.as_array()
                // 本行目的：遍历数组并拼接 dst 字段。
                .map(|items| items.iter().filter_map(|item| item.get("dst").and_then(|v| v.as_str())).collect::<String>())
                // 本行目的：过滤空结果。
                .filter(|output| !output.trim().is_empty())
        // 本行目的：结束提取闭包。
        };

        // 本行目的：尝试从 trans_result 字段读取结果。
        if let Some(output) = value.get("trans_result").and_then(extract_list) {
            // 本行目的：返回提取到的翻译结果。
            return Ok(Some(output));
        // 本行目的：结束 trans_result 提取。
        }

        // 本行目的：尝试从 data 字段读取结果。
        if let Some(output) = value.get("data").and_then(extract_list) {
            // 本行目的：返回提取到的翻译结果。
            return Ok(Some(output));
        // 本行目的：结束 data 提取。
        }

        // 本行目的：尝试从 result 嵌套字段读取结果。
        if let Some(result) = value.get("result") {
            // 本行目的：尝试从 result.trans_result 读取结果。
            if let Some(output) = result.get("trans_result").and_then(extract_list) {
                // 本行目的：返回提取到的翻译结果。
                return Ok(Some(output));
            // 本行目的：结束 result.trans_result 提取。
            }

            // 本行目的：尝试从 result.data 读取结果。
            if let Some(output) = result.get("data").and_then(extract_list) {
                // 本行目的：返回提取到的翻译结果。
                return Ok(Some(output));
            // 本行目的：结束 result.data 提取。
            }
        // 本行目的：结束 result 提取。
        }

        // 本行目的：未获取到结果时返回 None。
        Ok(None)
    // 本行目的：结束解析闭包。
    };

    // 变量：ai_request | 含义：翻译 API 请求构建器 | 类型：reqwest::RequestBuilder | 作用域：translate_to_cn_baidu
    // 本行目的：构建翻译请求。
    let ai_request = client
        // 本行目的：设置翻译 API 地址。
        .post("https://fanyi-api.baidu.com/ait/api/aiTextTranslate")
        // 本行目的：设置请求 JSON 体。
        .json(&serde_json::json!({
          // 本行目的：写入 APP ID。
          "appid": app_id,
          // 本行目的：写入待翻译文本。
          "q": text,
          // 本行目的：设置源语言为自动检测。
          "from": "auto",
          // 本行目的：设置目标语言为中文。
          "to": "zh",
          // 本行目的：指定模型类型。
          "model_type": "llm"
        // 本行目的：结束 JSON 构建。
        }))
        // 本行目的：设置鉴权信息。
        .bearer_auth(&api_key);
    // 变量：ai_response | 含义：翻译 API 响应 | 类型：reqwest::Response | 作用域：translate_to_cn_baidu
    // 本行目的：发送请求并获取响应。
    let ai_response = ai_request.send().await.map_err(|e| e.to_string())?;

    // 本行目的：仅在响应成功时解析结果。
    if ai_response.status().is_success() {
        // 变量：body | 含义：响应体字符串 | 类型：String | 作用域：translate_to_cn_baidu
        // 本行目的：读取响应体。
        let body = ai_response.text().await.map_err(|e| e.to_string())?;

        // 本行目的：解析翻译结果并处理错误。
        match parse_translation(&body) {
            // 本行目的：翻译成功时返回结果。
            Ok(Some(output)) => return Ok(output),
            // 本行目的：无结果时继续处理错误。
            Ok(None) => {}
            // 本行目的：解析出错时判断是否需要返回。
            Err(err) => {
                // 本行目的：除特定错误外直接返回。
                if !err.contains("52003") {
                    // 本行目的：返回解析错误。
                    return Err(err);
                // 本行目的：结束错误码判断。
                }
            // 本行目的：结束错误处理分支。
            }
        // 本行目的：结束解析匹配。
        }
    // 本行目的：结束响应成功判断。
    }

    // 本行目的：返回翻译结果为空的错误。
    Err("翻译结果为空".to_string())
// 本行目的：结束翻译函数。
}
