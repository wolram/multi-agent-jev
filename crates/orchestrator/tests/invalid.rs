use orchestrator::state::TaskState;

#[test]
fn invalid_transition_fails() {
    assert!(!TaskState::Open.can_transition(TaskState::Designed));
    assert!(TaskState::Open.can_transition(TaskState::Routed));
    assert!(!TaskState::Merged.can_transition(TaskState::Routed));
}
