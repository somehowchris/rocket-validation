# Rocket Validation

Welcome to the Rocket Validation crate. If you are looking to validate your Json, Form or Query structs using Rocket you have come to the right place!
## Why
Rocket is using Rusts powerful typing system. Which is amazing because you can be sure its what you want. But is it? How about kebab-case strings or phone number inputs, these aren’t really types.
You could implement a [custom deserializer](https://docs.serde.rs/serde/de/trait.Deserializer.html) for a wrapped type or write custom logic to validate it on endpoint calls, thats error prone and not ergonomic and doesn't allow you to return meaningful and contextual errors.

If you are coming from TypeScript you might have heard of [class-validator](https://github.com/typestack/class-validator) which is simple, declarative and can be implemented into middleware. Using [validator](https://github.com/Keats/validator) this crate achieves a similar result using rockets [guard](https://rocket.rs/v0.5-rc/guide/requests/#request-guards) mechanism.
> Anything implementing [Json](https://rocket.rs/v0.5-rc/guide/requests/#json), [FromRequest](https://rocket.rs/v0.5-rc/guide/requests/#custom-guards) or [FromForm](https://rocket.rs/v0.5-rc/guide/requests/#forms) as well as [`Validate`](https://docs.rs/validator/latest/validator/#example) are able to use the `Validated` guard of this crate, so you can be sure your data is validated once you receive it in your handler.

> Using rockets [catchers](https://rocket.rs/v0.5-rc/guide/requests/#error-catchers) you are able to route errors which occurs during validation to your user.

Current validation in rocket: Rocket has validation for FromForm structs but for nothing else.

## Usage

In order to get going, you need to depend on the `rocket-validation`.
Add this to your `Cargo.toml`
```toml
[dependencies]
rocket-validation = "0.1.0"
```

Now you can go on and implement your Validation
```rust
// Because we use rocket....
#[macro_use]
extern crate rocket;

// Some types for Json types
use rocket::serde::{json::Json, Deserialize, Serialize};

// Will be important for validation....
use rocket_validation::{Validate, Validated};

#[derive(Debug, Deserialize, Serialize, Validate)] // Implements `Validate`
#[serde(crate = "rocket::serde")]
pub struct HelloData {
	#[validate(length(min = 1))] // Your validation annotation
	name: String,
	#[validate(range(min = 0, max = 100))] // Your validation annotation
	age: u8,
}

#[post("/hello", format = "application/json", data = "<data>")]
fn validated_hello(data: /*Uses the `Validated` type*/ Validated<Json<HelloData>>) -> Json<HelloData> {
	Json(data.into_inner())
}

#[launch]
fn rocket() -> _ {
	rocket::build()
		.mount("/", routes![validated_hello])
}
```

### Exposing errors to clients

> Before you use the following, you should be aware of what errors you expose to your clients as well as what that means for security.

If you would like to respond invalid requests with some custom messages, you can implement the `validation_catcher` catcher to do so.
```rust
#[launch]
fn rocket() -> _ {
	rocket::build()
		.mount("/", routes![validated_hello])
		.register("/", catchers![rocket_validation::validation_catcher])
}
```