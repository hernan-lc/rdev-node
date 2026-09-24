// Post-build step: napi-rs emits a universal loader that references Android
// bindings, but rdev 0.5.3 does not support Android and this package ships no
// Android binaries. Remove the Android branch from the generated index.js so
// the loader does not falsely advertise Android support.
const { readFileSync, writeFileSync } = require('node:fs')
const path = require('node:path')

const loaderPath = path.join(__dirname, '..', 'index.js')
const androidBranch = "} else if (process.platform === 'android') {"
const nextBranch = "} else if (process.platform === 'win32') {"

const source = readFileSync(loaderPath, 'utf8')
const start = source.indexOf(androidBranch)
const end = source.indexOf(nextBranch, start)
if (start === -1 || end === -1 || end <= start) {
  console.error('strip-android: Android branch not found in index.js; loader template may have changed')
  process.exit(1)
}

const stripped = source.slice(0, start) + source.slice(end)
if (stripped.toLowerCase().includes('android')) {
  console.error('strip-android: index.js still mentions Android after stripping')
  process.exit(1)
}

writeFileSync(loaderPath, stripped)
console.log('strip-android: removed Android branch from index.js')
