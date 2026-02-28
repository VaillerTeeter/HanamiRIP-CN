/* 文件：lib.rs | 用途：向宿主程序导出百度翻译凭证（C ABI） | 关键对象：APP_ID_C、API_KEY_C、baidu_get_app_id、baidu_get_api_key */

// 引入 C 语言字符指针类型，用于定义跨语言导出函数的返回值。
use std::os::raw::c_char;

// 变量：APP_ID_C | 含义：携带以 \0 结尾的 AppId 常量字符串 | 类型：&str | 作用域：模块级只读常量
// 将编译期环境变量拼接为 C 风格字符串（末尾 NUL），避免运行时再分配。
const APP_ID_C: &str = concat!(env!("BAIDU_TRANSLATE_APP_ID"), "\0");
// 变量：API_KEY_C | 含义：携带以 \0 结尾的 ApiKey 常量字符串 | 类型：&str | 作用域：模块级只读常量
// 将编译期环境变量拼接为 C 风格字符串（末尾 NUL），保证 FFI 读取安全。
const API_KEY_C: &str = concat!(env!("BAIDU_TRANSLATE_API_KEY"), "\0");

/// 函数：baidu_get_app_id | 输入：无 | 输出：指向 APP_ID_C 的 C 字符串指针 | 可能失败：若调用方误用无效生命周期指针会导致未定义行为
/// 关闭 Rust 名字改编，确保导出符号名与外部调用约定一致。
#[no_mangle]
/// 使用 C 调用约定导出函数，供 DLL 调用方（如 C/C++/其他语言）直接访问。
pub extern "C" fn baidu_get_app_id() -> *const c_char {
    // 返回静态字符串底层地址；该地址在进程生命周期内稳定可读。
    APP_ID_C.as_ptr() as *const c_char
}

/// 函数：baidu_get_api_key | 输入：无 | 输出：指向 API_KEY_C 的 C 字符串指针 | 可能失败：若调用方把返回指针当可写内存使用会导致未定义行为
/// 关闭 Rust 名字改编，确保导出符号可被外部按固定名字定位。
#[no_mangle]
/// 使用 C ABI 导出，便于与宿主程序进行稳定的 FFI 对接。
pub extern "C" fn baidu_get_api_key() -> *const c_char {
    // 返回静态字符串底层地址；调用方只读访问即可。
    API_KEY_C.as_ptr() as *const c_char
}
