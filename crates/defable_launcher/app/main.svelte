<script>
    import { remote } from 'electron'
    import addon from './index.node'

    const form = {
        fableExecutable: ''
    }

    function launch () {
        addon.launch_fable(form.fableExecutable)
    }

    function openDialog () {
        let result = remote.dialog.showOpenDialog({ properties: [ 'openFile' ] })
        if (result) result = result[0]
        form.fableExecutable = result
    }
</script>

<main>
    <div class="options">
        <div class="game">
            <span class="label">Fable Executable</span>
            <div class="file-select">
                <input bind:value={form.fableExecutable} />
                <button on:click={openDialog}>Select File</button>
            </div>
            <button on:click={launch}>Launch</button>
        </div>
        <div class="mods">
        </div>
    </div>
    <div class="errors">

    </div>
</main>