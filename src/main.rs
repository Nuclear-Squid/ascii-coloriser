use std::env;           // command line args
use std::fs;            // file manipulation
use itertools::izip;    // zip more than 2 iterators
use colored::Colorize;  // 'cause why not

fn get_contents(file: &String) -> String {
    if file != "" {
        fs::read_to_string(file).expect(format!("Couldn't read from file `{}`", file).as_str())
    } else {
        String::from("")
    }
}

fn nb_chars(text: &String) -> usize {
    let text_as_vec: Vec<char> = text.chars().collect();
    text_as_vec.len()
}

fn max(x: i32, y: i32) -> i32 {
    if x > y {
        x
    } else {
        y
    }
}

fn get_padded_contents(file1: &String, file2: &String, file3: &String) -> (String, String, String) {
    fn pad (text: &String, n: i32) -> String {
        let mut rv_text = text.clone();
        for _ in 0..n {
            rv_text.push_str(" ");
        };
        rv_text
    }

    let files_contents: [String; 3] = [file1, file2, file3].map(|x| get_contents(&x));
    let max_len = files_contents.clone().map(|x| nb_chars(&x)).iter().fold(0, |acc, x| max(acc, *x as i32));

    let mut trucated_contents: Vec<String> = vec![];
    for mut x in files_contents.clone().into_iter() {
        x.pop();
        trucated_contents.push(x);
    }

    let mut padded_contents: Vec<String> = vec![];
    for x in trucated_contents.clone().into_iter() {
        padded_contents.push(pad(&x, max_len - (nb_chars(&x) as i32) - 1));
    };

    let out1 = padded_contents[0].clone();
    let out2 = padded_contents[1].clone();
    let out3 = padded_contents[2].clone();

    (out1, out2, out3)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let ((base_ascii_art, fg_color_map, bg_color_map), out_file) = {
        match args.len() {
            4 => (get_padded_contents(&args[1], &args[2], &String::from("")), args[3].clone()),
            5 => (get_padded_contents(&args[1], &args[2], &args[3]), args[4].clone()),
            _ => panic!("Wrong number of arguments, here's the correct syntax:
    ascii_coloriser <base_ascii_art> <fg_color_map> [<bg_color_map>] <out_file>"),
        }
    };

    println!("{}, {}, {},", base_ascii_art, fg_color_map, bg_color_map);
    for (a, b, c) in izip!(base_ascii_art.chars(), fg_color_map.chars(), bg_color_map.chars()) {
        println!("{} {} {}", a, b, c);
    };
}
