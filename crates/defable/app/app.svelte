<script>
    import Options from './components/options.svelte'
    import Mods from './components/mods.svelte'

    import addon from './index.node'
    import path from 'path'

    let page = 'options'
    let errors = []
    let mods = []

    let options = {
        fableExecutable: '',
        defaultCheat: true,
        defaultCheatDll: path.resolve(__dirname, './fable_cheat.dll'),
        cheatDll: path.resolve(__dirname, './fable_cheat.dll'),
        fullscreen: true,
        dimensions: ['', '', '', '']
    }

    const routes = {
        options: Options,
        mods: Mods,
    }

    const injectorHelper = path.join(__dirname, 'fable_injector_32bit_helper.exe')

    function launch () {
        const handle = addon.create_and_inject(options.fableExecutable, options.cheatDll, injectorHelper)
    }

    function dismissErrors () {
        errors = []
    }

    function edit () {

    }
</script>

<main>
    {#if errors.length > 0}
        <section class="errors">
            <img src="priority_high-24px.svg" alt="Oops.">
            <ul>
                {#each errors as error}
                    <li>{error}</li>
                {/each}
            </ul>
            <img src="static/close-24px.svg" alt="Dismiss" on:click={dismissErrors}>
        </section>
    {/if}
    <section class="btns">
        <button class='btn launch' on:click={launch}>
            <img src="static/power_settings_new-24px.svg" alt="">
            <span>Start</span>
        </button>
        <button class='btn edit' on:click={edit}>
            <img src="static/tune-24px.svg" alt="">
            <span>Edit</span>
        </button>
    </section>
    <nav>
        {#each Object.keys(routes) as pageName}
            <button
                class={`btn nav-btn ${pageName === page ? 'active' : ''}`}
                on:click={() => page = pageName}
            >
                <span>{pageName}</span>
            </button>
        {/each}
    </nav>
    <svelte:component this={routes[page] || Options} options={options} />
</main>