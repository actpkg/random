use act_sdk::prelude::*;

act_sdk::embed_skill!("skill/");

#[act_component]
mod component {
    use super::*;

    /// Generate a UUID.
    #[act_tool(
        description = "Generate a UUID (v4 random or v7 time-ordered)",
        read_only
    )]
    fn uuid(
        #[doc = "UUID version: 4 (random, default) or 7 (time-ordered)"] version: Option<u8>,
    ) -> ActResult<String> {
        match version.unwrap_or(4) {
            4 => Ok(uuid::Uuid::new_v4().to_string()),
            7 => Ok(uuid::Uuid::now_v7().to_string()),
            v => Err(ActError::invalid_args(format!(
                "Unsupported UUID version: {v}. Use 4 or 7."
            ))),
        }
    }

    /// Generate a random string.
    #[act_tool(description = "Generate a cryptographically random string", read_only)]
    fn random_string(
        #[doc = "Length of the string"] length: u32,
        #[doc = "Character set: 'alphanumeric' (default), 'hex', 'alpha', 'digits', 'ascii'"]
        charset: Option<String>,
    ) -> ActResult<String> {
        use rand::Rng;
        let mut rng = rand::rng();
        let len = length.min(10_000) as usize;

        let chars: &[u8] = match charset.as_deref().unwrap_or("alphanumeric") {
            "alphanumeric" => b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            "hex" => b"0123456789abcdef",
            "alpha" => b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "digits" => b"0123456789",
            "ascii" => b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:',.<>?/`~",
            other => {
                return Err(ActError::invalid_args(format!(
                    "Unknown charset: {other}. Use: alphanumeric, hex, alpha, digits, ascii"
                )));
            }
        };

        let result: String = (0..len)
            .map(|_| chars[rng.random_range(0..chars.len())] as char)
            .collect();
        Ok(result)
    }

    /// Generate a random integer in a range.
    #[act_tool(
        description = "Generate a cryptographically random integer in a range",
        read_only
    )]
    fn random_number(
        #[doc = "Minimum value (inclusive, default 0)"] min: Option<i64>,
        #[doc = "Maximum value (inclusive, default 100)"] max: Option<i64>,
    ) -> ActResult<String> {
        use rand::Rng;
        let min = min.unwrap_or(0);
        let max = max.unwrap_or(100);
        if min > max {
            return Err(ActError::invalid_args("min must be <= max"));
        }
        let n: i64 = rand::rng().random_range(min..=max);
        Ok(n.to_string())
    }
}
