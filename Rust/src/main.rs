use regex::Regex;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn search_in_file(filename: &str, pattern: &Regex) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for (linenumber, line) in reader.lines().enumerate() {
        let content = line.unwrap();
        if pattern.is_match(&content) {
            println!("{}:{}:{}", filename, linenumber + 1, content);
        } 
    }
}

fn main() {
    let args = vec!["searcher", "tz", "testdateien/testtext.txt"];
    
    let pattern = &args[1];
    let regex = Regex::new(pattern).unwrap();

    for filename in &args[2..] {
        search_in_file(filename, &regex);
    }
}
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