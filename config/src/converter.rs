use crate::BoxError;

pub trait Converter<T> {
    fn convert(&self, s: &str) -> Result<T, BoxError>;

    fn convert_from_env(&self, env_var: &str) -> Option<(String, Result<T, BoxError>)> {
        std::env::var(env_var).ok().map(|value| {
            let parse_result = self.convert(&value);
            (value, parse_result)
        })
    }
}

pub fn from_str<T>(s: &str) -> Result<T, BoxError>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + 'static,
{
    s.parse::<T>().map_err(|e| e.into())
}

pub fn into<T>(s: &str) -> Result<T, BoxError>
where
    T: From<String>,
{
    Ok(s.to_owned().into())
}

impl<T, F> Converter<T> for F
where
    F: for<'a> Fn(&'a str) -> Result<T, BoxError>,
{
    fn convert(&self, s: &str) -> Result<T, BoxError> {
        (self)(s)
    }
}
