#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod parsing;

/// Represents a token of a paragraph/page.
pub enum Token {
    /// A separator symbol.
    Separator(char),
    /// A word.
    Word(String),
}

impl Token {
    /// Tokenize a text.
    pub fn tokenize(text: &str) -> Vec<Token> {
        let mut word = String::new();
        let mut res = vec![];
        for ch in text.chars() {
            match ch {
                '\n' | ' ' | '.' | ',' | ':' | ';' |
                '[' | ']' | '(' | ')' |
                '“' | '"' | '”' | '’' |
                '—' => {
                    if word.len() > 0 {
                        res.push(Token::Word(word));
                        word = String::new();
                    }
                    res.push(Token::Separator(ch));
                }
                _ => word.push(ch),
            }
        }


        if word.len() > 0 {
            res.push(Token::Word(word));
        }

        res
    }
}

/// Defines the data structure of Translate documents.
pub type Data = Vec<(String, String)>;

/// Create empty data document.
///
/// This contains one item by default.
pub fn new() -> Data {
    vec![("".into(), "".into())]
}

/// Save data to file.
pub fn save(file: &str, data: &Data) -> Result<(), std::io::Error> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(file)?;
    writeln!(file, "[")?;
    let mut first = true;
    for (from, to) in data {
        if first {
            first = false;
        } else {
            writeln!(file, ",")?;
        }
        write!(file, "  [\n    {:?},\n    {:?}\n  ]", from, to)?;
    }
    writeln!(file, "\n]")?;
    Ok(())
}

/// Load data from file.
pub fn load(file: &str) -> Result<Data, String> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(file).map_err(|_|
        "Could not open file".to_string())?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).unwrap();
    let source = String::from_utf8(bytes)
        .map_err(|_| "Could not convert to UTF8 text".to_string())?;
    parsing::parse_str(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let a = parsing::parse_str(r#"[["a", "b"]]"#).unwrap();
        assert_eq!(a, vec![("a".to_string(), "b".to_string())]);

        let a = parsing::parse_str(r#"[["a", "b"], ["c", "d"]]"#).unwrap();
        assert_eq!(a, vec![
            ("a".to_string(), "b".to_string()),
            ("c".to_string(), "d".to_string()),
        ]);
    }
}
