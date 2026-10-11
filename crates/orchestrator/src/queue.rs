use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub id: String,
    pub title: String,
    pub body: String,
    pub paths: Vec<String>,
}

pub struct Queue {
    q: VecDeque<QueuedTask>,
}

impl Queue {
    pub fn new() -> Self {
        Self { q: VecDeque::new() }
    }

    pub fn push(&mut self, t: QueuedTask) {
        self.q.push_back(t);
    }

    pub fn claim(&mut self) -> Option<QueuedTask> {
        self.q.pop_front()
    }
}
