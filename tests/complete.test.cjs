const { test, describe, before } = require('node:test')
const assert = require('node:assert')
const { spawnSync } = require('node:child_process')
const path = require('node:path')
const rdev = require('../index.js')

describe('rdev-node Complete Test Suite', () => {
  describe('Binding & API Surface', () => {
    test('all core functions are exported and are functions', () => {
      const functions = [
        'getDisplaySize',
        'initSimulation',
        'isModifierKey',
        'normalizeKeyName',
        'simulateEvent',
        'startListener',
        'stopListener',
        'stringKeyToKeycode',
      ]
      for (const fn of functions) {
        assert.strictEqual(typeof rdev[fn], 'function', `Missing function: ${fn}`)
      }
    })

    test('all enums are exported and contain expected values', () => {
      // ButtonType
      assert.strictEqual(rdev.ButtonType.Left, 'Left')
      assert.strictEqual(rdev.ButtonType.Right, 'Right')
      assert.strictEqual(rdev.ButtonType.Middle, 'Middle')
      assert.strictEqual(rdev.ButtonType.Unknown, 'Unknown')

      // EventTypeValue
      assert.strictEqual(rdev.EventTypeValue.KeyPress, 'KeyPress')
      assert.strictEqual(rdev.EventTypeValue.KeyRelease, 'KeyRelease')
      assert.strictEqual(rdev.EventTypeValue.MouseMove, 'MouseMove')
      assert.strictEqual(rdev.EventTypeValue.ButtonPress, 'ButtonPress')
      assert.strictEqual(rdev.EventTypeValue.ButtonRelease, 'ButtonRelease')
      assert.strictEqual(rdev.EventTypeValue.Wheel, 'Wheel')

      // KeyCode (sampling)
      assert.strictEqual(rdev.KeyCode.KeyA, 'KeyA')
      assert.strictEqual(rdev.KeyCode.ControlLeft, 'ControlLeft')
      assert.strictEqual(rdev.KeyCode.Backspace, 'Backspace')
      assert.strictEqual(rdev.KeyCode.Escape, 'Escape')

      // NormalizedModifier
      assert.strictEqual(rdev.NormalizedModifier.Ctrl, 'Ctrl')
      assert.strictEqual(rdev.NormalizedModifier.Shift, 'Shift')
      assert.strictEqual(rdev.NormalizedModifier.Alt, 'Alt')
      assert.strictEqual(rdev.NormalizedModifier.Meta, 'Meta')
    })
  })

  describe('Utility Functions - isModifierKey', () => {
    const modifiers = [
      rdev.KeyCode.ControlLeft,
      rdev.KeyCode.ControlRight,
      rdev.KeyCode.ShiftLeft,
      rdev.KeyCode.ShiftRight,
      rdev.KeyCode.Alt,
      rdev.KeyCode.AltGr,
      rdev.KeyCode.MetaLeft,
      rdev.KeyCode.MetaRight,
    ]

    for (const key of modifiers) {
      test(`${key} is a modifier`, () => assert.strictEqual(rdev.isModifierKey(key), true))
    }

    const nonModifiers = [
      rdev.KeyCode.KeyA,
      rdev.KeyCode.Space,
      rdev.KeyCode.F1,
      rdev.KeyCode.Escape,
      rdev.KeyCode.Return,
      rdev.KeyCode.Backspace,
    ]
    for (const key of nonModifiers) {
      test(`${key} is NOT a modifier`, () => assert.strictEqual(rdev.isModifierKey(key), false))
    }
  })

  describe('Utility Functions - normalizeKeyName', () => {
    test('standard modifier normalization', () => {
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.ControlLeft), 'ctrl')
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.ControlRight), 'ctrl')
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.ShiftLeft), 'shift')
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.Alt), 'alt')
    })

    test('alphanumeric normalization', () => {
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.KeyA), 'keya')
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.Num1), 'num1')
    })

    test('special key normalization', () => {
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.Escape), 'escape')
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.Return), 'return')
      assert.strictEqual(rdev.normalizeKeyName(rdev.KeyCode.Space), 'space')
    })
  })

  describe('Utility Functions - stringKeyToKeycode', () => {
    test('mapping common aliases', () => {
      const cases = {
        a: rdev.KeyCode.KeyA,
        keya: rdev.KeyCode.KeyA,
        1: rdev.KeyCode.Num1,
        num1: rdev.KeyCode.Num1,
        esc: rdev.KeyCode.Escape,
        escape: rdev.KeyCode.Escape,
        enter: rdev.KeyCode.Return,
        return: rdev.KeyCode.Return,
        ctrl: rdev.KeyCode.ControlLeft,
        control: rdev.KeyCode.ControlLeft,
        controll: rdev.KeyCode.ControlLeft,
        controlleft: rdev.KeyCode.ControlLeft,
        controlright: rdev.KeyCode.ControlRight,
        shift: rdev.KeyCode.ShiftLeft,
        alt: rdev.KeyCode.Alt,
        up: rdev.KeyCode.UpArrow,
        uparrow: rdev.KeyCode.UpArrow,
      }
      for (const [input, expected] of Object.entries(cases)) {
        assert.strictEqual(rdev.stringKeyToKeycode(input), expected, `Failed mapping ${input}`)
      }
    })

    test('case insensitivity', () => {
      assert.strictEqual(rdev.stringKeyToKeycode('KEYA'), rdev.KeyCode.KeyA)
      assert.strictEqual(rdev.stringKeyToKeycode('Esc'), rdev.KeyCode.Escape)
    })

    test('numpad keys', () => {
      assert.strictEqual(rdev.stringKeyToKeycode('kp0'), rdev.KeyCode.Kp0)
      assert.strictEqual(rdev.stringKeyToKeycode('numpad0'), rdev.KeyCode.Kp0)
    })

    test('invalid inputs', () => {
      assert.strictEqual(rdev.stringKeyToKeycode('not-a-key'), null)
      assert.strictEqual(rdev.stringKeyToKeycode(''), null)
    })
  })

  describe('System & Display', () => {
    test('getDisplaySize format', () => {
      try {
        const size = rdev.getDisplaySize()
        assert.ok(typeof size.width === 'number' && size.width >= 0)
        assert.ok(typeof size.height === 'number' && size.height >= 0)
      } catch (e) {
        if (e.message.includes('NoDisplay')) {
          console.log('Skipping display size check: NoDisplay')
          return
        }
        throw e
      }
    })

    test('initSimulation lifecycle', () => {
      try {
        rdev.initSimulation()
      } catch (e) {
        if (e.message.includes('NoDisplay')) {
          console.log('Skipping initSimulation: NoDisplay')
          return
        }
        throw e
      }
    })
  })

  describe('Simulation Events', () => {
    let simulationInitialized = false
    before(() => {
      try {
        rdev.initSimulation()
        simulationInitialized = true
      } catch (e) {
        if (e.message.includes('NoDisplay')) {
          console.log('Skipping simulation events: NoDisplay')
        } else {
          throw e
        }
      }
    })

    const now = Date.now()
    const testEvents = [
      {
        name: 'Key Press',
        data: { eventType: rdev.EventTypeValue.KeyPress, keyPress: { key: rdev.KeyCode.KeyB }, time: now },
      },
      {
        name: 'Key Release',
        data: { eventType: rdev.EventTypeValue.KeyRelease, keyRelease: { key: rdev.KeyCode.KeyB }, time: now + 5 },
      },
      {
        name: 'Mouse Move',
        data: { eventType: rdev.EventTypeValue.MouseMove, mouseMove: { x: 50, y: 50 }, time: now + 10 },
      },
      {
        name: 'Button Press',
        data: {
          eventType: rdev.EventTypeValue.ButtonPress,
          buttonPress: { button: rdev.ButtonType.Right },
          time: now + 15,
        },
      },
      {
        name: 'Wheel',
        data: { eventType: rdev.EventTypeValue.Wheel, wheel: { deltaX: 5, deltaY: -5 }, time: now + 20 },
      },
    ]

    for (const { name, data } of testEvents) {
      test(`simulateEvent: ${name}`, (t) => {
        if (!simulationInitialized) {
          t.skip('No display available')
          return
        }
        assert.doesNotThrow(() => rdev.simulateEvent(data))
      })
    }

    test('simulateEvent validation - missing keyPress', (t) => {
      if (!simulationInitialized) {
        t.skip('No display available')
        return
      }
      assert.throws(
        () => {
          rdev.simulateEvent({ eventType: rdev.EventTypeValue.KeyPress, time: now })
        },
        { message: /Missing key_press/ },
      )
    })

    test('simulateEvent validation - missing mouseMove', (t) => {
      if (!simulationInitialized) {
        t.skip('No display available')
        return
      }
      assert.throws(
        () => {
          rdev.simulateEvent({ eventType: rdev.EventTypeValue.MouseMove, time: now })
        },
        { message: /Missing mouse_move/ },
      )
    })
  })

  describe('Listener API', () => {
    test('stopListener reports false when nothing is running', () => {
      assert.strictEqual(rdev.stopListener(), false)
    })

    test('startListener guards duplicates and stopListener releases', (t) => {
      try {
        rdev.initSimulation()
      } catch (e) {
        t.skip(`No display available: ${e.message}`)
        return
      }
      const errors = []
      const result = rdev.startListener(
        () => {},
        (message) => errors.push(message),
      )
      assert.strictEqual(result, undefined)
      assert.throws(() => rdev.startListener(() => {}), { message: /already running/ })
      assert.strictEqual(rdev.stopListener(), true)
      assert.strictEqual(rdev.stopListener(), false)
      assert.deepStrictEqual(errors, [])
    })

    test('listener receives a simulated event (child process)', { timeout: 120000 }, (t) => {
      const child = path.join(__dirname, 'listener-child.cjs')
      // Generous outer timeout: the child always exits by itself within 15s;
      // this only guards against a stalled runner killing the suite.
      const proc = spawnSync(process.execPath, [child], {
        encoding: 'utf8',
        timeout: 60000,
      })
      if (proc.error) {
        assert.fail(`listener child failed to run: ${proc.error.message}`)
      }
      const output = `${proc.stdout}${proc.stderr}`
      if (output.includes('SKIP:')) {
        t.skip(
          output
            .trim()
            .split('\n')
            .find((line) => line.includes('SKIP:')),
        )
        return
      }
      assert.strictEqual(proc.status, 0, `listener child exited ${proc.status}: ${output}`)
      assert.ok(output.includes('RECEIVED'), `listener child never saw the event: ${output}`)
    })
  })
})
