use std::io::{self, Write};

pub(crate) fn run(database_name: &str) -> io::Result<()> {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("{database_name}> ");
        io::stdout().flush()?;

        line.clear();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }

        match line.trim() {
            "" => {}
            "exit" => break,
            "help" => println!("Available commands are help and exit."),
            command => eprintln!("Unknown command '{command}'. Type 'help'."),
        }
    }

    Ok(())
}
