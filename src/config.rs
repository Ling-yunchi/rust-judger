use std::error::Error;
use clap::Parser;

#[derive(Parser,Debug)]
#[clap(author, version, about,long_about=None]
pub struct Config {
    pub id: String,
    pub language: String,
    pub file_path: String,
    pub time_limit: i32,
    pub memory_limit: i32,
    pub test_case_number: i32,
    pub test_case_paths: Vec<Case>,
}

impl Config {
    pub fn new(args: Vec<String>) -> Result<Config, Box<dyn Error>> {
        if args.len() < 6 {
            return Err("not enough arguments".into());
        }

        let mut config = Config {
            id: args[1].clone(),
            language: args[2].clone(),
            file_path: args[3].clone(),
            time_limit: args[4].parse::<i32>()?,
            memory_limit: args[5].parse::<i32>()?,
            test_case_number: args[6].parse::<i32>()?,
            test_case_paths: Vec::new(),
        };

        if config.test_case_number < 1 {
            return Err("test_case_number must be greater than 0".into());
        }

        if args.len() < (7 + config.test_case_number).try_into().unwrap() {
            return Err("not enough test cases".into());
        }

        for i in 7..(7 + config.test_case_number).try_into().unwrap() {
            let case = args[i].split('#').collect::<Vec<&str>>();
            config.test_case_paths.push(Case {
                input: case[0].to_string(),
                output: case[1].to_string(),
            });
        }
        let config = config;
        Ok(config)
    }
}

#[derive(Debug)]
pub struct Case {
    pub input: String,
    pub output: String,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_one_case() {
        let args = vec![
            "program-runner".to_string(),
            "judge-id".to_string(),
            "c++".to_string(),
            "main.cpp".to_string(),
            "1000".to_string(),
            "1000".to_string(),
            "1".to_string(),
            "input.txt#output.txt".to_string(),
        ];
        let config = super::Config::new(args).unwrap();

        println!("{:?}", config);

        assert_eq!(config.id, "judge-id".to_string());
        assert_eq!(config.language, "c++");
        assert_eq!(config.file_path, "main.cpp".to_string());
        assert_eq!(config.time_limit, 1000);
        assert_eq!(config.memory_limit, 1000);
        assert_eq!(config.test_case_number, 1);
        assert_eq!(config.test_case_paths.len(), 1);
        assert_eq!(config.test_case_paths[0].input, "input.txt");
        assert_eq!(config.test_case_paths[0].output, "output.txt");
    }

    #[test]
    fn test_many_case() {
        let args = vec![
            "program-runner".to_string(),
            "c++".to_string(),
            "main.cpp".to_string(),
            "1000".to_string(),
            "1000".to_string(),
            "3".to_string(),
            "input.txt#output.txt".to_string(),
            "input2.txt#output2.txt".to_string(),
            "input3.txt#output3.txt".to_string(),
        ];
        let config = super::Config::new(args).unwrap();

        println!("{:?}", config);

        assert_eq!(config.language, "c++");
        assert_eq!(config.file_path, "main.cpp".to_string());
        assert_eq!(config.time_limit, 1000);
        assert_eq!(config.memory_limit, 1000);
        assert_eq!(config.test_case_number, 3);
        assert_eq!(config.test_case_paths.len(), 3);
        assert_eq!(config.test_case_paths[0].input, "input.txt");
        assert_eq!(config.test_case_paths[0].output, "output.txt");
        assert_eq!(config.test_case_paths[1].input, "input2.txt");
        assert_eq!(config.test_case_paths[1].output, "output2.txt");
        assert_eq!(config.test_case_paths[2].input, "input3.txt");
        assert_eq!(config.test_case_paths[2].output, "output3.txt");
    }

    #[test]
    fn test_not_have_enough_args() {
        let args = vec![
            "program-runner".to_string(),
            "c++".to_string(),
            "main.cpp".to_string(),
            "1000".to_string(),
            "1000".to_string(),
        ];
        let config = super::Config::new(args);
        assert!(config.is_err());
        assert_eq!(config.unwrap_err(), "not enough arguments");
    }

    #[test]
    fn test_case_number_is_not_positive() {
        let args = vec![
            "program-runner".to_string(),
            "c++".to_string(),
            "main.cpp".to_string(),
            "1000".to_string(),
            "1000".to_string(),
            "-1".to_string(),
            "input.txt#output.txt".to_string(),
        ];
        let config = super::Config::new(args);
        assert!(config.is_err());
        assert_eq!(
            config.unwrap_err(),
            "test_case_number must be greater than 0"
        );
    }

    #[test]
    fn test_case_number_is_not_enough() {
        let args = vec![
            "program-runner".to_string(),
            "c++".to_string(),
            "main.cpp".to_string(),
            "1000".to_string(),
            "1000".to_string(),
            "2".to_string(),
            "input.txt#output.txt".to_string(),
        ];
        let config = super::Config::new(args);
        assert!(config.is_err());
        assert_eq!(config.unwrap_err(), "not enough test cases");
    }
}
