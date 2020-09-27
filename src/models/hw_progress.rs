use crate::pub_imports::*;
use schema::hw_progress;

#[derive(Insertable, Queryable, Identifiable, AsChangeset, Debug)]
#[primary_key(homework_id, user_id)]
#[table_name = "hw_progress"]
pub struct HWProgressModel {
    pub homework_id: i32,
    pub user_id: i32,
    pub progress: i16,
    pub delta: i16,
    pub delta_date: chrono::NaiveDate,
    pub weight: i32,
    pub last_repeat_reset: Option<chrono::NaiveDate>,
}

#[derive(FromForm)]
pub struct ChangeProgressModel {
    pub id: i32,
    pub amount: i16,
    pub use_delta: bool,
}
