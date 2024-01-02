mod entities;
mod parser;
mod scanner;
mod utility;

use crate::entities::{FPoint, Precedence, Token};
use crate::parser::parser::parser;
use crate::scanner::scanner::tokenizer;
use crate::utility::utility::{read_lines};
use my_macro::unique_i16;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::{io, thread};

fn printer(points: &Vec<FPoint>, tokens: &BTreeMap<i16, Token>, name: &str) {
    let mut j = 1;
    let binding = (name.to_string() + ".txt");
    let path = Path::new(&binding);

    let mut file = match File::create(&path) {
        Err(why) => {
            println!("Could not print the compiled code :\n {:?}", why);
            return;
        }
        Ok(file) => file,
    };

    println!("\n\n=============={:?}==============\n\n", name);
    print!("{:}: ", j);
    writeln!(&mut file, "{:}: ", j).unwrap();
    for point in points {
        if point.meta_data.line > j {
            j += 1;
            println!();
            print!("{:}: ", j);
            writeln!(&mut file).unwrap();
            write!(&mut file, "{:}: ", j).unwrap();
        }
        print!("{:?}->{:?}, ", point.meta_data.raw, point.state);
        write!(&mut file, "{:?}->{:?}, ", point.meta_data.raw, point.state).unwrap();
    }
    println!();
    print!("{:?}", tokens);
    println!("\n\n");
    writeln!(&mut file, "\n\n").unwrap();
    writeln!(&mut file).unwrap();
    write!(&mut file, "{:?}", tokens).unwrap();
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    println!();
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {
    let precedence_tree: BTreeMap<i16, BTreeMap<i16, Precedence>> = precedence_tree_map![
            "S" = { "E;":"P" } > {  } < { "E+", "E*", "(", "id" },
            "E+" = { "E+":"E+" } > { "E;", "E)" } < { "E*", "(" , "id" },
            "E*" = { "E*":"E*" } > { "E;", "E)" ,"E+" } < { "(" , "id" },
            "E)" = {  } > { } < {  },
            "E;" = {  } > {  } < {  },
            "+" = {  } > {  } < {  },
            "-" = {  } > {  } < {  },
            "*" = {  } > {  } < {  },
            "/" = {  } > {  } < {  },
            "(" = { "E)":"id" } > {  } < { "id", "(" },
            ")" = {  } > {  } < {  },
            ";" = {  } > {  } < {  },
            "id" = { ")":"E)", "+":"E+", "-":"E+", "*":"E*", "/":"E*", ";":"E;" } > {  } < {  },
        ];
    pause();
    loop {
        let mut token_tree: BTreeMap<i16, Token> = hashed_tree_map![
            ";",
            ":",
            "(",
            ")",
            "{",
            "}",
            "[",
            "]",
            ",",
            "+",
            "-",
            "*",
            "/",
            "=",
            "program",
            "data",
            "division",
            "|",
            "integer",
            "float",
            "char",
            "procedure",
            "division",
            "set",
            "to",
            "unsigned",
            "get",
            "put",
            "repeat",
            "times",
            "or",
            "Or",
            "either",
            "neither",
            "nor",
            "both",
            "and",
            "And",
            "execute",
            "not",
            "LT",
            "LE",
            "GT",
            "GE",
            "NE",
            "EQ"
        ];
        pause();

        let mut points: Vec<FPoint> = Vec::new();

        let mut scanner_threats: Vec<thread::JoinHandle<(Vec<FPoint>, Vec<(i16, Token)>, i8)>> =
            Vec::new();

        match read_lines(Path::new("code.txt")) {
            Ok(lines) => {
                let mut i = 1;
                for line in lines {
                    if let Ok(ip) = line {
                        scanner_threats.push(thread::spawn(move || {
                            return tokenizer(ip, i.clone());
                        }));
                    }
                    i += 1;
                }
            }
            Err(err) => {
                println!("{:?}", err);
                continue;
            }
        }
        {
            let mut in_comment = false;

            for threat in scanner_threats {
                let mut data = threat.join().unwrap();
                if data.2 < 0 {
                    in_comment = false;
                }

                if in_comment == false {
                    points.append(&mut data.0);

                    for datum in data.1 {
                        if !token_tree.contains_key(&datum.0) {
                            token_tree.insert(datum.0, datum.1);
                        }
                    }
                }

                if data.2 > 0 {
                    in_comment = true;
                }
            }
            printer(&points, &token_tree, "tokenizer");
            let return_val = parser(points, &mut token_tree, &precedence_tree);
            points = return_val.0;
            printer(&points, &token_tree,"parser");
            if return_val.1 { println!("\n\ngeneral parser error: unclosed scope(s)\n\n"); }
        }
    }
}
