use std::env;

use cli_log::debug;

/// Determine whether we're in tmux.
///
/// This is called only once, and cached in `KittyImageRenderer`
#[allow(unreachable_code)]
pub fn is_tmux() -> bool {
    debug!("is_tmux ?");

    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            debug!("${env_var} = {env_val:?}");
            let env_val = env_val.to_ascii_lowercase();
            if env_val.contains("tmux") {
                debug!(" -> this terminal seems to be Tmux");
                return true;
            }
        }
    }
    false
}

/// Custom environment variable to store how deeply tmux is nested. Starts at 1 when there's no nesting.
pub fn get_tmux_nest_count() -> u32 {
    std::env::var("TMUX_NEST_COUNT")
        .map(|s| str::parse(&s).unwrap_or(1))
        .unwrap_or(1)
}

/// Determine whether we're in SSH.
///
/// This is called only once, and cached in `KittyImageRenderer`
#[allow(unreachable_code)]
pub fn is_ssh() -> bool {
    debug!("is_ssh ?");

    for env_var in ["SSH_CLIENT", "SSH_CONNECTION"] {
        if env::var(env_var).is_ok() {
            debug!(" -> this seems to be under SSH");
            return true;
        }
    }
    false
}
