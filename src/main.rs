use std::io::Write;
use std::process::{Command, Stdio};
use std::env;

// ANSI color codes as constants
const BOLD: &str = "\x1B[1m";
const BLUE: &str = "\x1B[34m";
const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: pd <search-term>");
        std::process::exit(1);
    }

    // Use all arguments after the program name as the search term
    let search_term = args[1..].join(" ");
    let results = search_packages(&search_term);
    print_results_with_pager(&results);
}

fn search_packages(term: &str) -> (Vec<(String, String)>, Vec<(String, String)>, Vec<(String, String)>) {
    (
        search_pacman(term),
        search_aur(term),
        search_flatpak(term),
    )
}

fn search_pacman(term: &str) -> Vec<(String, String)> {
    execute_search_command("pacman", &["-Ss", term])
}

fn search_aur(term: &str) -> Vec<(String, String)> {
    execute_search_command("yay", &["-Ss", "--aur", term])
}

fn search_flatpak(term: &str) -> Vec<(String, String)> {
    let output = Command::new("flatpak")
        .args(&["search", term])
        .output()
        .expect("Failed to execute flatpak command");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() >= 2 && parts[0].to_lowercase().contains(&term.to_lowercase()) {
                let description = parts.get(2).map_or("No description.", |&s| {
                    if s.trim().is_empty() { "No description." } else { s.trim() }
                });
                Some((parts[0].to_string(), description.to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn execute_search_command(command: &str, args: &[&str]) -> Vec<(String, String)> {
    let output = Command::new(command)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute {} command", command));

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .collect::<Vec<&str>>()
        .chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                let parts: Vec<&str> = chunk[0].splitn(2, '/').collect();
                if parts.len() == 2 {
                    let package_info: Vec<&str> = parts[1].splitn(2, ' ').collect();
                    if package_info.len() == 2 {
                        let description = if chunk[1].trim().is_empty() {
                            "No description.".to_string()
                        } else {
                            chunk[1].trim().to_string()
                        };
                        Some((package_info[0].to_string(), description))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn print_results_with_pager(results: &(Vec<(String, String)>, Vec<(String, String)>, Vec<(String, String)>)) {
    let (pacman, aur, flatpak) = results;
    
    let mut output = String::new();
    
    fn format_package_count(count: usize) -> String {
        if count == 1 {
            format!("1 package")
        } else {
            format!("{} packages", count)
        }
    }
    
    // Summary of results
    output.push_str(&format!("{}Pacman:{} {} | {}AUR:{} {} | {}Flatpak:{} {}\n\n",
        BOLD, RESET, format_package_count(pacman.len()),
        BOLD, RESET, format_package_count(aur.len()),
        BOLD, RESET, format_package_count(flatpak.len())
    ));

    fn print_category_results(output: &mut String, category_name: &str, results: &[(String, String)], color: &str) {
        if !results.is_empty() {
            output.push_str(&format!("{}{} Results:{}\n", BOLD, category_name, RESET));
            output.push_str(&format!("{}\n", "=".repeat(category_name.len() + 9)));
            for (package, description) in results {
                output.push_str(&format!("{}{}{}{}\n", BOLD, color, package, RESET));
                output.push_str(&format!("  {}\n\n", description));
            }
        }
    }

    print_category_results(&mut output, "Pacman", pacman, BLUE);
    print_category_results(&mut output, "AUR", aur, RED);
    print_category_results(&mut output, "Flatpak", flatpak, GREEN);

    // Replace all '~' characters with spaces
    let display_output = output.replace('~', " ");

    let mut pager = Command::new("less")
        .args(&["-R", "+Gg", "-~"]) // Added the "-~" option to suppress ~ symbols
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start pager");

    if let Some(mut pager_stdin) = pager.stdin.take() {
        pager_stdin.write_all(display_output.as_bytes()).expect("Failed to write to pager");
    }

    pager.wait().expect("Pager process wasn't running");
}
