use crate::BoxError;

/// Return type for functions that can be used with the `with` attribute.
pub type ParseResult<T> = Result<T, BoxError>;

pub trait Parser<T> {
    fn parse(&self, s: &str) -> ParseResult<T>;

    fn parse_from_env(&self, env_var: &str) -> Option<(String, Result<T, BoxError>)> {
        std::env::var(env_var).ok().map(|value| {
            let parse_result = self.parse(&value);
            (value, parse_result)
        })
    }
}

pub fn from_str<T>(s: &str) -> ParseResult<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    s.parse::<T>().map_err(|e| e.into())
}

pub fn into<T>(s: &str) -> ParseResult<T>
where
    T: From<String>,
{
    Ok(s.to_owned().into())
}

impl<T, F> Parser<T> for F
where
    F: for<'a> Fn(&'a str) -> ParseResult<T>,
{
    fn parse(&self, s: &str) -> ParseResult<T> {
        (self)(s)
    }
}
