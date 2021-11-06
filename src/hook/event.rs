//! Events types and utilities for working with them.

use std::thread::sleep;
use std::time::Duration;

use ffi::*;
use uiohook_sys as ffi;

pub use crate::hook::constants::{
    EventMask, EventMode, Key, MouseButton, MouseScrollDirection, MouseScrollKind,
};
use crate::hook::global::{post_event, postable_event};
use crate::PostEventError;

/// Contains data shared by all event types.
#[derive(Debug, Clone, Default)]
pub struct EventMetaData {
    /// This field contains a unix time stamp, number of milliseconds since the unix epoch.
    pub time: u128,
    /// The mask is meant to represent key combinations, for example when the user uses the Ctrl-C
    /// shortcut two events will be received one for the Ctrl and one for C, but they will have
    /// the same mask, to indicate they were pressed at the same time.
    /// Note that not all keys generate these masks, for example Windows will not set the mask if pressing
    /// the A, and D keys at the same time. This only works for "special" keys that can be part of combinations
    /// like Ctrl, Alt, and mouse buttons.
    pub mask: EventMask,
    /// This field indicates the mode the event is in.
    /// There are two possible modes, and a default.
    /// * [`Reserved`] - can only be set using the [`reserve_events`] function,
    /// if it is set, this event will not be propagated to userspace.
    ///
    /// * [`Synthetic`] - cannot be manually set, if the event was created using this
    /// library (specifically the API provided by HookEvent) it will automatically be set when the
    /// event is posted.
    ///
    /// [`Reserved`]: crate::hook::event::EventMode::RESERVED
    /// [`Synthetic`]: crate::hook::event::EventMode::SYNTHETIC
    /// [`reserve_events`]: crate::hook::global::reserve_events
    pub mode: EventMode,
}

impl EventMetaData {
    /// Check if the event was created by this library.
    ///
    /// This functionality is not supported by the OS, and is done with a little synchronization hack
    /// which does not guarantee that the flag will be set for all synthetic events, and does not guarantee,
    /// that some OS events will not be marked synthetic. Though the only time this can happen is if
    /// we post a synthetic event when the OS generates many events of the same type, in such a scenario
    /// the synthetic tag could be assigned to the wrong event, or not at all.
    ///
    /// To read more on how this is done, see the [`global`] module documentation.
    ///
    /// [`global`]: crate::hook::global
    ///
    /// # Example
    /// ```rust
    /// use uiohook_rs::hook::event::Key;
    /// use uiohook_rs::HookEvent;
    ///
    /// let event = HookEvent::keyboard(Key::L).press();
    ///
    /// assert!(event.is_synthetic());
    /// ```
    pub fn is_synthetic(&self) -> bool {
        self.mode.contains(EventMode::SYNTHETIC)
    }

    /// Check if the event was reserved, and did not get to the user..
    ///
    /// # Example
    /// ```rust
    /// use uiohook_rs::hook::event::Key;
    /// use uiohook_rs::hook::global::{register_hook, reserve_events};
    /// use uiohook_rs::{hook_start, HookEvent};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    ///
    /// // we only want to reserve synthetic events.
    /// reserve_events(|e| e.is_synthetic());
    ///
    /// register_hook(|e| {
    ///     // if we get a synthetic event it should be reserved.
    ///     if e.is_synthetic() {
    ///         assert!(e.is_reserved());
    ///     }
    /// });
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// // create an event and post it to test this.
    /// let event = HookEvent::keyboard(Key::L).press().post();
    ///
    /// handle.stop();
    /// ```
    pub fn is_reserved(&self) -> bool {
        self.mode.contains(EventMode::RESERVED)
    }
}

crate::map_native! {
    /// Container for data shared by all keyboard events.
    ///
    /// Note that this struct will not always be fully populated, when pressing a key on the keyboard
    /// the OS will usually generate three events:
    ///
    /// *   [`KeyPressed`] - here the `keycode` and `rawcode` fields will be populated, and the `keychar` filed
    ///     will have a junk value.
    ///
    /// *   [`KeyTyped`] - here the `keycode` will usually be undefined, the `rawcode` will be correct and
    ///     `keychar` will also be populated with the correct value.
    ///
    /// *   [`KeyReleased`] - finally the release event will have all the fields correctly populated.
    ///
    /// [`KeyPressed`]: crate::hook::event::EventKind::KeyPressed
    /// [`KeyTyped`]: crate::hook::event::EventKind::KeyTyped
    /// [`KeyReleased`]: crate::hook::event::EventKind::KeyReleased"
    keyboard_event_data => KeyboardEvent {
        /// The key the events effects, the keys are labeled according to an english
        /// keyboard layout but the event represents the key itself and not the character.
        /// For example in a hebrew keyboard the letter \"א\" and the letter \"T\" share the same key.
        /// This means that if the user is typing in hebrew and types in the letter \"א\" the value of this
        /// field will be [`Key::T`]
        keycode => keycode: Key,
        /// This field is simply the unicode representation of the keycode, though it always represents
        /// the english character, if we take the example given above, the `rawcode` will be 54
        rawcode => rawcode: u16,
        /// This field is also a unicode representation of the key effected by the event, only this
        /// field represents the character typed, meaning in the example above it will have the value 1488
        keychar => keychar: u16
    }
}

crate::map_native! {
    /// Container for data shared by all mouse events
    ///
    /// Note the existence three types of mouse button events, we have [`MousePressed`] and
    /// [`MouseReleased`] which fire one after the other and are self explanatory.
    /// The [`MouseClicked`] event is simply meant to represent that a full click occurred both a press and release
    ///
    /// [`MousePressed`]: crate::hook::event::EventKind::MousePressed
    /// [`MouseReleased`]: crate::hook::event::EventKind::MouseReleased
    /// [`MouseClicked`]: crate::hook::event::EventKind::MouseClicked
    mouse_event_data => MouseEvent {
        /// The mouse button associated with the event. In case the mouse just moved and a button
        /// is not clicked this field will have the value [`MouseButton::NoButton`]
        button => button: MouseButton,
        /// Number of clicks that occurred in this event, the value will usually be either 0 for
        /// events where the mouse only moved, or 1 for events where the a mouse button was pressed.
        /// A value of 2 indicates a double click, tough usually two events will be fired anyway
        clicks => clicks: u16,
        /// The horizontal position of the mouse
        x => x: i16,
        /// The vertical position of the mouse
        y => y: i16
    }
}

crate::map_native! {
    mouse_wheel_event_data => MouseWheelEvent {
        clicks => clicks: u16,
        /// The horizontal position of the mouse
        x => x: i16,
        /// The vertical position of the mouse
        y => y: i16,
        /// Possible values are `WHEEL_BLOCK_SCROLL` and `WHEEL_UNIT_SCROLL`, and are determined by the native platform
        type_ => kind: MouseScrollKind,
        /// The amount scrolled in this single event, this number is relatively meaningless, it seems
        /// that it is constant set by the OS and assign to each event basically representing the granularity
        /// of the scroll.
        amount => amount: u16,
        /// The rotation of the scroll represents might be a bit misleading, how can scrolling have a rotation ?
        /// This will usually have two values, one for scrolling up/left - negative value,
        /// and one for scrolling down/right - positive value
        rotation => rotation: i16,
        /// Mostly the mouse scrolls vertically, but in some cases it is possible to scroll horizontally.
        direction => direction: MouseScrollDirection
    }
}

/// The central piece of the library, all events are represented as different
/// variants of this enum, containing at least an instance of EventMetaData, and
/// the event specific data.
///
/// To build an event there are 2 stages:
/// 1.  First we choose the input device - mouse or keyboard, note that mouse is
///     seperated into mouse, and scroll, where mouse includes all mouse events except for scrolling.
/// 2.  Secondly we choose exactly what kind of event to create.
///     The scroll event doesn't have any specializations and requires the user to call [`build`] to
///     finish the process.
///     Keyboard and mouse events come in two common variants - click, and release.
///     It is also possible to create both events at once using
///     the `pair` method for both keyboard and mouse.
///
/// Note that the event meta data cannot be set, as the system will ignore it anyway.
/// The time field of the event is set by the system when it is dispatched,
/// the reserved field can only be set using the [`reserve_events`] API,
/// and the mask field is set automatically if two keys or buttons are pressed in close enough
/// succession.
///
/// There also some portability considerations for how events are created, the library attempts to
/// abstract most of them but creating drag events is fundamentally different on Windows than on Linux
/// and MacOS. Windows does not have a drag event, and to create one you must first create a mouse press
/// event, then a mouse move event, and finally a mouse release to achieve the dragging.
///
///
/// [`reserve_events`]: crate::hook::global::reserve_events
/// [`post_event`]: crate::hook::global::post_event
/// [`build`]: MouseWheelEventBuilder::build
///
/// # Creating a keyboard event
///
/// ```rust
/// use uiohook_rs::hook::event::{HookEvent, Key};
///
/// HookEvent::keyboard(Key::E).press();
/// ```
///
/// # Creating a press and release mouse events
///
/// ```rust
/// use uiohook_rs::hook::event::{HookEvent, MouseButton};
/// # use std::time::Duration;
///
/// HookEvent::mouse(MouseButton::Left).pair();
/// ```
///
/// # Creating a scroll event
///
/// ```rust
/// use uiohook_rs::hook::event::HookEvent;
///
/// HookEvent::scroll(/* amount */ 10, /* x */ 20, /* y */ 20).build();
/// ```
///
/// # Creating a keyboard sequence
///
/// ```rust
/// # use uiohook_rs::hook::global::{reserve_events, hook_start};
/// # // prevent these events from effecting the user when running tests
/// # let handle = hook_start().unwrap();
/// # unsafe { reserve_events(|e| e.is_synthetic()); }
///
/// use uiohook_rs::hook::event::{EventMask, HookEvent, Key, PairEventIterator};
/// use uiohook_rs::hook::global::post_event;
/// # use std::thread::sleep;
/// # use std::time::Duration;
///
/// let make_events = |k: Key| {
///     HookEvent::keyboard(k)
///         .with_mask(EventMask::LeftControl)
///         .pair()
/// };
/// let ctrl_c = vec![Key::LeftControl, Key::C];
/// let ctrl_v = vec![Key::LeftControl, Key::V];
///
/// ctrl_c
///     .into_iter()
///     .map(make_events)
///     .post_delayed_sequence(Duration::from_millis(1));
/// ctrl_v
///     .into_iter()
///     .map(make_events)
///     .post_delayed_sequence(Duration::from_millis(1));
///
/// # handle.stop();
/// ```
///
/// # Some more examples
///
/// ```rust
/// use uiohook_rs::hook::event::{
///     HookEvent, Key, MouseButton, MouseScrollDirection, MouseScrollKind,
/// };
///
/// // on linux this will work
/// #[cfg(target_os = "linux")]
/// let drag: HookEvent = HookEvent::mouse(MouseButton::Middle)
///     .with_clicks(2)
///     .dragged(50, 50);
///
/// // and on windows this will work
/// // see the HookEvent documentation to understand the difference.
/// #[cfg(target_os = "windows")]
/// let win_drag: std::array::IntoIter<HookEvent, 3> = HookEvent::mouse(MouseButton::Left)
///     .with_clicks(2)
///     .dragged(50, 50);
///
/// let scroll = HookEvent::scroll(10, 20, 20)
///     .with_direction(MouseScrollDirection::Vertical)
///     .with_kind(MouseScrollKind::Unit)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct HookEvent {
    pub metadata: EventMetaData,
    pub kind: EventKind,
}

/// This enum stores the the specific kind of the event and the data
/// unique to it.
///
/// For more information on the events and their data look at the
/// documentation for the data structs [`KeyboardEvent`], [`MouseEvent`], [`MouseWheelEvent`].
#[derive(Debug, Clone)]
pub enum EventKind {
    #[doc(hidden)]
    Enabled,
    #[doc(hidden)]
    Disabled,
    KeyTyped(KeyboardEvent),
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),
    MouseClicked(MouseEvent),
    MousePressed(MouseEvent),
    MouseReleased(MouseEvent),
    MouseMoved(MouseEvent),
    MouseDragged(MouseEvent),
    MouseWheel(MouseWheelEvent),
}

/// A more generic version of [`EventKind`].
pub enum EventType {
    Control,
    Keyboard,
    Mouse,
    MouseWheel,
}

impl HookEvent {
    /// Start creating a keyboard event that will affect the specified `key`.
    pub fn keyboard(key: Key) -> KeyboardEventBuilder {
        let meta = EventMetaData {
            mode: EventMode::SYNTHETIC,
            ..Default::default()
        };
        KeyboardEventBuilder {
            meta,
            event: KeyboardEvent {
                keycode: key,
                rawcode: key.into(),
                keychar: key.into(),
            },
        }
    }

    /// Start creating a mouse event that will affect the specified `button`.
    pub fn mouse(button: MouseButton) -> MouseEventBuilder {
        let meta = EventMetaData {
            mode: EventMode::SYNTHETIC,
            ..Default::default()
        };
        MouseEventBuilder {
            meta,
            event: MouseEvent {
                button,
                clicks: 0,
                x: 0,
                y: 0,
            },
        }
    }

    /// Start creating a mouse scroll event.
    pub fn scroll(amount: u16, x: i16, y: i16) -> MouseWheelEventBuilder {
        let meta = EventMetaData {
            mode: EventMode::SYNTHETIC,
            ..Default::default()
        };
        MouseWheelEventBuilder {
            meta,
            event: MouseWheelEvent {
                clicks: 0,
                x,
                y,
                kind: MouseScrollKind::Unit,
                amount,
                rotation: 0,
                direction: MouseScrollDirection::Vertical,
            },
        }
    }

    /// Wrapper around [`EventMetaData::is_synthetic`].
    pub fn is_synthetic(&self) -> bool {
        self.metadata.is_synthetic()
    }

    /// Wrapper around [`EventMetaData::is_reserved`].
    pub fn is_reserved(&self) -> bool {
        self.metadata.is_reserved()
    }

    /// Get a more generic event type then the one provided by [`EventKind`].
    ///
    /// # Example
    /// ```rust
    /// use uiohook_rs::hook::global::register_hook;
    /// use uiohook_rs::{hook_start, EventType};
    ///
    /// hook_start().expect("oops already running");
    ///
    /// // we can easily check if the user did anything with the mouse
    /// // instead of doing the same thing for all mouse event kinds.
    /// register_hook(|event| match event.get_type() {
    ///     EventType::Mouse => println!("got a mouse event!"),
    ///     _ => (),
    /// });
    /// ```
    pub fn get_type(&self) -> EventType {
        match self.kind {
            EventKind::Enabled => EventType::Control,
            EventKind::Disabled => EventType::Control,
            EventKind::KeyTyped(_) => EventType::Keyboard,
            EventKind::KeyPressed(_) => EventType::Keyboard,
            EventKind::KeyReleased(_) => EventType::Keyboard,
            EventKind::MouseClicked(_) => EventType::Mouse,
            EventKind::MousePressed(_) => EventType::Mouse,
            EventKind::MouseReleased(_) => EventType::Mouse,
            EventKind::MouseMoved(_) => EventType::Mouse,
            EventKind::MouseDragged(_) => EventType::Mouse,
            EventKind::MouseWheel(_) => EventType::MouseWheel,
        }
    }

    /// Post the event, this will simulate the user creating the same event through the use of the mouse and keyboard.
    /// Calling this function is equivalent to calling [`post_event`] with this self.
    /// Note that its impossible to post `Enabled` and `Disabled` events.
    ///
    /// # Example
    /// ```rust
    /// # use uiohook_rs::hook::global::reserve_events;
    /// # // prevent these events from effecting the user when running tests
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    ///
    /// use uiohook_rs::hook::event::{EventKind, Key};
    /// use uiohook_rs::hook::global::register_hook;
    /// use uiohook_rs::{hook_start, HookEvent};
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    ///
    /// let handle = hook_start().expect("oops hook already running");
    ///
    /// register_hook(|event| {
    ///     if event.is_synthetic() {
    ///         if let EventKind::KeyPressed(data) = &event.kind {
    ///             // the event reached our hook.
    ///             assert_eq!(data.keycode, Key::A);
    ///         }
    ///         // this cant happen because we only post one synthetic event.
    ///         unreachable!();
    ///     }
    /// });
    ///
    /// sleep(Duration::from_millis(1));
    ///
    /// HookEvent::keyboard(Key::A)
    ///     .press()
    ///     .post()
    ///     .expect("we are not posting enabled or disabled events how is this happening");
    ///
    /// handle.stop();
    /// ```
    pub fn post(self) -> Result<(), PostEventError> {
        post_event(self)
    }
}

/// Container holding a (press, release) event pair.
///
/// This container is mainly a convenience to allow easier handling of the common
/// use case of a press and release event pair.
///
/// Note that when calling one of the posting methods on this type,
/// it is impossible for only part of the event pair to be posted while the
/// other errors. If one of the events in the pair is invalid the function will return before
/// posting any of them.
pub struct EventPair {
    press: HookEvent,
    release: HookEvent,
}

impl From<EventPair> for (HookEvent, HookEvent) {
    fn from(pair: EventPair) -> Self {
        (pair.press, pair.release)
    }
}

impl From<(HookEvent, HookEvent)> for EventPair {
    fn from(pair: (HookEvent, HookEvent)) -> Self {
        EventPair {
            press: pair.0,
            release: pair.1,
        }
    }
}

impl EventPair {
    // We use this method to make sure both events are postable before posting.
    // The check performed in `post_event` is not enough because we dont want to post the first event
    // only to find out the second is not postable.
    pub(crate) fn postable(&self) -> Result<(), PostEventError> {
        postable_event(&self.press)?;
        postable_event(&self.release)
    }

    /// post both the press and release events one after the other press -> release,
    /// with no delay between them.
    pub fn post(self) -> Result<(), PostEventError> {
        self.postable()?;
        post_event(self.press)?;
        post_event(self.release)
    }

    /// This method will post both events with a delay between them, blocking until the delay is
    /// finished an both events have been posted.
    ///
    /// # Example
    /// ```rust
    /// # use uiohook_rs::hook::global::{reserve_events, hook_start, hook_stop};
    /// # // prevent these events from effecting the user when running tests
    /// # let handle = hook_start().unwrap();
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::Key;
    /// use uiohook_rs::HookEvent;
    ///
    /// let event_pair = HookEvent::keyboard(Key::Escape).pair();
    ///
    /// event_pair.post_delayed(Duration::from_millis(2));
    /// // the press event will be registered immediately
    /// // two milliseconds later the release event will be registered.
    /// // finally the function will exit and this called will execute.
    /// println!("done!");
    /// # hook_stop().unwrap();
    /// ```
    pub fn post_delayed(self, delay: Duration) -> Result<(), PostEventError> {
        self.postable()?;

        post_event(self.press)?;
        sleep(delay);
        post_event(self.release)
    }

    /// This method will post both events with a delay between them, unlike [`post_delayed`]
    /// this function does not block.
    ///
    /// Note that this function spawns a thread in order to be asynchronous, meaning
    /// that if you call this function with a long delay many times, the memory usage of your program
    /// could explode with many waiting threads.
    ///
    /// [`post_delayed`]: EventPair::post_delayed
    /// # Example
    /// ```rust
    /// # use uiohook_rs::hook::global::{reserve_events, hook_start, hook_stop};
    /// # // prevent these events from effecting the user when running tests
    /// # let handle = hook_start().unwrap();
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    /// # use std::time::Duration;
    /// # use std::thread::sleep;
    /// use uiohook_rs::hook::event::Key;
    /// use uiohook_rs::HookEvent;
    ///
    /// let event_pair = HookEvent::keyboard(Key::Escape).pair();
    ///
    /// event_pair.post_delayed_async(Duration::from_millis(2));
    /// // the press event will be registered immediately.
    /// // the function spawns a thread and returns
    ///
    /// // do some stuff here...
    /// sleep(Duration::from_millis(2));
    /// // two milliseconds later the release event will be registered.
    /// # hook_stop().unwrap();
    /// ```
    pub fn post_delayed_async(self, delay: Duration) -> Result<(), PostEventError> {
        self.postable()?;

        let (press, release) = self.into();
        post_event(press)?;

        std::thread::spawn(move || {
            sleep(delay);
            post_event(release).expect("post event error not caught by postable event check");
        });

        Ok(())
    }
}

/// This is an extension for [`Iterator`] making it easier
/// to post multiple event pairs.
///
/// The methods in this trait are mostly wrappers around [`EventPair`]'s posting methods
/// and have very similar semantics.
pub trait PairEventIterator: Iterator<Item = EventPair> + Sized {
    /// Post all event pairs in the iterator one by one with no delay. For each pair the press
    /// event is posted, then immediately the release, and only then the next pair is is posted.
    ///
    /// This method simply consumes the iterator calling [`EventPair::post`] on each pair.
    fn post(self) -> Result<(), PostEventError> {
        for ep in self {
            ep.post()?;
        }

        Ok(())
    }

    /// Post all events in the iterator with a delay between the press and release
    /// of each event.
    ///
    /// This method works similarly to [`post`], only there will be a `delay` between the press and
    /// release of each event. Similarly to [`EventPair::post_delayed`] this method blocks until
    /// all events have been posted
    ///
    /// [`post`]: PairEventIterator::post
    fn post_delayed(self, delay: Duration) -> Result<(), PostEventError> {
        for ep in self {
            ep.post_delayed(delay)?;
        }

        Ok(())
    }

    /// Post all events in the iterator with a delay between the press and release
    /// of each event. Unlike [`post_delayed`] this method doesnt block.
    ///
    /// Note that this method spawns a single thread in order to be asynchronous, meaning
    /// that if you call this function with a long delay many times, the memory usage of your program
    /// could explode with many waiting threads.
    ///
    /// Note that the order of the events in the iterator is preserved, only one thread
    /// is spawned and it the iterator is consumed normally.
    ///
    /// [`post_delayed`]: PairEventIterator::post_delayed
    fn post_delayed_async(self, delay: Duration) -> Result<(), PostEventError> {
        let events: Vec<EventPair> = self.collect();
        for ep in events.iter() {
            ep.postable()?;
        }

        std::thread::spawn(move || {
            events.into_iter().for_each(|ep| {
                ep.post_delayed(delay)
                    .expect("post event error not caught by postable event check")
            });
        });
        Ok(())
    }

    /// Post all event pairs in the iterator treating them as a sequence, first posting
    /// all the press events, and then the release.
    ///
    /// Note that because this is a sequence if *any* event in the iterator cannot be posted
    /// the function will immediately return, without posting *any* of the events.
    ///
    /// # Example
    /// ```rust
    /// # use uiohook_rs::hook::global::{reserve_events, hook_start, hook_stop};
    /// # // prevent these events from effecting the user when running tests
    /// # let handle = hook_start().unwrap();
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{EventMask, Key, PairEventIterator};
    /// use uiohook_rs::HookEvent;
    ///
    /// // first we create an iterator of keyboard events.
    /// let sequence = vec![Key::LeftControl, Key::C].into_iter().map(|k| {
    ///     HookEvent::keyboard(k)
    ///         .with_mask(EventMask::LeftControl)
    ///         .pair()
    /// });
    ///
    /// // calling this method will cause the system to receive the events
    /// // in the following order:
    /// // LeftControl press -> C press -> LeftControl -> release, C release
    /// // making it a valid Ctrl-C.
    /// sequence.post_sequence();
    ///
    /// # hook_stop().unwrap();
    /// ```
    fn post_sequence(self) -> Result<(), PostEventError> {
        let mut pres_vec = Vec::new();
        let mut release_vec = Vec::new();

        for ep in self {
            ep.postable()?;
            pres_vec.push(ep.press);
            release_vec.push(ep.release);
        }

        pres_vec.into_iter().for_each(|e| {
            post_event(e).expect("post event error not caught by postable event check")
        });
        release_vec.into_iter().for_each(|e| {
            post_event(e).expect("post event error not caught by postable event check")
        });

        Ok(())
    }

    /// Post all event pairs in the iterator treating them as a sequence, with a `delay`
    /// separating the press and release parts of the sequence.
    ///
    /// Similarly to [`EventPair::post_delayed`] this method will block for the specified
    /// delay and will return only after all the events have been posted.
    ///
    /// Note that because this is a sequence if *any* event in the iterator cannot be posted
    /// the function will immediately return, without posting *any* of the events.
    ///
    /// # Example
    /// ```rust
    /// # use uiohook_rs::hook::global::{reserve_events, hook_start, hook_stop};
    /// # // prevent these events from effecting the user when running tests
    /// # let handle = hook_start().unwrap();
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{EventMask, Key, PairEventIterator};
    /// use uiohook_rs::HookEvent;
    ///
    /// // first we create an iterator of keyboard events.
    /// let sequence = vec![Key::LeftControl, Key::C].into_iter().map(|k| {
    ///     HookEvent::keyboard(k)
    ///         .with_mask(EventMask::LeftControl)
    ///         .pair()
    /// });
    ///
    /// // calling this method will cause the system to receive the events
    /// // in the following order:
    /// // LeftControl press -> C press -> (5 millisecond wait) -> LeftControl -> release, C release
    /// // making it a valid Ctrl-C.
    /// sequence.post_delayed_sequence(Duration::from_millis(1));
    ///
    /// # hook_stop().unwrap();
    /// ```
    fn post_delayed_sequence(self, delay: Duration) -> Result<(), PostEventError> {
        let mut pres_vec = Vec::new();
        let mut release_vec = Vec::new();

        for ep in self {
            ep.postable()?;
            pres_vec.push(ep.press);
            release_vec.push(ep.release);
        }

        pres_vec.into_iter().for_each(|e| {
            post_event(e).expect("post event error not caught by postable event check")
        });
        sleep(delay);
        release_vec.into_iter().for_each(|e| {
            post_event(e).expect("post event error not caught by postable event check")
        });

        Ok(())
    }

    /// Post all event pairs in the iterator treating them as a sequence, with a `delay`
    /// separating the press and release parts of the sequence.
    ///
    /// Similarly to [`EventPair::post_delayed_async`] this method will not block and will
    /// spawn a thread to post the events in.
    ///
    /// Note that because this is a sequence if *any* event in the iterator cannot be posted
    /// the function will immediately return, without posting *any* of the events.
    ///
    /// # Example
    /// ```rust
    /// # use uiohook_rs::hook::global::{reserve_events, hook_start, hook_stop};
    /// # // prevent these events from effecting the user when running tests
    /// # let handle = hook_start().unwrap();
    /// # unsafe { reserve_events(|e| e.is_synthetic()); }
    /// # use std::time::Duration;
    /// use uiohook_rs::hook::event::{EventMask, Key, PairEventIterator};
    /// use uiohook_rs::HookEvent;
    ///
    /// // first we create an iterator of keyboard events.
    /// let sequence = vec![Key::LeftControl, Key::C].into_iter().map(|k| {
    ///     HookEvent::keyboard(k)
    ///         .with_mask(EventMask::LeftControl)
    ///         .pair()
    /// });
    ///
    /// // calling this method will cause the system to receive the events
    /// // in the following order:
    /// // LeftControl press -> C press -> (5 millisecond wait) -> LeftControl -> release, C release
    /// // making it a valid Ctrl-C.
    /// sequence.post_delayed_async_sequence(Duration::from_millis(1));
    /// // though the function will return before any event is posted.
    /// // we can now do other things while the events are being posted...
    ///
    /// # hook_stop().unwrap();
    /// ```
    fn post_delayed_async_sequence(self, delay: Duration) -> Result<(), PostEventError> {
        let mut pres_vec = Vec::new();
        let mut release_vec = Vec::new();

        for ep in self {
            ep.postable()?;
            pres_vec.push(ep.press);
            release_vec.push(ep.release);
        }

        std::thread::spawn(move || {
            pres_vec.into_iter().for_each(|e| {
                post_event(e).expect("post event error not caught by postable event check")
            });
            sleep(delay);
            release_vec.into_iter().for_each(|e| {
                post_event(e).expect("post event error not caught by postable event check")
            });
        });

        Ok(())
    }
}

impl<T> PairEventIterator for T where T: Iterator<Item = EventPair> {}

pub trait EventIterator: Iterator<Item = HookEvent> + Sized {
    fn post(self) -> Result<(), PostEventError> {
        for e in self {
            e.post()?;
        }

        Ok(())
    }

    fn post_delayed(self, delay: Duration) -> Result<(), PostEventError> {
        for e in self {
            e.post()?;
            sleep(delay);
        }

        Ok(())
    }

    fn post_delayed_async(self, delay: Duration) -> Result<(), PostEventError> {
        let mut res = Ok(());
        let mut postable = Vec::new();
        for e in self {
            res = postable_event(&e);
            if res.is_ok() {
                postable.push(e);
            } else {
                break;
            }
        }

        std::thread::spawn(move || {
            for e in postable {
                e.post()
                    .expect("failed to post event event though it is postable.");
                sleep(delay);
            }
        });

        res
    }
}

impl<T> EventIterator for T where T: Iterator<Item = HookEvent> {}

#[doc(hidden)]
pub struct KeyboardEventBuilder {
    meta: EventMetaData,
    event: KeyboardEvent,
}
impl KeyboardEventBuilder {
    pub fn with_mask(mut self, mask: EventMask) -> Self {
        self.meta.mask = mask;
        self
    }

    pub fn pair(self) -> EventPair {
        EventPair {
            press: HookEvent {
                metadata: self.meta.clone(),
                kind: EventKind::KeyPressed(self.event.clone()),
            },
            release: HookEvent {
                metadata: self.meta,
                kind: EventKind::KeyReleased(self.event),
            },
        }
    }

    pub fn press(self) -> HookEvent {
        HookEvent {
            metadata: self.meta,
            kind: EventKind::KeyPressed(self.event),
        }
    }

    pub fn release(self) -> HookEvent {
        HookEvent {
            metadata: self.meta,
            kind: EventKind::KeyReleased(self.event),
        }
    }
}

pub struct MouseEventBuilder {
    meta: EventMetaData,
    event: MouseEvent,
}
impl MouseEventBuilder {
    pub fn with_clicks(mut self, clicks: u16) -> Self {
        self.event.clicks = clicks;
        self
    }

    pub fn with_mask(mut self, mask: EventMask) -> Self {
        self.meta.mask = mask;
        self
    }

    pub fn pair(mut self) -> EventPair {
        self.event.clicks = std::cmp::max(self.event.clicks, 1);
        EventPair {
            press: HookEvent {
                metadata: self.meta.clone(),
                kind: EventKind::MousePressed(self.event.clone()),
            },
            release: HookEvent {
                metadata: self.meta,
                kind: EventKind::MouseReleased(self.event),
            },
        }
    }

    pub fn press(mut self) -> HookEvent {
        self.event.clicks = std::cmp::max(self.event.clicks, 1);
        HookEvent {
            metadata: self.meta,
            kind: EventKind::MousePressed(self.event),
        }
    }

    pub fn release(mut self) -> HookEvent {
        self.event.clicks = std::cmp::max(self.event.clicks, 1);
        HookEvent {
            metadata: self.meta,
            kind: EventKind::MouseReleased(self.event),
        }
    }

    pub fn moved(mut self, x: i16, y: i16) -> HookEvent {
        self.event.x = x;
        self.event.y = y;
        HookEvent {
            metadata: self.meta,
            kind: EventKind::MouseMoved(self.event),
        }
    }

    #[cfg_attr(rustdoc, doc(cfg(any(target_os = "linux", target_os = "macos"))))]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn dragged(mut self, x: i16, y: i16) -> HookEvent {
        self.event.clicks = std::cmp::max(self.event.clicks, 1);
        self.event.x = x;
        self.event.y = y;
        HookEvent {
            metadata: self.meta,
            kind: EventKind::MouseDragged(self.event),
        }
    }

    #[cfg_attr(rustdoc, doc(cfg(target_os = "windows")))]
    #[cfg(target_os = "windows")]
    pub fn dragged(mut self, x: i16, y: i16) -> std::array::IntoIter<HookEvent, 3> {
        self.event.clicks = std::cmp::max(self.event.clicks, 1);
        self.event.x = x;
        self.event.y = y;

        let press_release_data = MouseEvent {
            button: self.event.button,
            clicks: self.event.clicks,
            x: 0,
            y: 0,
        };

        let press_event = HookEvent {
            metadata: self.meta.clone(),
            kind: EventKind::MousePressed(press_release_data.clone()),
        };
        let move_event = HookEvent {
            metadata: self.meta.clone(),
            kind: EventKind::MouseMoved(self.event),
        };
        let release_event = HookEvent {
            metadata: self.meta,
            kind: EventKind::MouseReleased(press_release_data),
        };

        IntoIterator::into_iter([press_event, move_event, release_event])
    }
}

#[doc(hidden)]
pub struct MouseWheelEventBuilder {
    meta: EventMetaData,
    event: MouseWheelEvent,
}
impl MouseWheelEventBuilder {
    pub fn with_clicks(mut self, clicks: u16) -> Self {
        self.event.clicks = clicks;
        self
    }

    pub fn with_kind(mut self, kind: MouseScrollKind) -> Self {
        self.event.kind = kind;
        self
    }

    pub fn with_direction(mut self, direction: MouseScrollDirection) -> Self {
        self.event.direction = direction;
        self
    }

    pub fn with_rotation(mut self, rotation: i16) -> Self {
        self.event.rotation = rotation;
        self
    }

    pub fn with_mask(mut self, mask: EventMask) -> Self {
        self.meta.mask = mask;
        self
    }

    pub fn build(self) -> HookEvent {
        HookEvent {
            metadata: self.meta,
            kind: EventKind::MouseWheel(self.event),
        }
    }
}
