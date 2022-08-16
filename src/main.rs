use std::env;            // command line args
use std::fs;             // file manipulation
use colored::Colorize;  // 'cause why not ?
use itertools::izip;    // zip more than 2 iterables

#[derive(PartialEq)]
enum Output {
    Stdout,
    File(String),
}

struct FormattedChar {
    ascii: char,
    fg_color: Option<char>,
    bg_color: Option<char>,
    style: Option<char>,
}

struct FileContents {
    ascii_art: Vec<char>,
    fg_color_map: Vec<Option<char>>,
    bg_color_map: Vec<Option<char>>,
    style_map: Vec<Option<char>>,
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
 : use `bat ASCII_ART | sd -p '[^ ]' '-' > MAP_FILE`
to generate templates. (works with fg/bg color maps and
style maps, replace fullcaps names with your file's name)";

fn get_contents(file: &String) -> String {
    fs::read_to_string(file).expect(format!("Couldn't read from file `{}`", file).as_str())
}

// Terminates process in case of error, returns content of file in case of success
fn compatible_with_ascii_art(ascii_art_line_lengths: &Vec<usize>, other_file_name: &String) -> Result<Vec<Option<char>>, String> {
    let other_file_contents = get_contents(other_file_name);
    let other_file_line_lenghts: Vec<usize> =
        other_file_contents.lines()
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

        Ok(other_file_contents.chars().map(|x| Some(x)).collect())

    } else if nb_lines_ascii_art > nb_lines_other_file {
        Err(format!("{} : unexpected EOF in file {} at line {}.",
                 "Error".red(), other_file_name.cyan(), nb_lines_other_file))
    } else {
        Err(format!("{} : expected EOF in file {} at line {}.",
                 "Error".red(), other_file_name.cyan(), nb_lines_ascii_art))
    }
}

fn get_safe_contents(ascii_art_content: &String, file_name: &String) -> Vec<Option<char>> {
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

    let ascii_art_contents: String = get_contents(&args[1]);
    let ascii_art_chars: Vec<char> = ascii_art_contents.chars().collect();
    let ascii_art_len = ascii_art_chars.len();
    let mut rv_file_contents = FileContents {
        ascii_art: ascii_art_chars,
        fg_color_map: vec![None; ascii_art_len],
        bg_color_map: vec![None; ascii_art_len],
        style_map: vec![None; ascii_art_len],
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
                    get_safe_contents(&ascii_art_contents, &arg)
            },
            BgFile => {
                state = ReadingFlags;
                rv_file_contents.bg_color_map =
                    get_safe_contents(&ascii_art_contents, &arg)
            },
            StyleFile => {
                state = ReadingFlags;
                rv_file_contents.style_map =
                    get_safe_contents(&ascii_art_contents, &arg)
            },
            OutputFile => {
                state = ReadingFlags;
                rv_file_contents.output_file =
                    Output::File(arg.clone())
            },
        }
    };

    if state != ReadingFlags {
        eprintln!("expected file name after flag `{}`", args.last().unwrap());
        std::process::exit(5);
    };

    if rv_file_contents.fg_color_map[0] == None &&
            rv_file_contents.bg_color_map[0] == None &&
            rv_file_contents.style_map[0] == None {
        eprintln!("No transformation maps have been given, did you forget them ?");
        std::process::exit(4);
    };

    rv_file_contents
}

// color codes explained here: https://stackoverflow.com/questions/5947742/how-to-change-the-output-color-of-echo-in-linux
fn stylise_char(output_type: &Output, new_char: FormattedChar) -> String {
    if new_char.ascii == '\n' && *output_type != Output::Stdout {
        r"\n".to_string()  // explicitely write the '\' for it to work with `echo -e`
    } else {
        match *output_type {
            Output::Stdout => {
                let mut rv_string = String::from(new_char.ascii);
                match new_char.fg_color {
                    None | Some(' ') | Some('-') | Some('\n') => (),
                    Some('0') => rv_string = format!("{}", rv_string.black()),
                    Some('1') => rv_string = format!("{}", rv_string.red()),
                    Some('2') => rv_string = format!("{}", rv_string.green()),
                    Some('3') => rv_string = format!("{}", rv_string.yellow()),
                    Some('4') => rv_string = format!("{}", rv_string.blue()),
                    Some('5') => rv_string = format!("{}", rv_string.purple()),
                    Some('6') => rv_string = format!("{}", rv_string.cyan()),
                    Some('7') => rv_string = format!("{}", rv_string.white()),
                    Some('8') => rv_string = format!("{}", rv_string.bright_black()),
                    Some('9') => rv_string = format!("{}", rv_string.bright_red()),
                    Some('a') => rv_string = format!("{}", rv_string.bright_green()),
                    Some('b') => rv_string = format!("{}", rv_string.bright_yellow()),
                    Some('c') => rv_string = format!("{}", rv_string.bright_blue()),
                    Some('d') => rv_string = format!("{}", rv_string.bright_purple()),
                    Some('e') => rv_string = format!("{}", rv_string.bright_cyan()),
                    Some('f') => rv_string = format!("{}", rv_string.bright_white()),
                    Some(c)   => panic!("unkown color {}", c),
                };
                match new_char.bg_color {
                    None | Some(' ') | Some('-') | Some('\n') => (),
                    Some('0') => rv_string = format!("{}", rv_string.on_black()),
                    Some('1') => rv_string = format!("{}", rv_string.on_red()),
                    Some('2') => rv_string = format!("{}", rv_string.on_green()),
                    Some('3') => rv_string = format!("{}", rv_string.on_yellow()),
                    Some('4') => rv_string = format!("{}", rv_string.on_blue()),
                    Some('5') => rv_string = format!("{}", rv_string.on_purple()),
                    Some('6') => rv_string = format!("{}", rv_string.on_cyan()),
                    Some('7') => rv_string = format!("{}", rv_string.on_white()),
                    Some('8') => rv_string = format!("{}", rv_string.on_bright_black()),
                    Some('9') => rv_string = format!("{}", rv_string.on_bright_red()),
                    Some('a') => rv_string = format!("{}", rv_string.on_bright_green()),
                    Some('b') => rv_string = format!("{}", rv_string.on_bright_yellow()),
                    Some('c') => rv_string = format!("{}", rv_string.on_bright_blue()),
                    Some('d') => rv_string = format!("{}", rv_string.on_bright_purple()),
                    Some('e') => rv_string = format!("{}", rv_string.on_bright_cyan()),
                    Some('f') => rv_string = format!("{}", rv_string.on_bright_white()),
                    Some(c)   => panic!("unkown color {}", c),
                };
                match new_char.style {
                    None | Some(' ') | Some('-') | Some('\n') => (),
                    Some('1') => rv_string = format!("{}", rv_string.bold()),
                    Some('2') => rv_string = format!("{}", rv_string.dimmed()),
                    Some('3') => rv_string = format!("{}", rv_string.italic()),
                    Some('4') => rv_string = format!("{}", rv_string.underline()),
                    Some('5') => rv_string = format!("{}", rv_string.blink()),
                    Some('6') => rv_string = format!("{}", rv_string.blink()),
                    Some('7') => rv_string = format!("{}", rv_string.reverse()),
                    Some('8') => rv_string = format!("{}", rv_string.hidden()),
                    Some('9') => rv_string = format!("{}", rv_string.strikethrough()),
                    Some(c)   => panic!("unkown style {}", c),
                };
                rv_string
            },
            Output::File(_) => {
                let mut rv_string = r"\e[".to_string();
                match new_char.style {
                    None | Some(' ') | Some('-') | Some('\n') => (),
                    Some('1') => rv_string.push_str("1;"),
                    Some('2') => rv_string.push_str("2;"),
                    Some('3') => rv_string.push_str("3;"),
                    Some('4') => rv_string.push_str("4;"),
                    Some('5') => rv_string.push_str("5;"),
                    Some('6') => rv_string.push_str("6;"),
                    Some('7') => rv_string.push_str("7;"),
                    Some('8') => rv_string.push_str("8;"),
                    Some('9') => rv_string.push_str("9;"),
                    Some(c)   => panic!("unkown style {}", c),
                };
                match new_char.fg_color {
                    None | Some(' ') | Some('-') | Some('\n') => (),
                    Some('0') => rv_string.push_str("30;"),
                    Some('1') => rv_string.push_str("31;"),
                    Some('2') => rv_string.push_str("32;"),
                    Some('3') => rv_string.push_str("33;"),
                    Some('4') => rv_string.push_str("34;"),
                    Some('5') => rv_string.push_str("35;"),
                    Some('6') => rv_string.push_str("36;"),
                    Some('7') => rv_string.push_str("37;"),
                    Some('8') => rv_string.push_str("90;"),
                    Some('9') => rv_string.push_str("91;"),
                    Some('a') => rv_string.push_str("92;"),
                    Some('b') => rv_string.push_str("93;"),
                    Some('c') => rv_string.push_str("94;"),
                    Some('d') => rv_string.push_str("95;"),
                    Some('e') => rv_string.push_str("96;"),
                    Some('f') => rv_string.push_str("97;"),
                    Some(c)   => panic!("unkown color {}", c),
                };
                match new_char.bg_color {
                    None | Some(' ') | Some('-') | Some('\n') => (),
                    Some('0') => rv_string.push_str("40;"),
                    Some('1') => rv_string.push_str("41;"),
                    Some('2') => rv_string.push_str("42;"),
                    Some('3') => rv_string.push_str("43;"),
                    Some('4') => rv_string.push_str("44;"),
                    Some('5') => rv_string.push_str("45;"),
                    Some('6') => rv_string.push_str("46;"),
                    Some('7') => rv_string.push_str("47;"),
                    Some('8') => rv_string.push_str("100;"),
                    Some('9') => rv_string.push_str("101;"),
                    Some('a') => rv_string.push_str("102;"),
                    Some('b') => rv_string.push_str("103;"),
                    Some('c') => rv_string.push_str("104;"),
                    Some('d') => rv_string.push_str("105;"),
                    Some('e') => rv_string.push_str("106;"),
                    Some('f') => rv_string.push_str("107;"),
                    Some(c)   => panic!("unkown color {}", c),
                };
                match rv_string.chars().last() {
                    Some(';') => {
                        rv_string.pop();
                        format!(r"{}m{}\e[0m", rv_string, new_char.ascii)
                    },
                    Some('[') => new_char.ascii.to_string(),
                    _ => panic!("osaentroasentr"),
                }
            }
        }
    }
}
 
fn main() {
    let args: FileContents = arg_parse();

    let mut buffer = String::new();
    for (ascii_char, fg_char, bg_char, style_char)
        in izip!(args.ascii_art, args.fg_color_map, args.bg_color_map, args.style_map)
    {
        let current_char = FormattedChar {
            ascii: ascii_char,
            fg_color: fg_char,
            bg_color: bg_char,
            style: style_char,
        };

        buffer.push_str(stylise_char(&args.output_file, current_char).as_str());
    };

    match args.output_file {
        Output::Stdout  => println!("{}", buffer),
        Output::File(f) => {
            match fs::write(f, buffer) {
                Ok(_) => println!("Render complete"),
                Err(why) => println!("Error when writing render to output file : {}", why),
            }
        },
    }
} 
