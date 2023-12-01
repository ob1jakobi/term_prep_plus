use std::process::exit;
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
    const START_ITALICS: &str = "\x1B[3m";
    const END_ITALICS: &str = "\x1B[23m";

    /// High-level structure representing an Exam; has a name and a series of questions
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Exam {
        name: String,
        questions: HashSet<Question>,
    }

    /// The questions that comprise an Exam
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Question {
        q_type: String,
        prompt: String,
        choices: HashSet<String>,
        answer: Vec<String>,
        explanation: String,
        refs: Vec<String>,
    }

    /// The next three are required to utilize Questions as a HashSet; this helps ensure that
    /// the sequence of questions are not revealed in the same sequence (as would be the case if
    /// the Exam struct utilized a Vec<Question>)
    impl PartialEq<Self> for Question {
        fn eq(&self, other: &Self) -> bool {
            self.q_type == other.q_type
            && self.prompt == other.prompt
            && self.choices == other.choices
            && self.answer == other.answer
            && self.explanation == other.explanation
            && self.refs == other.refs
        }
    }
    impl Eq for Question {}

    impl Hash for Question {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.q_type.hash(state);
            self.prompt.hash(state);
            self.choices.iter().for_each(|choice| choice.hash(state));
            self.answer.iter().for_each(|ans| ans.hash(state));
            self.explanation.hash(state);
            self.refs.hash(state);
        }
    }

    impl Exam {
        /// Attempts to create an Exam if an exam JSON file exists and is properly formatted.
        pub fn new() -> Option<Self> {
            match env::current_dir() {
                Ok(cwd) if Self::create_asset_dir(&cwd) => Some(Self::get_exam(&cwd)),
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
        fn get_exam(cwd: &PathBuf) -> Exam {
            let result: Exam = loop {
                let assets_dir: PathBuf = Self::select_asset_directory(cwd);
                match Self::display_and_collect_available_exams(assets_dir) {
                    Some(empty_dir) if empty_dir.is_empty() => {
                        eprintln!("{}There are no available exam files in chosen directory{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                    },
                    Some(exam_dir) => {
                        // Get the appropriate exam from the list provided
                        let exam_path = loop {
                            let prompt = "Enter the exam number (e.g., '1', '2', '3', ...): ";
                            let index = Self::input(prompt).parse::<usize>().unwrap_or(usize::MAX) - 1;
                            match exam_dir.get(index) {
                                Some(exam) => break exam,
                                _ => eprintln!("{}Please make a valid selection!{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                            }
                        };
                        // Open the file and attempt to parse the contents into an exam
                        if let Ok(exam_file) = File::open(exam_path) {
                            let reader = BufReader::new(exam_file);
                            match serde_json::from_reader(reader) {
                                Ok(exam) => break exam,
                                Err(e) => eprintln!("{}Unable to parse JSON file:\t{}{}", RED_COLOR_CODE, e, RESET_COLOR_CODE),
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
        ///
        /// # Panics
        /// if the JSON file that was chosen doesn't match one of the 3 allowable `q_type` variations
        /// * `mc` - for multiple choice questions
        /// * `ms` - for multiple select questions
        /// * `ue` - for user entry
        pub fn study(&self) {
            // Counts the number of questions the user answers correctly
            let mut num_correct = 0;

            // Display the exam the user selected to study
            println!("\n\n{}Exam selected: {}{}", GREEN_COLOR_CODE, &self.name, RESET_COLOR_CODE);

            // Ask the user for desired number of questions and save result
            let num_questions: usize = loop {
                match Self::input("How many questions would you like to review? ").parse::<usize>() {
                    Ok(num) if num > 0 => break min(num, self.questions.len()),
                    _ => eprintln!("{}Please enter a positive number!{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                }
            };

            // Iterate over the number of questions the user specified
            for question in self.questions.iter().take(num_questions) {
                // Display the question prompt
                println!("\n{}", question.prompt);

                // logic depends on question type
                match question.q_type.as_ref() {
                    "mc" => {
                        let choices = Self::display_choices_and_collect(question);

                        // Get the user's answer based on the letter prefix printed above
                        let user_answer: String = loop {
                            let letter_choice: String = Self::input("Enter answer (e.g., 'a', 'b', 'c', ...): ");
                            let choice_as_index: usize = letter_choice
                                .chars()
                                .next()
                                .map_or(usize::MAX, |c| (c as u8 - b'a') as usize);
                            match choices.get(choice_as_index) {
                                Some(choice) => break choice.to_string(),
                                None => eprintln!("{}Please pick a valid answer!{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                            }
                        };

                        // Get the correct answer from the vector and print out user's result
                        match question.answer.get(0) {
                            Some(correct_ans) if user_answer.eq(correct_ans.as_str()) => {
                                println!("{}Correct!{}", GREEN_COLOR_CODE, RESET_COLOR_CODE);
                                num_correct += 1;
                            },
                            _ => {
                                println!("{}Incorrect...{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                                println!("{}The correct answer(s): {:#?}{}", YELLOW_COLOR_CODE, question.answer, RESET_COLOR_CODE);
                            },

                        }
                    },
                    "ms" => {
                        let choices = Self::display_choices_and_collect(question);
                        // Get the user's multiple select answer(s)
                        let mut user_sel = loop {
                            let mut has_bad_input = false;
                            let prompt = "Enter comma-separated answer (e.g., 'a, b', or 'c'): ";
                            let user_ans = Self::input(prompt).split(", ").filter_map(|choice| {
                                match choice.chars().next().map_or(usize::MAX, |c| (c as u8 - b'a') as usize) {
                                    num if choices.get(num).is_none() => {
                                        eprintln!("{}Please enter a valid selection from available choices{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                                        has_bad_input = true;
                                        None
                                    },
                                    num => choices.get(num),
                                }
                            })
                                .collect::<HashSet<&String>>();
                            if !has_bad_input {
                                break user_ans
                            }
                        };
                        // If # of user choices != number of answer, then it's incorrect
                        if user_sel.len() == question.answer.len() {
                            // Iterate over correct answers, removing each from user's choices
                            for ans in question.answer.iter() {
                                user_sel.remove(ans);
                            }
                        }
                        // If user answered correctly, then the HashSet should've had all items removed
                        if user_sel.is_empty() {
                            println!("{}Correct!{}", GREEN_COLOR_CODE, RESET_COLOR_CODE);
                            num_correct += 1;
                        } else {
                            println!("{}Incorrect...{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                            println!("{}The correct answer(s): {:#?}{}", YELLOW_COLOR_CODE, question.answer, RESET_COLOR_CODE);
                        }
                    },
                    "ue" => {
                        // Collect the hint(s), if any
                        let hints = Self::display_choices_and_collect(question);
                        // Get the user's input; display prompt and show hint(s), if available
                        let user_ans: String = loop {
                            match hints.len() {
                                num if num > 0 => {
                                    let input = Self::input("Enter your answer (or enter 'hint' to see hints): ");
                                    if input.eq_ignore_ascii_case("hint") {
                                        Self::display_hints(&hints);
                                    } else {
                                        break input
                                    }
                                },
                                _ => {
                                    let input = Self::input("Enter your answer: ");
                                    if input.eq_ignore_ascii_case("hint") {
                                        eprintln!("{}This question doesn't have any hints...{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                                    } else {
                                        break input
                                    }
                                }
                            }
                        };
                        let mut is_correct = false;
                        for answer in question.answer.iter() {
                            if user_ans.eq(answer) {
                                is_correct = true;
                                break;
                            }
                        }
                        if is_correct {
                            println!("{}Correct!{}", GREEN_COLOR_CODE, RESET_COLOR_CODE);
                            num_correct += 1;
                        } else {
                            println!("{}Incorrect...{}", RED_COLOR_CODE, RESET_COLOR_CODE);
                            println!("{}The correct answer(s): {:#?}{}", YELLOW_COLOR_CODE, question.answer, RESET_COLOR_CODE);
                        }
                    },
                    _ => panic!("{}q_type field not recognized{}", RED_COLOR_CODE, RESET_COLOR_CODE),
                }
                // Sleep for a bit so that the user can see the result before adding extra text
                std::thread::sleep(std::time::Duration::from_millis(500));

                // Only print the explanation if one is provided; self-explanatory questions don't need explanation
                if !question.explanation.is_empty() {
                    println!("{}Explanation: {}{}", YELLOW_COLOR_CODE, question.explanation, RESET_COLOR_CODE);
                }
                // Always print reference(s)
                println!("{}Reference(s):\n\t{}{}", CYAN_COLOR_CODE, question.refs.join("\n\t"), RESET_COLOR_CODE);

                // Sleep for a sec so that the user can see explanation & references
                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            // Ask whether or not to play again
            match Self::input("\n\nPlay again (Y/n)? ").chars().next().unwrap_or('n') {
                'y' | 'Y' => self.study(),
                _ => {
                    println!("\nYou got {}/{} questions correct.", num_correct, num_questions);
                    println!("Great progress studying!");
                }
            }
        }

        /// Helper function for displaying hints for user entry questions.
        fn display_hints(hints_ref: &Vec<String>) {
            hints_ref.iter().for_each(|hint| {
                println!("{}\t{}Hint: {}{}{}", BLUE_COLOR_CODE, START_ITALICS, hint, END_ITALICS, RESET_COLOR_CODE);
            })
        }

        /// Helper function that iterates over the `choices` field of the parameter `Question`.
        /// The way a choice/option will be displayed depends on the `q_type` field.
        fn display_choices_and_collect(question_ref: &Question) -> Vec<String> {
            question_ref.choices.iter().enumerate().filter_map(|(index, choice)| {
                match question_ref.q_type.as_str() {
                    "mc" | "ms" => {
                        println!("{}\t{}.) {}{}", BLUE_COLOR_CODE, (index as u8 + b'a') as char, choice, RESET_COLOR_CODE);
                        Some(choice.to_string())
                    },
                    "ue" if !choice.is_empty() => {
                        // Don't print hint(s) - let the user decide
                        Some(choice.to_string())
                    },
                    _ => {
                        // Executes if there's no hints provided for ue questions
                        None
                    },
                }
            })
                .collect()
        }
    }
}

fn main() {
    println!("{}", LOGO);
    if let Some(exam) = Exam::new() {
        exam.study();
    } else {
        eprintln!("Unable to study today...");
        exit(1);
    }
}
