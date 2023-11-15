use crate::exam::Exam;

const LOGO: &str = "

████████╗███████╗██████╗ ███╗   ███╗    ██████╗ ██████╗ ███████╗██████╗     ██████╗ ██╗     ██╗   ██╗███████╗
╚══██╔══╝██╔════╝██╔══██╗████╗ ████║    ██╔══██╗██╔══██╗██╔════╝██╔══██╗    ██╔══██╗██║     ██║   ██║██╔════╝
   ██║   █████╗  ██████╔╝██╔████╔██║    ██████╔╝██████╔╝█████╗  ██████╔╝    ██████╔╝██║     ██║   ██║███████╗
   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║    ██╔═══╝ ██╔══██╗██╔══╝  ██╔═══╝     ██╔═══╝ ██║     ██║   ██║╚════██║
   ██║   ███████╗██║  ██║██║ ╚═╝ ██║    ██║     ██║  ██║███████╗██║         ██║     ███████╗╚██████╔╝███████║
   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝    ╚═╝     ╚═╝  ╚═╝╚══════╝╚═╝         ╚═╝     ╚══════╝ ╚═════╝ ╚══════╝

";

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

    /// Color codes for changing the color of stdout
    const RED_COLOR_CODE: &str = "\x1b[31m";
    const BLUE_COLOR_CODE: &str = "\x1b[34m";
    const GREEN_COLOR_CODE: &str = "\x1b[32m";
    const YELLOW_COLOR_CODE: &str = "\x1b[33m";
    const CYAN_COLOR_CODE: &str = "\x1b[36m";
    const RESET_COLOR_CODE: &str = "\x1b[0m";

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
        /// Attempts to create an Exam if an exam JSON file exists and is properly formatted.
        pub fn new() -> Option<Self> {
            match env::current_dir() {
                Ok(cwd) if Self::create_asset_dir(&cwd) => Self::get_exam(&cwd),
                _ => {
                    eprintln!("{}Unable to create Exam{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                    None
                },
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
                    eprintln!("An error {} occurred creating the {} directory...", e, ASSETS_DIR);
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
            let result: Option<Exam> = loop {
                let asset_dir: PathBuf = Self::select_asset_directory(cwd);
                match Self::display_and_collect_available_exams(asset_dir) {
                    Some(empty_dir) if empty_dir.is_empty() => {
                        eprintln!("{}There are no available exam files in chosen directory{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                    },
                    Some(exam_dir) => {
                        let usr_choice_prefix: usize = loop {
                            match Self::input("Enter the exam file number (e.g., '1', '2', '3'): ").parse::<usize>() {
                                Ok(valid_index) if valid_index > 0 && valid_index <= exam_dir.len() => break valid_index - 1,
                                _ => eprintln!("{}Please make a valid selection!{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                            }
                        };
                        let exam_file_path: Option<&PathBuf> = exam_dir.get(usr_choice_prefix);
                        if exam_file_path.is_none() {
                            eprintln!("{}Unable to extract exam file for chosen exam{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                        } else if let Ok(exam_file) = File::open(exam_file_path.unwrap()) {
                            let reader: BufReader<File> = BufReader::new(exam_file);
                            match serde_json::from_reader(reader) {
                                Ok(exam) => break Some(exam),
                                Err(_) => eprintln!("{}Unable to parse JSON file{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                            }
                        } else {
                            eprintln!("{}Unable to open selected exam{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                        }
                    },
                    None => eprintln!("{}Unable to get list of exam files in chosen directory{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                }
            };
            result
        }

        /// Helper function that obtains the path to the directory where the user has stored their
        /// exam files. The user can opt to use the `assets` directory, which is created as one of
        /// the initial steps in the `Exam` constructor, or uses a different directory of the user's
        /// choosing.
        fn select_asset_directory(cwd: &PathBuf) -> PathBuf {
            loop {
                match Self::input("\nSearch default directory for exam files (Y/n)? ").chars().next().unwrap_or('n') {
                    'y' | 'Y' => break cwd.join(ASSETS_DIR),
                    'n' | 'N' => {
                        let user_dir = PathBuf::from(Self::input_confirm("Enter full path to exam directory: "));
                        if user_dir.exists() && user_dir.is_dir() {
                            break user_dir
                        } else {
                            eprintln!("{}Please enter a valid directory!{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                        }
                    },
                    _ => eprintln!("{}Please enter a valid option!{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                }
            }
        }

        /// Lists the exams that are available to study by the file extension ending in `json` at
        /// the directory provided. If the directory with the exam files exist, this display the
        /// exams with a number prefix and return an `Option` with the vector containing the file
        /// paths.
        fn display_and_collect_available_exams(dir: PathBuf) -> Option<Vec<PathBuf>> {
            if let Ok(entries) = fs::read_dir(&dir) {
                println!("\nThe following compatible exam files were found:");
                let exams: Vec<PathBuf> = entries
                    .filter(|e|
                         e.as_ref().is_ok_and(|e|
                             e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "json")
                         )
                    )
                    .enumerate()
                    .map(|(index, e)| {
                        let path: PathBuf = e.unwrap().path();
                        let filename: &str = path.file_name().unwrap().to_str().unwrap();
                        println!("\t{}{}.) {}{}", BLUE_COLOR_CODE, index + 1, filename, RESET_COLOR_CODE);
                        path
                    })
                    .collect();
                Some(exams)
            } else {
                eprintln!("{}Unable to read files in selected directory{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                None
            }
        }

        /// Helper function for displaying a prompt that the user can respond to in-line with the
        /// prompt.
        fn input(prompt: &str) -> String {
            let mut temp: String = String::new();
            while temp.trim().is_empty() {
                print!("{}", prompt);
                stdout().flush().expect("Unable to flush stdout...");
                stdin().read_line(&mut temp).expect("Unable to read from stdin");
            }
            temp.trim().to_string()
        }

        /// Helper function that prompts the user to enter info in-line with a prompt twice to
        /// verify the user's input is accurate.
        fn input_confirm(prompt: &str) -> String {
            loop {
                let in1: String = Self::input(prompt);
                let in2: String = Self::input("Confirm entry: ");
                if in1.eq(&in2) {
                    return in2;
                } else {
                    eprintln!("{}Entries must match!{}", RED_COLOR_CODE, RESET_COLOR_CODE);
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
            // For identifying the user's number of correct answers
            let mut num_correct: u8 = 0;
            println!("\n\n{}Exam selected: {}{}", GREEN_COLOR_CODE, &self.name, RESET_COLOR_CODE);
            // Get the number of questions to study; ensures value range is [1, self.questions.len)
            let num_questions: usize = loop {
                match Self::input("How many questions would you like to review? ").parse::<usize>() {
                    Ok(num) if num > 0 => break min(num, self.questions.len()),
                    _ => eprintln!("{}Please enter a positive number!{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                }
            };
            // Iterate over questions in range specified by user.
            for question in self.questions.iter().take(num_questions) {
                // Display the question prompt so the user knows the question to answer
                println!("\n{}", question.prompt);
                // Covert question.choices to vector for easy indexing; side effect of printing the
                // choice with a letter prefix so the user can use the prefix to select their answer
                let choices: Vec<String> = question.choices
                    .iter()
                    .enumerate()
                    .map(|(index, choice)| {
                        println!("{}\t{}.) {}{}", BLUE_COLOR_CODE, (index as u8 + b'a') as char, choice, RESET_COLOR_CODE);
                        choice.to_string()
                    })
                    .collect();
                // Get the user's answer; if the user's answer is a valid choice, then the choice
                // itself (i.e., the element from question.choices) will be returned.
                let user_answer: String = loop {
                    let letter_choice: String = Self::input("Enter answer (e.g., 'a', 'b', 'c'): ");
                    let choice_as_index: usize = letter_choice
                        .chars()
                        .next()
                        .map_or(usize::MAX, |c| (c as u8 - b'a') as usize);
                    match choices.get(choice_as_index) {
                        Some(choice) => break choice.to_string(),
                        None => eprintln!("{}Please pick a valid answer!{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                    }
                };
                // Let the user know if they've answered correctly; if so, increment num_correct
                if user_answer.eq(&question.answer) {
                    println!("{}Correct!{}", GREEN_COLOR_CODE, RESET_COLOR_CODE);
                    num_correct += 1;
                } else {
                    println!("{}Incorrect...{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                }
                // Only print the explanation if one is provided; self-explanatory questions don't need explanation
                if !question.explanation.is_empty() {
                    println!("{}Explanation: {}{}", YELLOW_COLOR_CODE, question.explanation, RESET_COLOR_CODE);
                }
                // Always print reference(s)
                println!("{}Reference(s):\n\t{}{}", CYAN_COLOR_CODE, question.refs.join("\n\t"), RESET_COLOR_CODE);
            }

            // Whether or not to play again
            match Self::input("\n\nPlay again (Y/n)? ").chars().next().unwrap_or('n') {
                'y' | 'Y' => self.study(),
                _ => {
                    println!("\nYou got {}/{} questions correct.", num_correct, num_questions);
                    println!("Great progress studying!");
                }
            }
        }
    }

}

fn main() {
    println!("{}", LOGO);
    if let Some(exam) = Exam::new() {
        exam.study();
    } else {
        println!("Unable to study...");
    }
}
