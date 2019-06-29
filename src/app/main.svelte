<script>
    import { Router } from '@jamen/svelte-router'
    import { fableDirectory } from './stores.js'
    import './client.js'

    import Selector from './views/selector.svelte'
    import Home from './views/home.svelte'
    import WadExtract from './views/wad-extract.svelte'
    import Big from './views/big.svelte'
    import Lost from './views/lost.svelte'

    const routes = {
        '/': Home,
        '/wad-extract': WadExtract,
        '/big': Big,
        404: Lost
    }
</script>

<style>
    main {
        width: 100%;
        height: 100%;
        background: #202020;
        font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
        font-size: 14px;
        color: #fff;
        overflow: auto;
    }
</style>

<main>
    {#if $fableDirectory}
        <Router {routes} />
    {:else}
        <Selector />
    {/if}
</main>