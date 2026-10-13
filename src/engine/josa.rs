//! 어절 뒤에 남는 부분이 조사와 문장부호뿐인지 확인한다.
//!
//! 조사는 앞에서부터 가장 긴 것을 반복해서 걷어 낸다.
//! `에게서는`은 `에게서`와 `는`으로 나뉜다.

/// 격조사, 접속조사, 보조사, 서술격 조사.
/// 적, 화, 들, 님 같은 접미사는 조사가 아니라서 넣지 않는다.
const JOSA: &[&str] = &[
    "이야말로",
    "으로부터",
    "으로서",
    "으로써",
    "이시여",
    "에게서",
    "한테서",
    "이라고",
    "이라는",
    "이든지",
    "이든가",
    "이라도",
    "이나마",
    "로부터",
    "야말로",
    "인들",
    "께서",
    "에서",
    "에게",
    "한테",
    "으로",
    "로서",
    "로써",
    "까지",
    "부터",
    "조차",
    "마저",
    "밖에",
    "대로",
    "만큼",
    "처럼",
    "같이",
    "보다",
    "마다",
    "커녕",
    "마는",
    "이나",
    "나마",
    "라도",
    "든지",
    "든가",
    "이랑",
    "하고",
    "이며",
    "라고",
    "라는",
    "이란",
    "이여",
    "이다",
    "더러",
    "보고",
    "마냥",
    "은",
    "는",
    "이",
    "가",
    "을",
    "를",
    "의",
    "에",
    "께",
    "로",
    "와",
    "과",
    "도",
    "만",
    "아",
    "야",
    "여",
    "나",
    "랑",
    "며",
    "요",
    "란",
];

/// `rest`가 비어 있거나 조사와 문장부호로만 이루어져 있으면 참이다.
pub fn remainder_is_josa_or_punct(mut rest: &str) -> bool {
    while !rest.is_empty() {
        if let Some(next) = strip_punct(rest) {
            rest = next;
            continue;
        }
        if let Some(next) = strip_longest_josa(rest) {
            rest = next;
            continue;
        }
        return false;
    }
    true
}

fn strip_longest_josa(rest: &str) -> Option<&str> {
    let mut best_len = 0;
    for josa in JOSA {
        if josa.len() > best_len && rest.starts_with(josa) {
            best_len = josa.len();
        }
    }
    if best_len == 0 {
        None
    } else {
        Some(&rest[best_len..])
    }
}

fn strip_punct(rest: &str) -> Option<&str> {
    let ch = rest.chars().next()?;
    if is_punct(ch) {
        Some(&rest[ch.len_utf8()..])
    } else {
        None
    }
}

fn is_punct(ch: char) -> bool {
    if ch.is_ascii_punctuation() {
        return true;
    }
    matches!(
        ch,
        '…' | '·' | '・' | '‧' | 'ㆍ'
            | '～' | '〜' | '―' | '–' | '—'
            | '、' | '。' | '，' | '．' | '！' | '？' | '；' | '：'
            | '「' | '」' | '『' | '』' | '【' | '】'
            | '〈' | '〉' | '《' | '》'
            | '（' | '）' | '［' | '］' | '｛' | '｝'
            | '“' | '”' | '‘' | '’' | '′' | '″'
            | '＇' | '＂'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_particles_and_punctuation() {
        assert!(remainder_is_josa_or_punct(""));
        assert!(remainder_is_josa_or_punct("는"));
        assert!(remainder_is_josa_or_punct("에게서는"));
        assert!(remainder_is_josa_or_punct("으로부터는"));
        assert!(remainder_is_josa_or_punct("은."));
        assert!(remainder_is_josa_or_punct(","));
        assert!(remainder_is_josa_or_punct("이다"));
        assert!(remainder_is_josa_or_punct("와"));
    }

    #[test]
    fn rejects_non_particle_tails() {
        assert!(!remainder_is_josa_or_punct("나무"));
        assert!(!remainder_is_josa_or_punct("나무는"));
        assert!(!remainder_is_josa_or_punct("차"));
        assert!(!remainder_is_josa_or_punct("공학"));
    }
}
