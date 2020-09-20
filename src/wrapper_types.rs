use rocket::{http::RawStr, request::FromFormValue};

pub trait RouteVec {
    fn set_root(self, base: &str) -> Self;
    fn add(self, other: Self) -> Self;
}

impl RouteVec for Vec<rocket::Route> {
    fn set_root(self, root: &str) -> Self {
        self.into_iter()
            .map(|r| r.map_base(|base| format!("{}{}", root, base)).unwrap())
            .collect()
    }

    fn add(mut self, mut other: Self) -> Self {
        self.append(&mut other);
        self
    }
}

#[derive(Copy, Clone)]
pub struct FormDate(pub chrono::NaiveDate);

impl<'v> FromFormValue<'v> for FormDate {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<FormDate, &'v RawStr> {
        match chrono::NaiveDate::parse_from_str(form_value, "%Y-%m-%d") {
            Ok(age) => Ok(FormDate(age)),
            _ => Err(form_value),
        }
    }
}
