import nodeResolve from '@rollup/plugin-node-resolve'
import replace from '@rollup/plugin-replace'
import svelte from 'rollup-plugin-svelte'
import { terser } from 'rollup-plugin-terser'
import { builtinModules } from 'module'

const devMode =  process.env.NODE_ENV === 'development'
const external = [
    'electron',
    './defable_editor.node',
    ...builtinModules,
    ...Object.keys(process.binding('natives'))
]

export default [
    {
        input: 'app/main.js',
        output: {
            file: 'out/main.js',
            format: 'cjs'
        },
        external,
        plugins: [
            replace({
                'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV)
            }),
            nodeResolve(),
            !devMode && terser({ module: true })
        ]
    },

    {
        input: 'app/app.js',
        output: {
            file: 'out/app.js',
            format: 'cjs'
        },
        external,
        plugins: [
            replace({
                'process.browser': true,
                'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV)
            }),
            svelte({
                generate: 'dom',
                hydratable: true,
                dev: devMode,
            }),
            nodeResolve(),
            !devMode && terser({ module: true })
        ]
    },

    {
        input: 'app/views/app.svelte',
        output: {
            file: 'out/app.ssr.js',
            format: 'cjs'
        },
        external,
        plugins: [
            replace({
                'process.browser': false,
                'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV)
            }),
            svelte({
                generate: 'ssr',
                hydratable: true,
                dev: devMode
            }),
            nodeResolve(),
            !devMode && terser({ module: true })
        ]
    }
]