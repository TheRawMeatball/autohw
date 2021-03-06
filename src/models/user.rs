use crate::{auth_guard::AuthUser, schema::users};
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub struct RegisterFormModel {
    pub name: String,
    pub password: String,
    pub confirm_password: String,
    class: i32,
    class_name: String,
}

impl RegisterFormModel {
    pub fn cname(&self) -> String {
        format!("{}-{}", self.class, self.class_name.clone())
    }
}

#[derive(FromForm, Debug)]
pub struct ChangeFormModel {
    pub name: String,
    pub password: String,
    pub confirm_password: String,
    class: i32,
    class_name: String,
    mo: u32,
    tu: u32,
    we: u32,
    th: u32,
    fr: u32,
    sa: u32,
    su: u32,
}

impl ChangeFormModel {
    pub fn cname(&self) -> String {
        format!("{}-{}", self.class, self.class_name.clone())
    }
}

impl ChangeFormModel {
    pub fn weights(&self) -> Vec<u32> {
        vec![
            self.mo, self.tu, self.we, self.th, self.fr, self.sa, self.su,
        ]
    }
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

#[derive(Identifiable, Queryable, AsChangeset)]
#[table_name = "users"]
pub struct DbUserModel {
    pub id: i32,
    pub name: String,
    pub pwhs: String,
    pub class_id: Option<i32>,
    pub day_weights: Vec<i32>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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
