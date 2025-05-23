use std::fmt::Display;

use argon2::{
    Argon2, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{
    Decode, Encode, Sqlite, Type,
    encode::IsNull,
    error::BoxDynError,
    sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Uuid(uuid::Uuid);

impl Uuid {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_hyphenated())
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(value: Uuid) -> Self {
        value.0
    }
}

impl AsRef<uuid::Uuid> for Uuid {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl Type<Sqlite> for Uuid {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for Uuid {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        <uuid::fmt::Hyphenated as Encode<Sqlite>>::encode_by_ref(&self.0.hyphenated(), args)
    }
}

impl Decode<'_, Sqlite> for Uuid {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        <uuid::fmt::Hyphenated as Decode<Sqlite>>::decode(value).map(|uuid| Uuid(uuid.into_uuid()))
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    sqlx::Type,
)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }

    pub fn format(&self, fmt: &str) -> String {
        self.0.format(fmt).to_string()
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format("%+"))
    }
}

impl From<DateTime> for chrono::DateTime<chrono::Utc> {
    fn from(value: DateTime) -> Self {
        value.0
    }
}

impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }
}

impl AsRef<chrono::DateTime<chrono::Utc>> for DateTime {
    fn as_ref(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    sqlx::Type,
)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct Date(chrono::NaiveDate);

impl Date {
    pub fn now() -> Self {
        Self(chrono::Utc::now().date_naive())
    }

    pub fn format(&self, fmt: &str) -> String {
        self.0.format(fmt).to_string()
    }

    pub fn is_in_past(&self) -> bool {
        self.0 < chrono::Utc::now().date_naive()
    }

    pub fn is_today(&self) -> bool {
        self.0 == chrono::Utc::now().date_naive()
    }

    pub fn is_in_future(&self) -> bool {
        self.0 > chrono::Utc::now().date_naive()
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format("%Y-%m-%d"))
    }
}

impl From<Date> for chrono::NaiveDate {
    fn from(value: Date) -> Self {
        value.0
    }
}

impl From<chrono::NaiveDate> for Date {
    fn from(value: chrono::NaiveDate) -> Self {
        Self(value)
    }
}

impl From<chrono::NaiveDateTime> for Date {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Self(value.date())
    }
}

impl AsRef<chrono::NaiveDate> for Date {
    fn as_ref(&self) -> &chrono::NaiveDate {
        &self.0
    }
}

#[derive(Debug, serde::Deserialize)]
#[repr(transparent)]
pub struct PasswordHash(SecretString);

impl PasswordHash {
    pub fn from_plain_password(plain_password: SecretString) -> Self {
        let hash = Argon2::default()
            .hash_password(
                plain_password.expose_secret().as_bytes(),
                &SaltString::generate(&mut OsRng),
            )
            .unwrap()
            .to_string();

        Self(hash.into())
    }

    pub fn from_hash(hash: SecretString) -> Self {
        Self(hash)
    }

    pub fn verify(&self, plain_password: SecretString) -> bool {
        Argon2::default()
            .verify_password(
                plain_password.expose_secret().as_bytes(),
                &argon2::password_hash::PasswordHash::new(self.0.expose_secret()).unwrap(),
            )
            .is_ok()
    }
}

impl Type<Sqlite> for PasswordHash {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for PasswordHash {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<Sqlite>>::encode_by_ref(&self.0.expose_secret().to_string(), args)
    }
}

impl Decode<'_, Sqlite> for PasswordHash {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        <String as Decode<Sqlite>>::decode(value)
            .map(|password_hash| PasswordHash::from_hash(password_hash.into()))
    }
}
