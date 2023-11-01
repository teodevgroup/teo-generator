use pad::{PadStr, Alignment};
use colored::Colorize;
use inflector::Inflector;

pub(crate) fn green_message(label: &str, content: String) {
    let label = label.to_sentence_case().pad(12, ' ', Alignment::Right, false) + " ";
    let label = label.green().bold();
    println!("{}{}", label, content);
}

pub(crate) fn yellow_message(label: &str, content: String) {
    let label = label.to_sentence_case().pad(12, ' ', Alignment::Right, false) + " ";
    let label = label.yellow().bold();
    println!("{}{}", label, content);
}

pub(crate) fn red_message(label: &str, content: String) {
    let label = label.to_sentence_case().pad(12, ' ', Alignment::Right, false) + " ";
    let label = label.red().bold();
    println!("{}{}", label, content);
}
