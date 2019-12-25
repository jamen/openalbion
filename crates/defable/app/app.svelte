<script>
    import { remote } from 'electron'
    import addon from './index.node'

    let errors = []
    let mods = []
    let fableExecutable = ''

    function launch () {
        addon.launch_fable(fableExecutable)
    }

    function openDialog () {
        let result = remote.dialog.showOpenDialog({ properties: [ 'openFile' ] })
        if (result) result = result[0]
        fableExecutable = result
    }

    function dismissErrors () {
        errors = []
    }
</script>

<main>
    {#if errors.length > 0}
        <div class="errors">
            <img src="static/dismiss.svg" alt="Dismiss" on:click={dismissErrors}>
            <ul>
                {#each errors as error}
                    <li>{error}</li>
                {/each}
            </ul>
        </div>
    {/if}
    <div class="options">
        <label>Fable executable<label>
        <div class="file-select">
            <input bind:value={fableExecutable} />
            <button on:click={openDialog}>Select File</button>
        </div>
    </div>
    <div class="mods">
        {#each mods as mod}
            <div class='mod'>

            </div>
        {/each}
    </div>
    <button on:click={launch}>Launch</button>
</main>