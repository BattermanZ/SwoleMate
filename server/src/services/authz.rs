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

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "workouts.read" => Some(Self::WorkoutsRead),
            "progress.read" => Some(Self::ProgressRead),
            "workouts.write" => Some(Self::WorkoutsWrite),
            _ => None,
        }
    }
}

pub fn normalize_scopes(raw: &str) -> Vec<String> {
    raw.split([' ', ','])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub fn has_scope(granted: &[String], required: McpScope) -> bool {
    granted.iter().any(|scope| scope == required.as_str())
}
