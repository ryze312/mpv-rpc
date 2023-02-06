pub fn truncate_string(current: &mut String, length: usize) {
    match current.char_indices().nth(length) {
        None => (),
        Some((index, _)) => current.truncate(index)
    }
}

pub fn truncate_string_fmt(current: &mut String, length: usize) {
    if current.chars().count() <= length {
        return;
    }

    truncate_string(current, length - 3);
    current.push_str("...");
}