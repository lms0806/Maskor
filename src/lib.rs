//! Maskor의 Node.js 바인딩.
//!
//! 검색 로직은 [`engine`]에 있고, 이 파일은 그 엔진을 TypeScript에서
//! 쓸 수 있는 함수와 클래스로 내보낸다.

mod engine;

use engine::{parse_mask, BlurOutput, Engine};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// 마스킹에 쓸 문자. 생략하면 `*`를 쓴다.
#[napi(object)]
pub struct BlurOptions {
    /// BMP 문자 1개. 예: `*`, `＊`, `█`.
    pub mask: Option<String>,
}

/// 정규화된 원문에서 단어가 맞은 구간.
#[napi(object)]
pub struct BlurSpan {
    /// 맞은 단어.
    pub word: String,
    /// UTF-16 코드 단위 기준 시작 위치.
    pub start: u32,
    /// UTF-16 코드 단위 기준 끝 위치. 이 위치의 문자는 포함하지 않는다.
    pub end: u32,
}

/// 블러 처리 결과.
#[napi(object)]
pub struct BlurResult {
    /// 단어를 마스킹 문자로 바꾼 문장.
    pub text: String,
    /// `text`와 같은 위치 기준의 일치 구간.
    pub spans: Vec<BlurSpan>,
}

fn to_result(output: BlurOutput) -> BlurResult {
    BlurResult {
        text: output.text,
        spans: output
            .spans
            .into_iter()
            .map(|span| BlurSpan {
                word: span.word,
                start: span.start,
                end: span.end,
            })
            .collect(),
    }
}

fn mask_from_options(options: Option<BlurOptions>) -> Result<char> {
    let mask = options.as_ref().and_then(|options| options.mask.as_deref());
    parse_mask(mask).map_err(|message| Error::new(Status::InvalidArg, message))
}

fn engine_from_words(words: Vec<String>) -> Result<Engine> {
    Engine::new(words).map_err(|message| Error::new(Status::GenericFailure, message))
}

/// 문장에서 `words`를 찾아 블러 처리한다.
///
/// 공백으로 나눈 어절이 단어로 시작하고, 뒤에 조사나 문장부호만 남을 때만 가린다.
/// `사과나무` 안의 `사과`처럼 더 긴 단어의 일부는 가리지 않는다.
/// 같은 단어 목록으로 문장을 여러 번 처리할 때는 [`Maskor`]를 만들어 재사용한다.
/// 입력은 NFC로 정규화되며, 위치는 JavaScript 문자열과 같은 UTF-16 기준이다.
#[napi]
pub fn blur(text: String, words: Vec<String>, options: Option<BlurOptions>) -> Result<BlurResult> {
    let engine = engine_from_words(words)?;
    let mask = mask_from_options(options)?;
    Ok(to_result(engine.blur(&text, mask)))
}

/// 단어 목록을 미리 컴파일해 두고 여러 문장을 처리하는 검색기.
#[napi]
pub struct Maskor {
    engine: Engine,
}

#[napi]
impl Maskor {
    /// `words`로 검색기를 만든다. 빈 문자열은 무시하고 중복은 제거한다.
    #[napi(constructor)]
    pub fn new(words: Vec<String>) -> Result<Self> {
        Ok(Self {
            engine: engine_from_words(words)?,
        })
    }

    /// 등록된 단어 수.
    #[napi(getter)]
    pub fn word_count(&self) -> u32 {
        self.engine.word_count()
    }

    /// 문장 하나를 블러 처리한다.
    #[napi]
    pub fn blur(&self, text: String, options: Option<BlurOptions>) -> Result<BlurResult> {
        let mask = mask_from_options(options)?;
        Ok(to_result(self.engine.blur(&text, mask)))
    }

    /// 여러 문장을 같은 단어 목록으로 블러 처리한다.
    #[napi]
    pub fn blur_all(
        &self,
        texts: Vec<String>,
        options: Option<BlurOptions>,
    ) -> Result<Vec<BlurResult>> {
        let mask = mask_from_options(options)?;
        Ok(texts
            .iter()
            .map(|text| to_result(self.engine.blur(text, mask)))
            .collect())
    }
}
