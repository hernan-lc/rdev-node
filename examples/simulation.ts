import { ButtonType, EventTypeValue, KeyCode, getDisplaySize, initSimulation, simulateEvent } from '../index'

// Initialize simulation (important for Linux/X11)
try {
  initSimulation()
  console.log('Simulation initialized.')
} catch (e) {
  console.error('Failed to initialize simulation:', e)
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

async function runDemo() {
  const size = getDisplaySize()
  console.log(`Display size: ${size.width}x${size.height}`)

  console.log('\n--- Keyboard Simulation ---')
  console.log('Typing "ABC"...')

  const keys = [KeyCode.KeyA, KeyCode.KeyB, KeyCode.KeyC]
  for (const key of keys) {
    simulateEvent({ eventType: EventTypeValue.KeyPress, keyPress: { key }, time: Date.now() })
    await sleep(50)
    simulateEvent({ eventType: EventTypeValue.KeyRelease, keyRelease: { key }, time: Date.now() })
    await sleep(50)
  }

  console.log('\n--- Mouse Simulation ---')
  console.log('Moving mouse to center and clicking...')

  const centerX = size.width / 2
  const centerY = size.height / 2

  simulateEvent({
    eventType: EventTypeValue.MouseMove,
    mouseMove: { x: centerX, y: centerY },
    time: Date.now(),
  })
  await sleep(100)

  simulateEvent({
    eventType: EventTypeValue.ButtonPress,
    buttonPress: { button: ButtonType.Left },
    time: Date.now(),
  })
  await sleep(50)
  simulateEvent({
    eventType: EventTypeValue.ButtonRelease,
    buttonRelease: { button: ButtonType.Left },
    time: Date.now(),
  })

  console.log('\n--- Wheel Simulation ---')
  console.log('Scrolling down...')
  simulateEvent({
    eventType: EventTypeValue.Wheel,
    wheel: { deltaX: 0, deltaY: 10 },
    time: Date.now(),
  })

  console.log('\nSimulation demo complete.')
}

runDemo().catch(console.error)
