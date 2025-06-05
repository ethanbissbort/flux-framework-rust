pub mod file_ops;
pub mod logging;
pub mod network;
pub mod system;
pub mod user_input;
pub mod validation;

// Re-export commonly used functions
pub use file_ops::{backup_file, safe_write_file};
pub use logging::{log_debug, log_error, log_info, log_warn};
pub use system::{detect_distro, has_systemd, is_root};
pub use user_input::{prompt_yes_no, prompt_with_default};
pub use validation::{validate_hostname, validate_ip, validate_port};