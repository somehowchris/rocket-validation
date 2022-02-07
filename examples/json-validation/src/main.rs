#![allow(clippy::cargo)]

#[macro_use]
extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_validation::{Validate, Validated};

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(crate = "rocket::serde")]
pub struct HelloData {
	#[validate(length(min = 1))]
	name: String,
	#[validate(range(min = 0, max = 100))]
	age: u8,
}

#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> Json<HelloData> {
	Json(HelloData { name, age })
}

#[post("/hello", format = "application/json", data = "<data>")]
fn validated_hello(data: Validated<Json<HelloData>>) -> Json<HelloData> {
	Json(data.0 .0)
}

#[launch]
fn rocket() -> _ {
	rocket::build()
		.mount("/", routes![hello, validated_hello])
		.register("/", catchers![rocket_validation::validation_catcher])
}
