/*
  这个 crate 用于导出两个 C ABI 函数，
  让其它模块（通过动态库加载）能读取百度翻译的密钥。
  密钥来自编译期环境变量，因此不会以明文形式写在源码里。
*/

use std::os::raw::c_char;

// 把环境变量拼接成以 \0 结尾的 C 字符串。
// 这样返回给 C 侧时不会越界。
const APP_ID_C: &str = concat!(env!("BAIDU_TRANSLATE_APP_ID"), "\0");
const API_KEY_C: &str = concat!(env!("BAIDU_TRANSLATE_API_KEY"), "\0");

// 导出 C ABI 函数：返回 APP ID 指针。
// #[no_mangle] 确保符号名不被 Rust 改写，方便动态库查找。
#[no_mangle]
pub extern "C" fn baidu_get_app_id() -> *const c_char {
  APP_ID_C.as_ptr() as *const c_char
}

// 导出 C ABI 函数：返回 API Key 指针。
#[no_mangle]
pub extern "C" fn baidu_get_api_key() -> *const c_char {
  API_KEY_C.as_ptr() as *const c_char
}
