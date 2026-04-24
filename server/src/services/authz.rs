#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpScope {
    WorkoutsRead,
    ProgressRead,
    WorkoutsWrite,
}

pub const ALLOWED_MCP_SCOPES: &[&str] = &[
    McpScope::WorkoutsRead.as_str(),
    McpScope::ProgressRead.as_str(),
    McpScope::WorkoutsWrite.as_str(),
];

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

pub fn all_scopes_supported(scopes: &[String]) -> bool {
    scopes
        .iter()
        .all(|scope| ALLOWED_MCP_SCOPES.iter().any(|allowed| allowed == scope))
}
