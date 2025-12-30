use std::path::PathBuf;

#[derive(Debug)]
pub enum Command {
    Add(PathBuf),
    Commit(String)
}

impl Command {
    pub fn parse(args: Vec<String>) -> Result<Self, String> {
        if args.len() != 3 {
            return Err("Missing arguments".into())
        };

        let command = args[1].to_lowercase();
        let arguments = args[2].clone();

        match command.as_str() {
            "add" => {
                let path = PathBuf::from(&arguments);
                Ok(Command::Add(path))
            },
            "commit" => {
                Ok(Command::Commit(arguments))
            },
            unknown => {
                Err(format!("Unexpected command {}", unknown))
            }
        }
    }
}
