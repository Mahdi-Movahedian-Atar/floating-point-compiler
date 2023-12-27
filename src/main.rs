mod utility;
mod entities;
mod scanner;

use std::collections::BTreeMap;
use std::{io, thread};
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use crate::entities::{FPoint, Token};
use crate::utility::utility::{read_lines, cs_hash};
use crate::scanner::scanner::tokenizer;

fn printer(points: &Vec<FPoint>, tokens: &BTreeMap<i16,Token>){
    let mut j = 1;
    let path = Path::new("compiled.txt");

    let mut file = match File::create(&path) {
        Err(why) => {
            println!("Could not print the compiled code :\n {:?}",why);
            return;
        },
        Ok(file) => file,
    };


    println!();
    print!("{:}: ",j);
    writeln!(& mut file,"{:}: ",j).unwrap();
    for point in points {
        if point.meta_data.line > j  {
            j+=1;
            println!();
            print!("{:}: ",j);
            writeln!(& mut file).unwrap();
            write!(& mut file,"{:}: ",j).unwrap();
        }
        print!("{:?}->{:?}, ",point.meta_data.raw,point.state);
        write!(& mut file,"{:?}->{:?}, ",point.meta_data.raw,point.state).unwrap();
    }
    println!();
    print!("{:?}",tokens);
    writeln!(& mut file).unwrap();
    write!(& mut file,"{:?}",tokens).unwrap();
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
    loop {
        let mut filename = String::new();
        println!("Locate a .txt file for compiling:");
        io::stdin().read_line(&mut filename).expect("failed to readline");
        let path = Path::new(filename.as_str());
        println!("{:?}",path);

        pause();

        let mut token_tree: BTreeMap<i16, Token> = hashed_tree_map![";", ":","(" ,")" ,"{" ,"}" ,"[" ,"]" ,"," ,"+" ,"-" ,"*" ,"/" ,"=",
                                                    "program", "data","division","|","integer","float","char","procedure","division",
                                                    "set","to","unsigned","get","put","repeat","times","or","either","neither","nor",
                                                    "both","and","execute","not","LT","LE","GT","GE","NE","EQ"];
        let mut points: Vec<FPoint> = Vec::new();

        let mut scanner_threats: Vec<thread::JoinHandle<(Vec<FPoint>, Vec<(i16, Token)>, i8)>> = Vec::new();

        match read_lines(Path::new(path)) {
            Ok(lines) => {
                let mut i = 1;
                for line in lines {
                    if let Ok(ip) = line {
                        scanner_threats.push(thread::spawn(move || { return tokenizer(ip, i.clone()); }));
                    }
                    i += 1;
                }
            }
            Err(err) => { println!("{:?}",err); continue; }
        }
        {
            let mut in_comment = false;

            for threat in scanner_threats {
                let mut data = threat.join().unwrap();
                if data.2 < 0 { in_comment = false; }

                if in_comment == false {
                    points.append(&mut data.0);

                    for datum in data.1 {
                        token_tree.insert(datum.0, datum.1);
                    }
                }

                if data.2 > 0 { in_comment = true; }
            }
        }
        printer(&points, &token_tree);
    }
}