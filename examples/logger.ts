import { EventTypeValue, normalizeKeyName, startListener } from '../index'

console.log('--- rdev-node Logger ---')
console.log('Listening for all input events. Press Ctrl+C to stop.\n')

startListener((event) => {
  const time = new Date(event.time).toISOString().split('T')[1].replace('Z', '')
  let detail = ''

  switch (event.eventType) {
    case EventTypeValue.KeyPress: {
      const key = event.keyPress?.key
      detail = key ? `Key: ${key} (Normalized: ${normalizeKeyName(key)})` : 'Key: (no data)'
      break
    }
    case EventTypeValue.KeyRelease:
      detail = `Key: ${event.keyRelease?.key}`
      break
    case EventTypeValue.MouseMove:
      detail = `Pos: ${event.mouseMove?.x}, ${event.mouseMove?.y}`
      break
    case EventTypeValue.ButtonPress:
      detail = `Button: ${event.buttonPress?.button}`
      break
    case EventTypeValue.ButtonRelease:
      detail = `Button: ${event.buttonRelease?.button}`
      break
    case EventTypeValue.Wheel:
      detail = `Delta: ${event.wheel?.deltaX}, ${event.wheel?.deltaY}`
      break
  }

  console.log(`[${time}] ${event.eventType.padEnd(15)} | ${detail}`)
})
