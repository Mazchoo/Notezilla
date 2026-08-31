/// Return a present non-empty compile-time env value for `$name`, otherwise empty.
macro_rules! key_from_env {
    ($name:literal) => {
        match option_env!($name) {
            Some(key) if !key.is_empty() => key,
            _ => "",
        }
    };
}

pub(crate) use key_from_env;

#[cfg(test)]
mod tests {
    #[test]
    /// Assert `key_from_env` reads a named env var and is empty when that var is unset.
    fn key_from_env_reads_named_var_or_empty() {
        assert_eq!(key_from_env!("NOTEZILLA_UNSET_ENV_VAR"), "");
        let gemini = match option_env!("GEMINI_API_KEY") {
            Some(key) if !key.is_empty() => key,
            _ => "",
        };
        assert_eq!(key_from_env!("GEMINI_API_KEY"), gemini);
    }
}
