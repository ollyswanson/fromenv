use crate::BoxError;

pub type ParserResult<T> = Result<T, BoxError>;

pub trait Parser<T> {
    fn parse(&self, s: &str) -> ParserResult<T>;

    fn parse_from_env(&self, env_var: &str) -> Option<(String, Result<T, BoxError>)> {
        std::env::var(env_var).ok().map(|value| {
            let parse_result = self.parse(&value);
            (value, parse_result)
        })
    }
}

pub fn from_str<T>(s: &str) -> ParserResult<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
{
    s.parse::<T>().map_err(|e| e.into())
}

pub fn into<T>(s: &str) -> ParserResult<T>
where
    T: From<String>,
{
    Ok(s.to_owned().into())
}

impl<T, F> Parser<T> for F
where
    F: for<'a> Fn(&'a str) -> ParserResult<T>,
{
    fn parse(&self, s: &str) -> ParserResult<T> {
        (self)(s)
    }
}
