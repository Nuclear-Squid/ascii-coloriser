use std::env;          // command line args
use std::fs;           // file manipulation
use colored::Colorize;  // 'cause why not ?
// use regex::Regex;  // mainly for replace_all method

enum Output {
    Stdout,
    File(String),
}

struct FileContents {
    ascii_art: String,
    fg_color_map: Option<String>,
    bg_color_map: Option<String>,
    style_map: Option<String>,
    output_file: Output,
}

#[derive(PartialEq)]
enum ArgParserStates {
    ReadingFlags,
    FgFile,
    BgFile,
    StyleFile,
    OutputFile,
}

static INCOMPATIBLE_FILES_HINT: &str ="\
 : use `bat ASCII_ART | sd -p '[^ ]' '·' > FG_COLOR_MAP`
or `bat ASCII_ART | sd -p '[^ ]' '-' > BG_COLOR_MAP`
to generate color map templates. (both commands work for
style templates, replace fullcaps names with your file's name)";

fn get_contents(file: &String) -> String {
    fs::read_to_string(file).expect(format!("Couldn't read from file `{}`", file).as_str())
}

// Terminates process in case of error, returns content of file in case of success
fn compatible_with_ascii_art(ascii_art_line_lengths: &Vec<usize>, other_file_name: &String) -> Result<String, String> {
    let other_file_contents = get_contents(other_file_name);
    let other_file_line_lenghts: Vec<usize> =
        other_file_contents .lines()
            .map(|l| l.chars().collect::<Vec<char>>().len())  // nb of utf-8 chars
            .collect();

    let nb_lines_ascii_art = ascii_art_line_lengths.len();
    let nb_lines_other_file = other_file_line_lenghts.len();

    if nb_lines_ascii_art == nb_lines_other_file {
        for (i, (len_1, len_2)) in std::iter::zip(ascii_art_line_lengths, other_file_line_lenghts).enumerate() {
            if *len_1 > len_2 {
                return Err(format!("{} : unexpected end-of-line in file {} at line {}, column {}.",
                         "Error".red(), other_file_name.cyan(), i, len_2,))
            } else if *len_1 < len_2 {
                return Err(format!("{} : expected end-of-line in file {} at line {}, column {}.",
                         "Error".red(), other_file_name.cyan(), i, len_1))
            } else {
                continue;
            }
        };

        Ok(other_file_contents)

    } else if nb_lines_ascii_art > nb_lines_other_file {
        Err(format!("{} : unexpected EOF in file {} at line {}.",
                 "Error".red(), other_file_name.cyan(), nb_lines_other_file))
    } else {
        Err(format!("{} : expected EOF in file {} at line {}.",
                 "Error".red(), other_file_name.cyan(), nb_lines_ascii_art))
    }
}

fn get_safe_contents(ascii_art_content: &String, file_name: &String) -> String {
    let ascii_art_line_lengths: Vec<usize> =
        ascii_art_content.lines()
        .map(|l| l.chars().collect::<Vec<char>>().len())
        .collect();

    match compatible_with_ascii_art(&ascii_art_line_lengths, file_name) {
        Ok(contents) => contents,
        Err(why) => {
            eprintln!("{}\n\n{}{}", why, "Hint".yellow(), INCOMPATIBLE_FILES_HINT);
            std::process::exit(1);
        }
    }
}

fn arg_parse() -> FileContents {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Too few arguments were given, it needs at least an ascii art\
        and a map file (preceded by the appropriate flag). Exemple :

        \rascii_coloriser my_awesome_ascii_art -fg color_map");
        std::process::exit(2);
    }

    let mut rv_file_contents = FileContents {
        ascii_art: get_contents(&args[1]),
        fg_color_map: None,
        bg_color_map: None,
        style_map: None,
        output_file: Output::Stdout,
    };

    use ArgParserStates::*;
    let mut state:ArgParserStates = ReadingFlags;

    for arg in args[2..].iter() {
        match state {
            ReadingFlags => {
                match arg.as_str() {
                    "-fg" => state = FgFile,
                    "-bg" => state = BgFile,
                    "-s"  => state = StyleFile,
                    "-o"  => state = OutputFile,
                    _     => {
                        eprintln!("Unknown flag `{}`", arg);
                        std::process::exit(3);
                    }
                }
            },
            FgFile => {
                state = ReadingFlags;
                rv_file_contents.fg_color_map =
                    Some(get_safe_contents(&rv_file_contents.ascii_art, &arg))
            },
            BgFile => {
                state = ReadingFlags;
                rv_file_contents.bg_color_map =
                    Some(get_safe_contents(&rv_file_contents.ascii_art, &arg))
            },
            StyleFile => {
                state = ReadingFlags;
                rv_file_contents.style_map =
                    Some(get_safe_contents(&rv_file_contents.ascii_art, &arg))
            },
            OutputFile => {
                state = ReadingFlags;
                rv_file_contents.output_file =
                    Output::File(get_safe_contents(&rv_file_contents.ascii_art, &arg))
            },
        }
    };

    if state != ReadingFlags {
        eprintln!("expected file name after flag `{}`", args.last().unwrap());
        std::process::exit(5);
    };

    if rv_file_contents.fg_color_map == None &&
            rv_file_contents.bg_color_map == None &&
            rv_file_contents.style_map == None {
        eprintln!("No transformation maps have been given, did you forget them ?");
        std::process::exit(4);
    };

    rv_file_contents
}
 
fn main() {
    let args: FileContents = arg_parse();
}
