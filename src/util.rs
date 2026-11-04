pub fn format_count(value: u64) -> String {
    if value >= 1_000_000_000 {
        format!("{:.1}B", value as f64 / 1_000_000_000.0)
    } else if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

pub fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

pub fn format_date(iso: &str) -> String {
    let date = iso.get(0..10).unwrap_or(iso);
    let mut parts = date.split('-');
    let (Some(year), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next()) else {
        return date.to_owned();
    };
    let month = match month {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        _ => return date.to_owned(),
    };
    let day = day.trim_start_matches('0');
    format!("{month} {day}, {year}")
}

pub fn initials(name: &str) -> String {
    let mut letters = name
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2);
    let mut out = String::new();
    if let Some(first) = letters.next() {
        out.push(first.to_ascii_uppercase());
    }
    if let Some(second) = letters.next() {
        out.push(second.to_ascii_uppercase());
    }
    if out.is_empty() { "?".to_owned() } else { out }
}

pub fn truncate_to_width(
    text: &str,
    max_width: f32,
    mut width_of: impl FnMut(&str) -> f32,
) -> String {
    if width_of(text) <= max_width {
        return text.to_owned();
    }
    let mut end = text.len();
    while end > 0 {
        end = text[..end]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
            .unwrap_or(0);
        let candidate = format!("{}…", &text[..end]);
        if width_of(&candidate) <= max_width {
            return candidate;
        }
    }
    "…".to_owned()
}

pub fn strip_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_compact() {
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(4_842), "4.8K");
        assert_eq!(format_count(4_428_309), "4.4M");
        assert_eq!(format_count(4_528_325_764), "4.5B");
    }

    #[test]
    fn numbers_use_grouping() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(42), "42");
        assert_eq!(format_number(1_234), "1,234");
        assert_eq!(format_number(12_345_678), "12,345,678");
    }

    #[test]
    fn dates_are_human() {
        assert_eq!(format_date("2026-09-17T16:05:51.936Z"), "Sep 17, 2026");
    }

    #[test]
    fn initials_skip_symbols() {
        assert_eq!(initials("DOORS 🍬🍭"), "D");
        assert_eq!(initials("Natural Disaster Survival"), "ND");
    }

    #[test]
    fn html_is_stripped() {
        assert_eq!(strip_html("<p>Hello &amp; bye</p>"), "Hello & bye");
    }
}
