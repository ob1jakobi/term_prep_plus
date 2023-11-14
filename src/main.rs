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

        /// Gets the appropriate exam directory from the user for the study session, attempts to
        /// get the appropriate `Exam` via an `Option` depending on whether the JSON file exists.
        fn get_exam(cwd: &PathBuf) -> Option<Exam> {
            let asset_dir: PathBuf = Self::select_asset_directory(cwd);
            match Self::display_and_collect_available_exams(asset_dir) {
                Some(exam_vec) if exam_vec.is_empty() => {
                    println!("There are no exam files in the desired directory");
                    None
                },
                Some(exam_vec) => {
                    let user_choice: usize = loop {
                        match Self::input("Enter the number for the exam you want to study (i.e. '1', '2', '3'): ").parse::<usize>() {
                            Ok(num) if num > 0 && num <= exam_vec.len() => break num - 1,
                            _ => println!("Please enter a valid number!"),
                        }
                    };
                    match exam_vec.get(user_choice) {
                        Some(exam_path) => {
                            match File::open(exam_path) {
                                Ok(exam_file) => {
                                    let reader = BufReader::new(exam_file);
                                    match serde_json::from_reader(reader) {
                                        Ok(exam) => Some(exam),
                                        Err(e) => {
                                            eprintln!("Unable to parse JSON exam file; Error: {}", e);
                                            None
                                        },
                                    }
                                },
                                Err(e) => {
                                    eprintln!("An error occurred opening the exam file; Error: {}", e);
                                    None
                                },
                            }
                        },
                        None => {
                            eprintln!("Unable to get the desired exam from the list of available exams");
                            None
                        }
                    }
                },
                None => {
                    eprintln!("No exams available to study with at desired directory");
                    None
                },
            }
        }

        /// Helper function that obtains the path to the directory where the user has stored their
        /// exam files. The user can opt to use the `assets` directory, which is created as one of
        /// the initial steps in the `Exam` constructor, or uses a different directory of the user's
        /// choosing.
        fn select_asset_directory(cwd: &PathBuf) -> PathBuf {
            let asset_dir: PathBuf = loop {
                let use_default_dir = Self::input("Search default directory for exam files (Y/n)? ");
                if use_default_dir.eq_ignore_ascii_case("y") {
                    break cwd.join(ASSETS_DIR)
                } else if use_default_dir.eq_ignore_ascii_case("n") {
                    let user_dir = PathBuf::from(Self::input_confirm("Enter the desired exam file directory: "));
                    if user_dir.exists() && user_dir.is_dir() {
                        break user_dir
                    } else {
                        println!("Please enter a valid directory!");
                    }
                } else {
                    println!("Please enter a valid option!");
                }
            };
            asset_dir
        }

        /// Lists the exams that are available to study by the file extension ending in `json` at
        /// the directory provided. If the directory with the exam files exist, this display the
        /// exams with a number prefix and return an `Option` with the vector containing the file
        /// paths.
        fn display_and_collect_available_exams(dir: PathBuf) -> Option<Vec<PathBuf>> {
            let mut exam_number: usize = 1;
            match fs::read_dir(&dir) {
                Ok(entries) => {
                    let exam_files: Vec<PathBuf> = entries
                        .filter_map(|entry| {
                            match entry {
                                Ok(entry) if entry.path().is_file() && entry.path().extension().map_or(false, |ext| ext == "json") => {
                                    println!("\t{}.) {}", exam_number, entry.path().file_name().expect("Unable to get filename").to_string_lossy());
                                    exam_number += 1;
                                    Some(entry.path())
                                },
                                _ => None
                            }
                        })
                        .collect();
                    Some(exam_files)
                },
                Err(e) => {
                    eprintln!("Unable to read files in provided directory; Error: {}", e);
                    None
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
