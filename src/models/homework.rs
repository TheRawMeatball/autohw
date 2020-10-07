use std::cmp::Ordering;

use crate::pub_imports::*;
use schema::homework;
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub struct AddHomeworkModel {
    pub due_date: Option<FormDate>,
    pub detail: String,
    pub amount: i16,
    pub weekday: Option<i32>,
    pub for_self: bool,
}

#[derive(FromForm)]
pub struct SetWeightModel {
    pub id: i32,
    pub weight: i32,
}

#[derive(Insertable)]
#[table_name = "homework"]
pub struct NewHomeworkModel<'a> {
    pub due_date: Option<chrono::NaiveDate>,
    pub day_of_week: Option<i32>,
    pub detail: &'a str,
    pub amount: i16,
    pub class_id: Option<i32>,
    pub user_id: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct DbHomeworkModel {
    pub id: i32,
    pub due_date: Option<chrono::NaiveDate>,
    pub detail: String,
    pub amount: i16,
    pub day_of_week: Option<i32>,
    pub class_id: Option<i32>,
    pub user_id: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct ProgressHomeworkModel {
    pub id: i32,
    pub due_date: Option<chrono::NaiveDate>,
    pub detail: String,
    pub amount: i16,
    pub day_of_week: Option<i32>,
    pub progress: i16,
    pub delta_progress: i16,
    pub delta_date: chrono::NaiveDate,
    pub weight: i32,
    pub last_repeat_reset: Option<chrono::NaiveDate>,
}

#[derive(Serialize, Clone, Debug)]
pub struct UserHomework {
    pub due_date: DueDate,
    pub amount: i16,
    pub progress: i16,
    pub detail: String,
    /// For `homework` table
    pub db_id: i32,
    pub delta: i16,
    pub weight: i32,
}

#[derive(Serialize, Debug)]
pub struct DailyHomework {
    pub hw: UserHomework,
    pub amount: i32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DueDate {
    Date(chrono::NaiveDate),
    Repeat(i32),
}

impl From<ProgressHomeworkModel> for UserHomework {
    fn from(m: ProgressHomeworkModel) -> Self {
        Self {
            amount: m.amount,
            db_id: m.id,
            detail: m.detail,
            progress: m.progress,
            delta: m.delta_progress,
            due_date: match m.due_date {
                Some(d) => DueDate::Date(d),
                None => DueDate::Repeat(m.day_of_week.unwrap()),
            },
            weight: m.weight,
        }
    }
}

impl PartialOrd for DueDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (DueDate::Repeat(_), DueDate::Date(_)) => Some(Ordering::Less),
            (DueDate::Date(_), DueDate::Repeat(_)) => Some(Ordering::Greater),
            (DueDate::Repeat(d1), DueDate::Repeat(d2)) => d1.partial_cmp(d2),
            (DueDate::Date(d1), DueDate::Date(d2)) => d1.partial_cmp(d2),
        }
    }
}

impl Ord for DueDate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
