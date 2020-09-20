use crate::{auth_guard::AuthUser, schema::users};
use serde::{Serialize, Deserialize};

#[derive(FromForm)]
pub struct RegisterFormModel {
    pub name: String,
    pub password: String,
    pub confirm_password: String,
    pub class_name: String,
}

#[derive(FromForm)]
pub struct LoginFormModel {
    pub name: String,
    pub password: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUserModel<'a> {
    pub name: &'a str,
    pub pwhs: &'a str,
    pub class_id: Option<i32>,
}

#[derive(Identifiable, Queryable)]
#[table_name = "users"]
pub struct DbUserModel {
    pub id: i32,
    pub name: String,
    pub pwhs: String,
    pub class_id: Option<i32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: i32,
    pub class_name: Option<String>,
    pub class_id: Option<i32>,
}

impl From<AuthUser> for User {
    fn from(u: AuthUser) -> Self {
        Self {
            name: u.name,
            id: u.id,
            class_name: u.class_name,
            class_id: u.class_id,
        }
    }
}