use crate::cef_string::CefString;

pub struct ProcessMessage {
    pub(crate) inner: *mut chromium_sys::cef_process_message_t,
}

impl ProcessMessage {
    pub fn new(name: &str) -> Self {
        let name = CefString::new_raw(name);
        let inner = unsafe { chromium_sys::cef_process_message_create(&name) };
        Self { inner }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let get_name = (*self.inner).get_name.unwrap();
            CefString::from_raw(get_name(self.inner))
        }
    }

    pub fn write_string(&mut self, index: usize, data: &str) -> bool {
        unsafe {
            let args = self.get_arg_list();
            let set_string = (*args).set_string.unwrap();
            let data = CefString::new_raw(data);

            set_string(args, index, &data) == 1
        }
    }

    pub fn read_string(&mut self, index: usize) -> Option<String> {
        unsafe {
            let args = self.get_arg_list();
            let get_string = (*args).get_string.unwrap();
            let get_type = (*args).get_type.unwrap();

            let ty: ValueType = get_type(args, index).into();
            if ty != ValueType::String {
                return None;
            }

            let data = get_string(args, index);
            Some(CefString::from_raw(data))
        }
    }

    pub fn write_int(&mut self, index: usize, data: i32) -> bool {
        unsafe {
            let args = self.get_arg_list();
            let set_int = (*args).set_int.unwrap();

            set_int(args, index, data) == 1
        }
    }

    pub fn read_int(&mut self, index: usize) -> Option<i32> {
        unsafe {
            let args = self.get_arg_list();
            let get_int = (*args).get_int.unwrap();
            let get_type = (*args).get_type.unwrap();

            let ty: ValueType = get_type(args, index).into();
            if ty != ValueType::Int {
                return None;
            }

            Some(get_int(args, index))
        }
    }

    fn get_arg_list(&self) -> *mut chromium_sys::cef_list_value_t {
        unsafe {
            let get_argument_list = (*self.inner).get_argument_list.unwrap();
            get_argument_list(self.inner)
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ProcessId {
    Browser = chromium_sys::cef_process_id_t_PID_BROWSER,
    Renderer = chromium_sys::cef_process_id_t_PID_RENDERER,
}

impl From<chromium_sys::cef_process_id_t> for ProcessId {
    fn from(value: chromium_sys::cef_process_id_t) -> Self {
        match value {
            chromium_sys::cef_process_id_t_PID_BROWSER => Self::Browser,
            chromium_sys::cef_process_id_t_PID_RENDERER => Self::Renderer,
            _ => unreachable!(),
        }
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq)]
enum ValueType {
    Invalid = chromium_sys::cef_value_type_t_VTYPE_INVALID,
    Null = chromium_sys::cef_value_type_t_VTYPE_NULL,
    Bool = chromium_sys::cef_value_type_t_VTYPE_BOOL,
    Int = chromium_sys::cef_value_type_t_VTYPE_INT,
    Double = chromium_sys::cef_value_type_t_VTYPE_DOUBLE,
    String = chromium_sys::cef_value_type_t_VTYPE_STRING,
    Binary = chromium_sys::cef_value_type_t_VTYPE_BINARY,
    Dictionary = chromium_sys::cef_value_type_t_VTYPE_DICTIONARY,
    List = chromium_sys::cef_value_type_t_VTYPE_LIST,
}

impl From<chromium_sys::cef_value_type_t> for ValueType {
    fn from(value: chromium_sys::cef_value_type_t) -> Self {
        match value {
            chromium_sys::cef_value_type_t_VTYPE_INVALID => Self::Invalid,
            chromium_sys::cef_value_type_t_VTYPE_NULL => Self::Null,
            chromium_sys::cef_value_type_t_VTYPE_BOOL => Self::Bool,
            chromium_sys::cef_value_type_t_VTYPE_INT => Self::Int,
            chromium_sys::cef_value_type_t_VTYPE_DOUBLE => Self::Double,
            chromium_sys::cef_value_type_t_VTYPE_STRING => Self::String,
            chromium_sys::cef_value_type_t_VTYPE_BINARY => Self::Binary,
            chromium_sys::cef_value_type_t_VTYPE_DICTIONARY => Self::Dictionary,
            chromium_sys::cef_value_type_t_VTYPE_LIST => Self::List,
            _ => unreachable!(),
        }
    }
}
