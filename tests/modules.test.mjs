import assert from 'node:assert'
import { describe, test } from 'node:test'
import { KeyCode, normalizeKeyName, startListener, stopListener } from 'rdev-node'

describe('rdev-node ESM entry', () => {
  test('ESM import works through the package name', () => {
    assert.strictEqual(typeof startListener, 'function')
    assert.strictEqual(typeof stopListener, 'function')
    assert.strictEqual(normalizeKeyName(KeyCode.ControlLeft), 'ctrl')
  })

  test('ESM named imports match the CommonJS binding', async () => {
    const { createRequire } = await import('node:module')
    const require = createRequire(import.meta.url)
    const cjs = require('../index.js')
    const esm = await import('../index.mjs')
    for (const name of [
      'ButtonType',
      'EventTypeValue',
      'getDisplaySize',
      'initSimulation',
      'isModifierKey',
      'KeyCode',
      'NormalizedModifier',
      'normalizeKeyName',
      'simulateEvent',
      'startListener',
      'stopListener',
      'stringKeyToKeycode',
    ]) {
      assert.strictEqual(esm[name], cjs[name], `ESM/CJS mismatch: ${name}`)
    }
  })
})
