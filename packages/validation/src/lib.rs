use rocket::data::Outcome as DataOutcome;
use rocket::data::{Data, FromData};

use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use std::fmt::Debug;
use validator::{Validate, ValidationErrors};

pub extern crate validator;

#[derive(Debug)]
pub struct Validated<T>(pub T);

fn match_outcome<L: Validate>(data: &L) -> Result<(), (Status, ValidationErrors)> {
    let validation_outcome = data.validate();

    match validation_outcome {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("{:?}", err);
            Err((Status::BadRequest, err))
        }
    }
}

#[rocket::async_trait]
impl<'r, D: Validate + Deserialize<'r>> FromData<'r> for Validated<Json<D>> {
    type Error = Result<ValidationErrors, <Json<D> as rocket::data::FromData<'r>>::Error>;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> DataOutcome<'r, Self> {
        let data_outcome: Outcome<Json<D>, (Status, <Json<D> as FromData>::Error), Data> =
            <Json<D> as FromData<'r>>::from_data(req, data).await;

        match data_outcome {
            Outcome::Failure((status, err)) => Outcome::Failure((status, Err(err))),
            Outcome::Forward(err) => Outcome::Forward(err),
            Outcome::Success(data) => match match_outcome(&data.0) {
                Ok(_) => Outcome::Success(Validated(data)),
                Err((status, err)) => Outcome::Failure((status, Ok(err))),
            },
        }
    }
}

#[rocket::async_trait]
impl<'r, D: Validate + FromRequest<'r>> FromRequest<'r> for Validated<D> {
    type Error = Result<ValidationErrors, D::Error>;
    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let data_outcome = D::from_request(req).await;

        match data_outcome {
            Outcome::Failure((status, err)) => Outcome::Failure((status, Err(err))),
            Outcome::Forward(err) => Outcome::Forward(err),
            Outcome::Success(data) => match match_outcome(&data) {
                Ok(_) => Outcome::Success(Validated(data)),
                Err((status, err)) => Outcome::Failure((status, Ok(err))),
            },
        }
    }
}

use rocket::form;
use rocket::form::{DataField, FromForm, ValueField};

/* #[rocket::async_trait]
impl<'r, D: Validate + FromFormField<'r>> FromFormField<'r> for Validated<D> {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match D::from_value(field) {
            Ok(data) => {
                match match_outcome(&data) {
                    Ok(_) => Ok(Validated(data)),
                    Err(err) => {Err(form::Errors::new())},
                }
            },
            Err(err) => Err(err),
        }
    }

    async fn from_data(field: DataField<'r, '_>) -> form::Result<'r, Self> {
        match D::from_data(field).await {
            Ok(data) => {
                match match_outcome(&data) {
                    Ok(_) => Ok(Validated(data)),
                    Err(err) => {Err(form::Errors::new())},
                }
            },
            Err(err) => Err(err),
        }
    }
}
 */

#[rocket::async_trait]
impl<'r, T: Validate + FromForm<'r>> FromForm<'r> for Validated<T> {
    type Context = T::Context;

    fn init(opts: form::Options) -> Self::Context {
        T::init(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: ValueField<'r>) {
        T::push_value(ctxt, field)
    }

    async fn push_data(ctxt: &mut Self::Context, field: DataField<'r, '_>) {
        T::push_data(ctxt, field).await
    }

    fn finalize(this: Self::Context) -> form::Result<'r, Self> {
        let final_data = T::finalize(this);

        match final_data {
            Err(err) => Err(err),
            Ok(data) => match match_outcome(&data) {
                Ok(_) => Ok(Validated(data)),
                Err(_err) => Err(form::Errors::new()),
            },
        }
    }
}
