use crate::schema::classes;

#[derive(Insertable)]
#[table_name = "classes"]
pub struct NewClass<'a> {
    pub name: &'a str,
}

#[derive(Identifiable, Queryable)]
#[table_name = "classes"]
pub struct DbClassModel {
    pub id: i32,
    pub name: String,
}
