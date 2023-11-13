use crate::exam::Exam;

mod exam {
    use std::collections::HashSet;
    use std::{env, fs};
    use std::cmp::min;
    use std::fs::File;
    use std::hash::{Hash, Hasher};
    use std::io::{BufReader, ErrorKind, stdin, stdout, Write};
    use std::path::PathBuf;
    use serde::{Serialize, Deserialize};

    /// The default directory for storing JSON-formatted exam files
    const ASSETS_DIR: &str = "assets";

    /// High-level structure representing an Exam; has a name and a series of questions
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Exam {
        name: String,
        questions: HashSet<Question>,
    }

    /// The questions that comprise an Exam
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Question {
        prompt: String,
        choices: HashSet<String>,
        answer: String,
        explanation: String,
        refs: Vec<String>,
    }

    /// The next three are required to utilize Questions as a HashSet; this helps ensure that
    /// the sequence of questions are not revealed in the same sequence (as would be the case if
    /// the Exam struct utilized a Vec<Question>)
    impl PartialEq<Self> for Question {
        fn eq(&self, other: &Self) -> bool {
            self.prompt == other.prompt
            && self.choices == other.choices
            && self.answer == other.answer
            && self.explanation == other.explanation
            && self.refs == other.refs
        }
    }
    impl Eq for Question {}

    impl Hash for Question {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.prompt.hash(state);
            self.choices.iter().for_each(|choice| choice.hash(state));
            self.answer.hash(state);
            self.explanation.hash(state);
            self.refs.hash(state);
        }
    }

    impl Exam {
        /// Constructs an optional `Exam` since the methods used to construct an `Exam` might not
        /// succeed.
        ///
        /// # Panics
        ///
        /// In the event that the program cannot obtain the current working directory to establish
        /// an `assets` directory - as well as pull JSON-formatted exam files, the program will
        /// panic.
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

        /// Helper function that ensures the creation of the default `assets` directory for storing
        /// JSON-formatted exam files.
        ///
        /// # Argument
        ///
        /// * `cwd` - a reference to the current working directory as a `PathBuf` reference.
        ///
        /// # Returns
        ///
        /// * `bool` - If the `assets` directory already exists, or if the `assets` directory was
        ///   created without any errors, then the program will print out the applicable message and
        ///   return `true` - otherwise the program will print an error message to `stderr` and return
        ///   `false`.
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
            println!("Checking available exam files in the default location...");
            // Create an iterable of all the applicable exam files in assets
            if let Some(exams) = Self::collect_available_exams(cwd) {
                if exams.is_empty() {
                    println!("\tThere are no available exams in the default {} directory.", ASSETS_DIR);
                    None
                } else {
                    println!("Please choose one of the following exams:");
                    for (index, exam_path) in exams.iter().enumerate() {
                        if let Some(file_name) = exam_path.file_name() {
                            if let Some(exam_file_name) = file_name.to_str() {
                                println!("\t{}.) {}", index + 1, exam_file_name);
                            }
                        }
                    }
                    // TODO: Get the user's input and return the option of the exam selected
                }
            } else {
                println!("Unable to collect the exams from the default {} directory.", ASSETS_DIR);
                None
            }
        }

        fn collect_available_exams(cwd: &PathBuf) -> Option<Vec<PathBuf>> {
            let assets_dir: PathBuf = cwd.join(ASSETS_DIR);
            if let Ok(entries) = fs::read_dir(&assets_dir) {
                let exam_files: Vec<PathBuf> = entries
                    .filter_map(|entry| {
                        match entry {
                            Ok(entry) => {
                                let path = entry.path();
                                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                                    Some(path)
                                } else {
                                    None
                                }
                            },
                            _ => None
                        }
                    })
                    .collect();
                Some(exam_files)
            } else {
                println!("\tThere was an error accessing the {} directory...", ASSETS_DIR);
                None
            }
        }


        /*
        fn get_exam(cwd: &PathBuf) -> Option<Exam> {
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
            let assets_dir: PathBuf = cwd.join(ASSETS_DIR);
            println!("Available exams files include:");
            if let Ok(entries) = fs::read_dir(&assets_dir) {
                let exam_files: Vec<PathBuf> = entries
                    .filter_map(|entry| {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                                Some(path)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                if exam_files.is_empty() {
                    println!("\tThere are no exam files available in the {} directory...", ASSETS_DIR);
                    false
                } else {
                    for (index, exam) in exam_files.iter().enumerate() {
                        if let Some(exam_file) = exam.file_name() {
                            if let Some(exam_file_name) = exam_file.to_str() {
                                println!("\t{}.) {:?}", index + 1, exam_file_name);
                            }
                        }
                    }
                    true
                }
            } else {
                println!("\tThere was an error accessing the {} directory", ASSETS_DIR);
                false
            }
        }
        */

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

        /// Method for studying questions from an exam in the `assets` directory. This will ask the
        /// user how many questions they'd like to study. If the user enters a number of questions
        /// that exceeds the number of questions in the exam JSON file, then the entire contents of
        /// the exam file will be studied. After the study session has completed, a ratio of the
        /// number of questions correctly answered to the number of questions studied will be
        /// displayed.
        pub fn study(&self) {
            let mut num_correct: u8 = 0;
            println!("\n\nExam selected: {}", &self.name);

            let num_questions: usize = loop {
                match Self::input("How many questions would you like to review? ").parse::<usize>() {
                    Ok(num) if num > 0 => break min(num, self.questions.len()),
                    _ => println!("Please enter a positive number!"),
                }
            };

            for question in self.questions.iter().take(num_questions) {
                println!("\n{}", question.prompt);

                let choices: Vec<String> = question.choices
                    .iter()
                    .enumerate()
                    .map(|(index, choice)| {
                        println!("{}.) {}", (index as u8 + b'a') as char, choice);
                        choice.to_string()
                    }).collect();

                let user_answer_ind = loop {
                    let user_ans: String = Self::input("Enter answer (e.g., 'a', 'b', 'c'): ");
                    if let Some(index) = user_ans.chars().next().map(|c| (c as u8 - b'a') as usize) {
                        if index < choices.len() {
                            break index;
                        }
                    }
                    println!("Please make a valid selection!");
                };

                if choices[user_answer_ind].eq_ignore_ascii_case(&question.answer) {
                    println!("\nCorrect!");
                    num_correct += 1;
                } else {
                    println!("\nIncorrect...");
                }

                println!("Explanation: {}", question.explanation);
                println!("Reference(s):");
                question.refs.iter().for_each(|r| println!("\t{}", r));
            }

            // Whether or not to play again
            if !Self::input("Play again (Y/n)? ").eq_ignore_ascii_case("y") {
                println!("\nYou got {}/{} questions correct.", num_correct, num_questions);
                println!("Great progress studying!");
            } else {
                self.study();
            }
        }
    }

}

fn main() {
    if let Some(exam) = Exam::new() {
        exam.study();
    } else {
        println!("Unable to study...");
    }
}
