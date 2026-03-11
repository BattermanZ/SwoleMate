#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpScope {
    WorkoutsRead,
    ProgressRead,
    WorkoutsWrite,
}

impl McpScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkoutsRead => "workouts.read",
            Self::ProgressRead => "progress.read",
            Self::WorkoutsWrite => "workouts.write",
        }
    }
}
