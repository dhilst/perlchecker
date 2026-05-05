pub const DEFAULT_MAX_LOOP_UNROLL: usize = 9;
pub const DEFAULT_MAX_PATHS: usize = 1024;
pub const DEFAULT_SOLVER_TIMEOUT_MS: u32 = 10000;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_loop_unroll: usize,
    pub max_paths: usize,
    pub solver_timeout_ms: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_loop_unroll: DEFAULT_MAX_LOOP_UNROLL,
            max_paths: DEFAULT_MAX_PATHS,
            solver_timeout_ms: DEFAULT_SOLVER_TIMEOUT_MS,
        }
    }
}
