import resolve from 'rollup-plugin-node-resolve'
import replace from '@rollup/plugin-replace'
import commonjs from 'rollup-plugin-commonjs'
import svelte from 'rollup-plugin-svelte'
import { terser } from 'rollup-plugin-terser'
import pkg from './package.json'
import { builtinModules } from "module"

const mode = process.env.NODE_ENV;
const dev = mode === 'development';
const legacy = !!process.env.SAPPER_LEGACY_BUILD;

const onwarn = (warning, onwarn) => (warning.code === 'CIRCULAR_DEPENDENCY' && /[/\\]@sapper[/\\]/.test(warning.message)) || onwarn(warning);
const dedupe = importee => importee === 'svelte' || importee.startsWith('svelte/');

const external = [
	'electron',
	'./index.node',
	...builtinModules,
	...Object.keys(pkg.dependencies || {})
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
				'process.browser': false,
				'process.env.NODE_ENV': JSON.stringify(mode)
			}),
			resolve({ dedupe }),
			commonjs(),
			!dev && terser({ module: true })
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
				'process.env.NODE_ENV': JSON.stringify(mode)
			}),
			svelte({ dev }),
			resolve({ dedupe }),
			commonjs(),
			!dev && terser({ module: true })
		],
		onwarn
	}
]