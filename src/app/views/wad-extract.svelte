<script>
    import { fableDirectory } from '../stores.js'
    import * as wad from '../../parser/wad.js'
    import bytes from 'bytes'
    import { join, basename } from 'path'
    import { remote } from 'electron'

    const wadFile = join($fableDirectory, 'data/Levels/FinalAlbion.wad')
    const wadTask = wad.parseFile(wadFile)

    let filter = ''

    async function extractWadFiles () {
        try {
            await wad.extractWad(wadFile, $fableDirectory)
        } catch (error) {
            console.error(error)
        }
    }
</script>

<style>
    .file-list {
        display: flex;
        flex-direction: column;
    }
</style>

<div class='wad'>
    <button on:click={extractWadFiles}>Extract WAD Files</button>
    {#await wadTask}
        <p>Loading WAD.</p>
    {:then wadData}
        <div class='file-list'>
            {#each wadData.files as file}
                <span class='file'>{basename(file.name)}</span>
            {/each}
        </div>
    {:catch err}
        <p>Failed to read WAD: {err.message}</p>
    {/await}
</div>