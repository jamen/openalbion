<script>
    import { remote } from 'electron'
    import { fableDirectory } from '../stores.js'

    let failed = false

    function showDialog () {
        remote.dialog.showOpenDialog({
            title: 'Select Fable\'s directory',
            properties: [ 'openDirectory' ]
        }, (files) => {
            if (files && files.length === 1) {
                $fableDirectory = files[0]
                failed = false
            } else {
                failed = true
            }
        })
    }

    showDialog()
</script>

<style>
    .selector {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        width: 100%;
        height: 100%;
    }

    .text {
        text-align: center;
        font-size: 28px;
        margin-bottom: 10px;
    }

    .error {
        color: #d81b1b;
        font-weight: bold;
    }

    .retry {
        text-decoration: underline;
        cursor: pointer;
    }

    p {
        margin: 0;
    }
</style>

<div class='selector'>
    <p class='text'>Open Fable's directory to begin</p>
    {#if failed}
        <p class='error'>Opening Fable failed. <span class='retry' on:click={showDialog}>Try again?</span></p>
    {/if}
</div>