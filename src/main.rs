use std::env;           // command line args
use std::fs;            // file manipulation
use itertools::izip;    // zip more than 2 iterators
use colored::Colorize;  // 'cause why not
use regex::Regex;       // mainly for replace_all method

fn get_contents(file: &String) -> String {
    fs::read_to_string(file).expect(format!("Couldn't read from file `{}`", file).as_str())
}

fn nb_chars(text: &String) -> usize {
    let text_as_vec: Vec<char> = text.chars().collect();
    text_as_vec.len()
}

fn compatible_files(input: Vec<&String>) -> Result<(), String> {
    assert!(!input.is_empty());

    // Check if every files have the same number of characters
    let lenghts: Vec<usize> = input.iter().map(|x| nb_chars(x)).collect();

    if lenghts.iter().min() == lenghts.iter().max() {
        // Check if new_line characters match across files
        let regexp = Regex::new(r"[^\n]").unwrap();
        let head = regexp.replace_all(input[0], " ");

        for s in input[1..].iter() {
            if regexp.replace_all(*s, " ") != head {
                return Err(String::from("uuu"))
            }
        };

        Ok(())
    } else {
        let mut rv_string = String::new();
        let mut i = 1;
        for l in lenghts.iter() {
            rv_string.push_str(format!("Lenght of string n°{} : {}\n", i, l).as_str());
            i += 1;
        };
        Err(rv_string)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (base_ascii_art, fg_color_map, maybe_bg_color_map, out_file) = {
        match args.len() {
            4 => (get_contents(&args[1]), get_contents(&args[2]), None, &args[3]),
            5 => (get_contents(&args[1]), get_contents(&args[2]), Some(get_contents(&args[3])), &args[4]),
            _ => panic!("Wrong number of arguments, here's the correct syntax:
    ascii_coloriser <base_ascii_art> <fg_color_map> [<bg_color_map>] <out_file>"),
        }
    };

    if let Some(bg_color_map) = maybe_bg_color_map {
        match compatible_files(vec![&base_ascii_art, &fg_color_map, &bg_color_map]) {
            Ok(_)    => (),
            Err(why) => panic!("Ascii art and color maps don't have the amount\
            of characters :\n{}\nHint : use `bat ASCII_ART | sd -p '[^ ]' '·' > FG_COLOR_MAP`
            \ror `bat ASCII_ART | sd -p '[^ ]' '-' > BG_COLOR_MAP` to generate color map templates.
            \r(replace fullcaps names with your file names of course)", why),
        }
    }
}
