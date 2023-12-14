use libc::{c_void, size_t, wchar_t};
use widestring::WideCString;

#[repr(C)]
#[derive(Debug)]
pub enum StreamKind {
    _General = 0,
    Video,
    _Audio,
    _Text,
    _Other,
    Image,
    _Menu,
    _Max,
}

#[repr(C)]
#[derive(Debug)]
pub enum InfoKind {
    Name = 0,
    Text,
    _Measure,
    _Options,
    _NameText,
    _MeasureText,
    _Info,
    _HowTo,
    _Max,
}

#[allow(dead_code)]
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
    fn MediaInfo_Get(
        handle: *mut c_void,
        stream_kind: StreamKind,
        stream_number: size_t,
        parameter: *const wchar_t,
        info_kind: InfoKind,
        search_kind: InfoKind,
    ) -> *const wchar_t;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MediaInfo {
    handle: *mut c_void,
    is_open: bool,
}

impl MediaInfo {
    pub(crate) fn new() -> Self {
        let handle = unsafe { MediaInfo_New() };

        let mi = Self {
            handle,
            is_open: false,
        };
        mi.option("Info_Version", "22.12;hazelrs;0.1.0;");
        //mi.option("Complete", "1");
        mi.option("Output", "JSON");

        mi
    }

    pub(crate) fn open(&self, path: &str) -> bool {
        let wchar = WideCString::from_str(path).expect("failed to create wchar");
        let result = unsafe { MediaInfo_Open(self.handle, wchar.as_ptr() as *const i32) };

        result == 1
    }

    #[allow(dead_code)]
    pub(crate) fn is_open(&self) -> bool {
        self.is_open
    }

    pub(crate) fn close(&self) {
        unsafe {
            MediaInfo_Close(self.handle);
        }
    }

    pub(crate) fn _inform(&self, info_stream: Option<&str>, parameter: Option<&str>) -> String {
        if let (Some(p), Some(s)) = (parameter, info_stream) {
            self.option("Inform", &format!("{s};%{p}%"));
        };

        let result = unsafe {
            WideCString::from_ptr_str(MediaInfo_Inform(self.handle, 0 as size_t) as *const u32)
        };

        result.to_string_lossy()
    }

    pub(crate) fn get_string(&self, stream_kind: StreamKind, field: &str) -> String {
        let wchar = WideCString::from_str(field).expect("failed to create wchar");
        let result = unsafe {
            WideCString::from_ptr_str(MediaInfo_Get(
                self.handle,
                stream_kind,
                0,
                wchar.as_ptr() as *const i32,
                InfoKind::Text,
                InfoKind::Name,
            ) as *const u32)
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
