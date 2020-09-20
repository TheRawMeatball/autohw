use crate::pub_imports::*;
use models::class::DbClassModel;
use models::user::DbUserModel;
use schema::homework;
use serde::{Serialize, Deserialize};

#[derive(FromForm)]
pub struct AddHomeworkModel {
    pub due_date: Option<FormDate>,
    pub detail: String,
    pub amount: i16,
    pub weekday: Option<i32>,
    pub for_self: bool,
}

#[derive(Insertable)]
#[table_name = "homework"]
pub struct NewHomeworkModel<'a> {
    pub due_date: Option<chrono::NaiveDate>,
    pub weekday: Option<i32>,
    pub detail: &'a str,
    pub amount: i16,
    pub class_id: Option<i32>,
    pub user_id: Option<i32>,
}

#[derive(Associations, Identifiable, Queryable, Serialize, Deserialize)]
#[belongs_to(parent = "DbUserModel", foreign_key = "user_id")]
#[belongs_to(parent = "DbClassModel", foreign_key = "class_id")]
#[table_name = "homework"]
pub struct DbHomeworkModel {
    pub id: i32,
    pub due_date: Option<chrono::NaiveDate>,
    pub detail: String,
    pub amount: i16,
    pub weekday: Option<i32>,
    pub class_id: Option<i32>,
    pub user_id: Option<i32>,
}
