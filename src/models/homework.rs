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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
