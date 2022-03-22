//! Parts: Split the input to tokens with the common naming conventions.

use std::{borrow::Cow, ops::Deref};

#[derive(Debug, PartialEq)]
pub struct Parts(Vec<String>);

impl Parts {
    pub fn to_pascalcase(&self) -> String {
        self.iter().fold(String::new(), |mut acc, token| {
            for (idx, c) in token.chars().enumerate() {
                // We convert the first character of token to uppercase,
                // then the rest to lowercase. As we've converted all the characters
                // in token to lowercase (see From::<&str>::from), we just push those
                // characters to `acc` with no any modification.
                if idx == 0 {
                    acc.push(c.to_ascii_uppercase());
                } else {
                    acc.push(c);
                }
            }
            acc
        })
    }

    pub fn to_camelcase(&self) -> String {
        self.iter().enumerate().fold(String::new(), |mut acc, (token_idx, token)| {
            for (c_idx, c) in token.chars().enumerate() {
                // If the token is not in the first order ("camel" in ["camel", "case"]),
                // we convert the first character of token to uppercase (["camel", "Case"]), 
                // then the rest to lowercase. As we've converted all the characters
                // in token to lowercase (see From::<&str>::from), we just push those
                // characters to `acc` with no any modification.
                if token_idx != 0 && c_idx == 0 {
                    acc.push(c.to_ascii_uppercase());
                } else {
                    acc.push(c);
                }
            }
            acc
        })
    }

    pub fn to_snakecase(&self) -> String {
        self.join("_")
    }

    pub fn to_kebabcase(&self) -> String {
        self.join("-")
    }

    pub fn to_screaming_snakecase(&self) -> String {
        // The logic is just like what to_snakecase() does,
        // but we convert all the tokens to uppercase first.
        self.iter().map(|s| s.to_uppercase()).collect::<Vec<String>>().join("_")
    }

    pub fn to_screaming_kebabcase(&self) -> String {
        // The logic is just like what to_kebabcase() does,
        // but we convert all the tokens to uppercase first.
        self.iter().map(|s| s.to_uppercase()).collect::<Vec<String>>().join("-")
    }
}

impl Deref for Parts {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

impl From<&str> for Parts {
    fn from(input: &str) -> Self {
        let mut input = Cow::Borrowed(input);
        let mut parts = Vec::new();
        let mut buf: String = String::new();
        
        // If the input contains with '_' or '-', we convert
        // all the character in `input` to lowercase so it can split correctly.
        //
        // For example, passing "HelLo_WoRld" will get ["hello", "world"].
        if input.contains('_') || input.contains('-') {
            input = Cow::Owned(input.to_ascii_lowercase());
        }
        
        // If the input is all uppercase,
        // we returns a vector with only the lowercased input.
        //
        // For example, passing "HELLOWORLD" will get ["helloworld"].
        if input.chars().all(|c| c.is_ascii_uppercase()) {
            return Parts(vec![input.to_ascii_lowercase()]);
        }
        
        // Separate the flush part as macro.
        //
        // We don't write this in closure, as closure will take
        // the mutable reference of `parts` and `buf`,
        // which is not expected.
        macro_rules! flush {
            () => {
                if !buf.is_empty() {
                    parts.push(buf.clone());
                    buf.clear();
                }
            };
        }
        
        for c in input.chars() {
            if matches!(c, '_' | '-') {
                // If we saw '_' or '-', we flush the buffer. For example:
                //
                // buf
                // ~~~~~v [flush point]
                // hello_world
                //       ~~~~~ next buf
                flush!();
            } else if c.is_ascii_uppercase() {
                // If we saw an uppercase character (which will appear if
                // the `input` is named with PascalCase or camelCase),
                // we flush the buffer. For example:
                //
                // buf
                // ~~~~~v [flush point]
                // HelloWorld
                //      ~~~~~ next buf (all lowercased)
                flush!();
                buf.push(c.to_ascii_lowercase());
            } else {
                // Otherwise, we push to buf.
                buf.push(c);
            }
        }
        
        // Flush the buffer eventually.
        parts.push(buf);
        // Construct `Parts`.
        Parts(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::Parts;

    #[test]
    fn test_parts_from_str() {
        let expected = ["hello", "world"];

        assert_eq!(*Parts::from("HelloWorld"), &expected[..]);
        assert_eq!(*Parts::from("helloWorld"), &expected[..]);
        assert_eq!(*Parts::from("HELLOWORLD"), ["helloworld"].as_slice());
        assert_eq!(*Parts::from("helloworld"), ["helloworld"].as_slice());
        assert_eq!(*Parts::from("HelLo_WoRld"), &expected[..]);
        assert_eq!(*Parts::from("hello_world"), &expected[..]);
        assert_eq!(*Parts::from("HELLO_WORLD"), &expected[..]);
        assert_eq!(*Parts::from("Hello_World"), &expected[..]);
        assert_eq!(*Parts::from("hello-world"), &expected[..]);
        assert_eq!(*Parts::from("HELLO-WORLD"), &expected[..]);
        assert_eq!(*Parts::from("Hello-World"), &expected[..]);
    }
}
