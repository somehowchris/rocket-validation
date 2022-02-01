#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]

#[allow(unused_imports)]
#[macro_use]
pub extern crate validator;
pub use validator::{Validate, ValidationErrors};

use rocket::{
	data::{Data, FromData, Outcome as DataOutcome},
	form,
	form::{DataField, FromForm, ValueField},
	http::Status,
	outcome::Outcome,
	request::{FromRequest, Request},
	serde::{json::Json, Deserialize},
};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Validated<T>(pub T);

impl<T> Validated<T> {
	#[inline]
	pub fn into_inner(self) -> T {
		self.0
	}
}

#[inline]
fn match_outcome<L: Validate>(data: &L) -> Result<(), (Status, ValidationErrors)> {
	let validation_outcome = data.validate();

	match validation_outcome {
		Ok(_) => Ok(()),
		Err(err) => Err((Status::BadRequest, err)),
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
				Err(err) => Err(err
					.1
					.into_errors()
					.into_iter()
					.map(|e| form::Error {
						name: Some(e.0.to_owned().into()),
						kind: form::error::ErrorKind::Validation(std::borrow::Cow::Borrowed(e.0)),
						value: None,
						entity: form::error::Entity::Value,
					})
					.collect::<Vec<_>>()
					.into()),
			},
		}
	}
}
