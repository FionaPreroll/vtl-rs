const GIT_HASH: &str = env!("GIT_HASH");
const BUILD_TIME: &str = env!("BUILD_TIME");
const RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub fn get_build_info() -> String {
    format!(
        "Build: {}\nGit: {}\nCompiled with: {}",
        BUILD_TIME, GIT_HASH, RUSTC_VERSION
    )
}

// Value encapsulates the duality of characters and numbers in VTL
#[derive(Clone, Debug)]
pub enum Value {
    Number(i64),
    Char(char),
}

impl Value {
    // Convert any Value to i64 (numbers are returned as-is, chars are converted to their ASCII value)
    pub fn to_i64(&self) -> i64 {
        match self {
            Value::Number(num) => *num,
            Value::Char(ch) => *ch as i64,
        }
    }

    // Convert an i64 to Value (convert it to char if it fits in the ASCII range)
    pub fn from_i64(value: i64) -> Self {
        if (0..=255).contains(&value) {
            Value::Char(value as u8 as char)
        } else {
            Value::Number(value)
        }
    }
}
