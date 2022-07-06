// folds a single line

pub fn fold(line: &str) -> String {
    fold_with_max_length(line, 75)
}

pub fn fold_with_max_length(line: &str, max_length: usize) -> String {
    let mut new_line_buf = String::new();

    // add line characters to the new line buffer
    // until the line is too long
    let mut current_line_length = 0;
    for c in line.chars() {
        if current_line_length + c.len_utf8() > max_length {
            // add a newline
            new_line_buf.push('\r');
            new_line_buf.push('\n');
            // ad a whitespace (in our case " ")
            new_line_buf.push(' ');
            // add the current character
            new_line_buf.push(c);
            // update the current line length
            current_line_length = ' '.len_utf8() + c.len_utf8();
        } else {
            new_line_buf.push(c);
            current_line_length += c.len_utf8();
        }
    }
    new_line_buf
}

#[cfg(test)]
pub fn fold_and_join(lines: &[String]) -> String {
    // normal fold each line
    let mut folded_lines = Vec::new();
    for line in lines {
        folded_lines.push(fold(line));
    }
    // join the lines
    let out = folded_lines.join("\r\n");
    // add a newline at the end
    out + "\r\n"
}

#[cfg(test)]
mod tests {
    use crate::unfold::Unfold;

    use super::*;

    // unfold, then fold then unfold is without issue
    #[test]
    fn unfold_fold_unfold_is_without_issue() {
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
                let unfold_1 = Unfold::new(std::io::BufReader::new(file))
                    .map(|e| e.unwrap())
                    .collect::<Vec<_>>();
                // fold again, then join
                let fold_and_join_result = fold_and_join(&unfold_1);
                // build a BufRead
                let buf = std::io::Cursor::new(fold_and_join_result);
                // unfold again
                let unfold_2 = Unfold::new(std::io::BufReader::new(buf));
                // then compare first and second unfold
                for (line_1, line_2) in unfold_1.into_iter().zip(unfold_2) {
                    assert_eq!(line_1, line_2.unwrap());
                }
            }
        }
    }

    // the next test is a bit more difficult:
}
