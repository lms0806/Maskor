# Maskor

문장과 단어 목록을 받아, 문장 안의 그 단어만 블러 처리하는 라이브러리입니다. 검색 엔진은 Rust로 작성했고, Node.js에서는 TypeScript 타입과 함께 사용할 수 있습니다.

한국어는 단어 뒤에 조사가 바로 붙습니다. Maskor는 공백으로 단어를 자르지 않고 문장 전체를 검색해서 `홍길동은` 안의 `홍길동`처럼 붙어 있는 단어도 찾습니다.

## 준비

- Node.js 14 이상
- Rust 1.88 이상 (rustup 권장)
- Windows에서는 MSVC 빌드 도구

```bash
npm install
npm run build
```

## 사용법

```ts
import { Maskor, blur } from 'maskor'

const once = blur('홍길동은 서울에 산다', ['홍길동'])
once.text // '***은 서울에 산다'
once.spans // [{ word: '홍길동', start: 0, end: 3 }]

const maskor = new Maskor(['홍길동', '주민등록번호'])
maskor.blur('홍길동의 주민등록번호는 비공개입니다').text
// '***의 ********는 비공개입니다'

maskor.blur('서울특별시', { mask: '█' }).text
// 단어 목록에 없으면 문장을 그대로 반환합니다.
```

같은 단어 목록으로 문장을 여러 번 처리할 때는 `new Maskor(words)`로 검색기를 만들어 재사용하세요. 단어 목록은 이때 한 번만 컴파일됩니다.

```ts
const maskor = new Maskor(['비밀', '대외비'])

maskor.blur('이 문서는 비밀이다').text // '이 문서는 **이다'
maskor.blurAll(['비밀 회의', '공개 회의']).map((result) => result.text)
// ['** 회의', '공개 회의']
```

## 검색 규칙

- 단어는 정규식이 아니라 문자 그대로 찾습니다. `.`이나 `*`도 특수 문자가 아닙니다.
- 같은 위치에서 여러 단어가 맞으면 더 긴 단어를 선택합니다. `서울`과 `서울특별시`가 함께 있으면 `서울특별시`를 한 덩어리로 가립니다.
- 겹치지 않게 왼쪽부터 진행합니다.
- 입력 문장과 단어는 NFC로 맞춘 뒤 비교합니다. 자모가 풀려 들어온 `가`도 완성된 `가`와 같게 봅니다.
- 빈 문자열은 무시하고, 같은 단어는 한 번만 등록합니다.
- `mask`는 `*`, `＊`, `█`처럼 UTF-16 기준 한 단위인 문자 1개입니다. 생략하면 `*`입니다. 일치 구간은 글자 수만큼 이 문자로 바뀌므로, 정규화된 원문과 결과의 길이가 같습니다.

`spans`의 `start`와 `end`는 JavaScript 문자열과 같은 UTF-16 위치입니다. `end` 위치는 포함하지 않습니다. 입력에 서로게이트 페어가 없는 한글 문장이라면 일반적인 글자 위치와 같습니다.

## API

```ts
function blur(text: string, words: string[], options?: BlurOptions): BlurResult

class Maskor {
  constructor(words: string[])
  readonly wordCount: number
  blur(text: string, options?: BlurOptions): BlurResult
  blurAll(texts: string[], options?: BlurOptions): BlurResult[]
}

interface BlurOptions {
  mask?: string
}

interface BlurResult {
  text: string
  spans: BlurSpan[]
}

interface BlurSpan {
  word: string
  start: number
  end: number
}
```

## 개발

```bash
npm test
```

Rust 단위 테스트와 Node.js 테스트를 순서대로 실행합니다. 네이티브 모듈을 다시 만들 때는 `npm run build`를 실행하세요.
