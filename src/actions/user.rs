use crate::models::user::*;
use crate::pub_imports::*;
use diesel::{prelude::*, PgConnection};

pub fn add_user(
    model: &RegisterFormModel,
    conn: &PgConnection,
) -> Result<DbUserModel, UserApiError> {
    if model.name.len() < 4 {
        println!("asdasd");
        return Err(UserApiError::UsernameTooShort);
    } else if model.password != model.confirm_password {
        return Err(UserApiError::MismatchedPasswords);
    } else if model.password.len() < 6 {
        return Err(UserApiError::PasswordTooShort);
    }

    use schema::users::dsl::*;

    if let Some(_) = users
        .filter(name.eq(&model.name))
        .first::<DbUserModel>(conn)
        .optional()
        .unwrap()
    {
        Err(UserApiError::UserExists)
    } else {
        let new_user = NewUserModel {
            name: &model.name,
            pwhs: &bcrypt::hash(&model.password, bcrypt::DEFAULT_COST).unwrap(),
            class_id: {
                Some(
                    super::class::get_or_make_class(&model.class_name, conn)
                        .unwrap()
                        .id,
                )
            },
        };

        match diesel::insert_into(users)
            .values(new_user)
            .get_result::<DbUserModel>(conn)
        {
            Ok(u) => Ok(u),
            Err(e) => Err(e.into()),
        }
    }
}

pub fn login(model: &LoginFormModel, conn: &PgConnection) -> Result<DbUserModel, UserApiError> {
    use schema::users::dsl::*;

    if let Some(user) = users
        .filter(name.eq(&model.name))
        .first::<DbUserModel>(conn)
        .optional()
        .unwrap()
    {
        if bcrypt::verify(&model.password, &user.pwhs).unwrap() {
            Ok(user)
        } else {
            Err(UserApiError::WrongPassword)
        }
    } else {
        Err(UserApiError::UserNotFound)
    }
}

#[derive(Debug)]
pub enum UserApiError {
    UserExists,
    UserNotFound,
    WrongPassword,
    MismatchedPasswords,
    PasswordTooShort,
    UsernameTooShort,
    DieselError(diesel::result::Error),
}
impl std::fmt::Display for UserApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self)).unwrap();
        Ok(())
    }
}
impl std::error::Error for UserApiError {}
impl From<diesel::result::Error> for UserApiError {
    fn from(e: diesel::result::Error) -> Self {
        UserApiError::DieselError(e)
    }
}
