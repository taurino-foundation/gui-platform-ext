#[cfg(windows)]
mod win;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod linux;

#[cfg(any(
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
pub mod undecorated_resizing;

pub mod dpi;
pub mod image;
pub mod resources;
pub mod util;

use crate::dpi::{PhysicalRect, PhysicalSize};

// ─────────────────────────────────────────────
// Error handling
// ─────────────────────────────────────────────

use std::sync::{Arc, Mutex};


pub type ArcMut<T> = Arc<Mutex<T>>;

pub fn arc<T>(t: T) -> Arc<T> {
    Arc::new(t)
}

pub fn arc_mut<T>(t: T) -> ArcMut<T> {
    Arc::new(Mutex::new(t))
}

pub fn apply_shadow_correction(
    decorations: bool,
    window_size: &mut PhysicalSize<u32>, // oder was auch immer der Typ ist
) -> anyhow::Result<u32> {
    #[allow(unused_mut)]
    let mut shadow_width = 0;

    #[cfg(windows)]
    if decorations {
        use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRect, WS_OVERLAPPEDWINDOW};
        let mut rect = windows::Win32::Foundation::RECT::default();
        let result = unsafe { AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, false) };
        if result.is_ok() {
            shadow_width = (rect.right - rect.left) as u32;
            // rect.bottom is made out of shadow, and we don't care about it
            window_size.height += -rect.top as u32;
        }
    }

    Ok(shadow_width)
}

pub fn find_monitor_for_position(
    monitors: impl Iterator<Item = tao::monitor::MonitorHandle>,
    window_position: crate::dpi::Position,
) -> Option<tao::monitor::MonitorHandle> {
    monitors.into_iter().find(|m| {
        let monitor_pos = m.position();
        let monitor_size = m.size();

        // type annotations required for 32bit targets.
        let window_position = window_position.to_physical::<i32>(m.scale_factor());

        monitor_pos.x <= window_position.x
            && window_position.x < monitor_pos.x + monitor_size.width as i32
            && monitor_pos.y <= window_position.y
            && window_position.y < monitor_pos.y + monitor_size.height as i32
    })
}

pub fn error(_err: &'static str) {
    #[cfg(windows)]
    win::dialog::error(_err);

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    linux::dialog::error(_err);

    #[cfg(target_os = "macos")]
    mac::dialog::error(_err);
}

// ─────────────────────────────────────────────
// Monitor extensions
// ─────────────────────────────────────────────

pub trait MonitorExt {
    /// Get the work area of this monitor.
    ///
    /// ## Platform-specific
    ///
    /// - **Android / iOS**: Unsupported.
    fn work_area(&self) -> PhysicalRect<i32, u32>;
}

#[cfg(mobile)]
impl MonitorExt for tao::monitor::MonitorHandle {
    fn work_area(&self) -> PhysicalRect<i32, u32> {
        PhysicalRect {
            size: self.size(),
            position: self.position(),
        }
    }
}

// ─────────────────────────────────────────────
// Window extensions
// ─────────────────────────────────────────────

pub trait WindowExt {
    /// Enable or disable the window.
    ///
    /// ## Platform-specific
    ///
    /// - **Android / iOS**: Unsupported.
    fn set_enabled(&self, enabled: bool);

    /// Whether the window is enabled or disabled.
    ///
    /// ## Platform-specific
    ///
    /// - **Android / iOS**: Unsupported, always returns `true`.
    fn is_enabled(&self) -> bool;

    /// Center the window.
    ///
    /// ## Platform-specific
    ///
    /// - **Android / iOS**: Unsupported.
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    fn center(&self) {}

    /// Clears the window surface, i.e. makes it transparent.
    #[cfg(windows)]
    fn draw_surface(
        &self,
        surface: &mut softbuffer::Surface<std::sync::Arc<tao::window::Window>, std::sync::Arc<tao::window::Window>>,
        background_color: Option<tao::window::RGBA>,
    );
}

#[cfg(mobile)]
impl WindowExt for tao::window::Window {
    fn set_enabled(&self, _: bool) {}

    fn is_enabled(&self) -> bool {
        true
    }
}

// ─────────────────────────────────────────────
// Window positioning
// ─────────────────────────────────────────────

#[cfg(desktop)]
pub fn calculate_window_center_position(
    window_size: tao::dpi::PhysicalSize<u32>,
    target_monitor: tao::monitor::MonitorHandle,
) -> tao::dpi::PhysicalPosition<i32> {
    let work_area = target_monitor.work_area();

    tao::dpi::PhysicalPosition::new(
        (work_area.size.width as i32 - window_size.width as i32) / 2 + work_area.position.x,
        (work_area.size.height as i32 - window_size.height as i32) / 2 + work_area.position.y,
    )
}

// ─────────────────────────────────────────────
// Public dependency re-exports
// ─────────────────────────────────────────────

pub use {anyhow, getrandom, log, raw_window_handle, serde, tao, wry};

// These names would conflict with our own `dpi` and `image` modules.
pub use ::image as image_crate;

// ─────────────────────────────────────────────
// Cross-platform desktop dependencies
// ─────────────────────────────────────────────

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "windows",
    target_os = "macos",
))]
pub use ::muda;

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "windows",
    target_os = "macos",
))]
pub use ::tray_icon;

// ─────────────────────────────────────────────
// Windows
// ─────────────────────────────────────────────

#[cfg(windows)]
pub use ::once_cell;

#[cfg(windows)]
pub use ::softbuffer;

#[cfg(windows)]
pub use ::webview2_com;

#[cfg(windows)]
pub use ::windows;

// ─────────────────────────────────────────────
// macOS
// ─────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub use ::objc2;

#[cfg(target_os = "macos")]
pub use ::objc2_app_kit;

#[cfg(target_os = "macos")]
pub use ::objc2_web_kit;

// ─────────────────────────────────────────────
// Linux / BSD
// ─────────────────────────────────────────────

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub use ::gtk;

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub use ::webkit2gtk;



/// Identifier of a window.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct WindowId(u32);

impl From<u32> for WindowId {
  fn from(value: u32) -> Self {
    Self(value)
  }
}

impl WindowId {
    pub fn get(self) -> u32 {
        self.0
    }
}

/// Identifier of a webview.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct WebViewId(u32);

impl From<u32> for WebViewId {
  fn from(value: u32) -> Self {
    Self(value)
  }
}


impl WebViewId {
    pub fn get(self) -> u32 {
        self.0
    }
}

// ─────────────────────────────────────────────
// Synchronization helpers
// ─────────────────────────────────────────────
#[macro_export]
macro_rules! unsafe_impl_sync_send {
    ($type:ty) => {
        unsafe impl Send for $type {}
        unsafe impl Sync for $type {}
    };
}

#[macro_export]
macro_rules! set_property_some {
    ($builder:ident, $property:ident, &$value:expr) => {
        if let Some(value) = &$value {
            $builder = $builder.$property(value);
        }
    };
    ($builder:ident, $property:ident, $value:expr) => {
        if let Some(value) = $value {
            $builder = $builder.$property(value.clone());
        }
    };
}

#[macro_export]
macro_rules! set_property {
    ($builder:ident, $property:ident, $value:expr) => {
        $builder = $builder.$property($value);
    };
}


#[macro_export]
macro_rules! lock {
    ($value:expr) => {
        $value
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock {}.", stringify!($value)))
    };
}

#[macro_export]
macro_rules! lock_force {
    ($value:expr) => {
        $value.lock().unwrap()
    };
}




use tao::{
        event::Event,
        event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    }};

/// Internal messages delivered through the Tao user-event channel.
///
/// These messages provide a thread-safe mechanism for scheduling engine work on
/// the event-loop thread.
///
/// Code executing outside the GUI event loop should generally send one of these
/// messages instead of manipulating Tao state directly.
pub enum EventLoopMessage {
    /// Requests graceful termination of the entire application.
    ///
    /// The request is processed by [`EngineEventHandler`] on the event-loop
    /// thread.
    Shutdown,

    /// Executes a dynamically supplied Taurino event-loop operation.
    ///
    /// This variant allows engine subsystems to schedule arbitrary operations
    /// that require access to the Tao event-loop target and its
    /// [`ControlFlow`].
    TaskWithTarget(EngineEvent),
    /// Executes a dynamically supplied event-loop operation.
    ///
    /// This variant allows engine subsystems to schedule arbitrary operations
    /// that require to run on the Tao event-loop target thread
    Task(Box<dyn FnOnce() + Send>),

      #[cfg(target_os = "macos")]
      SetDockVisibility(bool),
      RequestExit(i32),
}
pub type EngineLoop = EventLoop<EventLoopMessage>;
pub type EngineLoopBuilder = EventLoopBuilder<EventLoopMessage>;
pub type EngineWindowTarget = EventLoopWindowTarget<EventLoopMessage>;
pub type EngineLoopProxy = EventLoopProxy<EventLoopMessage>;
pub type EngineLoopClosed = EventLoopClosed<EventLoopMessage>;
pub type EngineLoopEvent<'a> = Event<'a, EventLoopMessage>;
pub type EngineCallback = Pin<Box<dyn Fn(&TaurinoWindowTarget, &mut ControlFlow) -> anyhow::Result<()> + Send>>;

/// Type-erased operation scheduled for execution on the Tao event-loop thread.
///
/// The callback receives:
///
/// - the current [`TaurinoWindowTarget`], allowing creation or manipulation of
///   event-loop-bound objects;
/// - mutable [`ControlFlow`], allowing the callback to influence future
///   event-loop execution.
///
/// The callback is `Send` because it may be created on another thread before
/// being transported through the Tao user-event channel.
///
/// It is boxed and pinned to provide one stable, type-erased representation for
/// arbitrary callback implementations.
///
pub struct EngineEvent(EngineCallback);

impl Debug for EngineEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineEvent").finish()
    }
}

impl EngineEvent {
    pub fn new<F: Fn(&EngineWindowTarget, &mut ControlFlow) -> anyhow::Result<()> + Send + 'static>(f: F) -> Self {
        Self(Box::pin(f))
    }
}

impl Deref for EngineEvent {
    type Target = Pin<Box<dyn Fn(&EngineWindowTarget, &mut ControlFlow) -> anyhow::Result<()> + Send>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
