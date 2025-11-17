use regex::Regex;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::fs;

// Funktion zum Durchsuchen einer Datei nach dem Muster
fn search_in_file(filename: &Path, pattern: &Regex) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for (linenumber, line) in reader.lines().enumerate() {
        let content = line.unwrap();
        if pattern.is_match(&content) {
            println!("Filename: {} |||| Line Number:{} |||| Content:{}", filename.display(), linenumber + 1, content);
        }  
    }
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
fn handle_path(path: &Path, pattern: &Regex) -> Result<(), Box<dyn std::error::Error>> {
    match check_path(path) {
        PathType::File => {
            search_in_file(path, pattern);
        }
        PathType::Directory => {
            for entry in fs::read_dir(path)? {
                let entry = entry?; 
                let child_path = entry.path();
                handle_path(&child_path, pattern)?; 
            }
        }
        _ => {}
    }
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("testdateien");
    let regex = Regex::new("df")?;

    handle_path(path, &regex)?;

    Ok(())
}
    /* 
    let args = vec!["searcher", "dsfs", "testdateien/testtext.txt"];
    
    let pattern = &args[1];
    let regex = Regex::new(pattern).unwrap();

    for filename in &args[2..] {
        search_in_file(filename, &regex);
    }*/

/* --- IGNORE ---
usage: searcher [OPTIONS] PATTERN [PATH ...]

-A, --after-context <n>     n Folgezeilen je Treffer
-B, --before-context <n>    n Vorzeilen je Treffer
-C, --context <n>           n Vor- und Folgezeilen (Kurzform für -A/-B)
-c, --color                 farbige Hervorhebung der Match-Teile
-h, --hidden                auch versteckte Dateien/Ordner durchsuchen
--help                      Hilfe anzeigen
-i, --ignore-case           Groß-/Kleinschreibung ignorieren
--no-heading                keine Dateiblocks, sondern eine Zeile pro Treffer

*/