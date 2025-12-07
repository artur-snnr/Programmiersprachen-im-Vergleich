use regex::Regex;
use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use std::path::Path;
use std::fs;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

// Funktion zum Hervorheben der Treffer in der Ausgabe
fn highlight_regex(line: &str, re: &Regex, use_color: bool) -> String {
    if !use_color {
        return line.to_string();
    }
    re.replace_all(line, |caps: &regex::Captures| {
        format!("\x1b[31m{}\x1b[0m", &caps[0])
    })
    .to_string()
}

// Funktion zum Durchsuchen einer Datei nach dem Muster
fn search_in_file(filename: &Path, 
                    pattern: &Regex, 
                    with_heading: bool, 
                    use_color: bool, 
                    before_context: usize, 
                    after_context: usize) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    
    let mut lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => lines.push(l),
            Err(_) => return, 
        }
    }
    if lines.is_empty() {
        return;
    }
    // Alle Zeilen mit Treffer sammeln
    let mut match_indices: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if pattern.is_match(line) {
            match_indices.push(i); // Treffer gefunden
        }
    }

    if match_indices.is_empty() {
        return;
    }

    let match_set: HashSet<usize> = match_indices.iter().copied().collect();

    let mut ranges: Vec<(usize, usize)> = Vec::new();

    for &idx in &match_indices {
        let start = idx.saturating_sub(before_context);
        let end = (idx + after_context).min(lines.len() - 1);

        if let Some((last_start, last_end)) = ranges.last_mut() {
            if start <= *last_end + 1 {
                if end > *last_end {
                    *last_end = end;
                }
            } else {
                ranges.push((start, end));
            }
        } else {
            ranges.push((start, end));
        }
    }
    // no-heading Implementierung
    if with_heading {
    println!("{}",filename.display());
    }
    
    let file_display = filename.display();

    // Für jeden Treffer einen Kontextbereich [start, end] bauen
    for (range_idx, (start, end)) in ranges.iter().enumerate() {
        // Blöcke ausgeben, mit "--" dazwischen
        if with_heading && range_idx > 0 {
            println!("--");
        }

        for line_idx in *start..=*end {
            let line = &lines[line_idx];
            let is_match = match_set.contains(&line_idx);

            let sep = if is_match { ":" } else { "-" };

            let out_line = if is_match && use_color {
                highlight_regex(line, pattern, true)
            } else {
                line.to_string()
            };

            if with_heading {
                println!("{}{}{}", line_idx + 1, sep, out_line);
            } else {
                println!("{}{}{}{}{}", file_display, sep, line_idx + 1, sep, out_line);
            }
        }
    }
}

// Binärdatei-Erkennung
fn is_text_file(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = [0u8; 1024]; // die ersten 1 KB einlesen um zu prüfen ob es Text ist
    let n = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    if n == 0 { // leere Datei
        return true;
    }

    std::str::from_utf8(&buffer[..n]).is_ok()
}

// versteckte Dateien/Ordner
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}


// checken ob Pfad eine Datei, ein Verzeichnis ist oder nicht exisitiert
#[derive(PartialEq, Eq)]
enum PathType {
    File,
    Directory,
    NotFound,
    Other,
}
fn check_path(path: &Path) -> PathType {
    if !path.exists() { return PathType::NotFound; }
    if path.is_file() { return PathType::File; }
    if path.is_dir()  { return PathType::Directory; }
    PathType::Other
}

//  Datei/Verzeichnis Handling 
fn handle_path(path: &Path, 
                    pattern: &Regex, 
                    include_hidden: bool, 
                    with_heading: bool, 
                    use_color: bool, 
                    before_context: usize, 
                    after_context: usize) -> Result<(), Box<dyn std::error::Error>> {
    match check_path(path) {
        PathType::File => {
            if !include_hidden && is_hidden(path) {
                return Ok(());
            }

            if is_text_file(path) {
                search_in_file(path, pattern, with_heading, use_color, before_context, after_context);
            } else {
                eprintln!("Binär-Datei gefunden: {}", path.display());
            }
        }
        PathType::Directory => {
            for entry in fs::read_dir(path)? {
                let entry = entry?; 
                let child_path = entry.path();
                
                // versteckte Dateien/Ordner ggf. überspringen
                if !include_hidden && is_hidden(&child_path) {
                    continue;
                }

                handle_path(&child_path, pattern, include_hidden, with_heading, use_color, before_context, after_context)?;
            }
        }
        _ => {}
    }
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let help = "
        usage: searcher [OPTIONS] PATTERN [PATH ...]

        -A, --after-context <n>     n Folgezeilen je Treffer
        -B, --before-context <n>    n Vorzeilen je Treffer
        -C, --context <n>           n Vor- und Folgezeilen (Kurzform für -A/-B)
        -c, --color                 farbige Hervorhebung der Match-Teile
        -h, --hidden                auch versteckte Dateien/Ordner durchsuchen
        --help                      Hilfe anzeigen
        -i, --ignore-case           Groß-/Kleinschreibung ignorieren
        --no-heading                keine Dateiblocks, sondern eine Zeile pro Treffer ";

    let mut args = env::args().skip(1);

    let mut before_context: usize = 0;
    let mut after_context: usize = 0;
    let mut use_color = false;
    let mut include_hidden = false;
    let mut ignorecase = false;
    let mut with_heading = true;

    let mut pattern: Option<String> = None;
    let mut paths: Vec<PathBuf> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                println!("{}", help);
                return Ok(());
            }
            "-c" | "--color" => use_color = true,
            "-i" | "--ignore-case" => ignorecase = true,
            "-h" | "--hidden" => include_hidden = true,
            "--no-heading" => with_heading = false,
            "-A" | "--after-context" => {
                let n = args.next().expect("missing value for -A/--after-context");
                after_context = n.parse().expect("invalid number for -A/--after-context");
            }
            "-B" | "--before-context" => {
                let n = args.next().expect("missing value for -B/--before-context");
                before_context = n.parse().expect("invalid number for -B/--before-context");
            }
            "-C" | "--context" => {
                let n = args.next().expect("missing value for -C/--context");
                let n: usize = n.parse().expect("invalid number for -C/--context");
                before_context = n;
                after_context = n;
            }
            _ => {
                if arg.starts_with('-') && pattern.is_none() {
                    eprintln!("Unbekannte Option: {}", arg);
                    eprintln!("{}", help);
                    std::process::exit(1);
                }
                // erstes Nicht-Option-Argument = Pattern
                if pattern.is_none() {
                    pattern = Some(arg);
                } else {
                    // restliche = Pfade
                    paths.push(PathBuf::from(arg));
                }
            }
        }
    }

    let pattern = match pattern {
        Some(p) => p,
        None => {
            eprintln!("Fehler: kein PATTERN angegeben.");
            eprintln!("{}", help);
            std::process::exit(1);
        }
    };

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    let mut pat = pattern;
    if ignorecase {
        pat = format!("(?i){}", pat);
    }
    let regex = Regex::new(&pat)?;

    for path in paths {
        handle_path(
            &path,
            &regex,
            include_hidden,
            with_heading,
            use_color,
            before_context,
            after_context,
        )?;
    }
    Ok(())
}