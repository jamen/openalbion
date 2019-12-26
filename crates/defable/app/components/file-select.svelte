<script>
    import { remote } from 'electron'

    export let value = ''
    export let disabled = false

    function select () {
        let result = remote.dialog.showOpenDialog({ properties: [ 'openFile' ] })
        if (result) {
            value = result[0]
        }
    }
</script>

<div class="file-select">
    <input class="field" type="text" bind:value={value} disabled={disabled ? 'disabled' : ''}>
    <button class="btn" on:click={select} disabled={disabled ? 'disabled' : ''}>Select</button>
</div>