//! 여러 단어를 한 번에 찾는 마스킹 엔진.
//!
//! 단어 목록으로 Aho–Corasick 자동자(automaton)를 한 번 만들고,
//! 문장은 공백 기준 어절로 나누어 검색한다.
//! 어절이 단어로 시작하고 뒤에 조사나 문장부호만 남을 때만 가린다.
//! 같은 어절에서 여러 단어가 맞으면 더 긴 단어를 택한다.

mod josa;

use std::borrow::Cow;
use std::collections::HashSet;

use aho_corasick::AhoCorasick;
use unicode_normalization::{is_nfc, UnicodeNormalization};

use josa::remainder_is_josa_or_punct;

/// 블러 처리된 문장과, 정규화된 원문 기준의 일치 구간.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlurOutput {
    /// 일치 구간을 마스킹 문자로 바꾼 문장.
    pub text: String,
    /// 정규화된 원문에서의 일치 구간. 끝 위치는 포함하지 않는다.
    pub spans: Vec<BlurSpan>,
}

/// 한 단어가 맞은 구간.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlurSpan {
    /// 실제로 맞은 문자열.
    pub word: String,
    /// UTF-16 코드 단위 기준 시작 위치.
    pub start: u32,
    /// UTF-16 코드 단위 기준 끝 위치. 이 위치는 포함하지 않는다.
    pub end: u32,
}

/// 재사용 가능한 단어 검색기.
#[derive(Debug)]
pub struct Engine {
    ac: AhoCorasick,
    word_count: u32,
}

impl Engine {
    /// 단어 목록으로 검색기를 만든다.
    ///
    /// 빈 문자열은 건너뛰고, NFC로 맞춘 뒤 중복을 제거한다.
    /// 처음 나온 순서를 유지한다.
    pub fn new(words: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Self, String> {
        let mut patterns = Vec::new();
        let mut seen = HashSet::new();

        for word in words {
            let word: String = word.as_ref().nfc().collect();
            if word.is_empty() || !seen.insert(word.clone()) {
                continue;
            }
            patterns.push(word);
        }

        let ac = AhoCorasick::builder()
            .build(&patterns)
            .map_err(|err| format!("단어 목록으로 검색기를 만들지 못했습니다: {err}"))?;

        let word_count = u32::try_from(patterns.len()).unwrap_or(u32::MAX);
        Ok(Self { ac, word_count })
    }

    /// 등록된 단어 수.
    pub fn word_count(&self) -> u32 {
        self.word_count
    }

    /// `text`에서 단어를 찾아 `mask`로 가린다.
    ///
    /// 입력은 NFC로 정규화한 뒤, 공백으로 나눈 어절마다 검사한다.
    /// 마스킹 문자는 일치 구간의 UTF-16 코드 단위마다 하나씩 들어가서,
    /// 정규화된 원문과 결과의 길이가 같다. `mask`는 BMP 문자여야 한다.
    pub fn blur(&self, text: &str, mask: char) -> BlurOutput {
        debug_assert!(
            mask.len_utf16() == 1,
            "마스킹 문자는 UTF-16 코드 단위 1개여야 합니다"
        );

        let normalized = normalize_nfc(text);
        if self.word_count == 0 {
            return BlurOutput {
                text: normalized.into_owned(),
                spans: Vec::new(),
            };
        }

        let mut text_out = String::with_capacity(normalized.len());
        let mut spans = Vec::new();
        let mut utf16_at = 0u32;
        let mut rest = normalized.as_ref();

        while !rest.is_empty() {
            let (is_eojeol, len) = next_segment(rest);
            let segment = &rest[..len];
            rest = &rest[len..];

            if !is_eojeol {
                text_out.push_str(segment);
                utf16_at += utf16_len(segment);
                continue;
            }

            let Some(end) = self.mask_end(segment) else {
                text_out.push_str(segment);
                utf16_at += utf16_len(segment);
                continue;
            };

            let matched = &segment[..end];
            let suffix = &segment[end..];
            let units = utf16_len(matched);
            let start = utf16_at;
            let span_end = utf16_at + units;
            spans.push(BlurSpan {
                word: matched.to_owned(),
                start,
                end: span_end,
            });
            for _ in 0..units {
                text_out.push(mask);
            }
            utf16_at = span_end;
            text_out.push_str(suffix);
            utf16_at += utf16_len(suffix);
        }

        BlurOutput {
            text: text_out,
            spans,
        }
    }

    /// 어절 앞에서 시작하는 단어 중, 나머지가 조사·문장부호인 가장 긴 단어의 끝 위치.
    fn mask_end(&self, eojeol: &str) -> Option<usize> {
        let mut best: Option<usize> = None;
        for mat in self.ac.find_overlapping_iter(eojeol) {
            if mat.start() != 0 {
                continue;
            }
            if remainder_is_josa_or_punct(&eojeol[mat.end()..]) {
                best = Some(best.map_or(mat.end(), |end| end.max(mat.end())));
            }
        }
        best
    }
}

/// 다음 조각이 어절이면 참과 바이트 길이를, 공백이면 거짓과 바이트 길이를 반환한다.
fn next_segment(text: &str) -> (bool, usize) {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return (false, 0);
    };
    let is_eojeol = !first.is_whitespace();
    let mut end = first.len_utf8();
    for ch in chars {
        if ch.is_whitespace() == is_eojeol {
            break;
        }
        end += ch.len_utf8();
    }
    (is_eojeol, end)
}

fn normalize_nfc(text: &str) -> Cow<'_, str> {
    if is_nfc(text) {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(text.nfc().collect())
    }
}

fn utf16_len(text: &str) -> u32 {
    text.encode_utf16().count() as u32
}

/// JavaScript에 넘길 마스킹 문자를 확인한다.
pub fn parse_mask(mask: Option<&str>) -> Result<char, String> {
    let Some(mask) = mask else {
        return Ok('*');
    };

    let mut chars = mask.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) if ch.len_utf16() == 1 => Ok(ch),
        _ => Err("mask는 '*' 같은 BMP 문자 1개여야 합니다".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blur(text: &str, words: &[&str]) -> BlurOutput {
        Engine::new(words.iter().copied()).unwrap().blur(text, '*')
    }

    #[test]
    fn masks_word_with_particle() {
        let result = blur("홍길동은 서울에 산다", &["홍길동"]);
        assert_eq!(result.text, "***은 서울에 산다");
        assert_eq!(result.spans.len(), 1);
        assert_eq!(result.spans[0].word, "홍길동");
        assert_eq!((result.spans[0].start, result.spans[0].end), (0, 3));
    }

    #[test]
    fn masks_every_occurrence() {
        let result = blur("김철수는 김철수와 만났다", &["김철수"]);
        assert_eq!(result.text, "***는 ***와 만났다");
        assert_eq!(
            result
                .spans
                .iter()
                .map(|span| (span.start, span.end))
                .collect::<Vec<_>>(),
            vec![(0, 3), (5, 8)]
        );
    }

    #[test]
    fn prefers_longer_word() {
        let result = blur("서울특별시 서울", &["서울", "서울특별시"]);
        assert_eq!(result.text, "***** **");
        assert_eq!(result.spans[0].word, "서울특별시");
        assert_eq!(result.spans[1].word, "서울");
    }

    #[test]
    fn keeps_literal_characters() {
        let result = blur("기호는 . 와 * 이다", &[".", "*"]);
        assert_eq!(result.text, "기호는 * 와 * 이다");
        assert_eq!(blur("1,000원이다", &["1,000"]).text, "1,000원이다");
        assert_eq!(blur("1,000은 남는다", &["1,000"]).text, "*****은 남는다");
    }

    #[test]
    fn matches_decomposed_hangul() {
        let text = "\u{1100}\u{1161}는";
        let result = blur(text, &["가"]);
        assert_eq!(result.text, "*는");
        assert_eq!(result.spans[0].word, "가");
        assert_eq!((result.spans[0].start, result.spans[0].end), (0, 1));
    }

    #[test]
    fn preserves_utf16_length_around_emoji() {
        let source = "비밀 😊 비밀";
        let result = blur(source, &["비밀"]);
        assert_eq!(result.text, "** 😊 **");
        assert_eq!(
            result
                .spans
                .iter()
                .map(|span| (span.start, span.end))
                .collect::<Vec<_>>(),
            vec![(0, 2), (6, 8)]
        );
        assert_eq!(
            result.text.encode_utf16().count(),
            source.encode_utf16().count()
        );
    }

    #[test]
    fn ignores_empty_and_duplicate_words() {
        let engine = Engine::new(["", "금지어", "금지어", "\u{1100}\u{1161}"]).unwrap();
        assert_eq!(engine.word_count(), 2);
        assert_eq!(engine.blur("금지어만", '*').text, "***만");
        assert_eq!(engine.blur("가는", '*').text, "*는");
    }

    #[test]
    fn leaves_stem_inside_longer_word() {
        let result = blur("사과나무는 크다", &["사과"]);
        assert_eq!(result.text, "사과나무는 크다");
        assert!(result.spans.is_empty());
        assert_eq!(blur("자동차는 빠르다", &["자동"]).text, "자동차는 빠르다");
    }

    #[test]
    fn masks_particle_forms_only() {
        assert_eq!(blur("사과는 달다", &["사과"]).text, "**는 달다");
        assert_eq!(blur("사과나무는 크다", &["사과나무"]).text, "****는 크다");
        assert_eq!(blur("자동차는 빠르다", &["자동차"]).text, "***는 빠르다");
        assert_eq!(blur("사과, 배", &["사과"]).text, "**, 배");
        assert_eq!(blur("학교에서는 만났다.", &["학교"]).text, "**에서는 만났다.");
        assert_eq!(blur("홍길동은.", &["홍길동"]).text, "***은.");
        assert_eq!(
            blur("서울은 서울특별시는 크다", &["서울", "서울특별시"]).text,
            "**은 *****는 크다"
        );
    }

    #[test]
    fn returns_text_when_nothing_matches() {
        let result = blur("평범한 문장", &["비밀"]);
        assert_eq!(result.text, "평범한 문장");
        assert!(result.spans.is_empty());
    }

    #[test]
    fn rejects_multi_character_mask() {
        assert!(parse_mask(Some("**")).is_err());
        assert!(parse_mask(Some("")).is_err());
        assert!(parse_mask(Some("😊")).is_err());
        assert_eq!(parse_mask(None).unwrap(), '*');
        assert_eq!(parse_mask(Some("█")).unwrap(), '█');
    }
}
