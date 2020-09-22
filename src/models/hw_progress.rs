use crate::pub_imports::*;
use schema::hw_progress;

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[primary_key(homework_id, user_id)]
#[table_name = "hw_progress"]
pub struct HWProgressModel {
    pub user_id: i32,
    pub homework_id: i32,
    pub progress: i16,
    pub delta: i16,
    pub delta_date: chrono::NaiveDate,
}
