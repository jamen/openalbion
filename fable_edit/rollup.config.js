import svelte from 'rollup-plugin-svelte'
import { builtinModules } from "module"

export default [
    {
        input: 'src/fable_edit.js',
        external: [
            ...builtinModules,
            'electron',
            './native'
        ],
        output: {
            file: 'out/fable_edit.js',
            format: 'cjs'
        }
    },
    {
        input: 'src/index.js',
        external: [
            ...builtinModules,
            'electron',
            './native'
        ],
        output: {
            file: 'out/index.js',
            format: 'cjs'
        },
        plugins: [
            svelte()
        ]
    }
]