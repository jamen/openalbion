<script>
    import FileSelect from './file-select.svelte'

    export let options = {}

    function selectCheatDll () {
        options.cheatDll = openDialog()
    }

    function useDefaultCheat (event) {
        options.defaultCheat = event.target.checked
        options.cheatDll = options.defaultCheat ? options.defaultCheatDll : options.cheatDll
    }
</script>

<section class="page options">
    <div class="list options-list">
        <div class="option">
            <h2>Fable.exe</h2>
            <FileSelect bind:value={options.fableExecutable} />
        </div>
        <div class="option">
            <h2>DLL Cheat</h2>
            <span>Use default cheat?</span>
            <input type="checkbox" on:click={useDefaultCheat} checked={options.defaultCheat}>
            <FileSelect bind:value={options.cheatDll} disabled={options.defaultCheat} />
        </div>
        <div class="option">
            <h2>Window</h2>
            <span>Use full-screen?</span>
            <input type="checkbox" bind:checked={options.fullscreen}>
            <div class="dim">
                <span class='pre'>Dimensions:</span>
                <input class='field' bind:value={options.dimensions[0]} disabled={options.fullscreen} placeholder='x' type="text">
                <span class='x'>,</span>
                <input class='field' bind:value={options.dimensions[1]} disabled={options.fullscreen} placeholder='y' type="text">
                <span class='x'>,</span>
                <input class='field' bind:value={options.dimensions[2]} disabled={options.fullscreen} placeholder='width' type="text">
                <span class='x'>,</span>
                <input class='field' bind:value={options.dimensions[3]} disabled={options.fullscreen} placeholder='height' type="text">
            </div>
        </div>
    </div>
</section>