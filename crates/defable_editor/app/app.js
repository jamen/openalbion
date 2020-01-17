import App from './views/app.svelte'
import addon from './defable_editor.node'

console.log(addon.hello())

new App({
    target: document.body,
    hydrate: true
})

