<!-- https://svelte.dev/playground/d65a4e9f0ae74d1eb1b08d13e428af32?version=3.35.0 --><!-- https://svelte.dev/playground/d65a4e9f0ae74d1eb1b08d13e428af32?version=3.35.0 -->

<script lang="ts">
    // based on sug6gestions from:
    // Inclusive Components by Heydon Pickering https://inclusive-components.design/toggle-button/
    // On Designing and Building Toggle Switches by Sara Soueidan https://www.sarasoueidan.com/blog/toggle-switch-design/
    // and this example by Scott O'hara https://codepen.io/scottohara/pen/zLZwNv 


    export let label: string;
    export let value: bool | string = 'on';
    export let size: number = 1;
    let checked = true;


	const uniqueID = Math.floor(Math.random() * 100)

    function handleClick(event){
        const target = event.target
        const state = target.getAttribute('aria-checked')
        checked = state === 'true' ? false : true
        value = checked === true ? 'on' : 'off'
    }
	
    const slugify = (str = "") =>
    str.toLowerCase().replace(/ /g, "-").replace(/\./g, "");
</script>

<div class="toggle-switch" style="--size={size}">
    <span>{label}</span>
    <button
    role="switch"
    aria-checked={checked}
    aria-labelledby={`switch-${uniqueID}`}
    on:click={handleClick}
    style="--size={size}">
    </button>
    
</div>

<style>
    .toggle-switch {
        display: flex;
        box-sizing: border-box;
        align-items: center;
        width: 100%;
    }

    .toggle-switch span {
        margin-left: calc(var(--size) / 4);
        font-size: var(--size);
        text-overflow: ellipsis;
        overflow: hidden;
        max-width: calc(100% - var(--size) * 2.3)
    }

    .toggle-switch button {
        flex-shrink: 0;
        width: calc(var(--size) * 2.3);
        height: calc(var(--size) * 1.1);
        background: var(--background-3);
        /* margin: 0 0 0 calc(var(--size)); */
        position: relative;
        border: none;
        cursor: pointer;
        margin-left: auto;
    }

    .toggle-switch button::before {
        content: '';
        position: absolute;
        width: var(--size);
        height: var(--size);
        background: var(--foreground-3);
        /* top: 0.15em; */
        /* right: 0.5em; */
        top: calc(var(--size) / 20);
        right: calc(var(--size) * 1.2);
        transition: transform 0.2s;
    }

    .toggle-switch button[aria-checked="true"] {
        background-color: var(--accent-0);
    }

    .toggle-switch button[aria-checked="true"]::before {
        transform: translateX(calc(var(--size) / 0.9));
        transition: transform 0.2s;
        background-color: var(--foreground-1);
    }

    .toggle-switch button {
        border-radius: var(--size);
    }

    .toggle-switch button::before {
        border-radius: 100%;
    }
</style>