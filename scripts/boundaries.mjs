import path from 'node:path'
import { parseSync } from 'oxc-parser'
import { parse } from 'svelte/compiler'

const sourceRoot = path.resolve('ui/src')

function visit(node, check, seen = new WeakSet()) {
  if (!node || typeof node !== 'object' || seen.has(node)) return
  seen.add(node)
  if (typeof node.type === 'string') check(node)
  for (const value of Object.values(node)) {
    if (Array.isArray(value)) {
      for (const child of value) visit(child, check, seen)
    } else visit(value, check, seen)
  }
}

export function checkImports(file, source) {
  const owner = path.relative(sourceRoot, path.resolve(file)).split(path.sep).join('/')
  let ast
  if (file.endsWith('.svelte')) ast = parse(source, { modern: true })
  else {
    const result = parseSync(file, source, { sourceType: 'module' })
    if (result.errors.length) throw new Error(`${file}: ${result.errors[0].message}`)
    ast = result.program
  }
  const errors = []
  visit(ast, (node) => {
    if (node.type === 'HtmlTag') errors.push(`${file}: raw HTML is not allowed in shared UI`)
    let specifier
    if (
      [
        'ImportDeclaration',
        'ExportNamedDeclaration',
        'ExportAllDeclaration',
        'ImportExpression',
      ].includes(node.type)
    ) {
      specifier = node.source?.value
      if (node.type === 'ImportExpression' && typeof specifier !== 'string') {
        errors.push(`${file}: dynamic imports require a literal module path`)
      }
    } else if (node.type === 'CallExpression' && node.callee?.name === 'require') {
      specifier = node.arguments[0]?.value
      if (typeof specifier !== 'string') errors.push(`${file}: require needs a literal module path`)
    }
    if (typeof specifier !== 'string') return
    const report = (message) => errors.push(`${file}: ${message}`)
    if (specifier.startsWith('@tauri-apps/') && !owner.startsWith('lib/native/')) {
      report('Tauri IPC belongs in ui/src/lib/native; components use feature controllers.')
    }
    if (specifier.startsWith('node:')) report('Frontend runtime cannot import Node.js APIs.')
    if (specifier.startsWith('.')) {
      const target = path
        .relative(sourceRoot, path.resolve(path.dirname(path.resolve(file)), specifier))
        .split(path.sep)
        .join('/')
      if (target.startsWith('../'))
        report('Production UI must not import sibling projects or native source.')
      if (owner.startsWith('components/') && /^(features|lib\/native)\//.test(target)) {
        report('Shared presentation components must not import feature controllers or IPC.')
      }
      if (owner.startsWith('lib/') && /^(app|components|features)\//.test(target)) {
        report('Shared libraries cannot depend on UI features or components.')
      }
    }
  })
  return errors
}
