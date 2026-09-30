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

pub fn error(err: &'static str) {
  #[cfg(windows)]
  win::dialog::error(err);

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  linux::dialog::error(err);

  #[cfg(target_os = "macos")]
  mac::dialog::error(err);
}

pub trait MonitorExt {
  /// Get the work area of this monitor.
  ///
  /// ## Platform-specific
  ///
  /// - **Android / iOS**: Unsupported.
  fn work_area(&self) -> crate::dpi::PhysicalRect<i32, u32>;
}

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
  fn center(&self) {}

  /// Clears the window surface, making it transparent.
  #[cfg(windows)]
  fn draw_surface(
    &self,
    surface: &mut softbuffer::Surface<
      std::sync::Arc<tao::window::Window>,
      std::sync::Arc<tao::window::Window>,
    >,
    background_color: Option<tao::window::RGBA>,
  );
}

pub fn calculate_window_center_position(
  window_size: tao::dpi::PhysicalSize<u32>,
  target_monitor: tao::monitor::MonitorHandle,
) -> tao::dpi::PhysicalPosition<i32> {
  let work_area = target_monitor.work_area();

  tao::dpi::PhysicalPosition::new(
    (work_area.size.width as i32 - window_size.width as i32) / 2
      + work_area.position.x,
    (work_area.size.height as i32 - window_size.height as i32) / 2
      + work_area.position.y,
  )
}

// ─────────────────────────────────────────────
// Public dependency re-exports
// ─────────────────────────────────────────────

pub use {
  anyhow,
  getrandom,
  log,
  serde,
  tao,
};

// These names would conflict with our own `dpi` and `image` modules.
pub use ::dpi as dpi_crate;
pub use ::image as image_crate;

// ─────────────────────────────────────────────
// Windows
// ─────────────────────────────────────────────

#[cfg(windows)]
pub use {
  once_cell,
  softbuffer,
  windows,
};

// ─────────────────────────────────────────────
// macOS
// ─────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub use {
  objc2,
  objc2_app_kit,
  
};

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
pub use {
  gtk,
  webkit2gtk,
};






#[macro_export]
macro_rules! lock {
    ($value:expr) => {
        $value.lock().map_err(|_| {
            anyhow::anyhow!("Failed to lock {}.", stringify!($value))
        })
    };
}

#[macro_export]
macro_rules! lock_force {
    ($value:expr) => {
        $value.lock().unwrap()
    };
}
