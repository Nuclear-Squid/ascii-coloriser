use std::env;      // command line args
use std::fs;       // file manipulation
use regex::Regex;  // mainly for replace_all method

fn get_contents(file: &String) -> String {
    fs::read_to_string(file).expect(format!("Couldn't read from file `{}`", file).as_str())
}

// Checks that color/style maps are compatible with the ascii art
fn compatible_files(input_files: &[String]) -> Result<(), String> {
    assert!(!input_files.is_empty());

    fn find_first_diff(string_a: &str, string_b: &str) -> Result<(), String> {
        let mut i = 0;
        let vec_a: Vec<char> = string_a.chars().collect();
        let vec_b: Vec<char> = string_b.chars().collect();

        while (vec_a.get(i), vec_b.get(i)) != (None, None) {
            match (vec_a.get(i), vec_b.get(i)) {
                (None, None) => (),
                (Some(_), None) => return Err(format!(
                        "unexpected end-of-line character at column {}", i)),
                (None, Some(_)) => return Err(format!(
                        "expected end-of-line character at column {}", i)),
                (Some(a), Some(b)) =>
                    match (a, b) {
                        (' ', ' ') => (),
                        ('-', '-') => (),
                        (' ', '-') => return Err(format!(
                                "expected white-space at column {}", i)),
                        ('-', ' ') => return Err(format!(
                                "unexpected white-space at column {}", i)),
                        _ => return Err(format!(
                                "weird character at line {}", i)),
                    },
            };
            i += 1;
        };

        Ok(())
    }

    let file_contents: Vec<String> = input_files.iter()
        .map(|x| get_contents(x))
        .collect();

    let re = Regex::new(r"[^ \n]").unwrap();
    // `re.replace_all` returns a temporary, so
    //  we extend it's lifetime with a let bind
    let ascii_significant_chars = re.replace_all(&file_contents[0], "-");
    let ascii_file: Vec<&str> = ascii_significant_chars.lines().collect();

    for (file_index, file) in file_contents[1..].iter().enumerate() {
        // Same here with `current_file`
        let file_significant_chars = re.replace_all(&file, "-");
        let current_file: Vec<&str> = file_significant_chars.lines().collect();

        for (i, line) in current_file.iter().enumerate() {
            match ascii_file.get(i) {
                None => return Err(format!("Error: expected EOF at the end of \
                           line {} in file {}", i, input_files[file_index + 1])),
                Some(ascii_file_line) => {
                    match find_first_diff(ascii_file_line, line) {
                        Ok(_) => continue,
                        Err(why) => return Err(format!("Error in file `{}` at line {} \
                                    :\n{}", input_files[file_index + 1], i + 1, why)),
                    }
                }
            }
        }
    };

    Ok(())
}

const INCOMPATIBLE_FILES_ERROR_MESSAGE: &str = "
Hint : use `bat ASCII_ART | sd -p '[^ ]' '·' > FG_COLOR_MAP`
or `bat ASCII_ART | sd -p '[^ ]' '-' > BG_COLOR_MAP`
to generate color map templates. (both commands work for
style templates, replace fullcaps names with your file's name)";

fn main() {
    let args: Vec<String> = env::args().collect();

    // Send slice of `args` to get rid of $0
    match compatible_files(&args[1..]) {
        Ok(_) => (),
        Err(why) => {
            eprintln!("{}\n{}", why, INCOMPATIBLE_FILES_ERROR_MESSAGE);
            std::process::exit(1);
        },
    }

    println!("Ouais ils sont bien les fichiers");
}
