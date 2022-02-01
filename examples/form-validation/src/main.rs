#![allow(clippy::cargo)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_validation;

use rocket::{
	form::Form,
	serde::{json::Json, Deserialize, Serialize},
};
use rocket_validation::Validated;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Validate, FromForm)]
#[serde(crate = "rocket::serde")]
struct HelloData<'a> {
	#[validate(length(min = 3))]
	name: &'a str,
	#[validate(range(min = 0, max = 100))]
	age: u8,
}

#[get("/hello?<name>&<age>")]
fn hello(name: &'_ str, age: u8) -> Json<HelloData> {
	Json(HelloData { name, age })
}

#[post("/hello", data = "<data>")]
fn validated_hello(data: Form<Validated<HelloData>>) -> Json<HelloData> {
	Json(data.0)
}

#[launch]
fn rocket() -> _ {
	rocket::build().mount("/", routes![hello, validated_hello])
}
