import assert from 'node:assert/strict'
import { createRequire } from 'node:module'
import { test } from 'node:test'

const require = createRequire(import.meta.url)
const { Maskor, blur } = require('../index.js')

test('문장에 붙은 조사는 남기고 단어만 가린다', () => {
  const result = blur('홍길동은 서울에 산다', ['홍길동'])

  assert.equal(result.text, '***은 서울에 산다')
  assert.deepEqual(result.spans, [{ word: '홍길동', start: 0, end: 3 }])
})

test('더 긴 단어 일부인 말은 가리지 않는다', () => {
  assert.equal(blur('사과나무는 크다', ['사과']).text, '사과나무는 크다')
  assert.equal(blur('사과는 달다', ['사과']).text, '**는 달다')
  assert.equal(blur('자동차는 빠르다', ['자동']).text, '자동차는 빠르다')
  assert.equal(blur('자동차는 빠르다', ['자동차']).text, '***는 빠르다')
})

test('더 긴 단어를 우선하고 마스킹 문자를 바꿀 수 있다', () => {
  const maskor = new Maskor(['서울', '서울특별시'])

  assert.equal(maskor.wordCount, 2)
  assert.equal(maskor.blur('서울특별시 서울', { mask: '█' }).text, '█████ ██')
})

test('같은 검색기로 여러 문장을 처리한다', () => {
  const maskor = new Maskor(['비밀', ''])
  const results = maskor.blurAll(['비밀 회의', '공개 회의'])

  assert.deepEqual(
    results.map((result) => result.text),
    ['** 회의', '공개 회의'],
  )
})

test('빈 단어와 중복 단어를 걸러낸다', () => {
  const maskor = new Maskor(['', '금지어', '금지어'])

  assert.equal(maskor.wordCount, 1)
  assert.equal(maskor.blur('금지어가 있다').text, '***가 있다')
})

test('마스킹 문자가 한 글자가 아니면 거절한다', () => {
  const maskor = new Maskor(['비밀'])

  assert.throws(() => maskor.blur('비밀', { mask: '**' }), /BMP 문자 1개/)
  assert.throws(() => blur('비밀', ['비밀'], { mask: '' }), /BMP 문자 1개/)
})
