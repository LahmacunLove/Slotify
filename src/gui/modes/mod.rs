pub mod dj_mode;
pub mod guest_mode;
pub mod admin_mode;

pub use dj_mode::DjMode;
pub use guest_mode::GuestMode;
pub use admin_mode::AdminMode;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    DJ,
    Guest,
    Admin,
}