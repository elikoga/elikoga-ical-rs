// see https://icalendar.org/iCalendar-RFC-5545/3-1-content-lines.html

// Lines of text SHOULD NOT be longer than 75 octets,
// excluding the line break. Long content lines SHOULD
// be split into a multiple line representations using
// a line "folding" technique. That is, a long line can
// be split between any two characters by inserting a
// CRLF immediately followed by a single linear white-space
// character (i.e., SPACE or HTAB). Any sequence of CRLF
// followed immediately by a single linear white-space
// character is ignored (i.e., removed) when processing
// the content type.

use std::io::BufRead;

use eyre::Context;
use eyre::{eyre, Result};

#[derive(Debug, Clone)]
pub struct Unfold<B: BufRead> {
    read: B,
    last_line: Option<Vec<u8>>,
}

impl<B: BufRead> Unfold<B> {
    pub fn new(read: B) -> Unfold<B> {
        Unfold {
            read,
            last_line: None,
        }
    }
}

impl<B: BufRead> Iterator for Unfold<B>
where
    B: BufRead,
{
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut byte_buf = match self.last_line.take() {
            Some(buf) => buf,
            None => {
                let mut buf = Vec::new();
                match self
                    .read
                    .read_until(b'\r', &mut buf)
                    .wrap_err("first read_until failed")
                {
                    // read until CR
                    Ok(0) => return None, // EOF
                    Ok(_) => (),
                    Err(e) => return Some(Err(e)),
                };
                // assumption: the line does not begin with whitespace
                // assert that
                assert!(!buf.is_empty()); // it's not empty
                assert!(buf[0] != b' '); // it's not a space
                assert!(buf[0] != b'\t'); // it's not a tab

                // assumption: the next character is a newline
                // assert that
                let mut newline_buf: [u8; 1] = [0; 1];
                match self
                    .read
                    .read_exact(&mut newline_buf)
                    .wrap_err("first read_exact for \\n failed")
                {
                    Ok(_) => (),
                    Err(e) => return Some(Err(e)),
                };
                assert!(newline_buf[0] == b'\n');

                // since the line ends correctly, we can remove the CR
                buf.pop();
                buf
            }
        };

        loop {
            // now look at the next line
            let mut next_line_buf = Vec::new();
            match self
                .read
                .read_until(b'\r', &mut next_line_buf)
                .wrap_err("read_until failed")
            {
                // read until CR
                Ok(0) => return None, // EOF
                Ok(_) => (),
                Err(e) => return Some(Err(e)),
            };
            // assumption: the next character is a newline
            // assert that
            let mut newline_buf: [u8; 1] = [0; 1];
            match self
                .read
                .read_exact(&mut newline_buf)
                .wrap_err("read_exact for \\n failed")
            {
                Ok(_) => (),
                Err(e) => return Some(Err(e)),
            };
            assert!(newline_buf[0] == b'\n');

            // since the line ends correctly, we can remove the CR
            next_line_buf.pop();
            // if the next line is empty, we can fail with an error, since empty lines are not allowed
            if next_line_buf.is_empty() {
                return Some(Err(eyre!("empty line")));
            }
            // if the line does not begin with whitespace, we are done
            if next_line_buf[0] != b' ' && next_line_buf[0] != b'\t' {
                // we are done
                // save the next_line_buf
                self.last_line = Some(next_line_buf);
                // return the byte_buf
                let string = String::from_utf8(byte_buf).wrap_err("from_utf8 failed");
                return Some(string);
            }

            // since it begins with whitespace, we need to combine the two lines
            // remove the whitespace from the next line
            // and add into byte_buf
            byte_buf.extend_from_slice(&next_line_buf[1..]);
        }
    }
}

// tests

#[cfg(test)]
mod tests {

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
                let unfold = super::Unfold::new(std::io::BufReader::new(file));
                for (line_number, line) in unfold.enumerate() {
                    let line = line.unwrap();
                    println!("{}: {}", line_number, line);
                }
            }
        }
    }
}
