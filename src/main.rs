#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

fn main() {
    eprintln!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => init(),
        "cat-file" => cat_file(args),
        _ => println!("unknown command: {}", args[1]),
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory") 
}

fn cat_file(args: Vec<String>) {
    if args[2] == "-p" {
        let object_id = &args[3];
        let object_path = format!(".git/objects/{}/{}", &object_id[0..2], &object_id[2..]);
        let contents = fs::read_to_string(object_path).unwrap();
        println!("{}", contents);
    }
}