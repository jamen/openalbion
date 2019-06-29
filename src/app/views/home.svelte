<script>
    import { fableDirectory } from '../stores.js'
    import { spawn } from 'child_process'
    import { join } from 'path'
    import { Link } from '@jamen/svelte-router'

    const fableExecutable = join($fableDirectory, 'Fable.exe')
    let proc = null

    function toggleFableProcess () {
        if (proc) {
            proc.kill()
            proc.on('close', () => proc = null)
        } else {
            proc = spawn(fableExecutable, {
                detached: true,
                windowsHide: true,
                stdio: 'ignore'
            })
        }
    }
</script>

<style>
    .home {
        display: flex;
    }
</style>

<div class='home'>
    <Link href='/wad-extract'>View WAD</Link>
    <Link href='/big'>View BIG</Link>
    <button on:click={toggleFableProcess}>
        {#if proc === null}
            Launch Fable
        {:else}
            Stop Fable
        {/if}
    </button>
</div>