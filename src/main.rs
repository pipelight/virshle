use pipeligh_utils::files::CastError;
use serde_json::Value;
/**
Returns a Config struct from a provided toml file path.
*/
pub fn tml(file_path: &str) -> Result<Value> {
    let string = fs::read_to_string(file_path).into_diagnostic()?;
    let res = toml::from_str(&string);
    match res {
        Ok(res) => Ok(res),
        Err(e) => {
            let err = TomlError::new(e, &string);
            Err(err.into())
        }
    }
}
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use std::path::Path;

    #[test]
    fn load_toml_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = Path::new("../examples/machinetest.txt");
        assert!(res.is_ok());
    }
}
