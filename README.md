# Maskor

문장과 단어 목록을 받아, 문장 안의 그 단어만 블러 처리하는 라이브러리입니다. 검색 엔진은 Rust로 작성했고, Node.js에서는 TypeScript 타입과 함께 사용할 수 있습니다.

한국어는 단어 뒤에 조사가 바로 붙습니다. Maskor는 문장을 공백 기준 어절로 나눈 뒤, 어절이 그 단어로 시작하고 뒤에 조사나 문장부호만 남아 있을 때만 가립니다.

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

blur('사과나무는 크다', ['사과']).text // '사과나무는 크다'
blur('사과는 달다', ['사과']).text // '**는 달다'
blur('자동차는 빠르다', ['자동차']).text // '***는 빠르다'

const maskor = new Maskor(['홍길동', '주민등록번호'])
maskor.blur('홍길동의 주민등록번호는 비공개입니다').text
// '***의 ******는 비공개입니다'

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
