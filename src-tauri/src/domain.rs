use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub notes: Option<String>,
    pub status: String, // "todo" | "doing" | "done"
    pub due_at: Option<i64>,
    pub remind_at: Option<i64>,
    pub notified: bool,
    pub group_id: Option<i64>,
    pub sort_order: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    // Recurrence
    pub recur_type: Option<String>,       // "daily"|"weekly"|"monthly"|"yearly"
    pub recur_interval: Option<i64>,      // 1=毎週 2=隔週
    pub recur_weekdays: Option<Vec<u8>>,  // [0-6], 0=Sun
    pub recur_month_rule: Option<String>, // "day"|"last_day"|"last_weekday"
    pub recur_month_day: Option<i64>,     // 1-31
    // Priority: 0=none 1=low 2=medium 3=high
    pub priority: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskWithTags {
    #[serde(flatten)]
    pub task: Task,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTask {
    pub title: String,
    pub notes: Option<String>,
    pub due_at: Option<i64>,
    pub remind_at: Option<i64>,
    pub group_id: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
    pub new_tag_names: Option<Vec<String>>,
    // Recurrence
    pub recur_type: Option<String>,
    pub recur_interval: Option<i64>,
    pub recur_weekdays: Option<Vec<u8>>,
    pub recur_month_rule: Option<String>,
    pub recur_month_day: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: String,
    pub notes: Option<String>,
    pub due_at: Option<i64>,
    pub remind_at: Option<i64>,
    pub group_id: Option<i64>,
    // Recurrence
    pub recur_type: Option<String>,
    pub recur_interval: Option<i64>,
    pub recur_weekdays: Option<Vec<u8>>,
    pub recur_month_rule: Option<String>,
    pub recur_month_day: Option<i64>,
    pub priority: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: f64,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}
