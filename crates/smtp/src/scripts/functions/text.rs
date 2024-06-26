/*
 * Copyright (c) 2023 Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Server.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use nlp::tokenizers::types::{TokenType, TypesTokenizer};
use sieve::{runtime::Variable, Context};

use crate::config::scripts::SieveContext;

use super::{html::html_to_tokens, ApplyString};

pub fn fn_trim<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| Variable::from(s.trim()))
}

pub fn fn_trim_end<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| Variable::from(s.trim_end()))
}

pub fn fn_trim_start<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| Variable::from(s.trim_start()))
}

pub fn fn_len<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    match &v[0] {
        Variable::String(s) => s.len(),
        Variable::Array(a) => a.len(),
        v => v.to_string().len(),
    }
    .into()
}

pub fn fn_to_lowercase<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| Variable::from(s.to_lowercase()))
}

pub fn fn_to_uppercase<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| Variable::from(s.to_uppercase()))
}

pub fn fn_is_uppercase<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| {
        s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
            .into()
    })
}

pub fn fn_is_lowercase<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| {
        s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_lowercase())
            .into()
    })
}

pub fn fn_has_digits<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].transform(|s| s.chars().any(|c| c.is_ascii_digit()).into())
}

pub fn tokenize_words(v: &Variable) -> Variable {
    v.to_string()
        .split_whitespace()
        .filter(|word| word.chars().all(|c| c.is_alphanumeric()))
        .map(|word| Variable::from(word.to_string()))
        .collect::<Vec<_>>()
        .into()
}

pub fn fn_tokenize<'x>(ctx: &'x Context<'x, SieveContext>, mut v: Vec<Variable>) -> Variable {
    let (urls, urls_without_scheme, emails) = match v[1].to_string().as_ref() {
        "html" => return html_to_tokens(v[0].to_string().as_ref()).into(),
        "words" => return tokenize_words(&v[0]),
        "uri" | "url" => (true, true, true),
        "uri_strict" | "url_strict" => (true, false, false),
        "email" => (false, false, true),
        _ => return Variable::default(),
    };

    match v.remove(0) {
        v @ (Variable::String(_) | Variable::Array(_)) => {
            TypesTokenizer::new(v.to_string().as_ref(), &ctx.context().psl)
                .tokenize_numbers(false)
                .tokenize_urls(urls)
                .tokenize_urls_without_scheme(urls_without_scheme)
                .tokenize_emails(emails)
                .filter_map(|t| match t.word {
                    TokenType::Url(text) if urls => Variable::from(text.to_string()).into(),
                    TokenType::UrlNoScheme(text) if urls_without_scheme => {
                        Variable::from(format!("https://{text}")).into()
                    }
                    TokenType::Email(text) if emails => Variable::from(text.to_string()).into(),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .into()
        }
        v => v,
    }
}

pub fn fn_count_spaces<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .as_ref()
        .chars()
        .filter(|c| c.is_whitespace())
        .count()
        .into()
}

pub fn fn_count_uppercase<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .as_ref()
        .chars()
        .filter(|c| c.is_alphabetic() && c.is_uppercase())
        .count()
        .into()
}

pub fn fn_count_lowercase<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .as_ref()
        .chars()
        .filter(|c| c.is_alphabetic() && c.is_lowercase())
        .count()
        .into()
}

pub fn fn_count_chars<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string().as_ref().chars().count().into()
}

pub fn fn_eq_ignore_case<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .eq_ignore_ascii_case(v[1].to_string().as_ref())
        .into()
}

pub fn fn_contains<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    match &v[0] {
        Variable::String(s) => s.contains(v[1].to_string().as_ref()),
        Variable::Array(arr) => arr.contains(&v[1]),
        val => val.to_string().contains(v[1].to_string().as_ref()),
    }
    .into()
}

pub fn fn_contains_ignore_case<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    let needle = v[1].to_string();
    match &v[0] {
        Variable::String(s) => s.to_lowercase().contains(&needle.to_lowercase()),
        Variable::Array(arr) => arr.iter().any(|v| match v {
            Variable::String(s) => s.eq_ignore_ascii_case(needle.as_ref()),
            _ => false,
        }),
        val => val.to_string().contains(needle.as_ref()),
    }
    .into()
}

pub fn fn_starts_with<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .starts_with(v[1].to_string().as_ref())
        .into()
}

pub fn fn_ends_with<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string().ends_with(v[1].to_string().as_ref()).into()
}

pub fn fn_lines<'x>(_: &'x Context<'x, SieveContext>, mut v: Vec<Variable>) -> Variable {
    match v.remove(0) {
        Variable::String(s) => s
            .lines()
            .map(|s| Variable::from(s.to_string()))
            .collect::<Vec<_>>()
            .into(),
        val => val,
    }
}

pub fn fn_substring<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .chars()
        .skip(v[1].to_usize())
        .take(v[2].to_usize())
        .collect::<String>()
        .into()
}

pub fn fn_strip_prefix<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    let prefix = v[1].to_string();
    v[0].transform(|s| {
        s.strip_prefix(prefix.as_ref())
            .map(Variable::from)
            .unwrap_or_default()
    })
}

pub fn fn_strip_suffix<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    let suffix = v[1].to_string();
    v[0].transform(|s| {
        s.strip_suffix(suffix.as_ref())
            .map(Variable::from)
            .unwrap_or_default()
    })
}

pub fn fn_split<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .split(v[1].to_string().as_ref())
        .map(|s| Variable::from(s.to_string()))
        .collect::<Vec<_>>()
        .into()
}

pub fn fn_rsplit<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .rsplit(v[1].to_string().as_ref())
        .map(|s| Variable::from(s.to_string()))
        .collect::<Vec<_>>()
        .into()
}

pub fn fn_split_once<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .split_once(v[1].to_string().as_ref())
        .map(|(a, b)| {
            Variable::Array(
                vec![Variable::from(a.to_string()), Variable::from(b.to_string())].into(),
            )
        })
        .unwrap_or_default()
}

pub fn fn_rsplit_once<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    v[0].to_string()
        .rsplit_once(v[1].to_string().as_ref())
        .map(|(a, b)| {
            Variable::Array(
                vec![Variable::from(a.to_string()), Variable::from(b.to_string())].into(),
            )
        })
        .unwrap_or_default()
}

/**
 * `levenshtein-rs` - levenshtein
 *
 * MIT licensed.
 *
 * Copyright (c) 2016 Titus Wormer <tituswormer@gmail.com>
 */
pub fn fn_levenshtein_distance<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    let a = v[0].to_string();
    let b = v[1].to_string();

    let mut result = 0;

    /* Shortcut optimizations / degenerate cases. */
    if a == b {
        return result.into();
    }

    let length_a = a.chars().count();
    let length_b = b.chars().count();

    if length_a == 0 {
        return length_b.into();
    } else if length_b == 0 {
        return length_a.into();
    }

    /* Initialize the vector.
     *
     * This is why it’s fast, normally a matrix is used,
     * here we use a single vector. */
    let mut cache: Vec<usize> = (1..).take(length_a).collect();
    let mut distance_a;
    let mut distance_b;

    /* Loop. */
    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + 1
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }

    result.into()
}

pub fn fn_detect_language<'x>(_: &'x Context<'x, SieveContext>, v: Vec<Variable>) -> Variable {
    whatlang::detect_lang(v[0].to_string().as_ref())
        .map(|l| l.code())
        .unwrap_or("unknown")
        .into()
}
