use crate::error_handling::{BadInputErrorHandler, Handle};
use crate::{EMAIL_VERIFICATION, PASSWORD_VERIFICATION, USERNAME_VERIFICATION};
use async_graphql::{Context, Error, InputObject, Object, Result};
use sea_orm::{entity::Set, ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter};
use tracing::{error, info};
#[derive(Default)]
pub struct UserQuery;

#[derive(InputObject)]
pub struct LoginInput {
    account: String,
    password: String,
}

#[Object]
impl UserQuery {
    async fn log_in(&self, ctx: &Context<'_>, input: LoginInput) -> Result<String> {
        info!("Query.UserQuery.logIn accepted one request");
        let db = ctx.data_unchecked::<DbConn>();
        let mut bad_input_error_handler = ctx.data_unchecked::<BadInputErrorHandler>().clone();
        let user = crate::users::Entity::find()
            .filter(
                Condition::any()
                    .add(crate::users::Column::Username.eq(input.account.clone()))
                    .add(crate::users::Column::Email.eq(input.account.clone())),
            )
            .one(db)
            .await
            .expect("failed to query database");
        if let Some(user) = user {
            if user.password == input.password {
                info!("Query.UserQuery.logIn send a response token: fake token");
                return Ok("fake token".to_string());
            } else {
                error!("bad input: wrong password");
                bad_input_error_handler.append("password", "wrong password");
            }
        } else {
            error!("bad input: user not found");
            bad_input_error_handler.append("account", "user not found");
        }
        if !bad_input_error_handler.is_none() {
            return Err(bad_input_error_handler.to_err());
        } else {
            return Err(Error::new("unexpected error"));
        }
    }
}

#[derive(Default)]
pub struct UserMutation;

#[derive(InputObject)]
pub struct RegisterInput {
    username: String,
    email: Option<String>,
    password: String,
    confirm_password: String,
}
#[Object]
impl UserMutation {
    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<String> {
        info!("Mutation.UserMutation.register accepted one request");
        let mut bad_input_error_handler = ctx.data_unchecked::<BadInputErrorHandler>().clone();
        let username = input.username.trim();
        if username.is_empty() {
            bad_input_error_handler.append("username", "empty username is not allowed");
        } else if !USERNAME_VERIFICATION.is_match(username) {
            bad_input_error_handler.append("username", "invalid username")
        }
        let password = input.password.trim();
        if password.is_empty() {
            bad_input_error_handler.append("password", "empty password is not allowed");
        } else if !PASSWORD_VERIFICATION.is_match(password) {
            bad_input_error_handler.append(
                "password",
                "your password is too weak or the length is not in the range [8,16]",
            );
        }
        let confirm_password = input.confirm_password.trim();
        if confirm_password.is_empty() {
            bad_input_error_handler.append("confirm_password", "empty password is not allowed");
        } else if confirm_password != password {
            bad_input_error_handler.append(
                "confirm_password",
                "confirm password not match the password",
            );
        }
        let email = match input.email {
            Some(email) => {
                if !EMAIL_VERIFICATION.is_match(email.trim()) {
                    bad_input_error_handler.append("email", "not a valid email address");
                    None
                } else {
                    Some(email)
                }
            }
            None => None,
        };
        if bad_input_error_handler.is_none() {
            let user = crate::users::ActiveModel {
                username: Set(username.to_owned()),
                password: Set(password.to_owned()),
                email: Set(email.to_owned()),
                ..Default::default()
            };
            let db = ctx.data_unchecked::<DbConn>();
            match crate::users::Entity::insert(user).exec(db).await {
                Ok(_) => return Ok("fake token".to_string()),
                Err(err) => {
                    error!("{:?}", err);
                    return Err(Error::new_with_source(err));
                }
            }
        }
        return Err(bad_input_error_handler.to_err());
    }
}
