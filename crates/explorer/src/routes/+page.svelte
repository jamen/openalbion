<script>
    import { invoke } from "@tauri-apps/api/tauri";
    import { onMount } from "svelte";

    let message = "";

    onMount(async () => {
        message = await invoke("greet", { name: "world" });
    });
</script>

{message}
