//! What touches the system around an app: its window, app icons from
//! desktop entries, the shell's settings file and scoped redraws.

pub mod app;
pub mod app_icon;
pub mod launch;
pub mod marker;
pub mod process;
pub mod secrets;
pub mod sensors;
pub mod service;
pub mod settings;
pub mod shell_ipc;
pub mod sounds;
pub mod usb_c;
