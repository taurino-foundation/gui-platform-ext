#[cfg_attr(not(windows), allow(unused_imports))]
pub use imp::*;

#[cfg(not(windows))]
mod imp {}

#[cfg(windows)]
mod imp {
  use std::{iter::once, os::windows::ffi::OsStrExt};

  use once_cell::sync::Lazy;
  use windows::{
    core::{HRESULT, PCSTR, PCWSTR},
    Win32::{
      Foundation::*,
      Graphics::Gdi::*,
      System::LibraryLoader::{GetProcAddress, LoadLibraryW},
      UI::{HiDpi::*, WindowsAndMessaging::*},
    },
  };

  /// Encodes an OS string as a zero-terminated UTF-16 string.
  pub fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(once(0)).collect()
  }

  /// Dynamically loads a function from a Windows DLL.
  ///
  /// `function` must be zero-terminated. `library` is converted to a
  /// zero-terminated UTF-16 string by this function.
  pub(super) fn get_function_impl(library: &str, function: &str) -> FARPROC {
    let library = encode_wide(library);

    assert_eq!(
      function.as_bytes().last(),
      Some(&0),
      "function name must be zero-terminated"
    );

    // The module intentionally remains loaded for the lifetime of the process,
    // because function pointers obtained from it are stored in static Lazy values.
    let module =
      unsafe { LoadLibraryW(PCWSTR::from_raw(library.as_ptr())) }.unwrap_or_default();

    if module.is_invalid() {
      return None;
    }

    unsafe { GetProcAddress(module, PCSTR::from_raw(function.as_ptr())) }
  }

  macro_rules! get_function {
    ($lib:expr, $func:ident) => {
      $crate::util::get_function_impl($lib, concat!(stringify!($func), '\0'))
        .map(|function| unsafe { std::mem::transmute::<_, $func>(function) })
    };
  }

  type GetDpiForWindow = unsafe extern "system" fn(hwnd: HWND) -> u32;

  type GetDpiForMonitor = unsafe extern "system" fn(
    hmonitor: HMONITOR,
    dpi_type: MONITOR_DPI_TYPE,
    dpi_x: *mut u32,
    dpi_y: *mut u32,
  ) -> HRESULT;

  type GetSystemMetricsForDpi =
    unsafe extern "system" fn(nindex: SYSTEM_METRICS_INDEX, dpi: u32) -> i32;

  static GET_DPI_FOR_WINDOW: Lazy<Option<GetDpiForWindow>> =
    Lazy::new(|| get_function!("user32.dll", GetDpiForWindow));

  static GET_DPI_FOR_MONITOR: Lazy<Option<GetDpiForMonitor>> =
    Lazy::new(|| get_function!("shcore.dll", GetDpiForMonitor));

  static GET_SYSTEM_METRICS_FOR_DPI: Lazy<Option<GetSystemMetricsForDpi>> =
    Lazy::new(|| get_function!("user32.dll", GetSystemMetricsForDpi));

  /// Returns the effective DPI for the specified window.
  ///
  /// The implementation selects the newest DPI API supported by the running
  /// Windows version and falls back to older APIs when necessary.
  pub fn hwnd_dpi(hwnd: HWND) -> u32 {
    unsafe {
      if let Some(get_dpi_for_window) = *GET_DPI_FOR_WINDOW {
        // Windows 10 Anniversary Update (1607) or later.
        let dpi = get_dpi_for_window(hwnd);

        if dpi == 0 {
          // GetDpiForWindow returns 0 for an invalid HWND.
          USER_DEFAULT_SCREEN_DPI
        } else {
          dpi
        }
      } else if let Some(get_dpi_for_monitor) = *GET_DPI_FOR_MONITOR {
        // Windows 8.1 or later.
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

        if monitor.is_invalid() {
          return USER_DEFAULT_SCREEN_DPI;
        }

        let mut dpi_x = 0;
        let mut dpi_y = 0;

        if get_dpi_for_monitor(
          monitor,
          MDT_EFFECTIVE_DPI,
          &mut dpi_x,
          &mut dpi_y,
        )
        .is_ok()
          && dpi_x != 0
        {
          dpi_x
        } else {
          USER_DEFAULT_SCREEN_DPI
        }
      } else if IsProcessDPIAware().as_bool() {
        // Vista or later.
        //
        // For a DPI-aware process, the application is responsible for
        // performing the corresponding scaling.
        let hdc = GetDC(Some(hwnd));

        if hdc.is_invalid() {
          return USER_DEFAULT_SCREEN_DPI;
        }

        let dpi = GetDeviceCaps(Some(hdc), LOGPIXELSX);

        let _ = ReleaseDC(Some(hwnd), hdc);

        if dpi > 0 {
          dpi as u32
        } else {
          USER_DEFAULT_SCREEN_DPI
        }
      } else {
        // For a DPI-unaware process Windows performs the scaling itself.
        // Returning 96 prevents the application from applying scaling again.
        USER_DEFAULT_SCREEN_DPI
      }
    }
  }

  /// Returns a system metric adjusted for the requested DPI when supported.
  ///
  /// On Windows versions without `GetSystemMetricsForDpi`, this falls back to
  /// the legacy `GetSystemMetrics` API.
  pub fn get_system_metrics_for_dpi(
    nindex: SYSTEM_METRICS_INDEX,
    dpi: u32,
  ) -> i32 {
    unsafe {
      if let Some(get_system_metrics_for_dpi) = *GET_SYSTEM_METRICS_FOR_DPI {
        get_system_metrics_for_dpi(nindex, dpi)
      } else {
        GetSystemMetrics(nindex)
      }
    }
  }
}