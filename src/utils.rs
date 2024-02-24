use rocket::{http::Status, response::Responder};
use serde::Serializer;
use time::{
    format_description::well_known::Rfc3339, macros::format_description, Date, OffsetDateTime,
};

pub(crate) fn to_isoish_time<S>(dt: &OffsetDateTime, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(
        &dt.format(&Rfc3339)
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
    )
}

pub(crate) fn to_isoish_date<S>(dt: &Date, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(
        &dt.format(format_description!("[year]-[month]-[day]"))
            .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
    )
}

pub(crate) type Result<T, U = MyError> = core::result::Result<T, U>;
pub(crate) struct MyError(pub(crate) color_eyre::Report);
impl<T: Into<color_eyre::Report>> From<T> for MyError {
    fn from(value: T) -> Self {
        MyError(value.into())
    }
}
impl<'r, 'o: 'r> Responder<'r, 'o> for MyError {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::InternalServerError)
    }
}
