use crate::models::class::*;
use crate::pub_imports::*;
use diesel::{prelude::*, PgConnection};

pub fn get_or_make_class(
    get_name: &str,
    conn: &PgConnection,
) -> Result<DbClassModel, ClassApiError> {
    use schema::classes::dsl::*;

    match classes
        .filter(name.eq(get_name))
        .first::<DbClassModel>(conn)
        .optional()
    {
        Ok(opt) => {
            if let Some(model) = opt {
                Ok(model)
            } else {
                let x = diesel::insert_into(classes)
                    .values(NewClass { name: &get_name })
                    .get_result(conn)
                    .unwrap();
                Ok(x)
            }
        }
        Err(e) => Err(ClassApiError::DieselError(e)),
    }
}

pub fn get_class_by_id(
    get_id: i32,
    conn: &PgConnection,
) -> Result<Option<DbClassModel>, ClassApiError> {
    use schema::classes::dsl::*;

    match classes
        .filter(id.eq(get_id))
        .first::<DbClassModel>(conn)
        .optional()
    {
        Ok(opt) => Ok(opt),
        Err(e) => Err(ClassApiError::DieselError(e)),
    }
}

pub fn set_blackboard(
    get_id: i32,
    new_blackboard: String,
    conn: &PgConnection,
) -> Result<(), ClassApiError> {
    use schema::classes::dsl::*;

    let mut class = classes
        .filter(id.eq(get_id))
        .first::<DbClassModel>(conn)
        .map_err(ClassApiError::DieselError)?;

    class.blackboard = new_blackboard;

    class.save_changes::<DbClassModel>(conn).unwrap();
    Ok(())
}

#[derive(Debug)]
pub enum ClassApiError {
    DieselError(diesel::result::Error),
}
impl std::fmt::Display for ClassApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self)).unwrap();
        Ok(())
    }
}
impl std::error::Error for ClassApiError {}
impl From<diesel::result::Error> for ClassApiError {
    fn from(e: diesel::result::Error) -> Self {
        ClassApiError::DieselError(e)
    }
}
