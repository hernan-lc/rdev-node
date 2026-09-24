import { EventTypeValue, type KeyCode, startListener, stringKeyToKeycode } from '../index'

/**
 * A simple but effective Shortcut Manager
 */
class ShortcutManager {
  private pressedKeys = new Set<KeyCode>()
  private shortcuts = new Map<string, () => void>()

  constructor() {
    startListener((event) => {
      if (event.eventType === EventTypeValue.KeyPress && event.keyPress) {
        this.pressedKeys.add(event.keyPress.key)
        this.checkShortcuts()
      } else if (event.eventType === EventTypeValue.KeyRelease && event.keyRelease) {
        this.pressedKeys.delete(event.keyRelease.key)
      }
    })
  }

  /**
   * Register a shortcut like "Ctrl+Shift+S"
   */
  register(combo: string, callback: () => void) {
    const parts = combo.split('+').map((p) => p.trim())
    const normalizedParts = parts
      .map((p) => {
        const code = stringKeyToKeycode(p)
        if (!code) throw new Error(`Invalid key: ${p}`)
        return code
      })
      .sort()

    this.shortcuts.set(normalizedParts.join(','), callback)
    console.log(`Registered shortcut: ${combo}`)
  }

  private checkShortcuts() {
    const sortedPressed = Array.from(this.pressedKeys).sort().join(',')
    const handler = this.shortcuts.get(sortedPressed)
    if (handler) handler()
  }
}

const manager = new ShortcutManager()

manager.register('ControlLeft + ShiftLeft + S', () => {
  console.log('✅ Shortcut triggered: Save (Ctrl+Shift+S)')
})

manager.register('Alt + Q', () => {
  console.log('✅ Shortcut triggered: Quit (Alt+Q)')
  process.exit(0)
})

console.log('Shortcut manager active. Try Ctrl+Shift+S or Alt+Q.')
