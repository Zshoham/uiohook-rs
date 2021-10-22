use std::collections::HashSet;
use std::mem;

use once_cell::sync::Lazy;
use strum::IntoEnumIterator;

use crate::hook::event::{
    EventKind, EventMetaData, HookEvent, Key, KeyboardEvent, MouseButton, MouseEvent,
    MouseWheelEvent,
};
use crate::hook::global::HookId;

mod constants;

pub mod event;
pub mod global;

static KEY_SET: Lazy<HashSet<Key, ahash::RandomState>> = Lazy::new(|| Key::iter().collect());
static MOUSE_BUTTON_SET: Lazy<HashSet<MouseButton, ahash::RandomState>> =
    Lazy::new(|| MouseButton::iter().collect());

impl HookEvent {
    fn as_keyboard(&self) -> Option<(&EventMetaData, &KeyboardEvent)> {
        match &self.kind {
            EventKind::KeyPressed(data) => Some((&self.metadata, data)),
            EventKind::KeyReleased(data) => Some((&self.metadata, data)),
            EventKind::KeyTyped(data) => Some((&self.metadata, data)),
            _ => None,
        }
    }

    fn as_mouse(&self) -> Option<(&EventMetaData, &MouseEvent)> {
        match &self.kind {
            EventKind::MouseClicked(data) => Some((&self.metadata, data)),
            EventKind::MousePressed(data) => Some((&self.metadata, data)),
            EventKind::MouseReleased(data) => Some((&self.metadata, data)),
            EventKind::MouseMoved(data) => Some((&self.metadata, data)),
            EventKind::MouseDragged(data) => Some((&self.metadata, data)),
            _ => None,
        }
    }

    fn as_mouse_button(&self) -> Option<(&EventMetaData, &MouseEvent)> {
        match &self.kind {
            EventKind::MouseClicked(data) => Some((&self.metadata, data)),
            EventKind::MousePressed(data) => Some((&self.metadata, data)),
            EventKind::MouseReleased(data) => Some((&self.metadata, data)),
            _ => None,
        }
    }
}

/// This macro is meant to simplify creating global hooks using nicer syntax.
///
/// The macro uses the [`Hook`] struct and all the innovations return a [`Hook`].
/// The macro has 5 invocations to control how the hook is created.
///
/// * `($callback:expr)` - calling the macro with just a callback will create a hook equivalent
/// to calling [`Hook::new`] with the callback accepting all incoming events.
/// ```rust
/// # use uiohook_rs::{hook, HookEvent};
/// let h = hook!(|event: &HookEvent| println!("{:?}", event));
/// ```
///
/// * `(any($($event_kind:expr),+), $callback:expr)` - here we can call the macro specifying on which
/// event kinds we want the hook to be called.
#[macro_export]
macro_rules! hook {
    ($callback:expr) => { {
        let mut h = $crate::hook::Hook::new($callback);
        h.register();
        h
    } };
    (any($($event_kind:expr),+), $callback:expr) => { {
        let mut h = $crate::hook::Hook {
            hook: |event: &$crate::hook::event::HookEvent| {
                let cb = $callback;
                match &event.kind() {
                    $crate::hook::event::EventKind::Enabled(_) => (),
                    $crate::hook::event::EventKind::Disabled(_) => (),
                    $( $event_kind(_, _) => cb(&event)),+
                    _ => ()
                }
            }
            id: None
        };
        h.register()
        h
    } };
    ($event_kind:expr, $callback:expr) => { {
        let mut h = $crate::hook::Hook {
            hook: |event: &$crate::hook::event::HookEvent| {
                let cb = $callback;
                match event {
                    $crate::hook::event::HookEvent::Enabled(_) => (),
                    $crate::hook::event::HookEvent::Disabled(_) => (),
                    $event_kind(_, _) => cb(&event),
                    _ => ()
                }
            }
            id: None
        };
        h.register()
        h
    } };
    (none($($event_kind:expr),+), $callback:expr) => { {
        let mut h = Hook {
            hook: |event: &$crate::hook::event::HookEvent| {
                let cb = $callback;
                match event {
                    $crate::hook::event::HookEvent::Enabled(_) => (),
                    $crate::hook::event::HookEvent::Disabled(_) => (),
                    $( $event_kind => ()),+
                    _ => cb(&event)
                }
            }
            id: None
        };
        h.register();
        h
    } };
    (!$event_kind:expr, $callback:expr) => { {
        let mut h = $crate::hook::event::Hook {
            hook: |event: &$crate::hook::event::HookEvent| {
                let cb = $callback;
                match event {
                    $crate::hook::event::HookEvent::Enabled(_) => (),
                    $crate::hook::event::HookEvent::Disabled(_) => (),
                    $event_kind => (),
                    _ => cb(&event)
                }
            }
            id: None
        };
        h.register();
        h
    } };
}

/// This macro is meant to simplify creating keyboard hooks using nicer syntax.
///
/// The macro wraps functionality provided by [`Hook`]'s various constructors to make
/// them less verbose. Note that the macro returns a registered hook object from each invocation,
/// and that object should not be discarded, as when it goes out of scope it will unregister the hook.
///
/// We have a few ways of calling this macro in order to control what kind of keyboard hook will
/// be created:
///
/// * `($callback:expr)` - will create a generic keyboard hook, that will listen to any keyboard event.
/// and is equivalent to calling [`Hook::keyboard`].
/// ```rust
/// # use uiohook_rs::{EventMetaData, keyboard, hook::event::KeyboardEvent};
/// let h =
///     keyboard!(|meta: &EventMetaData, data: &KeyboardEvent| println!("{:?}, {:?}", meta, data));
/// ```
///
/// * `(any($($key:expr),+), $callback:expr)` - will create a keyboard hook that will only be called
/// if *any* of the specified keys are affected by the event.
/// ```rust
/// # use uiohook_rs::{EventMetaData, keyboard, hook::event::{KeyboardEvent, Key}};
/// let h = keyboard!(
///     any(Key::L, Key::R),
///     |meta: &EventMetaData, data: &KeyboardEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
/// Note that if you wish to hook only on a single key you can use `any(Key::{SomeKey})` but you can also
/// just remove the `any` and write something like this:
/// ```rust
/// # use uiohook_rs::{EventMetaData, keyboard, hook::event::{KeyboardEvent, Key}};
/// let h = keyboard!(Key::L, |meta: &EventMetaData, data: &KeyboardEvent| {
///     println!("{:?}, {:?}", meta, data)
/// });
/// ```
///
/// * `(none($($key:expr),+), $callback:expr)` - will create a keyboard hook that will only be called
/// if *none* of the specified keys are affected by the event.
/// ```rust
/// # use uiohook_rs::{EventMetaData, keyboard, hook::event::{KeyboardEvent, Key}};
/// let h = keyboard!(
///     none(Key::L, Key::R),
///     |meta: &EventMetaData, data: &KeyboardEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
/// Note that if you wish to hook on all keys except one you can use `none(Key::{SomeKey})` but you can also
/// this simplified syntax:
/// ```rust
/// # use uiohook_rs::{EventMetaData, keyboard, hook::event::{KeyboardEvent, Key}};
/// let h = keyboard!(
///     !Key::L,
///     |meta: &EventMetaData, data: &KeyboardEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
#[macro_export]
macro_rules! keyboard {
    ($callback:expr) => { {
        let mut h = $crate::hook::Hook::keyboard($callback);
        h.register();
        h
    } };
    (any($($key:expr),+), $callback:expr) => { {
        let mut h = $crate::hook::Hook::keys($crate::hook::HookOn::OneOf([$($key),+]), $callback);
        h.register();
        h
    } };
    (none($($key:expr),+), $callback:expr) => { {
        let mut h = $crate::hook::Hook::keys($crate::hook::HookOn::NoneOf([$($key),+]), $callback);
        h.register();
        h
    } };
    (! $key:expr, $callback:expr) => { {
        let mut h = $crate::hook::Hook::keys($crate::hook::HookOn::NoneOf([$key]), $callback);
        h.register();
        h
    } };
    ($key:expr, $callback:expr) => { {
        let mut h = $crate::hook::Hook::keys($crate::hook::HookOn::OneOf([$key]), $callback);
        h.register();
        h
    } };
}

/// This macro is meant to simplify creating mouse hooks using nicer syntax.
///
/// The macro wraps functionality provided by [`Hook`]'s various constructors to make
/// them less verbose. Note that the macro returns a registered hook object from each invocation,
/// and that object should not be discarded, as when it goes out of scope it will unregister the hook.
///
/// We have a few ways of calling this macro in order to control what kind of mouse hook will
/// be created:
///
/// * `($callback:expr)` - will create a generic mouse hook, that will listen to any keyboard event.
/// and is equivalent to calling [`Hook::mouse`].
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse, hook::event::MouseEvent};
/// let h = mouse!(|meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data));
/// ```
///
/// * `(any($($key:expr),+), $callback:expr)` - will create a mouse hook that will only be called
/// if *any* of the specified buttons are affected by the event.
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse, hook::event::{MouseEvent, MouseButton}};
/// let h = mouse!(
///     any(MouseButton::Left, MouseButton::Middle),
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
/// Note that if you wish to hook only on a single button you can use `any(MouseButton::{SomeButton})` but you can also
/// just remove the `any` and write something like this:
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse, hook::event::{MouseEvent, MouseButton}};
/// let h = mouse!(
///     MouseButton::Left,
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
///
/// * `(none($($key:expr),+), $callback:expr)` - will create a mouse hook that will only be called
/// if *none* of the specified keys are affected by the event.
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse, hook::event::{MouseEvent, MouseButton}};
/// let h = mouse!(
///     none(MouseButton::Left, MouseButton::Middle),
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
/// Note that if you wish to hook on all buttons except one you can use `none(MouseButton::{SomeButton})` but you can also
/// this simplified syntax:
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse, hook::event::{MouseEvent, MouseButton}};
/// let h = mouse!(
///     !MouseButton::Left,
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
#[macro_export]
macro_rules! mouse {
    ($callback:expr) => { {
        let mut h = $crate::hook::Hook::mouse($callback);
        h.register();
        h
    } };
    (any($($key:expr),+), $callback:expr) => { {
        let mut h = $crate::hook::Hook::mouse_buttons($crate::hook::HookOn::OneOf([$($key),+]), $callback);
        h.register();
        h
    } };
    (none($($key:expr),+), $callback:expr) => { {
        let mut h = $crate::hook::Hook::mouse_buttons($crate::hook::HookOn::NoneOf([$($key),+]), $callback);
        h.register();
        h
    } };
    (! $key:expr, $callback:expr) => { {
        let mut h = $crate::hook::Hook::mouse_buttons($crate::hook::HookOn::NoneOf([$key]), $callback);
        h.register();
        h
    } };
    ($key:expr, $callback:expr) => { {
        let mut h = $crate::hook::Hook::mouse_buttons($crate::hook::HookOn::OneOf([$key]), $callback);
        h.register();
        h
    } };
}

/// This macro is meant to simplify creating mouse drag hooks using nicer syntax.
///
/// The macro wraps functionality provided by [`Hook`]'s various constructors to make
/// them less verbose. Note that the macro returns a registered hook object from each invocation,
/// and that object should not be discarded, as when it goes out of scope it will unregister the hook.
///
/// We have a few ways of calling this macro in order to control what kind of hook will
/// be created:
///
/// * `($callback:expr)` - will create a generic mouse hook, that will listen to any keyboard event.
/// and is equivalent to calling [`Hook::mouse_drag`].
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse_drag, hook::event::MouseEvent};
/// let h =
///     mouse_drag!(|meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data));
/// ```
///
/// * `(any($($key:expr),+), $callback:expr)` - will create a mouse hook that will only be called
/// if *any* of the specified buttons are affected by the event.
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse_drag, hook::event::{MouseEvent, MouseButton}};
/// # #[cfg(any(target_os = "linux", target_os = "macos"))]
/// let h = mouse_drag!(
///     any(MouseButton::Left, MouseButton::Middle),
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
/// Note that if you wish to hook only on a single button you can use `any(MouseButton::{SomeButton})` but you can also
/// just remove the `any` and write something like this:
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse_drag, hook::event::{MouseEvent, MouseButton}};
/// # #[cfg(any(target_os = "linux", target_os = "macos"))]
/// let h = mouse_drag!(
///     MouseButton::Left,
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
///
/// * `(none($($key:expr),+), $callback:expr)` - will create a mouse hook that will only be called
/// if *none* of the specified keys are affected by the event.
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse_drag, hook::event::{MouseEvent, MouseButton}};
/// # #[cfg(any(target_os = "linux", target_os = "macos"))]
/// let h = mouse_drag!(
///     none(MouseButton::Left, MouseButton::Middle),
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
/// Note that if you wish to hook on all buttons except one you can use `none(MouseButton::{SomeButton})` but you can also
/// this simplified syntax:
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse_drag, hook::event::{MouseEvent, MouseButton}};
/// # #[cfg(any(target_os = "linux", target_os = "macos"))]
/// let h = mouse_drag!(
///     !MouseButton::Left,
///     |meta: &EventMetaData, data: &MouseEvent| println!("{:?}, {:?}", meta, data)
/// );
/// ```
#[macro_export]
macro_rules! mouse_drag {
    ($callback:expr) => { {
        let mut h = $crate::hook::Hook::mouse_drag($callback);
        h.register();
        h
    } };
    (any($($key:expr),+), $callback:expr) => { {
        #[cfg(any(target_os = "linux", target_os = "macos"))] {
            let mut h = $crate::hook::Hook::mouse_drag_buttons($crate::hook::HookOn::OneOf([$($key),+]), $callback);
            h.register();
            h
        }

        #[cfg(target_os = "windows")] {
            compile_error!("cant hook on mouse drag buttons on windows, see the `Hook::mouse_drag_button` function documentation");
        }
    } };
    (none($($key:expr),+), $callback:expr) => { {
        #[cfg(any(target_os = "linux", target_os = "macos"))] {
            let mut h = $crate::hook::Hook::mouse_drag_buttons($crate::hook::HookOn::NoneOf([$($key),+], $callback));
            h.register();
            h
        }

        #[cfg(target_os = "windows")] {
            compile_error!("cant hook on mouse drag buttons on windows, see the `Hook::mouse_drag_button` function documentation");
        }
    } };
    (!$key:expr, $callback:expr) => { {
        #[cfg(any(target_os = "linux", target_os = "macos"))] {
            let mut h = $crate::hook::Hook::mouse_drag_buttons($crate::hook::HookOn::NoneOf([$key], $callback));
            h.register();
            h
        }

        #[cfg(target_os = "windows")] {
            compile_error!("cant hook on mouse drag buttons on windows, see the `Hook::mouse_drag_button` function documentation");
        }
    } };
    ($key:expr, $callback:expr) => { {
        #[cfg(any(target_os = "linux", target_os = "macos"))] {
            let mut h = $crate::hook::Hook::mouse_drag_buttons($crate::hook::HookOn::OneOf([$key]), $callback);
            h.register();
            h
        }

        #[cfg(target_os = "windows")] {
            compile_error!("cant hook on mouse drag buttons on windows, see the `Hook::mouse_drag_button` function documentation");
        }
    } };
}

/// This macro is meant to simplify creating mouse wheel hooks using nicer syntax.
///
/// The macro wraps [`Hook::mouse_wheel`], and apart from returning an already registered hook doest,
/// add anything more to the function call, and exists more to provide a uniform API for creating
/// all hooks.
///
/// # Example
/// ```rust
/// # use uiohook_rs::{EventMetaData, mouse_wheel, hook::event::MouseWheelEvent};
/// let h = mouse_wheel!(|meta: &EventMetaData, data: &MouseWheelEvent| println!(
///     "{:?}, {:?}",
///     meta, data
/// ));
/// ````
#[macro_export]
macro_rules! mouse_wheel {
    ($callback:expr) => {{
        let mut h = $crate::hook::Hook::mouse_wheel($callback);
        h.register();
        h
    }};
}

/// Utility structs that helps express when the hook should be activated.
///
/// # Example
/// ```rust
/// use uiohook_rs::hook::event::{Key, MouseButton};
/// use uiohook_rs::hook::HookOn;
/// use uiohook_rs::Hook;
///
/// // here we create a hook that will be triggered when a keyboard event with
/// // *one of* the keys A or B is received.
/// let _ = Hook::keys(HookOn::OneOf([Key::A, Key::B]), |_, _| {
///     println!("got A or B")
/// });
///
/// // here we create a hook that will be triggered when one of the left. right or middle
/// // mouse buttons are pressed/released but not when one of the extra buttons is changed.
/// let _ = Hook::mouse_buttons(
///     HookOn::NoneOf([MouseButton::Extra1, MouseButton::Extra2]),
///     |_, _| println!("pressed some mouse button !"),
/// );
/// ````
pub enum HookOn<I: IntoIterator> {
    OneOf(I),
    NoneOf(I),
}

/// A hook handle that manages the internal state of the hook and the callback
/// to make it easier to create more specialized hooks.
///
/// The basic mechanism to create hooks in the library is [`register_hook`], though this method
/// of listening to events is crude because the events received need to be checked for their type
/// in order to achieve usefully functionality. This struct provides a nicer interface that
/// can be asked to listen to a specific key and will handle the forwarding only the appropriate events
/// to the provided callback.
///
/// Additionally this structs makes it easier to enable and disable the hook as it uses RAII instead
/// of working with the [`HookId`] provided by [`register_hook`].
///
/// [`register_hook`]: crate::hook::global::register_hook
/// [`HookId`]: crate::hook::global::HookId
///
/// # Example
/// ```rust
/// use uiohook_rs::hook::event::{Key, KeyboardEvent};
/// use uiohook_rs::hook::global::register_hook;
/// use uiohook_rs::hook::HookOn;
/// use uiohook_rs::{EventKind, EventMetaData, Hook, HookEvent};
///
/// // creating a hook that does something when either the C or V key is pressed
/// // on the keyboard.
/// register_hook(|event: &HookEvent| match &event.kind {
///     EventKind::KeyPressed(data) => match data.keycode {
///         Key::V | Key::C => println!("user pressed C or V"),
///         _ => (),
///     },
///     _ => (),
/// });
///
/// // here we can see how simple it is to specify what exactly the hook
/// // should be doing and the callback receives the appropriate data and meta data.
/// Hook::keys(
///     HookOn::OneOf([Key::C, Key::V]),
///     |meta: &EventMetaData, data: &KeyboardEvent| {
///         println!("user pressed C or V: {:?}, {:?}", meta, data)
///     },
/// );
/// ```
pub struct Hook {
    hook: Option<Box<dyn Fn(&HookEvent) + Sync + Send + 'static>>,
    id: Option<HookId>,
}

impl Hook {
    /// Create a hook that will listen to all events.
    ///
    /// This way of constructing a Hook adds no functionality over [`global::register_hook`],
    /// though it is still useful to have nicer control of the hook through the [`register`] and
    /// [`unregister`] methods.
    ///
    /// [`register`]: crate::hook::Hook::register
    /// [`unregister`]: crate::hook::Hook::unregister
    pub fn new<C>(callback: C) -> Hook
    where
        C: Fn(&HookEvent) + Sync + Send + 'static,
    {
        Hook {
            hook: Some(Box::new(callback)),
            id: None,
        }
    }

    /// Create a hook that will listen to all keyboard events.
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # reserve_events(|e| e.is_synthetic());
    /// # use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{
    ///     EventIterator, EventMask, Key, MouseButton, MouseScrollDirection, MouseScrollKind,
    /// };
    /// use uiohook_rs::hook::HookOn;
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static KEYBOARD: AtomicU8 = AtomicU8::new(0);
    ///
    /// let mut store_keyboard = Hook::keyboard(|_, _| {
    ///     KEYBOARD.fetch_add(1, Ordering::SeqCst);
    /// });
    ///
    /// store_keyboard.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we one left button and one right button event.
    /// for key in [Key::L, Key::F] {
    ///     HookEvent::keyboard(key).pair().post();
    /// }
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that both events were caught by `store_keyboard`
    /// // note that we have 4 events because we are catching both press and release.
    /// assert_eq!(KEYBOARD.load(Ordering::SeqCst), 4);
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn keyboard<C>(callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &KeyboardEvent) + Sync + Send + 'static,
    {
        let hook = move |event: &HookEvent| {
            if let Some((meta, data)) = event.as_keyboard() {
                callback(meta, data);
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Create a hook that will listen to all keyboard events
    /// but only activate for the keys specified by `keys`.
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # reserve_events(|e| e.is_synthetic());
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{
    ///     EventIterator, EventMask, Key, MouseButton, MouseScrollDirection, MouseScrollKind,
    /// };
    /// use uiohook_rs::hook::HookOn;
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static KEY_K: AtomicBool = AtomicBool::new(false);
    /// static KEY_L: AtomicBool = AtomicBool::new(false);
    ///
    /// let mut store_k = Hook::keys(HookOn::OneOf([Key::K]), |_, _| {
    ///     KEY_K.store(true, Ordering::SeqCst)
    /// });
    ///
    /// let mut store_l = Hook::keys(HookOn::OneOf([Key::L]), |_, _| {
    ///     KEY_L.store(true, Ordering::SeqCst)
    /// });
    ///
    /// store_k.register();
    /// store_l.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we create a mouse wheel event, and post it.
    /// HookEvent::keyboard(Key::L).pair().post();
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that only the `store_k` callback was called.
    /// assert!(!KEY_K.load(Ordering::SeqCst));
    /// assert!(KEY_L.load(Ordering::SeqCst));
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn keys<C, I>(keys: HookOn<I>, callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &KeyboardEvent) + Sync + Send + 'static,
        I: IntoIterator<Item = Key>,
    {
        let key_set: HashSet<Key, ahash::RandomState> = match keys {
            HookOn::OneOf(iter) => IntoIterator::into_iter(iter).collect(),
            HookOn::NoneOf(iter) => {
                let input_set = IntoIterator::into_iter(iter).collect();
                KEY_SET.difference(&input_set).cloned().collect()
            }
        };

        let hook = move |event: &HookEvent| {
            if let Some((meta, data)) = event.as_keyboard() {
                if key_set.contains(&data.keycode) {
                    callback(meta, data);
                }
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Create a hook that will listen to all mouse events.
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # reserve_events(|e| e.is_synthetic());
    /// # use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{
    ///     EventIterator, EventMask, MouseButton, MouseScrollDirection, MouseScrollKind,
    /// };
    /// use uiohook_rs::hook::HookOn;
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static MOUSE: AtomicU8 = AtomicU8::new(0);
    ///
    /// let mut store_mouse = Hook::mouse(|_, _| {
    ///     MOUSE.fetch_add(1, Ordering::SeqCst);
    /// });
    ///
    /// store_mouse.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we one left button and one right button event.
    /// for btn in [MouseButton::Left, MouseButton::Right] {
    ///     HookEvent::mouse(btn).pair().post();
    /// }
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that both events were caught by `store_mouse`
    /// // note that we have 4 events because we are catching both press and release.
    /// assert_eq!(MOUSE.load(Ordering::SeqCst), 4);
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn mouse<C>(callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &MouseEvent) + Sync + Send + 'static,
    {
        let hook = move |event: &HookEvent| {
            if let Some((meta, data)) = event.as_mouse() {
                callback(meta, data);
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Create a hook that will listen to all mouse events
    /// but only activate for the buttons specified by `buttons`.
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # reserve_events(|e| e.is_synthetic());
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{
    ///     EventIterator, EventMask, MouseButton, MouseScrollDirection, MouseScrollKind,
    /// };
    /// use uiohook_rs::hook::HookOn;
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static BUTTON_LEFT: AtomicBool = AtomicBool::new(false);
    /// static BUTTON_RIGHT: AtomicBool = AtomicBool::new(false);
    ///
    /// let mut store_left = Hook::mouse_buttons(HookOn::OneOf([MouseButton::Left]), |_, _| {
    ///     BUTTON_LEFT.store(true, Ordering::SeqCst)
    /// });
    ///
    /// let mut store_right = Hook::mouse_buttons(HookOn::OneOf([MouseButton::Right]), |_, _| {
    ///     BUTTON_RIGHT.store(true, Ordering::SeqCst)
    /// });
    ///
    /// store_left.register();
    /// store_right.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we create a mouse wheel event, and post it.
    /// HookEvent::mouse(MouseButton::Left).pair().post();
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that only the `store_left` callback was called.
    /// assert!(BUTTON_LEFT.load(Ordering::SeqCst));
    /// assert!(!BUTTON_RIGHT.load(Ordering::SeqCst));
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn mouse_buttons<C, I>(buttons: HookOn<I>, callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &MouseEvent) + Sync + Send + 'static,
        I: IntoIterator<Item = MouseButton>,
    {
        let button_set: HashSet<MouseButton, ahash::RandomState> = match buttons {
            HookOn::OneOf(iter) => IntoIterator::into_iter(iter).collect(),
            HookOn::NoneOf(iter) => {
                let input_set = IntoIterator::into_iter(iter).collect();
                MOUSE_BUTTON_SET.difference(&input_set).cloned().collect()
            }
        };

        let hook = move |event: &HookEvent| {
            if let Some((meta, data)) = event.as_mouse_button() {
                if button_set.contains(&data.button) {
                    callback(meta, data);
                }
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Create a hook that will only listen to [`MouseMoved`] events.
    ///
    /// [`MouseMoved`]: crate::hook::event::EventKind::MouseMoved
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # reserve_events(|e| e.is_synthetic());
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{
    ///     EventIterator, EventMask, MouseButton, MouseScrollDirection, MouseScrollKind,
    /// };
    /// use uiohook_rs::hook::HookOn;
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static MOVED: AtomicBool = AtomicBool::new(false);
    ///
    /// let mut store_moved = Hook::mouse_move(|_, _| MOVED.store(true, Ordering::SeqCst));
    ///
    /// store_moved.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we create a mouse wheel event, and post it.
    /// HookEvent::mouse(MouseButton::Left).moved(10, 10).post();
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that only the `store_moved` callback was called.
    /// assert!(MOVED.load(Ordering::SeqCst));
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn mouse_move<C>(callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &MouseEvent) + Sync + Send + 'static,
    {
        let hook = move |event: &HookEvent| {
            if let EventKind::MouseMoved(data) = &event.kind {
                callback(&event.metadata, data);
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Create a hook that will listen to all [`MouseDragged`] events.
    ///
    /// [`MouseDragged`]: crate::hook::event::EventKind::MouseDragged
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # reserve_events(|e| e.is_synthetic());
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{
    ///     EventIterator, EventMask, MouseButton, MouseScrollDirection, MouseScrollKind,
    /// };
    /// use uiohook_rs::hook::HookOn;
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static DRAGGED: AtomicBool = AtomicBool::new(false);
    ///
    /// let mut store_dragged = Hook::mouse_drag(|_, _| DRAGGED.store(true, Ordering::SeqCst));
    ///
    /// store_dragged.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we create a mouse wheel event, and post it.
    /// HookEvent::mouse(MouseButton::Left)
    ///     .with_mask(EventMask::LeftMouseButton)
    ///     .dragged(10, 10)
    ///     .post();
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that only the `store_dragged` callback was called.
    /// assert!(DRAGGED.load(Ordering::SeqCst));
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn mouse_drag<C>(callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &MouseEvent) + Sync + Send + 'static,
    {
        let hook = move |event: &HookEvent| {
            if let EventKind::MouseDragged(data) = &event.kind {
                callback(&event.metadata, data);
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Create a hook that will only listen to [`MouseDragged`] events
    /// where the buttons specified by `buttons` are used.
    ///
    /// This does not work on windows because windows does not put the mouse button in drag events
    /// if the mouse is dragged you might expect a series of event that looks like:
    /// left mouse button pressed -> mouse dragged -> mouse dragged -> left mouse button released.
    /// the two drag events will not have button information, only the press and release events will.
    ///
    /// This means that hooking on drag events with specific buttons is meaningless and the hook
    /// will never be activated on windows.
    ///
    /// [`MouseDragged`]: crate::hook::event::EventKind::MouseDragged
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # reserve_events(|e| e.is_synthetic());
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{
    ///     EventIterator, EventMask, MouseButton, MouseScrollDirection, MouseScrollKind,
    /// };
    /// use uiohook_rs::hook::HookOn;
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static BUTTON_LEFT: AtomicBool = AtomicBool::new(false);
    /// static BUTTON_RIGHT: AtomicBool = AtomicBool::new(false);
    ///
    /// let mut store_left =
    ///     Hook::mouse_drag_buttons(HookOn::OneOf([MouseButton::Left]), |meta, data| {
    ///         assert!(false, data);
    ///         BUTTON_LEFT.store(true, Ordering::SeqCst)
    ///     });
    ///
    /// let mut store_right = Hook::mouse_drag_buttons(HookOn::OneOf([MouseButton::Right]), |_, _| {
    ///     BUTTON_RIGHT.store(true, Ordering::SeqCst)
    /// });
    ///
    /// store_left.register();
    /// store_right.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we create a mouse wheel event, and post it.
    /// HookEvent::mouse(MouseButton::Left)
    ///     .with_mask(EventMask::LeftMouseButton)
    ///     .dragged(10, 10)
    ///     .post();
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that only the `store_left` callback was called.
    /// assert!(BUTTON_LEFT.load(Ordering::SeqCst));
    /// assert!(!BUTTON_RIGHT.load(Ordering::SeqCst));
    ///
    /// handle.stop().unwrap();
    /// ```
    #[cfg_attr(rustdoc, doc(cfg(any(target_os = "linux", target_os = "macos"))))]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mouse_drag_buttons<C, I>(buttons: HookOn<I>, callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &MouseEvent) + Sync + Send + 'static,
        I: IntoIterator<Item = MouseButton>,
    {
        let button_set: HashSet<MouseButton, ahash::RandomState> = match buttons {
            HookOn::OneOf(iter) => IntoIterator::into_iter(iter).collect(),
            HookOn::NoneOf(iter) => {
                let input_set = IntoIterator::into_iter(iter).collect();
                MOUSE_BUTTON_SET.difference(&input_set).cloned().collect()
            }
        };

        let hook = move |event: &HookEvent| {
            if let EventKind::MouseDragged(data) = &event.kind {
                if button_set.contains(&data.button) {
                    callback(&event.metadata, data);
                }
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Creates a hook that listens only to mouse scroll events.
    ///
    /// # Example
    ///```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{MouseScrollDirection, MouseScrollKind};
    /// use uiohook_rs::{hook_start, Hook, HookEvent};
    ///
    /// static WHEEL: AtomicBool = AtomicBool::new(false);
    /// static KEYBOARD: AtomicBool = AtomicBool::new(false);
    ///
    /// let mut store_wheel = Hook::mouse_wheel(|_, _| WHEEL.store(true, Ordering::SeqCst));
    /// let mut store_keyboard = Hook::keyboard(|_, _| KEYBOARD.store(true, Ordering::SeqCst));
    /// store_wheel.register();
    /// store_keyboard.register();
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // we create a mouse wheel event, and post it.
    /// HookEvent::scroll(3, 10, 10)
    ///     .with_kind(MouseScrollKind::Unit)
    ///     .with_rotation(1)
    ///     .with_direction(MouseScrollDirection::Vertical)
    ///     .build()
    ///     .post();
    ///
    /// // wait a little for the events to arrive.
    /// sleep(Duration::from_millis(5));
    ///
    /// // now we make sure that only the `store_wheel` callback was called.
    /// assert!(WHEEL.load(Ordering::SeqCst));
    /// assert!(!KEYBOARD.load(Ordering::SeqCst));
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn mouse_wheel<C>(callback: C) -> Hook
    where
        C: Fn(&EventMetaData, &MouseWheelEvent) + Sync + Send + 'static,
    {
        let hook = move |event: &HookEvent| {
            if let EventKind::MouseWheel(data) = &event.kind {
                callback(&event.metadata, data);
            }
        };

        Hook {
            hook: Some(Box::new(hook)),
            id: None,
        }
    }

    /// Register the hook so it will start listening.
    ///
    /// # Example
    ///```rust
    /// use uiohook_rs::{hook_start, keyboard, Hook};
    ///
    /// let mut log_all = Hook::new(|event| println!("{:?}", event));
    /// let handle = hook_start().expect("oops hook already running");
    /// // the log_all hook is not yet registered, meaning it wont catch any events.
    ///
    /// log_all.register();
    /// // now it will.
    ///
    /// // note that when using the macros the hooks are automatically registered.
    /// let log_kb = keyboard! {|meta, kb_event| println!("meta: {:?}\n event: {:?}", meta, kb_event)};
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn register(&mut self) {
        if let Some(callback) = mem::replace(&mut self.hook, None) {
            match self.id {
                Some(id) => global::register_boxed_hook_with_id(id, callback),
                None => self.id = Some(global::register_boxed_hook(callback)),
            }
        }
    }

    /// Unregister the hook, making it stop listening for events.
    ///
    /// # Example
    /// ```rust
    /// use uiohook_rs::{hook_start, keyboard, Hook};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    ///
    /// let mut log_all = Hook::new(|event| println!("{:?}", event));
    /// let handle = hook_start().expect("oops hook already running");
    /// // the log_all hook is not yet registered, meaning it wont catch any events.
    /// sleep(Duration::from_millis(1));
    ///
    /// log_all.register();
    /// // now it will.
    /// sleep(Duration::from_millis(1));
    ///
    /// log_all.unregister();
    /// // and now it will catch any event again.
    /// sleep(Duration::from_millis(1));
    ///
    /// handle.stop().unwrap();
    /// ```
    pub fn unregister(&mut self) {
        if let Some(id) = self.id {
            if let Some(callback) = global::unregister_hook(id) {
                self.hook = Some(callback);
            }
        }
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            global::drop_hook(id);
        }
    }
}
