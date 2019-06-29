import { builtinModules } from 'module'
import resolve from 'rollup-plugin-node-resolve'
import commonjs from 'rollup-plugin-commonjs'
import svelte from 'rollup-plugin-svelte'
// import { dependencies } from './package.json'

export default [
    {
        input: 'src/main.js',
        output: {
            file: 'dist/main.js',
            format: 'cjs'
        },
        external: [
            ...builtinModules,
            'electron'
        ],
        plugins: [
            resolve(),
            commonjs()
        ]
    },
    {
        input: 'src/app.js',
        output: {
            file: 'dist/app.js',
            format: 'cjs'
        },
        external: [
            ...builtinModules,
            'electron'
        ],
        plugins: [
            resolve(),
            commonjs(),
            svelte({
                css: file => file.write('dist/app.css')
            })
        ]
    }
]