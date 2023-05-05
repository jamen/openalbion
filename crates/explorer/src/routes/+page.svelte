<script lang="ts">
    import { open } from "@tauri-apps/api/dialog";

    let path: string | string[] | null = null;

    async function onClick() {
        path = await open({
            directory: true,
            multiple: false,
        });
    }
</script>

<button on:click={onClick}>Open File</button>
<span>{path}</span>
