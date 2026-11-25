use std::collections::HashMap;

pub const SCORE_CONTINUE_MATCH: f64 = 1.0;
pub const SCORE_SPACE_WORD_JUMP: f64 = 0.9;
pub const SCORE_NON_SPACE_WORD_JUMP: f64 = 0.8;
pub const SCORE_CHARACTER_JUMP: f64 = 0.17;
pub const SCORE_TRANSPOSITION: f64 = 0.1;

pub const PENALTY_SKIPPED: f64 = 0.999;
pub const PENALTY_CASE_MISMATCH: f64 = 0.9999;
pub const PENALTY_NOT_COMPLETE: f64 = 0.99;

#[inline]
fn is_gap_char(c: char) -> bool {
    matches!(
        c,
        '\\' | '/' | '_' | '+' | '.' | '#' | '"' | '@' | '[' | '(' | '{' | '&'
    )
}

#[inline]
fn is_space_char(c: char) -> bool {
    c.is_whitespace() || c == '-'
}

fn format_chars(chars: &[char]) -> Vec<char> {
    chars
        .iter()
        .map(|&c| {
            let lower = c.to_lowercase().next().unwrap_or(c);
            if is_space_char(lower) { ' ' } else { lower }
        })
        .collect()
}

fn find_char_from(haystack: &[char], needle: char, start: usize) -> Option<usize> {
    if start >= haystack.len() {
        return None;
    }
    haystack[start..]
        .iter()
        .position(|&c| c == needle)
        .map(|offset| start + offset)
}

fn compute_command_score_inner(
    string: &[char],
    abbreviation: &[char],
    lower_string: &[char],
    lower_abbreviation: &[char],
    string_index: usize,
    abbreviation_index: usize,
    memoized_results: &mut HashMap<(usize, usize), f64>,
) -> f64 {
    if abbreviation_index == abbreviation.len() {
        if string_index == string.len() {
            return SCORE_CONTINUE_MATCH;
        }
        return PENALTY_NOT_COMPLETE;
    }

    let memoize_key = (string_index, abbreviation_index);
    if let Some(&cached) = memoized_results.get(&memoize_key) {
        return cached;
    }

    let abbreviation_char = lower_abbreviation[abbreviation_index];
    let mut index = find_char_from(lower_string, abbreviation_char, string_index);
    let mut high_score: f64 = 0.0;

    while let Some(idx) = index {
        let mut score = compute_command_score_inner(
            string,
            abbreviation,
            lower_string,
            lower_abbreviation,
            idx + 1,
            abbreviation_index + 1,
            memoized_results,
        );

        if score > high_score {
            if idx == string_index {
                score *= SCORE_CONTINUE_MATCH;
            } else if idx > 0 && is_gap_char(string[idx - 1]) {
                score *= SCORE_NON_SPACE_WORD_JUMP;
                if string_index > 0 && idx - 1 > string_index {
                    let word_breaks = string[string_index..idx - 1]
                        .iter()
                        .filter(|&&c| is_gap_char(c))
                        .count();
                    if word_breaks > 0 {
                        score *= PENALTY_SKIPPED.powi(word_breaks as i32);
                    }
                }
            } else if idx > 0 && is_space_char(string[idx - 1]) {
                score *= SCORE_SPACE_WORD_JUMP;
                if string_index > 0 && idx - 1 > string_index {
                    let space_breaks = string[string_index..idx - 1]
                        .iter()
                        .filter(|&&c| is_space_char(c))
                        .count();
                    if space_breaks > 0 {
                        score *= PENALTY_SKIPPED.powi(space_breaks as i32);
                    }
                }
            } else {
                score *= SCORE_CHARACTER_JUMP;
                if string_index > 0 {
                    score *= PENALTY_SKIPPED.powi((idx - string_index) as i32);
                }
            }

            if string[idx] != abbreviation[abbreviation_index] {
                score *= PENALTY_CASE_MISMATCH;
            }
        }

        if abbreviation_index + 1 < lower_abbreviation.len() {
            let next_abbr = lower_abbreviation[abbreviation_index + 1];
            let prev_str = idx.checked_sub(1).and_then(|p| lower_string.get(p).copied());
            let should_transpose = (score < SCORE_TRANSPOSITION && prev_str == Some(next_abbr))
                || (next_abbr == abbreviation_char && prev_str != Some(abbreviation_char));

            if should_transpose {
                let transposed_score = compute_command_score_inner(
                    string,
                    abbreviation,
                    lower_string,
                    lower_abbreviation,
                    idx + 1,
                    abbreviation_index + 2,
                    memoized_results,
                );
                if transposed_score * SCORE_TRANSPOSITION > score {
                    score = transposed_score * SCORE_TRANSPOSITION;
                }
            }
        }

        if score > high_score {
            high_score = score;
        }

        index = find_char_from(lower_string, abbreviation_char, idx + 1);
    }

    memoized_results.insert(memoize_key, high_score);
    high_score
}

/// Computes a fuzzy subsequence match score in `[0.0, 1.0]` matching `bits-ui`'s
/// `computeCommandScore` algorithm (`1.0` = exact match, `0.0` = no match).
pub fn compute_command_score(value: &str, search: &str, keywords: &[String]) -> f64 {
    let combined = if keywords.is_empty() {
        value.to_string()
    } else {
        let mut s = String::with_capacity(
            value.len() + 1 + keywords.iter().map(|k| k.len() + 1).sum::<usize>(),
        );
        s.push_str(value);
        for kw in keywords {
            s.push(' ');
            s.push_str(kw);
        }
        s
    };

    let string_chars: Vec<char> = combined.chars().collect();
    let abbreviation_chars: Vec<char> = search.chars().collect();
    let lower_string = format_chars(&string_chars);
    let lower_abbreviation = format_chars(&abbreviation_chars);
    let mut memoized = HashMap::new();

    compute_command_score_inner(
        &string_chars,
        &abbreviation_chars,
        &lower_string,
        &lower_abbreviation,
        0,
        0,
        &mut memoized,
    )
}
