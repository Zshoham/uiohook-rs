use std::any::Any;

use thiserror::Error;
use uiohook_sys as ffi;

#[derive(Debug, Error)]
#[error("
    Trying to post invalid event type `{0}`, control events such as Enable and Disable cannot be posted.
    Please use hook_start, hook_stop or similar APIs.
")]
pub struct PostEventError(pub String);

#[cfg(target_os = "linux")]
#[derive(Debug, Error)]
pub enum HookError {
    #[error("Failed to allocate memory.")]
    OutOfMemory,
    #[error("Failed to open X11 display.")]
    XOpenDisplay,
    #[error("Unable to locate XRecord extension.")]
    XRecordNotFound,
    #[error("Unable to allocate XRecord range.")]
    XRecordAllocRange,
    #[error("Unable to allocate XRecord context.")]
    XRecordCreateContext,
    #[error("Failed to enable XRecord context.")]
    XRecordEnableContext,
    #[error("Could not retrieve XRecord context.")]
    XRecordGetContext,
    #[error("Encountered unknown error.")]
    Unknown(&'static str, Box<dyn Any + Send + 'static>),
}

#[cfg(target_os = "linux")]
impl From<u32> for HookError {
    fn from(native_error: u32) -> Self {
        match native_error {
            ffi::UIOHOOK_ERROR_OUT_OF_MEMORY => HookError::OutOfMemory,
            ffi::UIOHOOK_ERROR_X_OPEN_DISPLAY => HookError::XOpenDisplay,
            ffi::UIOHOOK_ERROR_X_RECORD_NOT_FOUND => HookError::XRecordNotFound,
            ffi::UIOHOOK_ERROR_X_RECORD_ALLOC_RANGE => HookError::XRecordAllocRange,
            ffi::UIOHOOK_ERROR_X_RECORD_CREATE_CONTEXT => HookError::XRecordCreateContext,
            ffi::UIOHOOK_ERROR_X_RECORD_ENABLE_CONTEXT => HookError::XRecordEnableContext,
            ffi::UIOHOOK_ERROR_X_RECORD_GET_CONTEXT => HookError::XRecordGetContext,
            _ => HookError::Unknown("unknown error code", Box::new(())),
        }
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, Error)]
pub enum HookError {
    #[error("Failed to allocate memory.")]
    OutOfMemory,
    #[error("Failed to register native windows hook.")]
    SetHookEx,
    #[error("Failed to retrieve handle for native windows hook.")]
    GetModuleHandle,
    #[error("Encountered unknown error.")]
    Unknown(&'static str, Box<dyn Any + Send + 'static>),
}

#[cfg(target_os = "windows")]
impl From<u32> for HookError {
    fn from(native_error: u32) -> Self {
        match native_error {
            ffi::UIOHOOK_ERROR_OUT_OF_MEMORY => HookError::OutOfMemory,
            ffi::UIOHOOK_ERROR_SET_WINDOWS_HOOK_EX => HookError::SetHookEx,
            ffi::UIOHOOK_ERROR_GET_MODULE_HANDLE => HookError::GetModuleHandle,
            _ => HookError::Unknown("unknown error code", Box::new(())),
        }
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Error)]
pub enum HookError {
    #[error("Failed to allocate memory.")]
    OutOfMemory,
    #[error("Failed to enable access for assistive devices.")]
    AXAPIDisabled,
    #[error("Failed to create apple event port.")]
    CreateEventPort,
    #[error("Failed to create apple run loop source.")]
    CreateRunLoopSource,
    #[error("Failed to acquire apple run loop.")]
    GetRunLoop,
    #[error("Failed to create apple run loop observer.")]
    CreateObserver,
    #[error("Encountered unknown error.")]
    Unknown(&'static str, Box<dyn Any + Send + 'static>),
}

#[cfg(target_os = "macos")]
impl From<u32> for HookError {
    fn from(native_error: u32) -> Self {
        match native_error {
            ffi::UIOHOOK_ERROR_OUT_OF_MEMORY => HookError::OutOfMemory,
            ffi::UIOHOOK_ERROR_SET_WINDOWS_HOOK_EX => HookError::AXAPIDisabled,
            ffi::UIOHOOK_ERROR_GET_MODULE_HANDLE => HookError::CreateEventPort,
            ffi::UIOHOOK_ERROR_CREATE_RUN_LOOP_SOURCE => HookError::CreateRunLoopSource,
            ffi::UIOHOOK_ERROR_GET_RUNLOOP => HookError::GetRunLoop,
            ffi::UIOHOOK_ERROR_CREATE_OBSERVER => HookError::CreateObserver,
            _ => HookError::Unknown("unknown error code", Box::new(())),
        }
    }
}

impl From<Box<dyn Any + Send + 'static>> for HookError {
    fn from(thread_panic: Box<dyn Any + Send + 'static>) -> Self {
        if let Some(s) = thread_panic.downcast_ref::<&'static str>() {
            HookError::Unknown(s, thread_panic)
        } else {
            HookError::Unknown("no panic info", thread_panic)
        }
    }
}
