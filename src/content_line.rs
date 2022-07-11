use std::{fmt::Display, str::FromStr};

use eyre::{eyre, Result};
use memchr::{memchr, memchr2, memchr3};

// parser for content lines

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContentLine {
    pub name: String,
    pub params: Vec<Param>,
    pub value: String, // special strings actually, see `value = *VALUE-CHAR` from the RFC
}

impl ContentLine {
    pub fn new(name: String, params: Vec<Param>, value: String) -> Self {
        Self {
            name,
            params,
            value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Param {
    name: String,
    values: Vec<String>, // assert that there is at least one value
}

impl Param {
    pub fn new(name: String, values: Vec<String>) -> Self {
        Self { name, values }
    }
}

impl Display for ContentLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        for param in &self.params {
            write!(f, ";{}=", param.name)?;
            for value in &param.values {
                // check if param.name contains ';', ':' or ',', if so, it is a quoted param
                let is_quoted = memchr3(b';', b':', b',', value.as_bytes()).is_some();
                if is_quoted {
                    write!(f, "\"{}\"", value)?;
                } else {
                    write!(f, "{}", value)?;
                }
            }
        }
        write!(f, ":{}", self.value)
    }
}

fn build_name(name: &[u8]) -> Result<String> {
    // just assert thatt it consists of Alphanumerics, Hyphens and Digits
    for c in name {
        if !(c.is_ascii_alphanumeric() || c.is_ascii_digit() || *c == b'-') {
            return Err(eyre!(
                "invalid name: {}",
                std::str::from_utf8(name).unwrap()
            ));
        }
    }
    Ok(std::str::from_utf8(name).unwrap().to_string())
}

fn build_qsafe(value: &[u8]) -> Result<String> {
    // just assert thatt it consists of QSAFE-CHAR
    // so any character except control characters and '"'
    for c in value {
        if (c.is_ascii_control() && *c != b'\t') || *c == b'"' {
            return Err(eyre!(
                "invalid qsafe: {}",
                std::str::from_utf8(value).unwrap()
            ));
        }
    }
    Ok(std::str::from_utf8(value).unwrap().to_string())
}

fn build_safe(value: &[u8]) -> Result<String> {
    // just assert thatt it consists of SAFE-CHAR
    // so any character except control characters and '"', ';', ':' and ','
    for c in value {
        if (c.is_ascii_control() && *c != b'\t')
            || *c == b'"'
            || *c == b';'
            || *c == b':'
            || *c == b','
        {
            return Err(eyre!(
                "invalid safe: {}",
                std::str::from_utf8(value).unwrap()
            ));
        }
    }
    Ok(std::str::from_utf8(value).unwrap().to_string())
}

fn build_value(value: &[u8]) -> Result<String> {
    // just assert thatt it consists of VALUE-CHAR
    // so any character except control characters
    for c in value {
        if c.is_ascii_control() && *c != b'\t' {
            return Err(eyre!(
                "invalid value: {}",
                std::str::from_utf8(value).unwrap()
            ));
        }
    }
    Ok(std::str::from_utf8(value).unwrap().to_string())
}

impl FromStr for ContentLine {
    type Err = eyre::Report;
    fn from_str(raw_line: &str) -> Result<ContentLine> {
        // parse by recursive descent
        let mut cursor = 0;
        // find first ';' or ':' using memchr2
        let name_end =
            memchr2(b';', b':', raw_line.as_bytes()).ok_or(eyre!("no ';' or ':' found"))?;
        // name is everything before the first ';' or ':'
        let name = build_name(&raw_line.as_bytes()[cursor..cursor + name_end])?;
        let mut params = Vec::new();
        cursor += name_end;
        // parse params
        while raw_line.as_bytes()[cursor] == b';' {
            cursor += 1;
            // find first '=' using memchr
            let param_name_end =
                memchr(b'=', raw_line[cursor..].as_bytes()).ok_or(eyre!("no '=' found"))?;
            // param name is everything before the first '='
            let param_name = build_name(&raw_line.as_bytes()[cursor..cursor + param_name_end])?;
            cursor += param_name_end;
            // parse param values
            let mut param_values = Vec::new();
            while {
                cursor += 1;
                if raw_line.as_bytes()[cursor] == b'"' {
                    cursor += 1;
                    // parse qsafe
                    let param_value_end = memchr(b'"', raw_line[cursor..].as_bytes())
                        .ok_or(eyre!("no '\"' found"))?;
                    let param_value =
                        build_qsafe(&raw_line.as_bytes()[cursor..cursor + param_value_end])?;
                    cursor += param_value_end;
                    param_values.push(param_value);
                    cursor += 1;
                } else {
                    // parse safe
                    let param_value_end = memchr3(b',', b';', b':', raw_line[cursor..].as_bytes())
                        .ok_or(eyre!("no ',' or ';' or ':' found"))?;
                    let param_value =
                        build_safe(&raw_line.as_bytes()[cursor..cursor + param_value_end])?;
                    cursor += param_value_end;
                    param_values.push(param_value);
                }
                raw_line.as_bytes()[cursor] == b','
            }
            /* do */
            { /* EMPTY */ }
            // construct param
            let param = Param {
                name: param_name,
                values: param_values,
            };
            params.push(param);
        }
        // assert the cursor is at ':'
        if raw_line.as_bytes()[cursor] != b':' {
            return Err(eyre!("no ':' found"));
        }
        cursor += 1;
        // the rest is the value
        // parse value
        let value = build_value(&raw_line.as_bytes()[cursor..])?;
        // construct content line
        Ok(ContentLine {
            name,
            params,
            value,
        })
    }
}

// tests
#[cfg(test)]
mod tests {
    use crate::{content_line::ContentLine, unfold::Unfold};
    use eyre::eyre;

    #[test]
    fn it_works_on_all_private_test_icals() {
        // go through all ./private-test-icals/*.ics files and go through all lines
        let folder = std::path::Path::new("./private-test-icals");
        let files = std::fs::read_dir(folder).unwrap();
        for file in files {
            let file = file.unwrap();
            let path = file.path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            if filename.ends_with(".ics") {
                let file = std::fs::File::open(path).unwrap();
                // bufread the file
                let unfold = Unfold::new(std::io::BufReader::new(file));
                for (line_number, line) in unfold.enumerate() {
                    let line = line.unwrap();
                    // parse the line
                    let content_line = line.parse::<ContentLine>().unwrap();
                    println!("{}: {:?}", line_number, content_line);
                    // rebuild the line
                    let rebuilt_line = content_line.to_string();
                    // and reparse it
                    let reparsed_line = rebuilt_line.parse::<ContentLine>().unwrap();
                    // assert that the parses are equal
                    if content_line != reparsed_line {
                        Err::<(), _>(eyre!(
                            "line {}: {} != {}",
                            line_number,
                            content_line,
                            reparsed_line
                        ))
                        .unwrap();
                    }
                }
            }
        }
    }
}
