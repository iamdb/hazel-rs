use libc::{c_void, size_t, wchar_t};
use widestring::WideCString;

extern "C" {
    fn MediaInfo_New() -> *mut c_void;
    fn MediaInfo_Close(handle: *mut c_void);
    fn MediaInfo_Open(handle: *mut c_void, path: *const wchar_t) -> size_t;
    fn MediaInfo_Option(
        handle: *mut c_void,
        parameter: *const wchar_t,
        value: *const wchar_t,
    ) -> *const wchar_t;
    fn MediaInfo_Inform(handle: *mut c_void, reserved: size_t) -> *const wchar_t;
}

pub struct MediaInfo {
    handle: *mut c_void,
}

impl MediaInfo {
    pub(crate) fn new() -> Self {
        let handle = unsafe { MediaInfo_New() };

        let mi = Self { handle };
        mi.option("Info_Version", "22.12;hazelrs;0.1.0;");
        mi.option("Output", "JSON");

        mi
    }

    pub(crate) fn open(&self, path: &str) -> bool {
        let wchar = WideCString::from_str(path).expect("failed to create cpath");
        let result = unsafe { MediaInfo_Open(self.handle, wchar.as_ptr() as *const i32) };

        result == 1
    }

    pub(crate) fn close(&self) {
        unsafe {
            MediaInfo_Close(self.handle);
        }
    }

    pub(crate) fn inform(&self) -> String {
        let result = unsafe {
            WideCString::from_ptr_str(MediaInfo_Inform(self.handle, 0 as size_t) as *const u32)
        };

        result.to_string_lossy()
    }

    pub(crate) fn option(&self, param: &str, value: &str) -> String {
        let result = unsafe {
            let param = WideCString::from_str(param).expect("failed to create cpath");
            let value = WideCString::from_str(value).expect("failed to create cpath");

            WideCString::from_ptr_str(MediaInfo_Option(
                self.handle,
                param.as_ptr() as *const i32,
                value.as_ptr() as *const i32,
            ) as *const u32)
        };

        result.to_string_lossy()
    }
}
