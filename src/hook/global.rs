//! Global hooks for keyboard and mouse events.
//!
//! The functions in this module are a direct mapping from the functions in libuiohook.
//! These functions provide global control over hooks. The [`hook_start`] and [`hook_stop`]
//! functions will start and stop all hooks.
//! The only way to prevent a hook from running is to unregister it.
//! Generally, it is recommended to use the more idiomatic API provided by [`Hook`].
//!
//! # Implementation Detail
//!
//! Running the hook requires two threads. One is the `hook_thread`,
//! which calls into the native `libuiohook` and blocks while the hook runs.
//! The second thread is the `control thread`, which listens for the native events from
//! the operating system and then distributes those events to all the registered hooks.
//! We need a second thread to listen for events coming from the OS because the callback that runs
//! natively for each event needs to run very fast.
//! Otherwise, some operating systems might discard the event or cause other problems.
//! This way, all that happens in the native callback is passing the event into the control thread,
//! which handles the user callbacks. This module exposes two functions to start the hook:
//!
//! *   [`hook_start`] - This function will spawn both control and hook threads and
//!      return a [`HookHandle`] that can be used to stop or wait for the threads to complete.
//!
//! *   [`hook_start_blocking`] - This function will spawn only the hook threads and use the
//!     current thread as the control thread, meaning it will block until the [`hook_stop`] is called.
//!     Obviously [`hook_stop`] cant be called from the current thread, thus it must be called either from
//!     a different thread (like UI or external event), or it can be called from the [`hook_start`] when some
//!     key or mouse button is pressed.
//!
//! ## Posting Events
//!
//! Posting events is relatively straight forward, for the most part what happens is the user posts
//! the event -> the event is parsed into its native representation -> the event is sent to the OS.
//! There are two exceptions to this simplicity:
//!
//! *   Reserved Events - The operating systems that support this functionality, require the event
//!     to first be dispatched from the operating system, and then the user must change the reserved
//!     flag to 1 from the the OS's event handler. This is problematic because this library intentionally
//!     prevents the user from running code inside the OS's hook thread because running for too long
//!     there might cause undefined behavior. The solution is to provide another callback that the
//!     user can set through [`reserve_events`] which will hopefully force the user to write minial code
//!     inside this callback.
//!
//! *   Synthetic Events - This functionality has no equivalent on the OS level, the way it is implemented
//!     is by doing two things, first all calls to [`post_event`] are synchronized to be in order.
//!     Secondly when posting the event we take event type (which is basically the only information guaranteed
//!     to not change after going through the OS) and store it inside an atomic variable - `SYNTHETIC`.
//!     Later when an event arrives at the OS handler we can check if its value is the same as `SYNTHETIC`
//!     if so we set the event mode to be synthetic nad change `SYNTHETIC` back to 0.
//!     The problem is, although usually the posted event will get to the OS handler in time, there
//!     could be a situation where the user generated the same event type as the one posted at exactly
//!     the right time so it also arrives before the posted event causing it to receive the synthetic mode
//!     instead of the posted event.
//!
//! ## Creating Hooks
//!
//! ```
//! use uiohook_rs::hook::event::{EventKind, HookEvent};
//! use uiohook_rs::hook::global::{hook_start, hook_stop, register_hook};
//! # use std::thread::sleep;
//! # use std::time::Duration;
//!
//! fn on_mouse_click(event: &HookEvent) {
//!     if let EventKind::MouseMoved(data) = &event.kind {
//!         println!("meta: {:?}, data: {:?}", &event.metadata, data)
//!     }
//! }
//!
//! let _id = register_hook(on_mouse_click);
//! let handle = hook_start().expect("oops hook is already running");
//! sleep(Duration::from_millis(5)); // the hook will print every mouse movement during this time.
//! handle.stop();
//! ```
//!
//! ## Removing Hooks
//!
//! ```
//! use uiohook_rs::hook::event::{EventKind, HookEvent};
//! use uiohook_rs::hook::global::{hook_start, hook_stop, register_hook, unregister_hook};
//! # use std::thread::sleep;
//! # use std::time::Duration;
//!
//! fn on_mouse_click(event: &HookEvent) {
//!     if let EventKind::MouseMoved(data) = &event.kind {
//!         println!("meta: {:?}, data: {:?}", &event.metadata, data)
//!     }
//! }
//!
//! let id = register_hook(on_mouse_click);
//! let handle = hook_start().expect("oops hook is already running");
//! sleep(Duration::from_millis(1)); // the hook will work during this time.
//! unregister_hook(id);
//! sleep(Duration::from_millis(1)); // the hook will not work anymore.
//! handle.stop();
//! ```
//!
//! ## Using Start Blocking
//!
//! ```
//! # use uiohook_rs::hook::global::reserve_events;
//! # // prevent these events from effecting the user when running tests
//! # unsafe {
//! #    reserve_events(|e| e.is_synthetic());
//! # }
//! # use std::thread::{self, sleep};
//! # use std::time::Duration;
//! use uiohook_rs::hook::event::{EventKind, HookEvent, MouseButton};
//! use uiohook_rs::hook::global::{hook_start_blocking, hook_stop, post_event, register_hook};
//!
//! fn on_mouse_click(event: &HookEvent) {
//!     match &event.kind {
//!         EventKind::MouseMoved(data) => {
//!             println!("meta: {:?}, data: {:?}", &event.metadata, data)
//!         }
//!         EventKind::MousePressed(_) => hook_stop().unwrap(),
//!         _ => (),
//!     }
//! }
//! let _id = register_hook(on_mouse_click);
//! // Here we programmatically create a mouse click event
//! // that will fire after the hook is started.
//! thread::spawn(|| {
//!     sleep(Duration::from_millis(10));
//!     HookEvent::mouse(MouseButton::Left).pair().post();
//! });
//! // This will block until a mouse click is received.
//! hook_start_blocking().unwrap();
//! ```
//!
//! [`Hook`]: crate::hook::Hook

// we only use DerefMut on windows.
#[allow(unused_imports)]
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use dashmap::DashMap;
use flume::{unbounded, Receiver, Sender};
use once_cell::sync::Lazy;
use parking_lot::{const_mutex, Condvar, Mutex};

use crate::error::{HookError, PostEventError};
use crate::hook::event::{EventKind, EventMetaData, HookEvent};

type HookCallback = Box<dyn Fn(&HookEvent) + Sync + Send>;
type HookFilter = Box<dyn Fn(&HookEvent) -> bool + Sync + Send>;

static RUNNING: AtomicBool = AtomicBool::new(false);
static ENABLED: (Mutex<bool>, Condvar) = (const_mutex(false), Condvar::new());

static EVENT_BUS: Lazy<(Sender<HookEvent>, Receiver<HookEvent>)> = Lazy::new(unbounded);
static HOOKS: Lazy<Arc<DashMap<HookId, HookCallback, ahash::RandomState>>> =
    Lazy::new(|| Arc::new(DashMap::with_hasher(ahash::RandomState::new())));

static RESERVE_CALLBACK: Mutex<Option<HookFilter>> = const_mutex(None);

mod native {
    use std::ffi::CStr;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use ffi::uiohook_event;
    use once_cell::sync::OnceCell;
    use parking_lot::{const_mutex, Mutex};
    use uiohook_sys as ffi;

    use crate::hook::constants::*;
    use crate::hook::event::{
        EventKind, EventMetaData, HookEvent, KeyboardEvent, MouseEvent, MouseWheelEvent,
    };
    use crate::hook::global::RESERVE_CALLBACK;
    use crate::HookError;

    static BASE_TIMESTAMP: OnceCell<u128> = OnceCell::new();
    static SYNTHETIC: AtomicU32 = AtomicU32::new(0);

    fn set_timestamp(metadata: &mut EventMetaData) {
        // libuiohook uses the system uptime as a timestamp, which means that if we shut down the computer,
        // and then run the library again the timestamp could be smaller than one acquired before shutting
        // down the computer. We don't want to make the system call for getting the current time on every call,
        // especially because we already have a timestamp. What we do here is calculate the difference
        // between unix time stamp and the system uptime, giving us the unix timestamp of the system startup
        // and in later calls all we need to do is add the system timestamp to the base time we calculated.
        metadata.time = BASE_TIMESTAMP.get_or_init(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime before unix epoch, what sorcery is this ?")
                .as_millis()
                - metadata.time as u128
        }) + metadata.time as u128;
    }

    fn set_mode(rusty_event: &mut HookEvent, native_event: &mut ffi::uiohook_event) {
        // we only need this mut on windows to possibly change the type in the following if.
        #[allow(unused_mut)]
        let mut event_type = native_event.type_;

        // This is used to make it possible to create synthetic drag events on windows.
        // Because on windows we cant actually create a drag event,
        // we need to do a press -> move -> release when the event sent by the OS will be a drag event
        // but the SYNTHETIC atomic will have the move type.
        // So on windows we do not distinguish between drag and move events in regard to weather
        // they are synthetic.
        #[cfg(target_os = "windows")]
        if event_type == NativeEventKind::EVENT_MOUSE_DRAGGED {
            event_type = NativeEventKind::EVENT_MOUSE_MOVED;
        }

        if SYNTHETIC
            .compare_exchange(event_type as u32, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            rusty_event.metadata.mode.insert(EventMode::SYNTHETIC);
        }

        if let Some(callback) = &*RESERVE_CALLBACK.lock() {
            if callback(rusty_event) {
                rusty_event.metadata.mode.insert(EventMode::RESERVED);
                native_event.reserved = EventMode::RESERVED.bits();
            }
        }
    }

    fn from_native(native: &mut ffi::uiohook_event) -> HookEvent {
        let mut meta = EventMetaData {
            time: native.time as u128,
            mask: native.mask.into(),
            mode: EventMode::from_bits(native.reserved).unwrap_or(EventMode::DEFAULT),
        };

        set_timestamp(&mut meta);

        #[inline(always)]
        fn from_keyboard(native: &mut ffi::uiohook_event) -> KeyboardEvent {
            // SAFETY: we assume that the native library sets the union to the type matching what is found in `native.type_`.
            unsafe { &mut native.data.keyboard }.into()
        }

        #[inline(always)]
        fn from_mouse(native: &mut ffi::uiohook_event) -> MouseEvent {
            // SAFETY: we assume that the native library sets the union to the type matching what is found in `native.type_`.
            unsafe { &mut native.data.mouse }.into()
        }

        #[inline(always)]
        fn from_mouse_wheel(native: &mut ffi::uiohook_event) -> MouseWheelEvent {
            // SAFETY: we assume that the native library sets the union to the type matching what is found in `native.type_`.
            unsafe { &mut native.data.wheel }.into()
        }

        let event_data = match native.type_ {
            NativeEventKind::EVENT_HOOK_ENABLED => EventKind::Enabled,
            NativeEventKind::EVENT_HOOK_DISABLED => EventKind::Disabled,
            NativeEventKind::EVENT_KEY_TYPED => EventKind::KeyTyped(from_keyboard(native)),
            NativeEventKind::EVENT_KEY_PRESSED => EventKind::KeyPressed(from_keyboard(native)),
            NativeEventKind::EVENT_KEY_RELEASED => EventKind::KeyReleased(from_keyboard(native)),
            NativeEventKind::EVENT_MOUSE_CLICKED => EventKind::MouseClicked(from_mouse(native)),
            NativeEventKind::EVENT_MOUSE_PRESSED => EventKind::MousePressed(from_mouse(native)),
            NativeEventKind::EVENT_MOUSE_RELEASED => EventKind::MouseReleased(from_mouse(native)),
            NativeEventKind::EVENT_MOUSE_MOVED => EventKind::MouseMoved(from_mouse(native)),
            NativeEventKind::EVENT_MOUSE_DRAGGED => EventKind::MouseDragged(from_mouse(native)),
            NativeEventKind::EVENT_MOUSE_WHEEL => EventKind::MouseWheel(from_mouse_wheel(native)),
        };

        HookEvent {
            metadata: meta,
            kind: event_data,
        }
    }

    fn into_native(event: HookEvent) -> ffi::uiohook_event {
        let mask = event.metadata.mask;

        let (event_type, event_data) = match event.kind {
            EventKind::Enabled => (
                NativeEventKind::EVENT_HOOK_ENABLED,
                ffi::_uiohook_event__bindgen_ty_1::default(),
            ),
            EventKind::Disabled => (
                NativeEventKind::EVENT_HOOK_DISABLED,
                ffi::_uiohook_event__bindgen_ty_1::default(),
            ),
            EventKind::KeyTyped(event_data) => (
                NativeEventKind::EVENT_KEY_TYPED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    keyboard: event_data.into(),
                },
            ),
            EventKind::KeyPressed(event_data) => (
                NativeEventKind::EVENT_KEY_PRESSED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    keyboard: event_data.into(),
                },
            ),
            EventKind::KeyReleased(event_data) => (
                NativeEventKind::EVENT_KEY_RELEASED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    keyboard: event_data.into(),
                },
            ),
            EventKind::MouseClicked(event_data) => (
                NativeEventKind::EVENT_MOUSE_CLICKED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    mouse: event_data.into(),
                },
            ),
            EventKind::MousePressed(event_data) => (
                NativeEventKind::EVENT_MOUSE_PRESSED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    mouse: event_data.into(),
                },
            ),
            EventKind::MouseReleased(event_data) => (
                NativeEventKind::EVENT_MOUSE_RELEASED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    mouse: event_data.into(),
                },
            ),
            EventKind::MouseMoved(event_data) => (
                NativeEventKind::EVENT_MOUSE_MOVED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    mouse: event_data.into(),
                },
            ),
            EventKind::MouseDragged(event_data) => (
                NativeEventKind::EVENT_MOUSE_DRAGGED,
                ffi::_uiohook_event__bindgen_ty_1 {
                    mouse: event_data.into(),
                },
            ),
            EventKind::MouseWheel(event_data) => (
                NativeEventKind::EVENT_MOUSE_WHEEL,
                ffi::_uiohook_event__bindgen_ty_1 {
                    wheel: event_data.into(),
                },
            ),
        };

        ffi::uiohook_event {
            type_: event_type,
            data: event_data,
            // we dont need to set the meta data here, since it will is ignored when the event is posted,
            // and the OS will create its own meta data.
            time: 0,
            mask: mask.into(),
            reserved: 0,
        }
    }

    extern "C" fn event_handler(event: *mut ffi::uiohook_event) {
        let (sender, _) = &*super::EVENT_BUS;

        // SAFETY: We assume that if the pointer is pointing to a valid uiohook_event,
        // as specified by the native library. Beyond that we only use the original event data
        // in order to construct our own representation, once the HookEvent struct is created it will
        // be used and the original pointer and data will not be read or mutated.
        // This means that the pointer can be safely freed when this function is complete.
        if let Some(mut native_event) = unsafe { event.as_mut() } {
            let mut rusty_event = from_native(native_event);
            set_mode(&mut rusty_event, &mut native_event);

            // We can ignore the send error here because our receiver is static and will not
            // be dropped until the end of the program.
            let _ = sender.send(rusty_event);
        }
    }

    #[cfg(feature = "logging")]
    extern "C" fn logger(level: ffi::log_level, raw_message: *const std::os::raw::c_char) -> bool {
        match unsafe { CStr::from_ptr(raw_message) }.to_str() {
            Ok(log_message) => match level {
                ffi::log_level::LOG_LEVEL_INFO => log::info!("{}", log_message),
                ffi::log_level::LOG_LEVEL_DEBUG => log::debug!("{}", log_message),
                ffi::log_level::LOG_LEVEL_WARN => log::warn!("{}", log_message),
                ffi::log_level::LOG_LEVEL_ERROR => log::error!("{}", log_message)
            },
            Err(_) => return false
        }

        return true;
    }

    #[cfg(feature = "logging")]
    pub fn enable_logging() {
        unsafe { ffi::hook_set_rusty_logger(Some(logger)) }
    }

    pub fn set_event_handler() {
        unsafe { ffi::hook_set_dispatch_proc(Some(event_handler)) }
    }

    pub fn post_event(event: HookEvent) {
        static POST_MUTEX: Mutex<()> = const_mutex(());

        let mut native_event = into_native(event);
        let _guard = POST_MUTEX.lock();
        SYNTHETIC.store(native_event.type_ as u32, Ordering::SeqCst);
        unsafe {
            ffi::hook_post_event(&mut native_event as *mut uiohook_event);
        };
    }

    pub fn hook_start() -> Result<(), HookError> {
        match unsafe { ffi::hook_run() as u32 } {
            ffi::UIOHOOK_SUCCESS => Ok(()),
            status => Err(status.into()),
        }
    }

    pub fn hook_stop() -> Result<(), HookError> {
        match unsafe { ffi::hook_stop() as u32 } {
            ffi::UIOHOOK_SUCCESS => Ok(()),
            status => Err(status.into()),
        }
    }
}

fn control_thread_main() -> JoinHandle<Result<(), HookError>> {
    #[cfg(feature = "logging")]
    native::enable_logging();
    native::set_event_handler();
    let hook_thread = thread::spawn(hook_thread_main);
    let (_, receiver) = &*EVENT_BUS;

    while let Ok(event) = receiver.recv() {
        if let EventKind::Enabled = &event.kind {
            // When we receive the enabled event from the OS we notify the conditional variable so
            // that the start function can complete.
            let (ref lock, ref cond) = ENABLED;
            let mut ready = lock.lock();
            *ready = true;
            cond.notify_all();
        }

        for hook in HOOKS.iter() {
            hook.value()(&event)
        }

        // If the event we received was of the hook being disabled
        // we can stop listening to the hook events.
        // After breaking out of the listening loop the control thread will
        // complete.
        if let EventKind::Disabled = event.kind {
            break;
        }
    }

    RUNNING.store(false, Ordering::SeqCst);
    hook_thread
}

fn hook_thread_main() -> Result<(), HookError> {
    // We need to send the Disabled event here in case the control thread has
    // started the event loop and is waiting for an event. If we don't send a
    // Disabled event the thread will wait indecently.
    if let Err(err) = native::hook_start() {
        let (sender, _) = &*EVENT_BUS;

        // We don't care about the error here because our channel is static, if the receiver is dropped
        // it means that the process is exiting anyway, and the control thread loop will be broken.
        let _ = sender.send(HookEvent {
            metadata: EventMetaData::default(),
            kind: EventKind::Disabled,
        });
        return Err(err);
    }

    Ok(())
}

/// Handle for the control and hook threads, used when starting the hook in non blocking "mode".
///
/// This is simply a wrapper type around [`std::thread::JoinHandle<T>`] that is simpler to use
/// in this case.
pub struct HookHandle {
    handle: JoinHandle<JoinHandle<Result<(), HookError>>>,
}

impl HookHandle {
    fn from_raw(handle: JoinHandle<JoinHandle<Result<(), HookError>>>) -> Self {
        HookHandle { handle }
    }

    /// Wait for both the control and hook threads to complete.
    ///
    /// This method will return on one of the following conditions:
    /// 1. The `control thread` has panicked for some reason. (you get the appropriate [`HookError::Unknown`])
    /// 2. The `hook thread` panicked for some reason. (you get the appropriate [`HookError::Unknown`])
    /// 3. There was an error when starting, or stopping the hook thread. (you get the appropriate [`HookError`])
    /// 4. The hook thread stopped, then the control thread stopped. (you get `Ok(())`)
    ///
    /// Note that if one of the threads panics the panic is included in the HookError,
    /// meaning that it is possible to continue the panicked into the thread that called wait
    /// using [`std::panic::resume_unwind`]
    ///
    /// # Example
    /// ```should_panic
    /// # use uiohook_rs::hook::global::{reserve_events, post_event};
    /// # // prevent these events from effecting the user when running tests
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    ///
    /// use uiohook_rs::hook::event::{EventKind, HookEvent, MouseButton};
    /// use uiohook_rs::hook::global::{hook_start, hook_stop, register_hook};
    /// use uiohook_rs::HookError;
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    ///
    /// fn on_mouse_click(event: &HookEvent) {
    ///     if let EventKind::MouseMoved(data) = &event.kind {
    ///         panic!("ahh!");
    ///     }
    /// }
    ///
    /// let _id = register_hook(on_mouse_click);
    /// let handle = hook_start().expect("oops hook is already running");
    /// sleep(Duration::from_millis(10)); // sleep to give everything time to set be set up.
    ///
    /// // The user moves his mouse...
    /// # HookEvent::mouse(MouseButton::NoButton).moved(10, 10).post();
    ///
    /// // we ignore all other errors and Ok states just to illustrate the resume unwind functionality,
    /// // since we already know the kind of error we get in this contrived example.
    /// if let Err(e) = handle.wait() {
    ///     match e {
    ///         HookError::Unknown(str, panic) => std::panic::resume_unwind(panic),
    ///         _ => (),
    ///     }
    /// }
    /// ```
    pub fn wait(self) -> Result<(), HookError> {
        match self.handle.join() {
            Ok(hook_thread) => hook_thread.join()?,
            Err(control_thread_panic) => Err(control_thread_panic.into()),
        }
    }

    /// Stop hook and wait for the control and hook threads to complete.
    /// This method is similar to calling [`hook_stop`] and then immodestly [`wait`].
    ///
    /// The difference between this method and manually stopping the hook and waiting is that,
    /// this method handles the errors of both [`hook_stop`] and [`wait`] internally.
    ///
    /// ## Manually Stopping
    /// ```rust
    /// use uiohook_rs::hook::event::HookEvent;
    /// use uiohook_rs::hook::global::{hook_start, hook_stop, register_hook};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// # fn some_hook(event: &HookEvent) {}
    ///
    /// let _id = register_hook(some_hook);
    /// let handle = hook_start().expect("oops hook is already running");
    /// sleep(Duration::from_millis(10));
    ///
    /// // we need to match on hook_stop
    /// // because it might fail before stopping the hook thread
    /// // in which case we do *not* want to wait for the handle.
    /// let res = match hook_stop() {
    ///     Ok(_) => handle.wait(),
    ///     Err(err) => Err(err),
    /// };
    /// ```
    /// ## Using Stop Method
    /// ```rust
    /// use uiohook_rs::hook::event::HookEvent;
    /// use uiohook_rs::hook::global::{hook_start, hook_stop, register_hook};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// # fn some_hook(event: &HookEvent) {}
    ///
    /// let _id = register_hook(some_hook);
    /// let handle = hook_start().expect("oops hook is already running");
    /// sleep(Duration::from_millis(10));
    ///
    /// // stop handles the match for us and returns a single Result.
    /// let res = handle.stop();
    /// ```
    ///
    /// [`wait`]: HookHandle::wait
    pub fn stop(self) -> Result<(), HookError> {
        match hook_stop() {
            Ok(_) => self.wait(),
            Err(err) => Err(err),
        }
    }
}

/// Starts listening for user events in a non blocking fashion, this function will spawn two
/// additional threads in order to handle user events and activate callbacks. More in depth explanation
/// is available at the module level documentation.
///
/// This function will return `Ok(HookHandle)` if this process did not call it already, otherwise
/// `None` will be returned.
pub fn hook_start() -> Option<HookHandle> {
    match RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            let control_thread = thread::spawn(control_thread_main);
            // After spawning the control thread, we dont want to return immediately because
            // the user could attempt to post an event before the control and hook threads were properly
            // initialized. We use the this condvar to wait until the control thread notifies us that
            // it is initialized.
            let (ref lock, ref cond) = ENABLED;
            let mut ready = lock.lock();
            if !*ready {
                cond.wait(&mut ready);
            }
            // We set ready back to false so that if `hook_stop` is called later and than this function
            // is called again we wont skip the wait on the condvar.
            *ready = false;

            Some(HookHandle::from_raw(control_thread))
        }
        Err(_) => None,
    }
}

/// Similar to hook start only it is blocking and spawns just one additional thread. See the module
/// level documentation for a better comparison.
///
/// If the hook has already been started, either by [`hook_start`] or this function from another thread
/// this function will return immediately with an `Ok(())` value.
///
/// Note that because this function is blocking you must call [`hook_stop`] from one of the
/// registered hooks, or another thread. Otherwise the hook thread will run indefinitely and this function
/// will never return.
pub fn hook_start_blocking() -> Result<(), HookError> {
    match RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => control_thread_main().join()?,
        Err(_) => Ok(()),
    }
}

/// This function attempts to stop the `hook_thread`, returning an error if unsuccessful.
///
/// If the function succeeds in stopping the `hook_thread` the `control_thread` will finish
/// quickly after. Use [`HookHandle::wait`] to block until the `control_thread` finishes as well.
/// It is generally easier to use the [`HookHandle::stop`], see its documentation for further comparison.
pub fn hook_stop() -> Result<(), HookError> {
    native::hook_stop()
}

/// HookId is a convenience type to represent the id of a hook.
///
/// [`register_hook`] returns this type and [`unregister_hook`] accepts it
/// to identify which hook should be unregistered.
pub type HookId = u128;

/// Register a hook handler that will be called each time the user creates a keyboard or mouse event.
/// This function only allows to register `global` hooks meaning the hook will be called on every user event.
/// One can of course match on the event type and only do work for some of the event types, but this
/// functionality is already provided by [`Hook`] which makes it easy to create more specialized
/// hooks.
///
/// The function returns a [`HookId`], that can be later used to unregister the hook.
///
/// [`Hook`]: crate::hook::Hook
pub fn register_hook<F: Fn(&HookEvent) + Sync + Send + 'static>(handler: F) -> HookId {
    register_boxed_hook(Box::new(handler))
}

pub(crate) fn register_boxed_hook(
    handler: Box<dyn Fn(&HookEvent) + Sync + Send + 'static>,
) -> HookId {
    static HOOK_ID: Mutex<u128> = const_mutex(0u128);

    // This is basically the `fetch_add`, only rust doest have
    // 128 bit atomic types on stable, so we use a mutex instead.
    let guard = &mut *HOOK_ID.lock();

    let id = {
        // We use wrapping add to guarantee that there is no overflow panic.
        // This code might theoretically be erroneous if we manage to overflow the
        // hook id and the hook id 0 is still in the hashmap, but that would require
        // calling this function 2^128 times which is practically impossible.
        let new_id = guard.wrapping_add(1);
        *guard = new_id;
        new_id
    };

    HOOKS.insert(id, handler);
    id
}

pub(crate) fn register_boxed_hook_with_id(
    id: HookId,
    handler: Box<dyn Fn(&HookEvent) + Sync + Send + 'static>,
) {
    HOOKS.insert(id, handler);
}

/// Unregister a hook handler, this will remove the handler corresponding to the [`HookId`],
/// and this handler will not be called anymore when new events arrive.
///
/// If the provided [`HookId`] does not correspond to a registered hook this function will return
///None, otherwise the unregistered hook will be returned.
pub fn unregister_hook(hook_id: HookId) -> Option<HookCallback> {
    HOOKS.remove(&hook_id).map(|(_, callback)| callback)
}

/// Exactly the same as [`unregister_hook`] except this function does not return anything,
/// if the [`HookId`] is valid the hook is dropped, otherwise nothing happens.
pub fn drop_hook(hook_id: HookId) {
    HOOKS.remove(&hook_id);
}

pub(crate) fn postable_event(event: &HookEvent) -> Result<(), PostEventError> {
    match &event.kind {
        EventKind::Enabled => Err(PostEventError("Enabled".into())),
        EventKind::Disabled => Err(PostEventError("Disabled".into())),
        _ => Ok(()),
    }
}

/// Post a [`HookEvent`], this will simulate the user creating the same event through the use of
/// the mouse and keyboard.
///
/// This function will return a `PostEventError` if the caller attempts to post
/// an [`Enabled`] or [`Disabled`] event. These events cant be posted as they are control events
/// internal to the library, in order to enable and disable the hook use the [`hook_start`] and
/// [`hook_stop`] API's respectfully.
///
/// See the [`HookEvent`] documentation for examples of how to create events.
///
/// [`Enabled`]: EventKind::Enabled
/// [`Disabled`]: EventKind::Disabled
pub fn post_event(event: HookEvent) -> Result<(), PostEventError> {
    let res = postable_event(&event);
    if res.is_ok() {
        native::post_event(event);
    }
    res
}

/// This function allows the caller to prevent some events from being propagated into userspace.
///
/// The function accepts a callback(filter) `Fn(&HookEvent) -> bool` accepting a hook event and and returning
/// weather or not it should be **reserved** meaning that if the function returns true the event
/// **wont** be propagated to userspace. The callback is called on every event that is sent to the
/// control thread right before it is sent.
///
/// Every call to this function **overwrites** the filter.
///
/// Unfortunately, support for this functionality is only available on Windows and macOS unfortunately.
/// For more information, see this issue from the native library discussing this: <https://github.com/kwhat/libuiohook/issues/57>.
///
/// # Safety
/// This function is marked unsafe not because it validates rust's memory safety guarantees, but because
/// it is unstable in its promise. First of all it is not supported on all platforms, only windows
/// and macOS. Secondly because it requires the filter closure to run inside the OS thread handling the
/// event, something like the closure running too long might cause the OS to drop the event
/// and stop executing the callback, meaning side effects of this function might not always happen consistently.
/// Finally the mechanism of preventing propagation is simply setting a reserved field to true, which means the
/// some processes could ignore this field and still use the event, or some process might get the event
/// before the reserved field is set.
///
/// # Example:
/// ```rust
/// use uiohook_rs::hook::event::EventMode;
/// use uiohook_rs::hook::global::{register_hook, reserve_events};
/// use uiohook_rs::hook_start;
///
/// hook_start().expect("oops already running..");
///
/// // while the hook is active all user events will be reserved, which means that the user
/// // will not be able to interact with the UI of the system for this duration.
/// reserve_events(|_| true);
///
/// register_hook(|e| assert!(e.metadata.mode.contains(EventMode::RESERVED)));
/// ```
#[cfg_attr(rustdoc, doc(cfg(any(target_os = "windows", target_os = "macos"))))]
#[cfg(any(rustdoc, target_os = "windows", target_os = "macos"))]
pub fn reserve_events<F: Fn(&HookEvent) -> bool + Sync + Send + 'static>(filter: F) {
    std::mem::swap(
        RESERVE_CALLBACK.lock().deref_mut(),
        &mut Some(Box::new(filter)),
    )
}

// we define an empty reserve_events function when in test mode to allow the tests
// to be cross platform with their use of reserve_events though obviously when running the
// tests on linux the events will not be reserved and you probably should run them in a headless
// container with something like xvfb
#[cfg(all(test, target_os = "linux"))]
#[doc(hidden)]
pub fn reserve_events<F: Fn(&HookEvent) -> bool + Sync + Send + 'static>(filter: F) {
    ()
}