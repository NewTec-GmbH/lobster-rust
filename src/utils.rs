/// Variuos utility functions not assosiated with any subfunction.

use regex::Regex;

pub(crate) fn trim_filename(filepath: &str) -> Option<String> {
    let file_re = Regex::new(r"(?:.*[/\\])?(?<file>[[:alnum:]]+)\.rs").unwrap();

    println!("Parsing from {} ...", &filepath);

    if let Some(cap) = file_re.captures(filepath) {
        let temp = cap.name("file").map(|g| g.as_str().to_string());
        println!("... to this {:?}", &temp);
        temp
    } else {
        None
    }
}