use crate::exam::Exam;

mod exam {
    use std::collections::HashSet;
    use std::{env, fs};
    use std::fs::File;
    use std::hash::{Hash, Hasher};
    use std::io::{BufReader, ErrorKind, stdin, stdout, Write};
    use std::path::PathBuf;
    use serde::{Serialize, Deserialize, Deserializer};
    use serde_json::{Result, Value};

    const ASSETS_DIR: &str = "assets";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Exam {
        name: String,
        questions: HashSet<Question>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Question {
        prompt: String,
        choices: HashSet<String>,
        answer: String,
        refs: Vec<String>,
    }

    impl PartialEq<Self> for Question {
        fn eq(&self, other: &Self) -> bool {
            self.prompt == other.prompt
            && self.choices == other.choices
            && self.answer == other.answer
            && self.refs == other.refs
        }
    }
    impl Eq for Question {}

    impl Hash for Question {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.prompt.hash(state);
            self.choices.iter().for_each(|choice| choice.hash(state));
            self.answer.hash(state);
            self.refs.hash(state);
        }
    }

    impl Exam {
        pub fn new() -> Option<Self> {
            // Get the current working directory
            let cwd: PathBuf = env::current_dir().expect("Unable to get cwd");
            // Create the assets directory (if necessary) and create the Exam object
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
            println!("Available exam files include:");
            if Self::show_available_exams(&cwd) {
                let exam_filename: String = Self::input_confirm("Enter filename of exam: ");
                let mut exam_file_path = PathBuf::from(cwd);
                exam_file_path.push(ASSETS_DIR);
                exam_file_path.push(&exam_filename);
                match File::open(cwd.join(exam_file_path)) {
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
            } else {
                None
            }
        }

        fn show_available_exams(cwd: &PathBuf) -> bool {
            let assets_dir = cwd.join(ASSETS_DIR);
            let mut count: u8 = 1;
            match fs::read_dir(assets_dir) {
                Ok(exams) => {
                    for exam in exams {
                        if let Ok(exam) = exam {
                            let exam_path = exam.path();
                            if let Some(exam_name) = exam_path.file_name() {
                                if let Some(exam_name_str) = exam_name.to_str() {
                                    if exam_name_str.ends_with(".json") {
                                        println!("\t{}.) {}", count, exam_name_str);
                                        count += 1;
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                    println!("\tUnable to print exam files...");
                    return false;
                },
                Err(_) => {
                    println!("\tThere are no available exams to choose from...");
                    false
                },
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
    if let Some(exam) = Exam::new() {
        println!("Successfully created an exam!\n{:#?}", exam);
    } else {
        println!("Unable to create the exam...");
    }
}
