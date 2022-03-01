use crate::communicate::send_result;
use crate::config::Config;
use phf::phf_map;
use serde::Serialize;
use serde_derive::Serialize;
use std::fmt::Display;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::{
    error::Error,
    process::{Command, Stdio},
};

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if let Err(err) = compile(&config.language, &config.file_path) {
        send_result(JudgeResult {
            id: config.id.clone(),
            result: JudgeCase::CompileError(err.to_string()),
            case: 0,
            time: 0,
            memory: 0,
        })
        .await;
    } else {
        for (i, case) in config.test_case_paths.iter().enumerate() {
            println!("Running test case: {:?}", case);
            let judge_case;
            let run_result =
                run_one_case(&case.input, config.time_limit, config.memory_limit).unwrap();
            println!("{:?}", run_result);

            let compare_result = compare_answer(PROGRAM_OUTPUT_FILE_NAME, &case.output).unwrap();
            println!("{:?}", compare_result);
            match compare_result {
                CompareResult::Ok => judge_case = JudgeCase::Accepted,
                CompareResult::Msg(msg) => judge_case = JudgeCase::WrongAnswer(msg),
            }
            println!("Judge result: {:?}", judge_case);
            send_result(JudgeResult {
                id: config.id.to_string(),
                result: judge_case,
                case: i as i32 + 1,
                time: run_result.time,
                memory: run_result.memory,
            })
            .await;
        }
    }
    Ok(())
}

const LANGUAGE_COMPILER: phf::Map<&'static str, &'static str> = phf_map! {
"c++"=> "g++",
"c"=> "gcc",
};
// const COMPILER_ARGS: phf::Map<&'static str, &'static str> = phf_map(&[
//     ("c", "-std=c11 -O2 -Wall -lm -DONLINE_JUDGE"),
//     ("c++", "-std=c++14 -O2 -Wall -lm -DONLINE_JUDGE"),
// ]);

const COMPILER_ARGS: phf::Map<&'static str, &'static str> = phf_map! {
    "c"=> "-std=c11 -O2 -Wall -lm -DONLINE_JUDGE",
    "c++"=> "-std=c++14 -O2 -Wall -lm -DONLINE_JUDGE",
};

// const COMPILER_ARGS: HashMap<&'static str, &'static str> = {
//     let mut map = HashMap::new();
//     map.insert("c++", "-std=c++14 -O2 -Wall -lm -DONLINE_JUDGE");
//     map.insert("c", "-std=c11 -O2 -Wall -lm -DONLINE_JUDGE");
//     map.insert("java", "-DONLINE_JUDGE");
//     map.insert("python", "-DONLINE_JUDGE");
//     map.insert("rust", "-DONLINE_JUDGE");
//     map.insert("javascript", "-DONLINE_JUDGE");
//     map
// };

const COMPILE_ERROR_INFO: &str = "compile_error.txt";
const COMPILED_PROGRAM_NAME: &str = "a.out";
const PROGRAM_OUTPUT_FILE_NAME: &str = "program_out.txt";

fn compile(language: &str, file_path: &str) -> Result<(), String> {
    let compiler = *LANGUAGE_COMPILER
        .get(language)
        .ok_or("language not supported")?;
    let compile_args = *COMPILER_ARGS
        .get(language)
        .ok_or("language not supported")?;
    let error_file = File::create(COMPILE_ERROR_INFO).unwrap();

    let output = Command::new(compiler)
        .arg(file_path)
        .args(compile_args.split_whitespace())
        .args(["-o", COMPILED_PROGRAM_NAME])
        .stderr(Stdio::from(error_file))
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        let mut error_file = File::open(COMPILE_ERROR_INFO).unwrap();
        let mut compile_error_info = String::new();
        let _ = error_file.read_to_string(&mut compile_error_info).unwrap();
        return Err(compile_error_info);
    }
    Ok(())
}

fn run_one_case(
    input_path: &str,
    time_limit: i32,
    memory_limit: i32,
) -> Result<RunResult, &'static str> {
    let input = std::fs::File::open(input_path).expect("input file not found");
    let output = std::fs::File::create(PROGRAM_OUTPUT_FILE_NAME).unwrap();

    println!("./{}", COMPILED_PROGRAM_NAME);

    // let timeout = Command::new("timeout")
    //     .arg(format!("{}s", time_limit / 1000))
    //     .arg(format!(
    //         "./{} < {} > {}",
    //         COMPILED_PROGRAM_NAME, input_path, PROGRAM_OUTPUT_FILE_NAME
    //     ))
    //     .output()
    //     .expect("failed to execute process");

    let result = Command::new(format!("./{}", COMPILED_PROGRAM_NAME))
        .stdin(Stdio::from(input))
        .stdout(Stdio::from(output))
        .output()
        .expect("failed to execute process");

    println!("{:?}", result);

    if !result.status.success() {
        return Err("program error");
    }

    Ok(RunResult { time: 0, memory: 0 })
}

#[derive(Debug, Serialize)]
pub struct JudgeResult {
    pub id: String,
    pub case: i32,
    pub result: JudgeCase,
    pub time: i32,
    pub memory: i32,
}

#[derive(Debug)]
pub enum JudgeCase {
    Accepted,
    WrongAnswer(String),
    RuntimeError(String),
    TimeLimitExceeded,
    MemoryLimitExceeded,
    CompileError(String),
}

impl Display for JudgeCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JudgeCase::Accepted => write!(f, "Accepted"),
            JudgeCase::WrongAnswer(answer) => write!(f, "WrongAnswer: {}", answer),
            JudgeCase::RuntimeError(error) => write!(f, "RuntimeError: {}", error),
            JudgeCase::TimeLimitExceeded => write!(f, "TimeLimitExceeded"),
            JudgeCase::MemoryLimitExceeded => write!(f, "MemoryLimitExceeded"),
            JudgeCase::CompileError(info) => write!(f, "CompileError: {}", info),
        }
    }
}

impl Serialize for JudgeCase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JudgeCase::Accepted => serializer.serialize_str("Accepted"),
            JudgeCase::WrongAnswer(answer) => {
                serializer.serialize_str(&format!("WrongAnswer: {}", answer))
            }

            JudgeCase::RuntimeError(error) => {
                serializer.serialize_str(&format!("RuntimeError: {}", error))
            }

            JudgeCase::TimeLimitExceeded => serializer.serialize_str("TimeLimitExceeded"),
            JudgeCase::MemoryLimitExceeded => serializer.serialize_str("MemoryLimitExceeded"),
            JudgeCase::CompileError(info) => {
                serializer.serialize_str(&format!("CompileError: {}", info))
            }
        }
    }
}

#[derive(Debug)]
struct RunResult {
    time: i32,
    memory: i32,
}

#[derive(PartialEq, Debug)]
enum CompareResult {
    Ok,
    Msg(String),
}

fn compare_answer(program_output: &str, answer: &str) -> Result<CompareResult, &'static str> {
    let mut program_output = std::fs::File::open(program_output).expect("file not found");
    let mut answer = std::fs::File::open(answer).expect("file not found");

    let program_output = BufReader::new(&mut program_output).lines();
    let answer = BufReader::new(&mut answer).lines();

    for (i, (line1, line2)) in answer.zip(program_output).enumerate() {
        if line1.is_err() || line2.is_err() {
            return Err("file format error");
        }
        let line1 = line1.unwrap();
        let line2 = line2.unwrap();

        // compare line length
        if line1.len() < line2.len() {
            return Ok(CompareResult::Msg(format!(
                "wrong answer Too long on line {}.",
                i + 1
            )));
        }

        // compare line content by character
        for (j, (c1, c2)) in line2.chars().zip(line1.chars()).enumerate() {
            if c1 != c2 {
                return Ok(CompareResult::Msg(format!(
                    "wrong answer On line {} column {}, read {}, expected {}.",
                    i + 1,
                    j + 1,
                    c1,
                    c2
                )));
            }
        }
    }

    // Ok
    Ok(CompareResult::Ok)
}

#[cfg(test)]
mod tests {

    use crate::config::Case;

    use super::*;

    #[test]
    fn test_compile() {
        let config = Config {
            id: "test".to_string(),
            language: "c++".to_string(),
            file_path: "main.cpp".to_string(),
            time_limit: 1000,
            memory_limit: 1000,
            test_case_number: 1,
            test_case_paths: vec![Case {
                input: "in.txt".to_string(),
                output: "out.txt".to_string(),
            }],
        };
        let result = compile(&config.language, &config.file_path);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_judge_one_case() {
        test_compile();
        let run_result = run_one_case("in.txt", 1000, 1000).unwrap_or_else(|err| {
            eprintln!("Err: {}", err);
            RunResult {
                time: -1,
                memory: -1,
            }
        });
        println!("{:?}", run_result);
    }

    #[test]
    fn test_compare_file() {
        let result = compare_answer(PROGRAM_OUTPUT_FILE_NAME, "out.txt").unwrap();
        println!("{:?}", result);
        assert_eq!(result, CompareResult::Ok);
    }
}
