#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Open,
    Routed,
    Designed,
    Implemented,
    Reviewed,
    ReadyToMerge,
    Merged,
    Blocked,
}

impl TaskState {
    pub fn can_transition(self, next: TaskState) -> bool {
        use TaskState::*;
        match (self, next) {
            (Open, Routed) => true,
            (Routed, Designed) => true,
            (Designed, Implemented) => true,
            (Implemented, Reviewed) => true,
            (Reviewed, ReadyToMerge) => true,
            (ReadyToMerge, Merged) => true,
            (s, Blocked) if s != Merged => true,
            (Blocked, Routed) | (Blocked, Designed) | (Blocked, Implemented) => true,
            (Blocked, Reviewed) | (Blocked, ReadyToMerge) => true,
            (Reviewed, Designed) | (Reviewed, Implemented) => true,
            _ => false,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TaskState::Open => "open",
            TaskState::Routed => "routed",
            TaskState::Designed => "designed",
            TaskState::Implemented => "implemented",
            TaskState::Reviewed => "reviewed",
            TaskState::ReadyToMerge => "ready_to_merge",
            TaskState::Merged => "merged",
            TaskState::Blocked => "blocked",
        }
    }
}
