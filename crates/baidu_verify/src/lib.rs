use std::os::raw::c_char;

const APP_ID_C: &str = concat!(env!("BAIDU_TRANSLATE_APP_ID"), "\0");
const API_KEY_C: &str = concat!(env!("BAIDU_TRANSLATE_API_KEY"), "\0");

#[no_mangle]
pub extern "C" fn baidu_get_app_id() -> *const c_char {
  APP_ID_C.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn baidu_get_api_key() -> *const c_char {
  API_KEY_C.as_ptr() as *const c_char
}
