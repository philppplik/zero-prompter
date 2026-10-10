//! Word normalisation for German and English.
//!
//! - lower-case, diacritics folded (`ä -> ae`, `ß -> ss`, `é -> e`) so that
//!   "Straße" == "Strasse" and STT spelling variants collapse;
//! - apostrophes and hyphens joined ("geht's" -> "gehts", "E-Mail" -> "email");
//! - other punctuation splits;
//! - digits expanded to number words in the selected language
//!   ("2026" -> "zweitausendsechsundzwanzig" / "two thousand twenty six"),
//!   because STT engines emit spoken forms;
//! - filler words ("äh", "ähm", "uh", "um") are removed from *hypotheses*.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Lang {
    #[default]
    De,
    En,
}

#[rustfmt::skip]
const FILLERS: &[&str] = &[
    "aeh", "aehm", "aem", "oehm", "ehm", "hm", "hmm", "mhm", "uh", "uhm", "um", "umm", "erm", "er",
    "ah",
];

/// `true` if the (already normalised) word is a hesitation filler.
pub fn is_filler(norm: &str) -> bool {
    FILLERS.contains(&norm)
}

fn fold_char(c: char, out: &mut String) {
    match c {
        'ä' | 'æ' => out.push_str("ae"),
        'ö' | 'œ' => out.push_str("oe"),
        'ü' => out.push_str("ue"),
        'ß' => out.push_str("ss"),
        'à' | 'á' | 'â' | 'ã' | 'å' | 'ā' => out.push('a'),
        'ç' | 'č' | 'ć' => out.push('c'),
        'è' | 'é' | 'ê' | 'ë' | 'ē' => out.push('e'),
        'ì' | 'í' | 'î' | 'ï' => out.push('i'),
        'ñ' | 'ń' => out.push('n'),
        'ò' | 'ó' | 'ô' | 'õ' | 'ø' => out.push('o'),
        'ù' | 'ú' | 'û' | 'ū' => out.push('u'),
        'ý' | 'ÿ' => out.push('y'),
        'š' | 'ś' => out.push('s'),
        'ž' | 'ź' | 'ż' => out.push('z'),
        'ł' => out.push('l'),
        _ => out.push(c),
    }
}

fn is_joiner(c: char) -> bool {
    matches!(c, '\'' | '’' | '`' | '´' | '-' | '\u{2010}' | '\u{2011}' | '\u{00AD}')
}

/// Normalises one raw script/STT word into zero or more normalised words.
pub fn normalize_word(raw: &str, lang: Lang) -> Vec<String> {
    let lower = raw.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut cur_is_digit = false;

    let flush = |cur: &mut String, is_digit: bool, out: &mut Vec<String>| {
        if cur.is_empty() {
            return;
        }
        if is_digit {
            out.extend(number_words(cur, lang));
        } else {
            out.push(std::mem::take(cur));
        }
        cur.clear();
    };

    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_digit() {
            if !cur_is_digit {
                flush(&mut cur, false, &mut out);
            }
            cur_is_digit = true;
            cur.push(c);
        } else if c.is_alphabetic() {
            if cur_is_digit {
                flush(&mut cur, true, &mut out);
            }
            cur_is_digit = false;
            fold_char(c, &mut cur);
        } else if is_joiner(c) {
            // joined: nothing to emit
        } else if (c == '.' || c == ',') && cur_is_digit && is_thousands_sep(&chars, i, lang) {
            // "1.000" (DE) / "1,000" (EN): drop the separator
        } else {
            flush(&mut cur, cur_is_digit, &mut out);
            cur_is_digit = false;
            match c {
                '%' => out.push(if lang == Lang::De { "prozent" } else { "percent" }.into()),
                '&' | '+' => out.push(if lang == Lang::De { "und" } else { "and" }.into()),
                '€' => out.push("euro".into()),
                '$' => out.push("dollar".into()),
                _ => {}
            }
        }
    }
    flush(&mut cur, cur_is_digit, &mut out);
    out
}

/// Separator at `i` is followed by exactly three digits and then a non-digit.
fn is_thousands_sep(chars: &[char], i: usize, lang: Lang) -> bool {
    let expected = if lang == Lang::De { '.' } else { ',' };
    if chars[i] != expected {
        return false;
    }
    let digits = chars[i + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
    digits == 3
}

/// Normalises a whole utterance (e.g. an STT hypothesis), dropping fillers.
pub fn normalize_hypothesis(text: &str, lang: Lang) -> Vec<String> {
    text.split_whitespace()
        .flat_map(|w| normalize_word(w, lang))
        .filter(|w| !is_filler(w))
        .collect()
}

/// Spells a digit string. Numbers >= 1 000 000 are spelled digit by digit.
pub fn number_words(digits: &str, lang: Lang) -> Vec<String> {
    let n: u64 = match digits.parse() {
        Ok(n) if digits.len() <= 7 && n < 1_000_000 => n,
        _ => {
            return digits
                .chars()
                .filter_map(|c| c.to_digit(10))
                .flat_map(|d| number_words(&d.to_string(), lang))
                .collect();
        }
    };
    match lang {
        Lang::De => vec![fold(&de_number(n))],
        Lang::En => en_number(n),
    }
}

fn fold(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 4);
    s.chars().for_each(|c| fold_char(c, &mut o));
    o
}

#[rustfmt::skip]
const DE_UNITS: [&str; 20] = [
    "null", "eins", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn",
    "elf", "zwölf", "dreizehn", "vierzehn", "fünfzehn", "sechzehn", "siebzehn", "achtzehn",
    "neunzehn",
];
#[rustfmt::skip]
const DE_TENS: [&str; 10] = [
    "", "", "zwanzig", "dreißig", "vierzig", "fünfzig", "sechzig", "siebzig", "achtzig", "neunzig",
];

fn de_below_100(n: u64) -> String {
    match n {
        0 => String::new(),
        1..=19 => DE_UNITS[n as usize].to_string(),
        _ => {
            let (t, u) = ((n / 10) as usize, (n % 10) as usize);
            if u == 0 {
                DE_TENS[t].to_string()
            } else {
                let unit = if u == 1 { "ein" } else { DE_UNITS[u] };
                format!("{unit}und{}", DE_TENS[t])
            }
        }
    }
}

fn de_below_1000(n: u64) -> String {
    let (h, r) = (n / 100, n % 100);
    let mut s = String::new();
    if h > 0 {
        if h > 1 {
            s.push_str(DE_UNITS[h as usize]);
        }
        s.push_str("hundert");
    }
    s.push_str(&de_below_100(r));
    s
}

fn de_number(n: u64) -> String {
    if n == 0 {
        return "null".into();
    }
    // Years 1100..=1999 are read "neunzehnhundert…".
    if (1100..2000).contains(&n) {
        return format!("{}hundert{}", de_below_100(n / 100), de_below_100(n % 100));
    }
    let (th, r) = (n / 1000, n % 1000);
    let mut s = String::new();
    if th > 0 {
        if th > 1 {
            let mut p = de_below_1000(th);
            if p.ends_with("eins") {
                p.truncate(p.len() - 1); // "einundzwanzig" ok, "…eins" -> "…ein"
            }
            s.push_str(&p);
        }
        s.push_str("tausend");
    }
    s.push_str(&de_below_1000(r));
    s
}

#[rustfmt::skip]
const EN_UNITS: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen",
];
#[rustfmt::skip]
const EN_TENS: [&str; 10] = [
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

fn en_below_100(n: u64, out: &mut Vec<String>) {
    match n {
        0 => {}
        1..=19 => out.push(EN_UNITS[n as usize].into()),
        _ => {
            out.push(EN_TENS[(n / 10) as usize].into());
            if n % 10 > 0 {
                out.push(EN_UNITS[(n % 10) as usize].into());
            }
        }
    }
}

fn en_below_1000(n: u64, out: &mut Vec<String>) {
    if n >= 100 {
        out.push(EN_UNITS[(n / 100) as usize].into());
        out.push("hundred".into());
    }
    en_below_100(n % 100, out);
}

fn en_number(n: u64) -> Vec<String> {
    let mut out = Vec::new();
    if n == 0 {
        out.push("zero".into());
        return out;
    }
    if (1100..2000).contains(&n) {
        en_below_100(n / 100, &mut out);
        match n % 100 {
            0 => out.push("hundred".into()),
            r @ 1..=9 => {
                out.push("oh".into());
                en_below_100(r, &mut out);
            }
            r => en_below_100(r, &mut out),
        }
        return out;
    }
    if n >= 1000 {
        en_below_1000(n / 1000, &mut out);
        out.push("thousand".into());
    }
    en_below_1000(n % 1000, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de(s: &str) -> Vec<String> {
        normalize_word(s, Lang::De)
    }

    #[test]
    fn umlauts_and_punctuation() {
        assert_eq!(de("Straße,"), ["strasse"]);
        assert_eq!(de("Grüße!"), ["gruesse"]);
        assert_eq!(de("„Köln“"), ["koeln"]);
        assert_eq!(de("geht's"), ["gehts"]);
        assert_eq!(de("E-Mail"), ["email"]);
        assert_eq!(de("Café"), ["cafe"]);
        assert_eq!(de("und/oder"), ["und", "oder"]);
    }

    #[test]
    fn german_numbers() {
        assert_eq!(de("21"), ["einundzwanzig"]);
        assert_eq!(de("100"), ["hundert"]);
        assert_eq!(de("1984"), ["neunzehnhundertvierundachtzig"]);
        assert_eq!(de("2026"), ["zweitausendsechsundzwanzig"]);
        assert_eq!(de("1.000"), ["tausend"]);
        assert_eq!(de("101000"), ["hunderteintausend"]);
        assert_eq!(de("30%"), ["dreissig", "prozent"]);
        assert_eq!(de("3,5"), ["drei", "fuenf"]);
        assert_eq!(de("12"), ["zwoelf"]);
    }

    #[test]
    fn english_numbers() {
        let en = |s| normalize_word(s, Lang::En);
        assert_eq!(en("42"), ["forty", "two"]);
        assert_eq!(en("2,500"), ["two", "thousand", "five", "hundred"]);
        // 1100..=1999 is read like a year ("fifteen hundred"), also with separator.
        assert_eq!(en("1,500"), ["fifteen", "hundred"]);
        assert_eq!(en("1905"), ["nineteen", "oh", "five"]);
        assert_eq!(en("2026"), ["two", "thousand", "twenty", "six"]);
        assert_eq!(en("don't"), ["dont"]);
    }

    #[test]
    fn mixed_alnum_and_big_numbers() {
        assert_eq!(de("3D"), ["drei", "d"]);
        assert_eq!(de("12345678").len(), 8);
    }

    #[test]
    fn hypothesis_drops_fillers() {
        assert_eq!(normalize_hypothesis("Äh also ähm hallo", Lang::De), ["also", "hallo"]);
    }
}
