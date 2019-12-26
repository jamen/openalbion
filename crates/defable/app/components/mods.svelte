<script>
    const mods = []

    function addMod () {

    }

    function removeMod () {

    }
</script>

<section class="page mods">
    <div class="list mods-list">
        {#each mods as mod}
            <div class='mod'>

            </div>
        {/each}
    </div>
    <div class="mods-options">
        <button class="btn add" on:click={addMod}>
            <img src="static/add-24px.svg" alt="">
            <span>Add</span>
        </button>
        <button class="btn remove" on:click={removeMod}>
            <img src="static/remove-24px.svg" alt="">
            <span>Remove</span>
        </button>
    </div>
</section>