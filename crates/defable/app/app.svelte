<script>
    import { remote } from 'electron'
    import addon from './index.node'
    import path from 'path'

    let errors = []
    let mods = []
    let fableExecutable = ''
    let defaultCheat = true
    let defaultCheatDll = path.resolve(__dirname, './fable_cheat.dll')
    let cheatDll = defaultCheatDll

    function launch () {
        addon.launch_fable(fableExecutable, cheatDll)
    }

    function selectFableExecutable () {
        fableExecutable = openDialog()
    }

    function selectCheatDll () {
        cheatDll = openDialog()
    }

    function openDialog () {
        let result = remote.dialog.showOpenDialog({ properties: [ 'openFile' ] })
        return result && result[0]
    }

    function dismissErrors () {
        errors = []
    }

    function addMod () {

    }

    function removeMod () {

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
    <section class="options">
        <h2>Options</h2>
        <div class="list options-list">
            <div class="option">
                <h3>Fable.exe</h3>
                <div class="file-select">
                    <input type="text" bind:value={fableExecutable} />
                    <button on:click={selectFableExecutable}>Select File</button>
                </div>
            </div>
            <div class="option">
                <h3>DLL Cheat</h3>
                <span>Use default cheat?</span>
                <input type="checkbox" bind:checked={defaultCheat} >
                <div class="file-select">
                    <input type="text" bind:value={cheatDll} disabled={defaultCheat ? 'disabled' : ''} value={defaultCheat ? defaultCheatDll : cheatDll} />
                    <button on:click={selectCheatDll} disabled={defaultCheat ? 'disabled' : ''}>Select File</button>
                </div>
            </div>
        </div>
    </section>
    <section class="mods">
        <h2>Mods</h2>
        <div class="list mods-list">
            {#each mods as mod}
                <div class='mod'>

                </div>
            {/each}
        </div>
    </section>
    <section class="mods-options">
        <button class="btn add" on:click={addMod}>
            <img src="static/add-24px.svg" alt="">
            <span>Add</span>
        </button>
        <button class="btn remove" on:click={removeMod}>
            <img src="static/remove-24px.svg" alt="">
            <span>Remove</span>
        </button>
    </section>
</main>