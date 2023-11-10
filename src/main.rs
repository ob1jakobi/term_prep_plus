mod exam {
    use std::collections::HashSet;
    use std::{env, fs};
    use std::fs::File;
    use std::io::{BufReader, ErrorKind, stdin, stdout, Write};
    use std::path::PathBuf;
    use serde::{Serialize, Deserialize, Deserializer};
    use serde_json::{Result, Value};

    const ASSETS_DIR: &str = "assets";

    #[derive(Debug, Deserialize, Serialize)]
    struct Exam {
        name: String,
        questions: HashSet<Question>,
    }

    #[derive(Eq, PartialEq, Hash, Debug, Deserialize, Serialize)]
    struct Question {
        prompt: String,
        choices: HashSet<String>,
        answer: String,
        refs: Vec<String>,
    }
    impl Exam {
        pub fn new() -> Option<Self> {
            // Create the assets directory
            let cwd: PathBuf = env::current_dir().expect("Unable to get cwd");
            if Self::create_asset_dir(&cwd) {
                Self::get_exam(&cwd)
            } else {
                None
            }
        }

        fn create_asset_dir(cwd: &PathBuf) -> bool {
            let assets_dir = cwd.join(ASSETS_DIR);
            match fs::create_dir(assets_dir) {
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    println!("The {} directory already exists; no need to create it...", ASSETS_DIR);
                    true
                },
                Err(e) => {
                    eprintln!("An error {} occurred...", e);
                    false
                },
                Ok(()) => {
                    println!("Created the {} directory", ASSETS_DIR);
                    true
                },
            }
        }

        fn get_exam(cwd: &PathBuf) -> Option<Exam> {
            let exam_filename: String = Self::input_confirm("Enter filename of exam: ");
            match File::open(cwd.join(exam_filename)) {
                Err(e) => {
                    eprintln!("Unable to open {} exam file... error: {}", &exam_filename, e);
                    None
                },
                Ok(exam_file) => {
                    let reader = BufReader::new(exam_file);
                    let exam: Exam = serde_json::from_reader(reader).expect("Error parsing JSON");
                    Some(exam)
                }
            }
        }

        fn input(prompt: &str) -> String {
            let mut temp: String = String::new();
            loop {
                temp.clear();
                print!("{}", prompt);
                stdout().flush().expect("Unable to flush stdout...");
                stdin().read_line(&mut temp).expect("Unable to get stdin...");
                let trimmed: String = String::from(temp.trim());
                if !trimmed.is_empty() {
                    return trimmed;
                }
                println!("Entry must not be empty!");
            }
        }

        fn input_confirm(prompt: &str) -> String {
            loop {
                let in1: String = Self::input(prompt);
                let in2: String = Self::input("Confirm entry: ");
                if in1.eq(&in2) {
                    return in2;
                } else {
                    println!("Entries must match!");
                }
            }
        }
    }

}

fn main() {
    println!("Hello, world!");
}
