use std::{
    fmt::Display,
    io::{BufRead, Cursor},
    iter::Peekable,
    str::FromStr,
};

use crate::{content_line::ContentLine, fold::fold, unfold::Unfold};
use eyre::{eyre, Result};

#[derive(Debug, PartialEq, Eq)]
pub struct ICalObject {
    pub object_type: String,
    pub properties: Vec<ContentLine>,
    pub sub_objects: Vec<ICalObject>,
}

impl ICalObject {
    fn from_peekable(
        mut peekable: &mut Peekable<impl Iterator<Item = Result<ContentLine>>>,
    ) -> Result<Self> {
        let mut properties = Vec::new();
        let mut sub_objects = Vec::new();
        let line = peekable.next().ok_or(eyre!("no line found"))??;
        if line.name != "BEGIN" {
            return Err(eyre!("expected BEGIN"));
        }
        let object_type = line.value.clone();
        while let Some(line) = match peekable.peek() {
            Some(Ok(line)) => Some(line),
            Some(Err(_)) => {
                // read then return the error
                let next = peekable.next().unwrap();
                next?;
                unreachable!()
            }
            None => None,
        } {
            if line.name == "END" {
                // get the next line
                let line = peekable.next().unwrap()?;
                // check that the object type matches
                if line.value != object_type {
                    return Err(eyre!("expected END:{}", object_type));
                }
                break;
            }
            // check if it's a begin property
            if line.name == "BEGIN" {
                sub_objects.push(ICalObject::from_iterator(&mut peekable)?);
            } else {
                // get line
                let line = peekable.next().unwrap()?;
                properties.push(line);
            }
        }
        Ok(ICalObject {
            object_type,
            properties,
            sub_objects,
        })
    }

    fn from_iterator(iterator: &mut impl Iterator<Item = Result<ContentLine>>) -> Result<Self> {
        let mut peekable = iterator.peekable();
        Self::from_peekable(&mut peekable)
    }
}

impl FromStr for ICalObject {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut cursor = Cursor::new(s);
        // unfold the string
        ICalObject::from_bufread(&mut cursor)
    }
}

impl ICalObject {
    fn from_bufread(read: &mut impl BufRead) -> Result<Self> {
        let mut unfolded =
            Unfold::new(read).flat_map(|line| line.map(|line| line.parse::<ContentLine>()));
        ICalObject::from_iterator(&mut unfolded)
    }
}

impl Display for ICalObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:{}\r\n", self.object_type)?;
        for line in &self.properties {
            write!(f, "{}\r\n", fold(&line.to_string()))?;
        }
        for object in &self.sub_objects {
            write!(f, "{}", object)?;
        }
        write!(f, "END:{}\r\n", self.object_type)
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::ICalObject;

    #[test]
    fn it_works_on_all_private_test_icals() {
        // go through all ./private-test-icals/*.ics files
        let folder = std::path::Path::new("./private-test-icals");
        let mut files = std::fs::read_dir(folder).unwrap();
        while let Some(file) = files.next() {
            let file = file.unwrap();
            let path = file.path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            if filename.ends_with(".ics") {
                let file = std::fs::File::open(path).unwrap();
                // bufread the file
                let mut bufreader = std::io::BufReader::new(file);
                // parse to ICalObject
                let ical = ICalObject::from_bufread(&mut bufreader).unwrap();
                // print the ICalObject
                println!("ICALICALICALICALSTART\n{}\nICALICALICALICALICALEND", ical);
                // check that the ICalObject is valid by parsing it again
                let ical2 = ical.to_string().parse().unwrap();
                assert_eq!(ical, ical2);
            }
        }
    }
}
