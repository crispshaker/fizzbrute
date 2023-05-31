use std::io::Write;

fn load_tokes_from_file(contents: &str) -> Vec<&str> {
    let mut output: Vec<&str> = Vec::new();
    for line in contents.lines() {
        output.push(line)
    }
    output
}

fn load_desired_string_from_file() -> String {
    std::fs::read_to_string("desired_string.txt").expect("Failed reading file")
}

fn write_to_file(filename: String, content: String) -> std::io::Result<()> {
    writeln!(std::fs::OpenOptions::new().write(true).append(true).open(filename)?,"{content}")
}

async fn eval_python_script(python_snippet: String, python_code: &str, char_to_replace: char, desired_string: &str) -> std::io::Result<()> {
    let python_code: String = python_code.replace(char_to_replace, &python_snippet);
    drop(python_snippet);

    let output: std::process::Output = tokio::process::Command::new("python").args(["-c", &python_code]).output().await.expect("Failed to execute Python program.");

    if output.stdout.is_empty() { /* Skip converting empty string to utf8 to save processing power */
        return Ok(());
    }

    if std::str::from_utf8(&output.stdout).expect("Failed to convert Python output to utf8").trim()== desired_string.trim() {
        write_to_file(String::from("output.txt"), python_code).expect("Write to file failed.");
    }
    Ok(())
}

fn update_console(length: usize, iteration: usize, total_number_of_iterations: usize, total_n_of_eval_timeout: u128, progressbar_size: usize) -> std::io::Result<()> {
    let progress: f32 = 100_f32 * iteration as f32 / total_number_of_iterations as f32;

    std::io::stdout().write_all(b"\x1B[2J").unwrap(); /* Send control character to clear screen */

    writeln!(std::io::stdout(), "Searching in length {length}", )?;
    writeln!(std::io::stdout(), "Currently in iteration {iteration} of {total_number_of_iterations} in total")?;

    write!(std::io::stdout(), "[")?;
    for i in 0..progressbar_size {
        if i <= (progress * progressbar_size as f32 / 100_f32) as usize {
            write!(std::io::stdout(), "=")?;
        } else {
            write!(std::io::stdout(), " ")?;
        }
    }

    writeln!(std::io::stdout(), "] {:.2}%", progress)?;
    writeln!(std::io::stdout(), "{} skipped due to timeout (~{:.2}%)", total_n_of_eval_timeout, 100_f32 * total_n_of_eval_timeout as f32 / iteration as f32)?;
    std::io::stdout().flush()?;
    Ok(())
}

async fn iterate_combinations(length: usize, tokens: Vec<&str>, python_code: &str, char_to_replace: char, desired_string: &str, print_every_x_iteration: usize, duration_till_timeout: std::time::Duration) {
    const PROGRESSBAR_SIZE: usize = 100;
    let mut indices: Vec<usize> = vec![0; length];
    let total_number_of_iterations: usize = tokens.len().pow(length as u32);
    let mut total_n_of_eval_timeout: u128 = 0;

    for iteration in 0..=total_number_of_iterations {
        if iteration % print_every_x_iteration == 0 {
            update_console(length, iteration, total_number_of_iterations, total_n_of_eval_timeout, PROGRESSBAR_SIZE).expect("Failed to print to screen.");
        }
        let python_snippet: String = indices.iter().map(|&index|tokens[index]).collect();

        if tokio::time::timeout(duration_till_timeout, eval_python_script(python_snippet, python_code, char_to_replace, desired_string)).await.is_err(){
            total_n_of_eval_timeout += 1;
        }

        let mut i: usize = length - 1;
        loop {
            indices[i] += 1;
            if indices[i] >= tokens.len() {
                indices[i] = 0;
                if i == 0 {
                    return;
                }
                i -= 1;
            } else {
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let contents: String = std::fs::read_to_string("tokens.txt").expect("Failed reading file");
    const PRINT_EVERY_X_ITERATION: usize = 100;
    const PYTHON_CODE: &str = "for i in range(1,101):print('FizzBuzz'[i%-3&4:�]or i)";
    const CHAR_TO_REPLACE: char = '�';
    const DURATION_TILL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);

    for length in 7..=9 {
        iterate_combinations(length, load_tokes_from_file(&contents), PYTHON_CODE, CHAR_TO_REPLACE, &load_desired_string_from_file(), PRINT_EVERY_X_ITERATION, DURATION_TILL_TIMEOUT).await;
    }
}
